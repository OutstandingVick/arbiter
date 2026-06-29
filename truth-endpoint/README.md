# Arbiter Truth Endpoint

Local x402-style football outcome resolver for the Arbiter demo.

Run:

```bash
npm run truth
```

Flow:

1. `GET /outcomes/0` returns `402 Payment Required`.
2. The agent signs the returned payment requirement.
3. The agent resends the request with `X-PAYMENT`.
4. The endpoint returns `winning_side` and a receipt-like `proof_ref`.

This is intentionally small and dependency-free for the submission prototype.
For production, the verification step should be swapped to the unchanged
Casper x402 reference facilitator.
