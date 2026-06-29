import { createHmac } from "node:crypto";
import { execFile } from "node:child_process";
import { writeFile } from "node:fs/promises";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

const config = {
  truthEndpoint: process.env.TRUTH_ENDPOINT_URL ?? "http://localhost:4020",
  x402Secret: process.env.X402_DEMO_SECRET ?? "arbiter-local-demo-secret",
  marketId: Number(process.env.ARBITER_MARKET_ID ?? "0"),
  nodeAddress: process.env.CASPER_NODE_ADDRESS ?? "https://node.testnet.casper.network",
  chainName: process.env.CASPER_CHAIN_NAME ?? "casper-test",
  secretKey: process.env.CASPER_SECRET_KEY ?? "/Users/macbook/.casper-client/secret_key.pem",
  publicKey: process.env.CASPER_PUBLIC_KEY ?? "/Users/macbook/.casper-client/public_key_hex",
  contractHash:
    process.env.ARBITER_CONTRACT_HASH ??
    "hash-11ee76f57a0c9c119e340329b3b507619bd07948b4e488dafdc250a3ce7203f6",
  dryRun: process.env.ARBITER_DRY_RUN === "1"
};

const proofRefPattern = /^x402:rcpt_[a-f0-9]{16}$/;

async function readPublicKey() {
  const { stdout } = await execFileAsync("cat", [config.publicKey]);
  return stdout.trim();
}

function signPayment(payload) {
  return createHmac("sha256", config.x402Secret).update(JSON.stringify(payload)).digest("hex");
}

async function payForOutcome(publicKey) {
  const outcomeUrl = `${config.truthEndpoint}/outcomes/${config.marketId}`;
  const challenge = await fetch(outcomeUrl);
  if (challenge.status !== 402) {
    throw new Error(`Expected 402 payment challenge, got HTTP ${challenge.status}`);
  }

  const challengeBody = await challenge.json();
  const requirement = challengeBody.accepts?.[0];
  if (!requirement) {
    throw new Error("Truth endpoint did not return x402 requirements");
  }

  const payment = {
    scheme: requirement.scheme,
    network: requirement.network,
    amount: requirement.amount,
    payer: publicKey,
    pay_to: requirement.pay_to,
    resource: requirement.resource,
    nonce: requirement.nonce,
    expires_at: requirement.expires_at,
    timestamp: new Date().toISOString()
  };
  const signedPayment = {
    ...payment,
    signature: signPayment(payment)
  };
  const encodedPayment = Buffer.from(JSON.stringify(signedPayment)).toString("base64url");

  const paid = await fetch(outcomeUrl, {
    headers: {
      "x-payment": encodedPayment
    }
  });
  if (!paid.ok) {
    throw new Error(`Truth endpoint rejected payment: HTTP ${paid.status} ${await paid.text()}`);
  }

  return paid.json();
}

function validateOutcome(outcome) {
  if (!Number.isInteger(outcome.market_id) || outcome.market_id !== config.marketId) {
    throw new Error(`Unexpected market id from truth endpoint: ${outcome.market_id}`);
  }
  if (!Number.isInteger(outcome.winning_side) || outcome.winning_side < 0 || outcome.winning_side > 255) {
    throw new Error(`Invalid winning side from truth endpoint: ${outcome.winning_side}`);
  }
  if (!proofRefPattern.test(outcome.proof_ref ?? "")) {
    throw new Error(`Invalid proof_ref from truth endpoint: ${outcome.proof_ref}`);
  }
}

async function casperTransaction(args) {
  const { stdout, stderr } = await execFileAsync("casper-client", args, {
    maxBuffer: 1024 * 1024 * 8
  });
  if (stderr.trim()) {
    console.error(stderr.trim());
  }
  return JSON.parse(stdout);
}

function baseCall(entryPoint, paymentAmount, sessionArgs) {
  return [
    "put-transaction",
    "invocable-entity",
    "--node-address",
    config.nodeAddress,
    "--chain-name",
    config.chainName,
    "--gas-price-tolerance",
    "1",
    "--secret-key",
    config.secretKey,
    "--contract-hash",
    config.contractHash,
    "--session-entry-point",
    entryPoint,
    "--payment-amount",
    paymentAmount,
    "--standard-payment",
    "true",
    ...sessionArgs.flatMap((arg) => ["--session-arg", arg])
  ];
}

async function submitResolution(outcome) {
  const args = baseCall("submit_resolution", "20000000000", [
    `market_id:u64='${config.marketId}'`,
    `winning_side:u8='${outcome.winning_side}'`,
    `proof_ref:string='${outcome.proof_ref}'`
  ]);
  return casperTransaction(args);
}

async function settleMarket() {
  const args = baseCall("settle", "20000000000", [`market_id:u64='${config.marketId}'`]);
  return casperTransaction(args);
}

function txHash(result) {
  return result?.result?.transaction_hash?.Version1 ?? result?.result?.transaction_hash;
}

async function waitForTransaction(hash, label) {
  for (let attempt = 1; attempt <= 20; attempt += 1) {
    const result = await casperTransaction([
      "get-transaction",
      "--node-address",
      config.nodeAddress,
      hash
    ]);
    const execution = result?.result?.execution_info?.execution_result?.Version2;
    if (execution) {
      if (execution.error_message) {
        throw new Error(`${label} failed: ${execution.error_message}`);
      }
      console.log(`${label} executed in block ${result.result.execution_info.block_height}`);
      return result;
    }
    await new Promise((resolve) => setTimeout(resolve, 5000));
  }
  throw new Error(`${label} was submitted but did not execute before timeout: ${hash}`);
}

async function main() {
  console.log("Arbiter agent starting");
  console.log(`market_id=${config.marketId}`);
  console.log(`truth_endpoint=${config.truthEndpoint}`);
  console.log(`dry_run=${config.dryRun}`);

  const publicKey = await readPublicKey();
  const outcome = await payForOutcome(publicKey);
  validateOutcome(outcome);
  console.log("Paid truth endpoint over x402-style flow");
  console.log(`outcome=${outcome.result} winning_side=${outcome.winning_side}`);
  console.log(`proof_ref=${outcome.proof_ref}`);

  if (config.dryRun) {
    console.log("Dry run enabled; skipping Casper submit_resolution and settle.");
    console.log(JSON.stringify({ outcome }, null, 2));
    return;
  }

  const resolutionTx = await submitResolution(outcome);
  const resolutionHash = txHash(resolutionTx);
  console.log(`submit_resolution tx=${resolutionHash}`);
  await waitForTransaction(resolutionHash, "submit_resolution");

  const settleTx = await settleMarket();
  const settleHash = txHash(settleTx);
  console.log(`settle tx=${settleHash}`);
  await waitForTransaction(settleHash, "settle");

  const proofTrail = {
    market_id: config.marketId,
    proof_ref: outcome.proof_ref,
    receipt: outcome.receipt,
    submit_resolution_tx: resolutionHash,
    settle_tx: settleHash
  };
  await writeFile(new URL("./proof-trail.latest.json", import.meta.url), JSON.stringify(proofTrail, null, 2));
  console.log(JSON.stringify(proofTrail, null, 2));
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
