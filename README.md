# Arbiter

**Autonomous settlement for the agent economy, on Casper.**

Arbiter is an autonomous agent that pays for a verified real-world outcome over
**x402**, settles the resulting obligation **on-chain**, and leaves a
**cryptographic proof trail** linking the payout to the receipt it paid for.

The live vertical is a **parimutuel football market**: stakers back an outcome,
the agent pays an x402 endpoint for the verified result, and the market settles
and pays out — with the x402 receipt referenced on-chain. The same engine
generalises to cross-border (corridor) settlement as a second adapter.

Built for the **Casper Agentic Buildathon 2026** (Casper Innovation Track).

---

## Why this, not another x402 primitive

The buildathon field is crowded with x402 *infrastructure* — receipt
primitives, gateways, kits, credit scores, spending guards. Arbiter is not
another primitive. It is the **working settlement application** that consumes
those primitives: real funds escrowed, a real outcome paid for, a real payout
made. It does something a user feels, and it closes the loop Casper's AI Toolkit
describes — one agent pays a service, acts on the result, and proves it.

---

## Architecture — two on-chain surfaces

Keep these separate; conflating them is what makes the build hard.

**Surface A — paying for truth (x402).** The agent calls the truth endpoint,
receives `402`, signs, and pays. The Casper x402 Facilitator settles this in the
CEP-18 x402 token. *We run the reference facilitator unchanged and fork the
reference resource server into our truth endpoint.* (`make-software/casper-x402`,
JS SDK under `js/`.)

**Surface B — settling the obligation (this contract).** The `ArbiterSettlement`
Odra contract escrows native CSPR, accepts the resolved outcome + an x402 proof
reference from the agent, takes a rake, and pays winners pro-rata. This is the
only bespoke contract.

```
        ARBITER AGENT  (perceive → pay x402 → reason → settle → prove)
              │ pays for outcome (Surface A)        │ submits resolution + settles (Surface B)
              ▼                                      ▼
     TRUTH ENDPOINT (x402 server)          ArbiterSettlement (Odra, this repo)
              │                                      │
   x402 FACILITATOR (reference, CEP-18)     CASPER TESTNET + CSPR.cloud events
```

---

## Repo layout

```
arbiter/
├── contracts/                  # Surface B — Odra settlement contract (Rust)  [DONE]
│   ├── src/arbiter_settlement.rs   # the contract + tests
│   ├── src/lib.rs
│   ├── Cargo.toml · Odra.toml · build.rs · rust-toolchain
│   └── bin/                        # wasm + schema build entrypoints
├── truth-endpoint/             # Surface A — x402-style outcome resolver       [DONE]
├── agent/                      # the Arbiter agent loop                        [DONE]
└── web/                        # demo proof viewer                             [DONE]
```

The contract is the spine and it is deployed on Casper testnet. The
truth-endpoint and agent are implemented as a small dependency-free demo loop:
the endpoint issues an x402-style HTTP 402 challenge, the agent pays it, receives
a proof reference, then calls `submit_resolution` and `settle` on Casper.

---

## The settlement contract

`ArbiterSettlement` — parimutuel, native-CSPR settlement.

**Entrypoints**

| Fn | Who | Effect |
|---|---|---|
| `init(resolver, treasury, rake_bps)` | deployer | sets the agent (resolver), treasury, and rake (bps, ≤1000) |
| `create_market(close_time) -> u64` | admin | opens a market, returns its id |
| `stake(market_id, side)` *(payable)* | anyone | escrows attached CSPR on a side, before `close_time` |
| `submit_resolution(market_id, winning_side, proof_ref)` | resolver | records the outcome + the x402 receipt reference |
| `settle(market_id)` | resolver | snapshots pools, sends rake to treasury, opens claims |
| `claim(market_id)` | winners | pulls pro-rata share of the post-rake pool |
| `set_resolver(resolver)` | admin | rotates the agent key |
| `get_market / get_pool / get_stake / get_total / get_resolver / market_count` | view | reads |

**Payout math (parimutuel).** Winners split the entire pool minus rake, pro-rata
to their winning-side stake:

```
rake        = total_pool * rake_bps / 10_000
payout_pool = total_pool - rake
your_payout = your_stake * payout_pool / winning_pool
```

If nobody backed the winning side, the whole pool routes to the treasury.

**Proof linkage.** `submit_resolution` stores `proof_ref` (the x402 receipt id)
in market state and re-emits it in the `Settled` event — this is the on-chain
end of the "prove" step that ties the payout to the truth the agent paid for.

**Design notes / hardening for the Final Round.** Settlement is pull-based
(`claim`) to avoid unbounded loops — production-safe by construction. The
resolver is a single provisioned key today (account abstraction is not yet
shipped on Casper); rotate via `set_resolver`. For the corridor vertical, swap
native-CSPR escrow for a CEP-18 settlement token via an `External<Erc20...>`
reference.

**Events:** `MarketCreated, Staked, Resolved, Settled, Claimed`.
**Errors:** `40_000–40_008` (Unauthorized, MarketNotFound, MarketNotOpen,
StakingClosed, ZeroStake, MarketNotResolved, MarketNotSettled, NothingToClaim,
InvalidRake).

---

## Build, test, deploy

Requires the Casper/Odra toolchain (the pinned nightly installs automatically
from `rust-toolchain`).

```bash
# one-time
cargo install cargo-odra --locked
rustup target add wasm32-unknown-unknown

cd contracts

# run the test suite (fast native VM)
cargo odra test

# run against the Casper execution engine backend
cargo odra test -b casper

# build the deployable wasm
cargo odra build
# -> wasm/ArbiterSettlement.wasm
```

Run the prototype checks:

```bash
npm run check
```

Run the local demo services:

```bash
npm run truth
npm run demo:dry-run
npm run web
```

The included tests cover the full happy path (stake → resolve → settle → claim,
with rake and pro-rata math asserted to the mote), double-claim rejection, and
the resolver/admin access checks.

**Deploy to Testnet:** build the wasm above, then deploy with `casper-client
put-deploy` (or Odra's livenet flow) against `https://node.testnet.casper.network`,
passing the runtime args `resolver`, `treasury`, `rake_bps`. See the Odra
livenet docs for the deployment harness.

---

## Status & timeline

The buildathon is two-phase. **Qualification closes June 30**; to advance on the
merit path you need a working Testnet prototype that produces an on-chain
transaction — this contract is that component. The **Final Round (July 6–19)** is
the jury contest, where the truth-endpoint, agent loop, demo video, brand, and
launch narrative get hardened.

- [x] Surface B — settlement contract + tests (this repo)
- [x] Surface A — x402-style truth endpoint
- [x] Arbiter agent loop — perceive → pay → reason → settle → prove
- [x] Testnet deployment + on-chain app interactions
- [x] Demo proof viewer + submission writeup
- [ ] Short demo recording
- [ ] Swap demo x402 verification for the unchanged Casper x402 facilitator
