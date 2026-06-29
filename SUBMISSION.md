# Arbiter — Casper Agentic Buildathon Submission

## Summary

Arbiter is an autonomous settlement application for the agent economy.

The prototype demonstrates a Casper-native settlement contract for a parimutuel
football market. Stakers back an outcome, an authorized resolver agent submits
the verified result with a proof reference, and the contract settles the pool
on-chain with pull-based winner claims.

The broader product architecture connects this settlement layer to x402:
the Arbiter agent pays an x402-gated truth endpoint for a verified real-world
outcome, then records the resulting proof reference on Casper while settling the
market obligation.

## Why It Fits The Buildathon

Many x402 projects focus on infrastructure primitives. Arbiter is the
application layer that consumes those primitives: an agent pays for truth,
acts on the result, and leaves an auditable on-chain settlement trail.

For the Casper qualification prototype, the core settlement layer is deployed
and usable on Casper testnet.

## Testnet Deployment

Network: Casper Testnet (`casper-test`)

Deploy account:

```text
account-hash-250c23eccc0d8765b8485fcc6812041d9c86de5191e4fd64ffdca33d2abe2fec
```

Contract package hash:

```text
hash-ff54178fd93b4080c7bbda2acc190db0427f70e38a6cdb6db72304ea6f255eca
```

Contract hash:

```text
contract-11ee76f57a0c9c119e340329b3b507619bd07948b4e488dafdc250a3ce7203f6
```

## Proof Transactions

Contract deploy transaction:

```text
73510503b81826ef4d2a78a9068888acc453a971c2eb325f493289816bf81a48
```

CSPR.live:

```text
https://testnet.cspr.live/transaction/73510503b81826ef4d2a78a9068888acc453a971c2eb325f493289816bf81a48
```

App-level transaction: `create_market(close_time)`

```text
b50fe93cc706fab4ee5fd0c64884e524322c7b803d2eb4aca089e261b0a7a0b6
```

CSPR.live:

```text
https://testnet.cspr.live/transaction/b50fe93cc706fab4ee5fd0c64884e524322c7b803d2eb4aca089e261b0a7a0b6
```

Created market:

```text
market_id: 0
close_time: 2026-07-04 15:35:21 UTC
```

Agent `submit_resolution(market_id, winning_side, proof_ref)` transaction:

```text
1bfb353ffd9440f2e06fdbc86639c15804d3db1865c2f3f7db6ea7eb7b55443c
```

CSPR.live:

```text
https://testnet.cspr.live/transaction/1bfb353ffd9440f2e06fdbc86639c15804d3db1865c2f3f7db6ea7eb7b55443c
```

Agent `settle(market_id)` transaction:

```text
3b75366a3d15e20b069e4b1df39450f03ddf38a6d9829c4930ba416f37d6a83c
```

CSPR.live:

```text
https://testnet.cspr.live/transaction/3b75366a3d15e20b069e4b1df39450f03ddf38a6d9829c4930ba416f37d6a83c
```

x402 proof reference stored with the resolution:

```text
x402:rcpt_35bb5efd4a6948d5
```

## Contract Capabilities

- `init(resolver, treasury, rake_bps)`
- `create_market(close_time)`
- `stake(market_id, side)` payable in native CSPR
- `submit_resolution(market_id, winning_side, proof_ref)`
- `settle(market_id)`
- `claim(market_id)`
- `set_resolver(resolver)`
- Read views for market, pool, stake, total, resolver, and market count

## Settlement Model

Arbiter uses parimutuel settlement:

```text
rake        = total_pool * rake_bps / 10_000
payout_pool = total_pool - rake
your_payout = your_stake * payout_pool / winning_pool
```

The deployed prototype uses a 2% rake:

```text
rake_bps = 200
```

Settlement is pull-based through `claim`, avoiding unbounded payout loops.

## Verification

Local tests passed before deployment:

```text
cargo odra test
cargo odra test -b casper
cargo odra build
```

The test suite covers:

- Full stake, resolve, settle, and claim flow
- 2% rake and pro-rata payout math
- Double-claim rejection
- Resolver-only resolution
- Admin-only market creation

## Current Status

Completed:

- Casper/Odra settlement contract
- Native OdraVM tests
- Casper backend tests
- Optimized WASM build
- Casper testnet deployment
- App-level `create_market` transaction on testnet
- x402-style truth endpoint
- Arbiter agent loop that pays the truth endpoint, submits resolution, and settles market `0`
- Static proof viewer
- Proof trail saved at `agent/proof-trail.latest.json`

Next:

- Short demo video
- Swap the demo x402 verifier for the unchanged Casper x402 reference facilitator
