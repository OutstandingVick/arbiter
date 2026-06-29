# Production Readiness Notes

This document separates what is production-grade in the current Arbiter
prototype from what is intentionally demo-scoped for the buildathon submission.

## Verified

- Odra native VM tests pass.
- Casper execution-engine backend tests pass.
- Optimized `ArbiterSettlement.wasm` builds successfully.
- Contract is deployed on Casper testnet.
- Market `0` was created on-chain.
- Agent paid the local x402-style truth endpoint, received a proof reference,
  submitted that proof reference on-chain, and settled market `0`.
- Proof viewer serves the deploy, market, resolution, settlement, and proof-ref
  trail.

## Production-Ready Parts

- Settlement contract architecture:
  - Pull-based winner claims.
  - Resolver-only resolution and settlement.
  - Admin-only market creation.
  - Rake capped at 10%.
  - No unbounded winner payout loops.
- Casper testnet deployment proof.
- On-chain proof reference path through `submit_resolution`.

## Demo-Scoped Parts

- The truth endpoint uses an x402-compatible HTTP 402 shape, but verifies a local
  HMAC payment header instead of the unchanged Casper x402 facilitator.
- Outcome data is fixed for market `0` to make the demo deterministic.
- Resolver, treasury, and deploy admin currently use the same testnet key.
- The proof viewer is a static local viewer, not an indexed production explorer.

## Hardening Added

- One-time payment challenges with nonce and expiry.
- Payment timestamp freshness checks.
- Payment replay detection.
- Agent-side validation of market id, winning side, and `proof_ref` format before
  any Casper transaction is submitted.
- Static viewer path traversal protection.
- Repo-level `npm run check` script for syntax and proof-reference checks.

## Before Mainnet

- Replace the demo HMAC verifier with the official Casper x402 facilitator flow.
- Use separate keys for deploy admin, resolver agent, and treasury.
- Add resolver key rotation runbook and emergency pause/operational policy.
- Add indexed event ingestion for proof viewer.
- Add multiple real-world oracle sources or an oracle provider SLA.
- Add load and failure-mode tests for endpoint and agent retries.
- Add CI for contract tests, schema generation, JS checks, and wasm build.

