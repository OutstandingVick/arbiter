# Arbiter Agent

The agent performs the core loop:

```text
perceive -> pay x402 -> reason -> submit_resolution -> settle -> prove
```

Dry run:

```bash
npm run demo:dry-run
```

Live Casper call:

```bash
npm run agent
```

The live call uses the deployed Casper testnet contract and the key in
`/Users/macbook/.casper-client/secret_key.pem`.

