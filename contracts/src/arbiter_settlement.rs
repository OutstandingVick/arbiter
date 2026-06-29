//! Arbiter — parimutuel settlement contract.
//!
//! A market is opened by the admin. Stakers back an outcome (a `side`) by
//! attaching native CSPR. After the real-world event resolves, the authorised
//! `resolver` (the Arbiter agent) submits the winning side together with a
//! `proof_ref` — a reference to the x402 receipt for the verified outcome it
//! paid for. `settle` then snapshots the pools and takes the rake; winners pull
//! their pro-rata share with `claim`.
//!
//! Settlement asset is native CSPR for this vertical. The x402 *payment for
//! truth* (the resolver paying the outcome endpoint) settles separately in the
//! CEP-18 x402 token at the agent layer; this contract only settles the market.

use odra::casper_types::U512;
use odra::prelude::*;

use crate::arbiter_settlement::errors::Error;
use crate::arbiter_settlement::events::*;

/// Lifecycle of a market.
#[odra::odra_type]
#[derive(Default)]
pub enum MarketStatus {
    /// Open for staking. Default state on creation.
    #[default]
    Open,
    /// Outcome submitted by the resolver; awaiting settlement.
    Resolved,
    /// Pools snapshotted, rake taken; winners may claim.
    Settled
}

/// Stored state of a single market.
#[odra::odra_type]
pub struct Market {
    /// Current lifecycle status.
    pub status: MarketStatus,
    /// Block time (ms) after which staking is closed.
    pub close_time: u64,
    /// Winning side, valid once `status` is `Resolved`/`Settled`.
    pub winning_side: u8,
    /// Reference to the x402 receipt that justified the resolution.
    pub proof_ref: String,
    /// Total staked across all sides (snapshot at settle).
    pub total_pool: U512,
    /// Total staked on the winning side (snapshot at settle).
    pub winning_pool: U512,
    /// Distributable pool after rake (snapshot at settle).
    pub payout_pool: U512,
    /// Rake taken to treasury at settle.
    pub rake: U512
}

/// Arbiter parimutuel settlement module.
#[odra::module(
    events = [MarketCreated, Staked, Resolved, Settled, Claimed],
    errors = Error
)]
pub struct ArbiterSettlement {
    /// Contract administrator (deployer). Creates markets, rotates resolver.
    admin: Var<Address>,
    /// The agent authorised to submit resolutions.
    resolver: Var<Address>,
    /// Recipient of the rake.
    treasury: Var<Address>,
    /// Rake in basis points (e.g. 200 = 2.00%). Capped at 1000 (10%).
    rake_bps: Var<u32>,
    /// Monotonic market id counter.
    market_count: Var<u64>,
    /// market_id -> Market.
    markets: Mapping<u64, Market>,
    /// running total staked per market (live during Open).
    market_total: Mapping<u64, U512>,
    /// (market_id, side) -> total staked on that side.
    pool: Mapping<(u64, u8), U512>,
    /// (market_id, side, staker) -> that staker's stake on the side.
    stake_of: Mapping<(u64, u8, Address), U512>
}

#[odra::module]
impl ArbiterSettlement {
    /// Initialise with the resolver agent, the treasury, and the rake.
    pub fn init(&mut self, resolver: Address, treasury: Address, rake_bps: u32) {
        if rake_bps > 1000 {
            self.env().revert(Error::InvalidRake);
        }
        self.admin.set(self.env().caller());
        self.resolver.set(resolver);
        self.treasury.set(treasury);
        self.rake_bps.set(rake_bps);
        self.market_count.set(0);
    }

    /// Open a new market. Admin only. Returns the new market id.
    pub fn create_market(&mut self, close_time: u64) -> u64 {
        self.assert_admin();
        let id = self.market_count.get_or_default();
        self.market_count.set(id + 1);
        self.markets.set(
            &id,
            Market {
                status: MarketStatus::Open,
                close_time,
                winning_side: 0,
                proof_ref: String::new(),
                total_pool: U512::zero(),
                winning_pool: U512::zero(),
                payout_pool: U512::zero(),
                rake: U512::zero()
            }
        );
        self.env().emit_event(MarketCreated {
            market_id: id,
            close_time
        });
        id
    }

    /// Stake the attached CSPR on `side` of `market_id`.
    #[odra(payable)]
    pub fn stake(&mut self, market_id: u64, side: u8) {
        let amount = self.env().attached_value();
        if amount.is_zero() {
            self.env().revert(Error::ZeroStake);
        }
        let market = self.load_market(market_id);
        if market.status != MarketStatus::Open {
            self.env().revert(Error::MarketNotOpen);
        }
        if self.env().get_block_time() >= market.close_time {
            self.env().revert(Error::StakingClosed);
        }

        let caller = self.env().caller();
        self.pool.add(&(market_id, side), amount);
        self.stake_of.add(&(market_id, side, caller), amount);
        self.market_total.add(&market_id, amount);

        self.env().emit_event(Staked {
            market_id,
            side,
            staker: caller,
            amount
        });
    }

    /// Submit the resolved outcome. Resolver only.
    ///
    /// `proof_ref` ties the on-chain resolution to the off-chain x402 receipt
    /// the agent paid for to obtain the verified outcome.
    pub fn submit_resolution(&mut self, market_id: u64, winning_side: u8, proof_ref: String) {
        if self.env().caller() != self.resolver.get_or_revert_with(Error::Unauthorized) {
            self.env().revert(Error::Unauthorized);
        }
        let mut market = self.load_market(market_id);
        if market.status != MarketStatus::Open {
            self.env().revert(Error::MarketNotOpen);
        }
        market.status = MarketStatus::Resolved;
        market.winning_side = winning_side;
        market.proof_ref = proof_ref.clone();
        self.markets.set(&market_id, market);

        self.env().emit_event(Resolved {
            market_id,
            winning_side,
            proof_ref
        });
    }

    /// Snapshot pools, take rake, and open claims. Resolver only.
    ///
    /// This is the agent's autonomous settlement action: it moves the rake
    /// on-chain and finalises the payout ratio. If nobody backed the winning
    /// side the entire pool routes to the treasury.
    pub fn settle(&mut self, market_id: u64) {
        if self.env().caller() != self.resolver.get_or_revert_with(Error::Unauthorized) {
            self.env().revert(Error::Unauthorized);
        }
        let mut market = self.load_market(market_id);
        if market.status != MarketStatus::Resolved {
            self.env().revert(Error::MarketNotResolved);
        }

        let total_pool = self.market_total.get_or_default(&market_id);
        let winning_pool = self.pool.get_or_default(&(market_id, market.winning_side));
        let treasury = self.treasury.get_or_revert_with(Error::Unauthorized);

        if winning_pool.is_zero() {
            // No winners: house collects the whole pool.
            if !total_pool.is_zero() {
                self.env().transfer_tokens(&treasury, &total_pool);
            }
            let winning_side = market.winning_side;
            let proof_ref = market.proof_ref.clone();
            market.total_pool = total_pool;
            market.winning_pool = U512::zero();
            market.payout_pool = U512::zero();
            market.rake = total_pool;
            market.status = MarketStatus::Settled;
            self.markets.set(&market_id, market);
            self.env().emit_event(Settled {
                market_id,
                winning_side,
                total_pool,
                payout_pool: U512::zero(),
                rake: total_pool,
                proof_ref
            });
            return;
        }

        let rake_bps = U512::from(self.rake_bps.get_or_default());
        let rake = total_pool * rake_bps / U512::from(10_000u32);
        let payout_pool = total_pool - rake;

        if !rake.is_zero() {
            self.env().transfer_tokens(&treasury, &rake);
        }

        market.total_pool = total_pool;
        market.winning_pool = winning_pool;
        market.payout_pool = payout_pool;
        market.rake = rake;
        market.status = MarketStatus::Settled;
        let winning_side = market.winning_side;
        let proof_ref = market.proof_ref.clone();
        self.markets.set(&market_id, market);

        self.env().emit_event(Settled {
            market_id,
            winning_side,
            total_pool,
            payout_pool,
            rake,
            proof_ref
        });
    }

    /// Claim winnings for a winning-side stake. Callable once per staker.
    pub fn claim(&mut self, market_id: u64) {
        let market = self.load_market(market_id);
        if market.status != MarketStatus::Settled {
            self.env().revert(Error::MarketNotSettled);
        }
        if market.winning_pool.is_zero() {
            self.env().revert(Error::NothingToClaim);
        }

        let caller = self.env().caller();
        let side = market.winning_side;
        let stake = self.stake_of.get_or_default(&(market_id, side, caller));
        if stake.is_zero() {
            self.env().revert(Error::NothingToClaim);
        }

        // Pro-rata share of the post-rake pool. Multiply before dividing.
        let payout = stake * market.payout_pool / market.winning_pool;

        // Zero the stake first to prevent re-entrant / double claims.
        self.stake_of.set(&(market_id, side, caller), U512::zero());
        self.env().transfer_tokens(&caller, &payout);

        self.env().emit_event(Claimed {
            market_id,
            staker: caller,
            amount: payout
        });
    }

    /// Rotate the resolver agent. Admin only.
    pub fn set_resolver(&mut self, resolver: Address) {
        self.assert_admin();
        self.resolver.set(resolver);
    }

    // ----- views -----

    /// Returns the full market record.
    pub fn get_market(&self, market_id: u64) -> Market {
        self.load_market(market_id)
    }

    /// Returns the total staked on a given side.
    pub fn get_pool(&self, market_id: u64, side: u8) -> U512 {
        self.pool.get_or_default(&(market_id, side))
    }

    /// Returns a staker's stake on a given side.
    pub fn get_stake(&self, market_id: u64, side: u8, staker: Address) -> U512 {
        self.stake_of.get_or_default(&(market_id, side, staker))
    }

    /// Returns the running total staked across all sides of a market.
    pub fn get_total(&self, market_id: u64) -> U512 {
        self.market_total.get_or_default(&market_id)
    }

    /// Returns the current resolver agent.
    pub fn get_resolver(&self) -> Address {
        self.resolver.get_or_revert_with(Error::Unauthorized)
    }

    /// Returns the number of markets created.
    pub fn market_count(&self) -> u64 {
        self.market_count.get_or_default()
    }

    // ----- internals -----

    fn assert_admin(&self) {
        if self.env().caller() != self.admin.get_or_revert_with(Error::Unauthorized) {
            self.env().revert(Error::Unauthorized);
        }
    }

    fn load_market(&self, market_id: u64) -> Market {
        self.markets
            .get(&market_id)
            .unwrap_or_revert_with(&self.env(), Error::MarketNotFound)
    }
}

/// Arbiter events.
pub mod events {
    use odra::casper_types::U512;
    use odra::prelude::*;

    /// A new market was opened.
    #[odra::event]
    pub struct MarketCreated {
        /// Market id.
        pub market_id: u64,
        /// Staking close time (ms).
        pub close_time: u64
    }

    /// A stake was placed on a side.
    #[odra::event]
    pub struct Staked {
        /// Market id.
        pub market_id: u64,
        /// Backed side.
        pub side: u8,
        /// Staker.
        pub staker: Address,
        /// Amount staked (motes).
        pub amount: U512
    }

    /// The resolver submitted the outcome.
    #[odra::event]
    pub struct Resolved {
        /// Market id.
        pub market_id: u64,
        /// Winning side.
        pub winning_side: u8,
        /// x402 receipt reference for the verified outcome.
        pub proof_ref: String
    }

    /// The market was settled and the rake taken.
    #[odra::event]
    pub struct Settled {
        /// Market id.
        pub market_id: u64,
        /// Winning side.
        pub winning_side: u8,
        /// Total pool (motes).
        pub total_pool: U512,
        /// Distributable pool after rake (motes).
        pub payout_pool: U512,
        /// Rake taken to treasury (motes).
        pub rake: U512,
        /// x402 receipt reference.
        pub proof_ref: String
    }

    /// A winner claimed their share.
    #[odra::event]
    pub struct Claimed {
        /// Market id.
        pub market_id: u64,
        /// Claiming staker.
        pub staker: Address,
        /// Amount paid out (motes).
        pub amount: U512
    }
}

/// Arbiter errors.
pub mod errors {
    use odra::prelude::*;

    /// Settlement errors.
    #[odra::odra_error]
    pub enum Error {
        /// Caller is not authorised for this action.
        Unauthorized = 40_000,
        /// Market id does not exist.
        MarketNotFound = 40_001,
        /// Market is not open for this action.
        MarketNotOpen = 40_002,
        /// Staking window has closed.
        StakingClosed = 40_003,
        /// Stake amount must be greater than zero.
        ZeroStake = 40_004,
        /// Market has not been resolved yet.
        MarketNotResolved = 40_005,
        /// Market has not been settled yet.
        MarketNotSettled = 40_006,
        /// Nothing available to claim.
        NothingToClaim = 40_007,
        /// Rake basis points out of range.
        InvalidRake = 40_008
    }
}

#[cfg(test)]
mod tests {
    use super::{
        events::{Claimed, MarketCreated, Resolved, Settled, Staked},
        ArbiterSettlement, ArbiterSettlementHostRef, ArbiterSettlementInitArgs, MarketStatus
    };
    use odra::casper_types::U512;
    use odra::host::{Deployer, HostEnv, HostRef};
    use odra::prelude::*;

    // 1 CSPR = 1e9 motes.
    const CSPR: u64 = 1_000_000_000;
    const RAKE_BPS: u32 = 200; // 2%
    const FAR_FUTURE: u64 = u64::MAX;

    // side ids for a 1X2 football market
    const HOME: u8 = 0;
    const AWAY: u8 = 2;

    struct Cast {
        env: HostEnv,
        admin: Address,
        resolver: Address,
        treasury: Address,
        alice: Address,
        bob: Address,
        carol: Address
    }

    fn setup() -> (Cast, ArbiterSettlementHostRef) {
        let env = odra_test::env();
        let admin = env.get_account(0);
        let resolver = env.get_account(1);
        let treasury = env.get_account(2);
        let alice = env.get_account(3);
        let bob = env.get_account(4);
        let carol = env.get_account(5);

        env.set_caller(admin);
        let contract = ArbiterSettlement::deploy(
            &env,
            ArbiterSettlementInitArgs {
                resolver,
                treasury,
                rake_bps: RAKE_BPS
            }
        );
        let cast = Cast {
            env: env.clone(),
            admin,
            resolver,
            treasury,
            alice,
            bob,
            carol
        };
        (cast, contract)
    }

    #[test]
    fn full_settlement_flow() {
        let (c, mut contract) = setup();

        // Admin opens a market.
        c.env.set_caller(c.admin);
        let market_id = contract.create_market(FAR_FUTURE);
        assert_eq!(market_id, 0);
        assert!(c.env.emitted_event(
            &contract,
            MarketCreated {
                market_id: 0,
                close_time: FAR_FUTURE
            }
        ));

        // Alice and Bob back HOME with 100 CSPR each; Carol backs AWAY with 200.
        let h = U512::from(100 * CSPR);
        let a = U512::from(200 * CSPR);

        c.env.set_caller(c.alice);
        contract.with_tokens(h).stake(market_id, HOME);
        c.env.set_caller(c.bob);
        contract.with_tokens(h).stake(market_id, HOME);
        c.env.set_caller(c.carol);
        contract.with_tokens(a).stake(market_id, AWAY);

        assert_eq!(contract.get_pool(market_id, HOME), U512::from(200 * CSPR));
        assert_eq!(contract.get_pool(market_id, AWAY), U512::from(200 * CSPR));
        assert_eq!(contract.get_total(market_id), U512::from(400 * CSPR));
        assert!(c.env.emitted_event(
            &contract,
            Staked {
                market_id,
                side: AWAY,
                staker: c.carol,
                amount: a
            }
        ));

        // Resolver submits HOME as the winner with an x402 proof reference.
        let proof = String::from("x402:rcpt_0x8f3ac41d");
        c.env.set_caller(c.resolver);
        contract.submit_resolution(market_id, HOME, proof.clone());
        assert_eq!(contract.get_market(market_id).status, MarketStatus::Resolved);
        assert!(c.env.emitted_event(
            &contract,
            Resolved {
                market_id,
                winning_side: HOME,
                proof_ref: proof.clone()
            }
        ));

        // Settle: rake = 2% of 400 = 8 CSPR to treasury; payout pool = 392.
        let treasury_before = c.env.balance_of(&c.treasury);
        c.env.set_caller(c.resolver);
        contract.settle(market_id);

        let market = contract.get_market(market_id);
        assert_eq!(market.status, MarketStatus::Settled);
        assert_eq!(market.total_pool, U512::from(400 * CSPR));
        assert_eq!(market.winning_pool, U512::from(200 * CSPR));
        assert_eq!(market.rake, U512::from(8 * CSPR));
        assert_eq!(market.payout_pool, U512::from(392 * CSPR));
        assert_eq!(
            c.env.balance_of(&c.treasury) - treasury_before,
            U512::from(8 * CSPR)
        );
        assert!(c.env.emitted_event(
            &contract,
            Settled {
                market_id,
                winning_side: HOME,
                total_pool: U512::from(400 * CSPR),
                payout_pool: U512::from(392 * CSPR),
                rake: U512::from(8 * CSPR),
                proof_ref: proof
            }
        ));

        // Alice claims: 100/200 * 392 = 196 CSPR.
        let alice_before = c.env.balance_of(&c.alice);
        c.env.set_caller(c.alice);
        contract.claim(market_id);
        assert_eq!(
            c.env.balance_of(&c.alice) - alice_before,
            U512::from(196 * CSPR)
        );
        assert!(c.env.emitted_event(
            &contract,
            Claimed {
                market_id,
                staker: c.alice,
                amount: U512::from(196 * CSPR)
            }
        ));

        // Bob claims the same.
        c.env.set_caller(c.bob);
        contract.claim(market_id);
        assert_eq!(
            contract.get_stake(market_id, HOME, c.bob),
            U512::zero()
        );
    }

    #[test]
    fn double_claim_reverts() {
        let (c, mut contract) = setup();
        c.env.set_caller(c.admin);
        let market_id = contract.create_market(FAR_FUTURE);

        c.env.set_caller(c.alice);
        contract.with_tokens(U512::from(100 * CSPR)).stake(market_id, HOME);

        c.env.set_caller(c.resolver);
        contract.submit_resolution(market_id, HOME, String::from("x402:rcpt"));
        contract.settle(market_id);

        c.env.set_caller(c.alice);
        contract.claim(market_id);
        // Second claim has nothing left.
        c.env.set_caller(c.alice);
        let res = contract.try_claim(market_id);
        assert!(res.is_err());
    }

    #[test]
    fn only_resolver_resolves() {
        let (c, mut contract) = setup();
        c.env.set_caller(c.admin);
        let market_id = contract.create_market(FAR_FUTURE);

        // A random account cannot resolve.
        c.env.set_caller(c.alice);
        let res = contract.try_submit_resolution(market_id, HOME, String::from("x402:rcpt"));
        assert!(res.is_err());
    }

    #[test]
    fn only_admin_creates_markets() {
        let (c, mut contract) = setup();
        c.env.set_caller(c.alice);
        let res = contract.try_create_market(FAR_FUTURE);
        assert!(res.is_err());
    }
}
