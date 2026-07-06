# Evidence And Verification

Use this file as the final submission evidence checklist.

## Public Links

| Artifact | Link |
|---|---|
| Repository | `https://github.com/OutstandingVick/arbiter` |
| Deploy transaction | https://testnet.cspr.live/transaction/73510503b81826ef4d2a78a9068888acc453a971c2eb325f493289816bf81a48 |
| Market creation transaction | https://testnet.cspr.live/transaction/b50fe93cc706fab4ee5fd0c64884e524322c7b803d2eb4aca089e261b0a7a0b6 |
| Resolution transaction | https://testnet.cspr.live/transaction/1bfb353ffd9440f2e06fdbc86639c15804d3db1865c2f3f7db6ea7eb7b55443c |
| Settlement transaction | https://testnet.cspr.live/transaction/3b75366a3d15e20b069e4b1df39450f03ddf38a6d9829c4930ba416f37d6a83c |
| Contract | https://testnet.cspr.live/contract/11ee76f57a0c9c119e340329b3b507619bd07948b4e488dafdc250a3ce7203f6 |

## Local Artifacts

| Artifact | Path |
|---|---|
| Proof trail JSON | `agent/proof-trail.latest.json` |
| Submission writeup | `SUBMISSION.md` |
| Production readiness notes | `PRODUCTION_READINESS.md` |
| Proof viewer | `web/index.html` |
| Deployable WASM | `contracts/wasm/ArbiterSettlement.wasm` |
| Contract source | `contracts/src/arbiter_settlement.rs` |

## Verification Commands

```bash
npm run check
```

```bash
cd contracts
cargo odra test
cargo odra test -b casper
cargo odra build
```

## Demo Commands

Terminal 1:

```bash
npm run truth
```

Terminal 2:

```bash
npm run demo:dry-run
```

Terminal 3:

```bash
npm run web
```

Then open the proof viewer and CSPR.live links.

## Screenshots

These are included for the final DoraHacks form:

| Screenshot | Notes |
|---|---|
| `docs/screenshots/proof-viewer.png` | Local proof viewer with deploy, market, resolution, settlement, contract hash, and proof ref. |
| `docs/screenshots/agent-dry-run.png` | Terminal-style dry run showing 402 payment, receipt, outcome, and proof ref. |
| `docs/screenshots/cspr-deploy.png` | Deploy evidence card with the public CSPR.live URL, deploy tx, package hash, and contract hash. |
| `docs/screenshots/cspr-resolution.png` | Direct CSPR.live capture of successful `submit_resolution`. |
| `docs/screenshots/cspr-settlement.png` | Direct CSPR.live capture of successful `settle`. |

The CSPR.live deploy transaction route intermittently renders a loading shell in
headless Chrome. The deploy evidence card preserves the exact public hash and
URL, while the resolution and settlement screenshots are direct explorer
captures.
