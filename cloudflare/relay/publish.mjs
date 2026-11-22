// Publishes target/release/market_agent.exe as the agents' update (relay PUT /update); agents pick it up at
// their next start or round. node cloudflare/relay/publish.mjs
import { readFileSync } from "node:fs";

const root = new URL("../../", import.meta.url);
const cfg = JSON.parse(readFileSync(new URL("ignored/market/relay_admin.json", root), "utf8"));
const exe = readFileSync(new URL("target/release/market_agent.exe", root));
const r = await fetch(`${cfg.url}/update`, {
  method: "PUT",
  headers: { authorization: `Bearer ${cfg.token}` },
  body: exe,
});
console.log(r.status, await r.text());
