# Arbiter Demo Video Script

Target length: 2-3 minutes.

## 0:00-0:20 - Hook

"Hi, I am Victor, and this is Arbiter: autonomous settlement for the agent
economy on Casper.

Most agents can call APIs, but financial agents need more than data. They need
to pay for verified truth, act on it, settle value, and prove why the value
moved. Arbiter demonstrates that full loop."

Show: README title and one-line pitch.

## 0:20-0:45 - Product Workflow

"The demo workflow is a parimutuel football market. Users stake CSPR on an
outcome. When the market closes, the Arbiter agent asks a truth endpoint for the
result. The endpoint returns an x402-style 402 payment challenge. The agent pays,
receives a receipt reference, submits the result to the Casper contract, and
settles the market."

Show: architecture section or proof viewer flow cards.

## 0:45-1:10 - Contract And Casper Proof

"The settlement contract is written with Odra. It supports market creation,
staking, resolver-only outcome submission, settlement, and pull-based claims.
Pull-based claims avoid unbounded payout loops, which makes the settlement model
safer for production."

Show: `contracts/src/arbiter_settlement.rs`, then `SUBMISSION.md`.

"This is deployed on Casper testnet. Here are the public proof transactions:
contract deploy, market creation, agent resolution, and settlement."

Show: CSPR.live links in `SUBMISSION.md` or browser tabs.

## 1:10-1:45 - Agent Run

"Now I will show the agent path. First the truth endpoint starts locally."

Show terminal:

```bash
npm run truth
```

"Then the agent runs the demo path. It receives the 402 challenge, signs the
payment payload, gets the verified outcome, validates the proof reference, and
prints the result. The already completed live run used the same path and then
called `submit_resolution` and `settle` on Casper testnet."

Show terminal:

```bash
npm run demo:dry-run
```

Point out these log lines:

- `Paid truth endpoint over x402-style flow`
- `outcome=...`
- `proof_ref=x402:...`

## 1:45-2:15 - Proof Viewer

"Finally, here is the proof viewer."

Show terminal:

```bash
npm run web
```

Open the local viewer.

"This page ties the full run together: network, market, deploy transaction,
market creation transaction, resolution transaction, settlement transaction,
contract hash, and the x402 proof reference."

Show: proof viewer and click one CSPR.live transaction.

## 2:15-2:45 - Why It Wins

"The important part is that Arbiter is not another x402 primitive. It is the
application layer that consumes paid truth and turns it into a real settlement
workflow. For this hackathon the vertical is a football market, but the same
pattern applies to insurance triggers, data marketplaces, prediction markets,
and cross-border settlement."

## 2:45-3:00 - Close

"That is Arbiter: an agent pays for truth, settles on Casper, and leaves a
verifiable proof trail."

End on: proof viewer or CSPR.live settlement transaction.

