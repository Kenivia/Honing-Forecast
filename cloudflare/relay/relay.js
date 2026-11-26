// Relay between market scanner PCs (crates/market, market_agent) and their controllers (market_mcp --remote,
// Discord). One Durable Object (Hub) holds every agent's WebSocket, forwards commands and keeps each machine's
// status; files (screenshots, profiles) go to R2 under files/<machine>/ (deleted after 7 days by a lifecycle
// rule, at most MAX_FILE each and DAY_BYTES a machine a day), and so does the agent's newest build (PUT /update,
// signed by publish.mjs; the agent checks the signature, the relay only stores it).
import { DurableObject } from "cloudflare:workers";

// an agent that disconnected is reported once it stayed away this long
const DOWN_MS = 2 * 60 * 1000;
// a scanning agent whose log did not move for this long is reported as stuck
const STUCK_MS = 20 * 60 * 1000;
const ALARM_MS = 60 * 1000;
const UPDATE = "update/market_agent.exe";
const MAX_FILE = 20 << 20;
const DAY_BYTES = 1 << 30;

const json = (v, status = 200) =>
  new Response(JSON.stringify(v), {
    status,
    headers: { "content-type": "application/json" },
  });

const bearer = (req) =>
  (req.headers.get("authorization") ?? "").replace(/^Bearer /, "");

const machineTokens = (env) => JSON.parse(env.MACHINE_TOKENS ?? "{}");

const hub = (env) => env.HUB.get(env.HUB.idFromName("hub"));

export default {
  async fetch(req, env) {
    const url = new URL(req.url);
    const path = url.pathname;
    const admin = bearer(req) === env.ADMIN_TOKEN;

    if (path === "/discord" && req.method === "POST") {
      return discordInteraction(req, env);
    }

    let m = path.match(/^\/agent\/([a-z0-9_-]+)$/);
    if (m) {
      if (machineTokens(env)[m[1]] !== bearer(req) || !bearer(req)) {
        return json({ error: "unauthorized" }, 401);
      }
      return hub(env).fetch(req);
    }

    m = path.match(/^\/files\/([a-z0-9_-]+)\/(.+)$/);
    if (m && req.method === "PUT") {
      if (!admin && machineTokens(env)[m[1]] !== bearer(req)) {
        return json({ error: "unauthorized" }, 401);
      }
      const size = Number(req.headers.get("content-length"));
      if (!(size > 0 && size <= MAX_FILE)) {
        return json({ error: `needs a content-length up to ${MAX_FILE}` }, 413);
      }
      if (!(await hub(env).allow(m[1], size))) {
        return json({ error: "daily upload budget used" }, 429);
      }
      const key = `files/${m[1]}/${decodeURIComponent(m[2])}`;
      await env.FILES.put(key, req.body);
      return json({ key });
    }

    // the agent's newest build and its SHA-256, for any machine
    const anyMachine = Object.values(machineTokens(env)).includes(bearer(req));
    if (path === "/update" && req.method === "GET" && (admin || anyMachine)) {
      const o = await env.FILES.head(UPDATE);
      const meta = o?.customMetadata ?? {};
      return json({ sha256: meta.sha256 ?? null, build: meta.build ?? null, sig: meta.sig ?? null });
    }
    if (path === "/update/exe" && req.method === "GET" && (admin || anyMachine)) {
      const o = await env.FILES.get(UPDATE);
      return o ? new Response(o.body) : json({ error: "not found" }, 404);
    }

    if (!admin) return json({ error: "unauthorized" }, 401);

    if (path === "/update" && req.method === "PUT") {
      const body = await req.arrayBuffer();
      const sha256 = [...new Uint8Array(await crypto.subtle.digest("SHA-256", body))]
        .map((b) => b.toString(16).padStart(2, "0"))
        .join("");
      const build = req.headers.get("x-build") ?? "";
      const sig = req.headers.get("x-signature") ?? "";
      await env.FILES.put(UPDATE, body, { customMetadata: { sha256, build, sig } });
      return json({ sha256, build, size: body.byteLength });
    }

    if (path === "/files" && req.method === "GET") {
      const list = await env.FILES.list({
        prefix: url.searchParams.get("prefix") ?? "",
        cursor: url.searchParams.get("cursor") ?? undefined,
      });
      return json({
        files: list.objects.map((o) => ({
          key: o.key,
          size: o.size,
          uploaded: o.uploaded,
        })),
        cursor: list.truncated ? list.cursor : null,
      });
    }
    m = path.match(/^\/files\/(.+)$/);
    if (m && req.method === "GET") {
      const o = await env.FILES.get(decodeURIComponent(m[1]));
      return o ? new Response(o.body) : json({ error: "not found" }, 404);
    }
    m = path.match(/^\/cmd\/([a-z0-9_-]+)$/);
    if (m && req.method === "POST") {
      return json(await hub(env).cmd(m[1], await req.json(), 120_000));
    }
    if (path === "/machines") {
      return json(await hub(env).machines());
    }
    return json({ error: "not found" }, 404);
  },
};

export class Hub extends DurableObject {
  constructor(ctx, env) {
    super(ctx, env);
    // replies awaited by id; only while a request waits, so hibernation never loses one
    this.pending = new Map();
    this.next = 1;
    // the agents' keep-alive, answered without waking the object
    this.ctx.setWebSocketAutoResponse(new WebSocketRequestResponsePair("ping", "pong"));
  }

  async fetch(req) {
    const machine = new URL(req.url).pathname.split("/").pop();
    if (req.headers.get("upgrade") !== "websocket") {
      return json({ error: "expected a websocket" }, 400);
    }
    for (const old of this.ctx.getWebSockets(machine)) {
      old.close(1000, "replaced");
    }
    const [client, server] = Object.values(new WebSocketPair());
    this.ctx.acceptWebSocket(server, [machine]);
    server.serializeAttachment({ machine });
    const rec = await this.record(machine);
    if (rec.down_told) {
      await post(this.env, `**${machine}** is back`);
    }
    Object.assign(rec, {
      connected: true,
      seen: Date.now(),
      down_at: null,
      down_told: false,
    });
    await this.save(machine, rec);
    await this.ensureAlarm();
    return new Response(null, { status: 101, webSocket: client });
  }

  async record(machine) {
    return (await this.ctx.storage.get(`m:${machine}`)) ?? { machine };
  }

  async save(machine, rec) {
    await this.ctx.storage.put(`m:${machine}`, rec);
  }

  async ensureAlarm() {
    if (!(await this.ctx.storage.getAlarm())) {
      await this.ctx.storage.setAlarm(Date.now() + ALARM_MS);
    }
  }

  async webSocketMessage(ws, data) {
    const { machine } = ws.deserializeAttachment();
    const msg = JSON.parse(data);
    if (msg.type === "reply") {
      // only from the machine the command went to
      const p = this.pending.get(msg.id);
      if (p?.machine !== machine) return;
      this.pending.delete(msg.id);
      p.resolve(msg);
      return;
    }
    const rec = await this.record(machine);
    rec.seen = Date.now();
    if (msg.type === "status") {
      if (rec.status?.state === "scanning" && msg.status.state !== "scanning") {
        rec.stuck_told = false;
      }
      // the stuck clock starts at the latest move into scanning, not at a log line from the wait
      if (rec.status?.state !== "scanning" && msg.status.state === "scanning") {
        rec.scan_at = Date.now();
      }
      rec.status = msg.status;
    } else if (msg.type === "event") {
      await this.event(machine, msg);
    }
    await this.save(machine, rec);
  }

  async event(machine, msg) {
    if (msg.kind === "error" || msg.kind === "info") {
      const icon = msg.kind === "error" ? "⚠️" : "ℹ️";
      // only the sender's own files
      const own = msg.file?.startsWith(`files/${machine}/`);
      const file = own ? await this.env.FILES.get(msg.file) : null;
      await post(
        this.env,
        `${icon} **${machine}**: ${msg.text}`,
        file && { name: msg.file.split("/").pop(), body: await file.blob() },
      );
    }
  }

  async webSocketClose(ws) {
    const { machine } = ws.deserializeAttachment();
    if (this.ctx.getWebSockets(machine).some((w) => w !== ws)) return;
    const rec = await this.record(machine);
    Object.assign(rec, { connected: false, down_at: Date.now() });
    await this.save(machine, rec);
  }

  async webSocketError(ws) {
    await this.webSocketClose(ws);
  }

  async alarm() {
    const now = Date.now();
    const recs = await this.ctx.storage.list({ prefix: "m:" });
    for (const [key, rec] of recs) {
      if (!rec.connected && rec.down_at && !rec.down_told && now - rec.down_at > DOWN_MS) {
        await post(this.env, `🔌 **${rec.machine}** disconnected`);
        rec.down_told = true;
      }
      const last = Math.max(Date.parse(rec.status?.log_at ?? "") || 0, rec.scan_at ?? 0);
      if (
        rec.connected &&
        rec.status?.state === "scanning" &&
        !rec.stuck_told &&
        now - last > STUCK_MS
      ) {
        await post(
          this.env,
          `⏳ **${rec.machine}** has logged nothing for ${span(now - last)}: ${rec.status.log}`,
        );
        rec.stuck_told = true;
      }
      await this.ctx.storage.put(key, rec);
    }
    await this.ctx.storage.setAlarm(now + ALARM_MS);
  }

  /// Sends a command to a machine's agent and waits for its reply.
  async cmd(machine, body, timeout) {
    const ws = this.ctx.getWebSockets(machine)[0];
    if (!ws) return { ok: false, text: `${machine} is not connected` };
    const id = this.next++;
    const reply = new Promise((resolve) => {
      this.pending.set(id, { machine, resolve });
      setTimeout(() => {
        if (this.pending.delete(id)) {
          resolve({ ok: false, text: `no reply from ${machine} in ${timeout / 1000} s` });
        }
      }, timeout);
    });
    ws.send(JSON.stringify({ ...body, id }));
    return reply;
  }

  /// Counts an upload against the machine's bytes for the day (UTC); false once over DAY_BYTES, told once.
  async allow(machine, size) {
    const day = new Date().toISOString().slice(0, 10);
    const key = `q:${machine}`;
    let q = (await this.ctx.storage.get(key)) ?? {};
    if (q.day !== day) q = { day, bytes: 0, told: false };
    if (q.bytes + size > DAY_BYTES) {
      if (!q.told) {
        await post(this.env, `🛑 **${machine}** used its upload budget for today; uploads refused until 00:00 UTC`);
        q.told = true;
        await this.ctx.storage.put(key, q);
      }
      return false;
    }
    q.bytes += size;
    await this.ctx.storage.put(key, q);
    return true;
  }

  async machines() {
    const recs = await this.ctx.storage.list({ prefix: "m:" });
    return [...recs.values()];
  }
}

// --- Discord: status posts through the bot, and the /start, /pause, /gap, /status commands

// agent text goes to Discord as is: no pings from it
const allowed_mentions = { parse: [] };

async function post(env, content, file) {
  if (!env.DISCORD_BOT_TOKEN) return;
  const url = `https://discord.com/api/v10/channels/${env.DISCORD_CHANNEL_ID}/messages`;
  const headers = { authorization: `Bot ${env.DISCORD_BOT_TOKEN}` };
  let body;
  if (file) {
    body = new FormData();
    body.append(
      "payload_json",
      JSON.stringify({ content, allowed_mentions, attachments: [{ id: 0, filename: file.name }] }),
    );
    body.append("files[0]", file.body, file.name);
  } else {
    headers["content-type"] = "application/json";
    body = JSON.stringify({ content, allowed_mentions });
  }
  const r = await fetch(url, { method: "POST", headers, body });
  if (!r.ok) console.log("discord post failed", r.status, await r.text());
}

const hex = (s) => new Uint8Array(s.match(/../g).map((b) => parseInt(b, 16)));

async function verified(req, env, body) {
  const sig = req.headers.get("x-signature-ed25519");
  const ts = req.headers.get("x-signature-timestamp");
  if (!sig || !ts) return false;
  const key = await crypto.subtle.importKey(
    "raw",
    hex(env.DISCORD_PUBLIC_KEY),
    { name: "Ed25519" },
    false,
    ["verify"],
  );
  return crypto.subtle.verify(
    "Ed25519",
    key,
    hex(sig),
    new TextEncoder().encode(ts + body),
  );
}

// largest whole unit only: s, min, h, d
const span = (ms) => {
  const s = Math.max(0, Math.floor(ms / 1000));
  if (s < 60) return `${s} s`;
  if (s < 3600) return `${Math.floor(s / 60)} min`;
  if (s < 86400) return `${Math.floor(s / 3600)} h`;
  return `${Math.floor(s / 86400)} d`;
};
const ago = (t) => `${span(Date.now() - t)} ago`;

function describe(rec) {
  const s = rec.status ?? {};
  if (!rec.connected) {
    return `**${rec.machine}**: disconnected (last seen ${ago(rec.seen)})`;
  }
  return [
    `**${rec.machine}**: ${s.state ?? "?"}${s.game ? "" : " (game not running)"}`,
    s.gap !== undefined &&
      `gap: ${s.gap ? `${s.gap} min` : "none"}${s.next ? `, next round <t:${s.next}:R>` : ""}`,
    s.last_pass &&
      `last pass${s.last_pass_at ? ` (${ago(Date.parse(s.last_pass_at))})` : ""}: ${s.last_pass}`,
    s.log && `log (${ago(Date.parse(s.log_at))}): ${s.log}`,
  ]
    .filter(Boolean)
    .join("\n");
}

async function discordInteraction(req, env) {
  const body = await req.text();
  if (!(await verified(req, env, body))) {
    return new Response("bad signature", { status: 401 });
  }
  const i = JSON.parse(body);
  if (i.type === 1) return json({ type: 1 });
  const say = (content) => json({ type: 4, data: { content, allowed_mentions } });
  const user = i.member?.user?.id ?? i.user?.id;
  if (!env.DISCORD_USERS.split(",").includes(user)) {
    return say("Not allowed.");
  }
  const h = hub(env);
  const recs = await h.machines();
  const asked = i.data.options?.find((o) => o.name === "machine")?.value;
  const name = i.data.name;
  if (name === "status") {
    const shown = recs.filter((r) => !asked || r.machine === asked);
    return say(shown.map(describe).join("\n\n") || "No machines yet.");
  }
  const live = recs.filter((r) => r.connected);
  const machine = asked ?? (live.length === 1 ? live[0].machine : null);
  if (!machine) {
    return say(
      `Which machine? Connected: ${live.map((r) => r.machine).join(", ") || "none"}`,
    );
  }
  // only these reach an agent from Discord (it takes nothing else unsigned anyway)
  const cmd = { op: name };
  if (name === "gap") {
    cmd.minutes = i.data.options?.find((o) => o.name === "minutes")?.value;
    if (!Number.isInteger(cmd.minutes) || cmd.minutes < 0 || cmd.minutes > 1440) {
      return say("Minutes must be a whole number from 0 to 1440.");
    }
  } else if (name !== "start" && name !== "pause") {
    return say("Unknown command.");
  }
  // Discord wants an answer within 3 s; these only set a value on the agent
  const r = await h.cmd(machine, cmd, 2500);
  return say(`**${machine}**: ${r.text}`);
}
