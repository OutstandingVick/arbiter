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

## Screenshots To Capture

Create these before submitting the final DoraHacks form:

- `docs/screenshots/proof-viewer.png`
- `docs/screenshots/agent-dry-run.png`
- `docs/screenshots/cspr-deploy.png`
- `docs/screenshots/cspr-resolution.png`
- `docs/screenshots/cspr-settlement.png`

