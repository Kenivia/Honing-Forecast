// Registers /start, /pause and /status on the server of the relay's channel (instant, unlike global commands).
// node cloudflare/relay/register.mjs   (reads the bot token from .dev.vars, the ids from wrangler.toml)
import { readFileSync } from "node:fs";

const dir = new URL(".", import.meta.url);
const vars = Object.fromEntries(
  readFileSync(new URL(".dev.vars", dir), "utf8")
    .split("\n")
    .filter((l) => l.includes("="))
    .map((l) => [l.slice(0, l.indexOf("=")), l.slice(l.indexOf("=") + 1).trim()]),
);
const toml = readFileSync(new URL("wrangler.toml", dir), "utf8");
const value = (k) => toml.match(new RegExp(`${k} = "([^"]+)"`))[1];
const app = value("DISCORD_APPLICATION_ID");
const headers = {
  authorization: `Bot ${vars.DISCORD_BOT_TOKEN}`,
  "content-type": "application/json",
};
const api = "https://discord.com/api/v10";

const channel = await (
  await fetch(`${api}/channels/${value("DISCORD_CHANNEL_ID")}`, { headers })
).json();
const machine = {
  name: "machine",
  description: "which machine (default: the only one connected)",
  type: 3,
  required: false,
};
const commands = [
  ["start", "Start scanning (when idle)"],
  ["pause", "Stop the pass; what it read so far is uploaded"],
  ["status", "What each machine is doing"],
].map(([name, description]) => ({ name, description, options: [machine] }));
const r = await fetch(`${api}/applications/${app}/guilds/${channel.guild_id}/commands`, {
  method: "PUT",
  headers,
  body: JSON.stringify(commands),
});
console.log(r.status, (await r.json()).map?.((c) => c.name) ?? "");
