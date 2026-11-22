// Builds market_agent with a build number (unix seconds), signs "hf-market-agent:<build>:<sha256>" with the
// private key in ignored/market/update_signing_key.pem (only here; the agent has the public key built in, so
// neither the relay nor its admin token can publish a build) and puts it as the agents' update (PUT /update).
// Agents take a newer build at their next start or round. pnpm relay-publish
import { readFileSync } from "node:fs";
import { execSync } from "node:child_process";
import { createHash, createPrivateKey, sign } from "node:crypto";

const root = new URL("../../", import.meta.url);
const file = (p) => readFileSync(new URL(p, root));
const build = Math.floor(Date.now() / 1000);
execSync("cargo build --release -p hf-market --bin market_agent", {
  cwd: root,
  stdio: "inherit",
  env: { ...process.env, MARKET_BUILD: String(build) },
});
const cfg = JSON.parse(file("ignored/market/relay_admin.json"));
const exe = file("target/release/market_agent.exe");
const sha256 = createHash("sha256").update(exe).digest("hex");
const key = createPrivateKey(file("ignored/market/update_signing_key.pem"));
const sig = sign(null, Buffer.from(`hf-market-agent:${build}:${sha256}`), key).toString("hex");
const r = await fetch(`${cfg.url}/update`, {
  method: "PUT",
  headers: { authorization: `Bearer ${cfg.token}`, "x-build": String(build), "x-signature": sig },
  body: exe,
});
console.log(r.status, await r.text());
