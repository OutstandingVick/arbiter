import { execFile } from "node:child_process";
import { access, readFile } from "node:fs/promises";
import { constants } from "node:fs";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const root = new URL("../", import.meta.url);

const sourceFiles = [
  "truth-endpoint/server.mjs",
  "agent/index.mjs",
  "web/server.mjs",
  "scripts/check.mjs"
];

async function checkSyntax(file) {
  await execFileAsync("node", ["--check", fileURLToPath(new URL(file, root))]);
  console.log(`ok syntax ${file}`);
}

async function checkExists(file) {
  await access(new URL(file, root), constants.R_OK);
  console.log(`ok exists ${file}`);
}

function assertIncludes(name, content, value) {
  if (!content.includes(value)) {
    throw new Error(`${name} is missing ${value}`);
  }
}

for (const file of sourceFiles) {
  await checkSyntax(file);
}

for (const file of [
  "contracts/wasm/ArbiterSettlement.wasm",
  "agent/proof-trail.latest.json",
  "SUBMISSION.md",
  "docs/JUDGE_PACKET.md",
  "docs/DEMO_SCRIPT.md",
  "docs/EVIDENCE.md",
  "docs/assets/architecture.svg",
  "docs/screenshots/proof-viewer.png",
  "docs/screenshots/agent-dry-run.png",
  "docs/screenshots/cspr-deploy.png",
  "docs/screenshots/cspr-resolution.png",
  "docs/screenshots/cspr-settlement.png",
  "web/index.html",
  "web/assets/agent-dry-run.png",
  "web/assets/cspr-resolution.png"
]) {
  await checkExists(file);
}

const submission = await readFile(new URL("SUBMISSION.md", root), "utf8");
const readme = await readFile(new URL("README.md", root), "utf8");
const viewer = await readFile(new URL("web/index.html", root), "utf8");
for (const value of [
  "73510503b81826ef4d2a78a9068888acc453a971c2eb325f493289816bf81a48",
  "b50fe93cc706fab4ee5fd0c64884e524322c7b803d2eb4aca089e261b0a7a0b6",
  "1bfb353ffd9440f2e06fdbc86639c15804d3db1865c2f3f7db6ea7eb7b55443c",
  "3b75366a3d15e20b069e4b1df39450f03ddf38a6d9829c4930ba416f37d6a83c",
  "x402:rcpt_35bb5efd4a6948d5"
]) {
  assertIncludes("SUBMISSION.md", submission, value);
  assertIncludes("README.md", readme, value);
  assertIncludes("web/index.html", viewer, value);
}

console.log("ok proof references");
console.log("production-readiness checks passed");
