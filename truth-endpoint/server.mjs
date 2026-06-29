import { createHmac, randomUUID, timingSafeEqual } from "node:crypto";
import { createServer } from "node:http";

const port = Number(process.env.PORT ?? 4020);
const secret = process.env.X402_DEMO_SECRET ?? "arbiter-local-demo-secret";
const payTo =
  process.env.X402_PAY_TO ??
  "account-hash-250c23eccc0d8765b8485fcc6812041d9c86de5191e4fd64ffdca33d2abe2fec";
const priceMotes = process.env.X402_PRICE_MOTES ?? "1000000";
const network = process.env.X402_NETWORK ?? "casper-test";
const challengeTtlMs = Number(process.env.X402_CHALLENGE_TTL_MS ?? 120_000);
const maxPaymentAgeMs = Number(process.env.X402_PAYMENT_MAX_AGE_MS ?? 120_000);

const outcomes = new Map([
  [
    "0",
    {
      market_id: 0,
      winning_side: 0,
      fixture: "Arbiter Demo FC vs Proof United",
      result: "HOME",
      source: "demo-oracle",
      resolved_at: "2026-06-27T15:35:21.000Z"
    }
  ]
]);

const receipts = new Map();
const challenges = new Map();
const usedSignatures = new Set();

function json(res, status, body, extraHeaders = {}) {
  res.writeHead(status, {
    "content-type": "application/json; charset=utf-8",
    "access-control-allow-origin": "*",
    "access-control-allow-methods": "GET,POST,OPTIONS",
    "access-control-allow-headers": "content-type,x-payment,x-402-payment",
    ...extraHeaders
  });
  res.end(JSON.stringify(body, null, 2));
}

function signPayment(payload) {
  return createHmac("sha256", secret).update(JSON.stringify(payload)).digest("hex");
}

function createChallenge(resource) {
  const nonce = randomUUID();
  const expiresAt = new Date(Date.now() + challengeTtlMs).toISOString();
  challenges.set(nonce, { resource, expiresAt });
  return { nonce, expires_at: expiresAt };
}

function verifyPayment(encoded, expectedResource) {
  let payment;
  try {
    payment = JSON.parse(Buffer.from(encoded, "base64url").toString("utf8"));
  } catch {
    return { ok: false, error: "Malformed x402 payment header" };
  }

  const { signature, ...payload } = payment;
  if (!signature) {
    return { ok: false, error: "Missing payment signature" };
  }
  if (payload.resource !== expectedResource) {
    return { ok: false, error: "Payment resource mismatch" };
  }
  if (!payload.nonce || typeof payload.nonce !== "string") {
    return { ok: false, error: "Missing payment nonce" };
  }
  const challenge = challenges.get(payload.nonce);
  if (!challenge) {
    return { ok: false, error: "Unknown or already-used payment nonce" };
  }
  if (challenge.resource !== expectedResource) {
    return { ok: false, error: "Payment nonce resource mismatch" };
  }
  if (Date.parse(challenge.expiresAt) < Date.now()) {
    challenges.delete(payload.nonce);
    return { ok: false, error: "Payment challenge expired" };
  }
  const paymentAge = Math.abs(Date.now() - Date.parse(payload.timestamp ?? ""));
  if (!Number.isFinite(paymentAge) || paymentAge > maxPaymentAgeMs) {
    return { ok: false, error: "Payment timestamp is outside the accepted window" };
  }
  if (payload.amount !== priceMotes) {
    return { ok: false, error: "Payment amount mismatch" };
  }
  if (payload.network !== network) {
    return { ok: false, error: "Payment network mismatch" };
  }

  const expected = signPayment(payload);
  const given = Buffer.from(signature, "hex");
  const wanted = Buffer.from(expected, "hex");
  if (given.length !== wanted.length || !timingSafeEqual(given, wanted)) {
    return { ok: false, error: "Invalid payment signature" };
  }
  if (usedSignatures.has(signature)) {
    return { ok: false, error: "Payment replay detected" };
  }
  usedSignatures.add(signature);
  challenges.delete(payload.nonce);

  const receipt = {
    id: `x402:rcpt_${randomUUID().replaceAll("-", "").slice(0, 16)}`,
    scheme: "x402-demo",
    network,
    amount: payload.amount,
    payer: payload.payer,
    pay_to: payTo,
    resource: payload.resource,
    payment_hash: createHmac("sha256", secret)
      .update(`${payload.payer}:${payload.resource}:${payload.timestamp}`)
      .digest("hex"),
    created_at: new Date().toISOString()
  };
  receipts.set(receipt.id, receipt);
  return { ok: true, payment: payload, receipt };
}

function paymentRequired(res, resource) {
  const challenge = createChallenge(resource);
  return json(
    res,
    402,
    {
      error: "Payment required",
      accepts: [
        {
          scheme: "x402-demo",
          network,
          amount: priceMotes,
          asset: "motes",
          pay_to: payTo,
          resource,
          ...challenge,
          facilitator: "reference-facilitator-compatible-demo",
          instructions:
            "Sign the payment payload and resend it as base64url JSON in the X-PAYMENT header."
        }
      ]
    },
    {
      "x-accepts-x402": "x402-demo",
      "x-payment-amount": priceMotes
    }
  );
}

const server = createServer(async (req, res) => {
  const url = new URL(req.url ?? "/", `http://${req.headers.host ?? "localhost"}`);

  if (req.method === "OPTIONS") {
    return json(res, 204, {});
  }

  if (req.method === "GET" && url.pathname === "/health") {
    return json(res, 200, { ok: true, service: "arbiter-truth-endpoint" });
  }

  if (req.method === "GET" && url.pathname.startsWith("/receipts/")) {
    const id = decodeURIComponent(url.pathname.split("/").slice(2).join("/"));
    const receipt = receipts.get(id);
    return receipt ? json(res, 200, receipt) : json(res, 404, { error: "Receipt not found" });
  }

  if (req.method === "GET" && url.pathname.startsWith("/outcomes/")) {
    const marketId = url.pathname.split("/").at(-1);
    const outcome = outcomes.get(marketId);
    if (!outcome) {
      return json(res, 404, { error: "Outcome not found", market_id: Number(marketId) });
    }

    const resource = `${url.origin}/outcomes/${marketId}`;
    const paymentHeader = req.headers["x-payment"] ?? req.headers["x-402-payment"];
    if (!paymentHeader || Array.isArray(paymentHeader)) {
      return paymentRequired(res, resource);
    }

    const verification = verifyPayment(paymentHeader, resource);
    if (!verification.ok) {
      return json(res, 402, { error: verification.error });
    }

    return json(res, 200, {
      ...outcome,
      proof_ref: verification.receipt.id,
      receipt: verification.receipt
    });
  }

  return json(res, 404, { error: "Not found" });
});

server.listen(port, () => {
  console.log(`Arbiter truth endpoint listening on http://localhost:${port}`);
  console.log(`x402 demo price: ${priceMotes} motes on ${network}`);
});
