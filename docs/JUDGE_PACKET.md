# Arbiter Judge Packet

This packet is written for a hackathon judge who needs to understand the
submission quickly and verify that it is more than a code sketch.

## One-line pitch

Arbiter is an autonomous settlement agent on Casper: it pays for verified truth,
uses that result to resolve a market, settles value on-chain, and publishes a
proof trail that links the paid receipt to the settlement transactions.

## Compelling description

Agentic applications need more than chat responses. They need to buy data, make
decisions, move value, and prove why the value moved. Arbiter demonstrates that
workflow on Casper with a parimutuel football market. Users stake CSPR on an
outcome, the Arbiter agent pays an x402-style truth endpoint for the verified
result, and the Casper settlement contract resolves the market, applies rake,
and lets winners claim pro-rata payouts. Every important action has a public
artifact: the x402 receipt reference, the Casper testnet deploy, the market
creation transaction, the resolution transaction, the settlement transaction,
and a local proof viewer.

## Requirement Coverage

The public DoraHacks track page was behind AWS WAF during this audit, so this
matrix is based on the visible project framing, Casper Agentic Buildathon
expectations, and the repository's own submission notes. Treat any item not
listed on the official page as a packaging risk to verify before final submit.

| Requirement / judging expectation | Coverage | Evidence | Fastest fix if challenged |
|---|---|---|---|
| Build on Casper | Clearly covered | Odra contract, WASM, Casper testnet deploy, CSPR.live links | Put the contract hash and deploy link in the DoraHacks description. |
| Produce on-chain testnet activity | Clearly covered | Deploy, `create_market`, `submit_resolution`, and `settle` tx links in `SUBMISSION.md` | Add screenshots of each CSPR.live page to `docs/screenshots/`. |
| Demonstrate agentic workflow | Clearly covered | `agent/index.mjs` performs perceive -> pay -> reason -> settle -> prove | In the demo, show the terminal logs line by line. |
| Use / align with x402 payment flow | Partially covered | HTTP 402 challenge, signed payment header, receipt id, nonce expiry, replay protection | State clearly that this is an x402-compatible demo verifier; show the swap path to the official Casper x402 facilitator. |
| Provide documentation and usage instructions | Clearly covered | `README.md`, `SUBMISSION.md`, `PRODUCTION_READINESS.md`, `docs/DEMO_SCRIPT.md` | Add a hosted demo link if time allows. |
| Public demo video | Missing until recorded | Script exists in `docs/DEMO_SCRIPT.md` | Record a 2-3 minute Loom/YouTube walkthrough using the script. |
| Public frontend / visible walkthrough | Partially covered | Local proof viewer in `web/index.html` | Host the proof viewer via GitHub Pages or include screenshots in the submission. |
| Verifiable proof artifacts | Clearly covered | `agent/proof-trail.latest.json`, CSPR.live links, proof viewer | Add terminal output screenshots and CSPR.live screenshots. |
| Production awareness / safety | Clearly covered | Pull-based claims, resolver permissions, replay protection, readiness notes | Mention the demo-scoped pieces honestly in the video. |
| Novelty / competitiveness | Clearly covered but presentation-sensitive | Application-level x402 consumer, not just infra | Lead with "agent-paid truth -> on-chain settlement -> proof" in the first 20 seconds. |

## Product Clarity

Problem: agents can talk and call APIs, but serious financial workflows need a
verifiable path from paid information to on-chain settlement. Without that path,
users cannot trust why funds moved.

Under-60-second test: the project is clear if the judge sees this sequence:

```text
stake CSPR -> agent pays for truth -> truth returns receipt -> agent resolves on Casper -> market settles -> proof viewer links everything
```

Current verdict: the project is a real workflow, not just a technical proof. It
has a deployed contract, agent loop, truth endpoint, and proof viewer. The main
weakness is presentation polish: the official x402 facilitator is not fully
wired, and the visual demo is local rather than hosted.

## Demo Strength

The demo should show visible proof at every stage:

- Terminal: `npm run truth` starts the x402-style endpoint.
- Terminal: `npm run demo:dry-run` shows the agent receiving 402, signing, and
  receiving a proof receipt.
- Browser: `npm run web` shows the proof viewer.
- Browser: CSPR.live links show deploy, market creation, resolution, settlement.
- Repo: `agent/proof-trail.latest.json` shows the receipt and tx hashes.

Avoid spending time live-deploying during the video. Use the already completed
testnet proof as the "this actually ran" evidence.

## README / Repo Quality

Strong:

- Clear architecture split between payment-for-truth and market settlement.
- Contract entrypoints and payout math are documented.
- Test, build, deploy, and demo commands exist.
- Proof transactions are public.

Weak:

- No screenshots checked into the repo.
- No hosted proof viewer link yet.
- x402 caveat must stay explicit so judges do not feel misled.
- The README is strong technically, but still dense for a tired judge.

Recommended sections now present or to keep visible:

- Judge quick path
- Live Casper testnet proof
- Current prototype honesty
- Demo flow
- Production readiness / demo-scoped pieces

## Proof Files To Create Before Submitting

Create these if there is time:

1. `docs/screenshots/proof-viewer.png` - browser screenshot of `npm run web`.
2. `docs/screenshots/agent-dry-run.png` - terminal screenshot of `npm run demo:dry-run`.
3. `docs/screenshots/cspr-deploy.png` - CSPR.live deploy transaction.
4. `docs/screenshots/cspr-resolution.png` - CSPR.live resolution transaction.
5. `docs/screenshots/cspr-settlement.png` - CSPR.live settlement transaction.

Fallback if external explorers fail during judging:

- `agent/proof-trail.latest.json`
- `SUBMISSION.md`
- `web/index.html`
- `contracts/wasm/ArbiterSettlement.wasm`
- local `npm run check`

## Competitiveness

Against a likely first-place submission, Arbiter's strongest asset is that it
closes a full economic loop. It is not just an agent chat UI or another payment
SDK. The weaker points are polish: no hosted UI, no submitted video yet, no
screenshots, and the x402 facilitator integration is demo-compatible rather
than final.

Top 5 highest-impact changes before submission:

1. Record the 2-3 minute demo video from `docs/DEMO_SCRIPT.md`.
2. Host the proof viewer or include screenshots in DoraHacks.
3. Put the four CSPR.live transaction links directly in the DoraHacks
   submission body.
4. Open the video with the product story, not the repo.
5. State the x402 demo scope honestly, then emphasize the production swap path.

