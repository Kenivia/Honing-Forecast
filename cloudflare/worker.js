import { unflatten } from "devalue";

const MARKET_URL =
  "https://marketdata-api.yrzhao1068589.workers.dev/v1/prices/latest";
const BIBLE_URL = "https://lostark.bible";
// Workers Cache keeps 200s for an hour; a hit never runs the Worker
const CACHE = "public, max-age=3600";
// the consolidated prices: kept short so new passes show quickly
const PRICES_CACHE = "public, max-age=60";
// upstream fetches per minute, across all of Cloudflare
const LIMITS = { market: 10, bible: 200 };
const MAX_UPLOAD = 300_000;
// one market scanner pass (crates/market); a full one is about 100 KB
const MAX_SCAN = 1_000_000;
// one upload's share of the lostark.bible limit
const MAX_CHARS = 40;
const BIBLE_REGIONS = { nae: "NA", euc: "CE" };
const HOUR_MS = 3600 * 1000;
const WEEK_MS = 7 * 24 * HOUR_MS;
// an exported character is looked up again this long after its last export
const FOLLOWUP_MS = 24 * HOUR_MS;
// a weekly reset, Thursday 10:00 UTC
const RESET = Date.UTC(2026, 0, 1, 10);

const UWUOWO_HEADERS = {
  "User-Agent":
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
  Accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
  "Accept-Language": "en-US,en;q=0.9",
  "Accept-Encoding": "gzip, deflate, br",
  Connection: "keep-alive",
  Referer: "https://honing-forecast.pages.dev/",
};

export default {
  async fetch(request, env, ctx) {
    try {
      const url = new URL(request.url);

      if (request.method === "OPTIONS") {
        return corsResponse(null, 204);
      }

      if (request.method === "GET") {
        const character = url.pathname.match(
          /^\/character\/([^/]+)\/([^/]+)(?:\/([^/]*))?$/,
        );
        if (character) {
          const [, region, char_name, suffix = ""] = character;
          return await handleCharacterProxy(region, char_name, suffix, env);
        }
        const market = url.pathname.match(/^\/market\/([^/]+)$/);
        if (market) {
          return await handleMarketProxy(
            market[1],
            url.searchParams.get("items") ?? "",
            env,
          );
        }
        const prices = url.pathname.match(/^\/prices\/([a-z]+)$/);
        if (prices) {
          return await handlePrices(
            prices[1],
            url.searchParams.get("items") ?? "",
            env,
          );
        }
        const scans = url.pathname.match(
          /^\/scans\/(?:latest\/([a-z]+)|(\d+)|history\/([a-z]+)\/(\d{4}-\d{2}))$/,
        );
        if (scans) {
          const [, latest, id, region, month] = scans;
          if (latest)
            return await handleScanGet(
              "region = ? ORDER BY started DESC",
              latest,
              env,
            );
          if (id) return await handleScanGet("id = ?", Number(id), env);
          return await handleHistory(region, month, env);
        }
        const bible = url.pathname.match(/^\/bible\/([^/]+)\/([^/]+)$/);
        if (bible) {
          return await handleBible(bible[1], decodeURIComponent(bible[2]), env);
        }
      }

      if (request.method === "POST" && url.pathname === "/snapshots") {
        // awaited so a failure gets the catch's CORS headers
        return await handleSnapshots(request, env, ctx);
      }

      if (request.method === "POST" && url.pathname === "/scans") {
        return await handleScan(request, env);
      }

      if (request.method === "POST" && url.pathname === "/moves") {
        return await handleMove(request, env);
      }

      return reply("Not Found", 404);
    } catch (e) {
      return reply(`Worker error: ${e.message}`, 500);
    }
  },

  async scheduled(controller, env, ctx) {
    await followUp(env, ctx);
  },
};

// One fetch from the source, unless this minute's are used up. Only cache misses get here.
async function allow(env, source) {
  const row = await env.DB.prepare(
    `INSERT INTO limits (source, minute, n) VALUES (?1, ?2, 1)
     ON CONFLICT (source) DO UPDATE SET n = CASE WHEN minute = ?2 THEN n + 1 ELSE 1 END, minute = ?2
     WHERE minute != ?2 OR n < ?3
     RETURNING n`,
  )
    .bind(source, Math.floor(Date.now() / 60_000), LIMITS[source])
    .first();
  return row !== null;
}

async function handleCharacterProxy(region, charName, suffix, env) {
  if (!(await allow(env, "bible"))) return reply("Rate limited", 429);
  const upstream = await fetch(
    `${BIBLE_URL}/character/${region}/${charName}` +
      (suffix ? `/${suffix}` : ""),
    { headers: UWUOWO_HEADERS },
  );
  const data = await upstream.text();
  if (!upstream.ok) {
    return reply(`Upstream error: ${upstream.status}, response: ${data}`, 502);
  }
  if (data == "") return reply(`Empty response: ${upstream.status}`, 502);
  return reply(data, 200, "text/html");
}

async function handleMarketProxy(region, items, env) {
  if (!(await allow(env, "market"))) return reply("Rate limited", 429);
  const upstream = await fetchMarket(region, items);
  if (!upstream.ok) return reply(`Upstream error: ${upstream.status}`, 502);
  return reply(await upstream.text(), 200, "application/json");
}

const fetchMarket = (region, items) =>
  fetch(MARKET_URL, {
    method: "POST",
    headers: {
      Accept: "*/*",
      "Accept-Language": "en-US,en;q=0.9",
      "Content-Type": "application/json",
      Priority: "u=4",
    },
    body: JSON.stringify({ region_slug: region, item_slugs: items.split(",") }),
  });

// The newest price of each item, from loa-buddy or the scanner's lowest listings (market_latest):
// [{item_slug, price, timestamp (s), source: "loa-buddy" | "hf"}]. Either source alone is enough.
async function handlePrices(region, items, env) {
  const wanted = new Set(items.split(","));
  const [loa, ours] = await Promise.allSettled([
    (async () => {
      if (!(await allow(env, "market"))) throw new Error("Rate limited");
      const upstream = await fetchMarket(region, items);
      if (!upstream.ok) throw new Error(`Upstream error: ${upstream.status}`);
      return await upstream.json();
    })(),
    env.DB.prepare(`SELECT body FROM market_latest WHERE region = ?`)
      .bind(region)
      .first(),
  ]);
  if (loa.status === "rejected" && ours.status === "rejected") {
    return reply(`No prices: ${loa.reason.message}`, 502);
  }
  const best = {};
  for (const row of loa.value ?? []) {
    best[row.item_slug] = { ...row, source: "loa-buddy" };
  }
  const latest = JSON.parse(ours.value?.body ?? "{}");
  for (const [slug, [price, at]] of Object.entries(latest)) {
    const timestamp = Date.parse(at) / 1000;
    if (wanted.has(slug) && !(best[slug]?.timestamp >= timestamp)) {
      best[slug] = { item_slug: slug, price, timestamp, source: "hf" };
    }
  }
  return reply(
    JSON.stringify(Object.values(best)),
    200,
    "application/json",
    PRICES_CACHE,
  );
}

// loa-buddy's slug of an in-game name: "Artisan's Metallurgy: Level 1" -> "artisans-metallurgy-level-1"
const slug = (name) =>
  name
    .toLowerCase()
    .replace(/'/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");

// A character's raids and gear, for uploads through the cache. Passing failures are not cached.
async function handleBible(region, name, env) {
  const [raids, gear] = await Promise.all([
    fetchRaids(region, name, env),
    fetchGear(region, name, env),
  ]);
  const cache = [raids, gear].every(
    (v) => typeof v !== "string" || v === "private" || v === "not found",
  );
  return reply(JSON.stringify({ raids, gear }), 200, "application/json", cache);
}

// Each character's raids and gear, or why there are none, through a loopback call each, so
// Workers Cache serves a character looked up within the hour and each has its own subrequests.
async function lookUp(ctx, region, names) {
  const raids = {};
  const gear = {};
  const bible_region = BIBLE_REGIONS[region];
  await Promise.all(
    Object.entries(names).map(async ([id, name]) => {
      if (!bible_region) return (raids[id] = gear[id] = "no region");
      if (!valid_name(name)) return (raids[id] = gear[id] = "no name");
      const res = await ctx.exports.default
        .fetch(
          `https://loopback/bible/${bible_region}/${encodeURIComponent(name)}`,
        )
        .then((res) => res.json())
        .catch((e) => ({
          raids: `error ${e.message}`,
          gear: `error ${e.message}`,
        }));
      raids[id] = res.raids;
      gear[id] = res.gear;
    }),
  );
  return { raids, gear };
}

const valid_name = (name) => !!name && !/^Newchar\d*$/.test(name);

// One roster's snapshots, stored with each character's recent raids from lostark.bible and
// without anything that names the character.
// one row per domain-move outcome a browser reports
async function handleMove(request, env) {
  const body = await request.text();
  if (body.length > 2000) return reply("Too large", 413);
  const { site, version, outcome, error, chunks, bytes } = JSON.parse(body);
  if (typeof outcome !== "string") return reply("Bad report", 400);
  const text = (v) => (typeof v === "string" ? v.slice(0, 300) : null);
  const int = (v) => (Number.isInteger(v) ? v : null);
  await env.DB.prepare(
    `INSERT INTO moves (received, site, version, outcome, error, chunks, bytes) VALUES (?, ?, ?, ?, ?, ?, ?)`,
  )
    .bind(
      Date.now(),
      text(site),
      text(version),
      text(outcome),
      text(error),
      int(chunks),
      int(bytes),
    )
    .run();
  return reply("ok", 200, "text/plain", false);
}

// One market scanner pass, kept whole; its points [key, day, value] are merged into its region's
// history, one row per month (a later value for the same key and day wins), and its materials' lowest
// listings into market_latest (a newer read wins). Only the scanner's token may post.
async function handleScan(request, env) {
  if (!(await authorized(request, env))) return reply("Unauthorized", 401);
  const body = await request.text();
  if (body.length > MAX_SCAN) return reply("Too large", 413);
  const scan = JSON.parse(body);
  const region = scan.region;
  if (typeof region !== "string" || typeof scan.started !== "string") {
    return reply("Bad scan", 400);
  }
  // a retry of a pass already stored
  const old = await env.DB.prepare(
    `SELECT id FROM market_scans WHERE region = ? AND started = ?`,
  )
    .bind(region, scan.started)
    .first();
  if (old) return reply(`stored before as ${old.id}`, 200, "text/plain", false);

  const months = {};
  for (const [key, day, value] of scan.points ?? []) {
    ((months[day.slice(0, 7)] ??= {})[key] ??= {})[day] = value;
  }
  const writes = [
    env.DB.prepare(
      `INSERT INTO market_scans (region, started, finished, received, body) VALUES (?, ?, ?, ?, ?)`,
    ).bind(region, scan.started, scan.finished ?? null, Date.now(), body),
  ];
  for (const [month, series] of Object.entries(months)) {
    const row = await env.DB.prepare(
      `SELECT body FROM market_history WHERE region = ? AND month = ?`,
    )
      .bind(region, month)
      .first();
    const history = row ? JSON.parse(row.body) : {};
    for (const [key, days] of Object.entries(series)) {
      Object.assign((history[key] ??= {}), days);
    }
    writes.push(
      env.DB.prepare(
        `INSERT INTO market_history (region, month, body) VALUES (?, ?, ?)
         ON CONFLICT (region, month) DO UPDATE SET body = excluded.body`,
      ).bind(region, month, JSON.stringify(history)),
    );
  }
  // each material's lowest listing, where newer than the one kept
  const kept = await env.DB.prepare(
    `SELECT body FROM market_latest WHERE region = ?`,
  )
    .bind(region)
    .first();
  const latest = JSON.parse(kept?.body ?? "{}");
  for (const m of scan.sections?.material ?? []) {
    const price = m.depth?.[0]?.[0];
    const key = slug(m.name);
    if (price === undefined || !m.at || latest[key]?.[1] >= m.at) continue;
    latest[key] = [price, m.at];
  }
  writes.push(
    env.DB.prepare(
      `INSERT INTO market_latest (region, body) VALUES (?, ?)
       ON CONFLICT (region) DO UPDATE SET body = excluded.body`,
    ).bind(region, JSON.stringify(latest)),
  );
  // one transaction: the pass, its history and the latest prices together
  const [inserted] = await env.DB.batch(writes);
  return reply(
    `stored as ${inserted.meta.last_row_id}`,
    200,
    "text/plain",
    false,
  );
}

async function authorized(request, env) {
  if (!env.MARKET_TOKEN) return false;
  const enc = new TextEncoder();
  const got = enc.encode(request.headers.get("Authorization") ?? "");
  const want = enc.encode(`Bearer ${env.MARKET_TOKEN}`);
  return (
    got.byteLength === want.byteLength &&
    crypto.subtle.timingSafeEqual(got, want)
  );
}

// One stored pass: {id, received, scan}. Not cached: one indexed row per request; a pass by id never changes.
async function handleScanGet(where, value, env) {
  const row = await env.DB.prepare(
    `SELECT id, received, body FROM market_scans WHERE ${where} LIMIT 1`,
  )
    .bind(value)
    .first();
  if (!row) return reply("No scan", 404);
  return reply(
    `{"id":${row.id},"received":${row.received},"scan":${row.body}}`,
    200,
    "application/json",
    typeof value === "number",
  );
}

// A region's month of history: {key: {day: value}}, {} when there is none.
async function handleHistory(region, month, env) {
  const row = await env.DB.prepare(
    `SELECT body FROM market_history WHERE region = ? AND month = ?`,
  )
    .bind(region, month)
    .first();
  return reply(row?.body ?? "{}", 200, "application/json", false);
}

async function handleSnapshots(request, env, ctx) {
  const body = await request.text();
  if (body.length > MAX_UPLOAD) {
    return reply("Too large", 413);
  }
  const { roster, region, version, snapshots } = JSON.parse(body);
  if (typeof roster !== "string" || !Array.isArray(snapshots)) {
    return reply("Bad upload", 400);
  }

  // the latest name of each character
  const names = {};
  for (const snapshot of snapshots) {
    for (const [id, char] of Object.entries(snapshot.chars ?? {})) {
      names[id] = char.name;
      delete char.name;
    }
  }
  const { raids, gear } = await lookUp(
    ctx,
    region,
    Object.fromEntries(Object.entries(names).slice(0, MAX_CHARS)),
  );
  const writes = [
    insertUpload(env, roster, region, version, snapshots, raids, gear),
  ];

  // the exported characters' follow-ups, pushed back by every export
  for (const id of new Set(snapshots.map((snapshot) => snapshot.by))) {
    if (!BIBLE_REGIONS[region] || !valid_name(names[id])) continue;
    writes.push(
      env.DB.prepare(
        `INSERT INTO pending (char, roster, region, version, name, due) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (char) DO UPDATE SET roster = ?2, region = ?3, version = ?4, name = ?5, due = ?6`,
      ).bind(
        id,
        roster,
        region,
        String(version),
        names[id],
        Date.now() + FOLLOWUP_MS,
      ),
    );
  }
  await env.DB.batch(writes);
  return reply("ok", 200, "text/plain", false);
}

const insertUpload = (env, roster, region, version, snapshots, raids, gear) =>
  env.DB.prepare(
    "INSERT INTO uploads (received, roster, region, version, snapshots, raids, gear) VALUES (?, ?, ?, ?, ?, ?, ?)",
  ).bind(
    Date.now(),
    roster,
    String(region),
    String(version),
    JSON.stringify(snapshots),
    JSON.stringify(raids),
    JSON.stringify(gear),
  );

// The characters whose follow-up is due, looked up again and stored as an upload without
// snapshots, one per roster. A rate-limited one stays for the next run.
async function followUp(env, ctx) {
  const { results } = await env.DB.prepare(
    "SELECT * FROM pending WHERE due <= ? ORDER BY due LIMIT ?",
  )
    .bind(Date.now(), MAX_CHARS)
    .all();
  const rosters = Object.groupBy(results, (row) => row.roster);
  for (const rows of Object.values(rosters)) {
    const { roster, region, version } = rows[0];
    const found = await lookUp(
      ctx,
      region,
      Object.fromEntries(rows.map((row) => [row.char, row.name])),
    );
    const done = rows.filter((row) => found.gear[row.char] !== "rate limited");
    if (done.length === 0) continue;
    const pick = (by) =>
      Object.fromEntries(done.map((row) => [row.char, by[row.char]]));
    await env.DB.batch([
      insertUpload(
        env,
        roster,
        region,
        `followup ${version}`,
        [],
        pick(found.raids),
        pick(found.gear),
      ),
      // unless an export pushed it back meanwhile
      ...done.map((row) =>
        env.DB.prepare("DELETE FROM pending WHERE char = ? AND due = ?").bind(
          row.char,
          row.due,
        ),
      ),
    ]);
  }
}

// The character's recent logs, reduced to what does not identify it, or why there are none.
async function fetchRaids(region, name, env) {
  try {
    if (!(await allow(env, "bible"))) return "rate limited";
    const upstream = await fetch(
      `${BIBLE_URL}/character/${region}/${encodeURIComponent(name)}/logs/__data.json`,
      { headers: UWUOWO_HEADERS },
    );
    if (!upstream.ok) return `error ${upstream.status}`;
    const page = (await upstream.json()).nodes
      .map((node) => (node?.type === "data" ? unflatten(node.data) : null))
      .find((data) => data && "logs" in data);
    if (!page) return "not found";
    if (!page.logsEnabled || !page.isPublic) return "private";
    return page.logs.map((log) => ({
      boss: log.boss,
      difficulty: log.difficulty,
      isBus: log.isBus,
      gearScore: Math.floor(log.gearScore),
      class: log.class,
      week: RESET + Math.floor((log.timestamp - RESET) / WEEK_MS) * WEEK_MS,
    }));
  } catch (e) {
    return `error ${e.message}`;
  }
}

// The character's gear levels in each loadout lostark.bible keeps (one per kind of content last
// entered: raid, chaos dungeon), times to the hour, or why there are none.
async function fetchGear(region, name, env) {
  try {
    if (!(await allow(env, "bible"))) return "rate limited";
    const upstream = await fetch(
      `${BIBLE_URL}/character/${region}/${encodeURIComponent(name)}/__data.json`,
      { headers: UWUOWO_HEADERS },
    );
    if (!upstream.ok) return `error ${upstream.status}`;
    const nodes = (await upstream.json()).nodes.map((node) =>
      node?.type === "data" ? unflatten(node.data) : null,
    );
    const header = nodes.find((data) => data?.header)?.header;
    const loadouts = nodes.find((data) => data && "loadouts" in data)?.loadouts;
    if (!header || !loadouts?.length) return "not found";
    return {
      ilvl: header.ilvl,
      refreshed: to_hour(header.lastRefreshTime),
      loadouts: loadouts.map((loadout) => ({
        kind: loadout.classification,
        updated: to_hour(loadout.lastUpdated),
        ilvl: loadout.itemLevel,
        items: loadout.items
          .filter((item) => item.data && "honing" in item.data)
          .map((item) => ({
            slot: item.slot,
            id: item.id,
            honing: item.data.honing,
            adv: item.data.advancedHoning,
          })),
      })),
    };
  } catch (e) {
    return `error ${e.message}`;
  }
}

const to_hour = (ms) => Math.floor(ms / HOUR_MS) * HOUR_MS;

function reply(body, status, type = "text/plain", cache = status === 200) {
  return corsResponse(
    new Response(body, {
      status,
      headers: {
        "Content-Type": type,
        "Cache-Control":
          typeof cache === "string" ? cache : cache ? CACHE : "no-store",
      },
    }),
  );
}

function corsResponse(response, status) {
  if (response === null) {
    return new Response(null, {
      status: status ?? 204,
      headers: corsHeaders(),
    });
  }
  const headers = new Headers(response.headers);
  for (const [k, v] of Object.entries(corsHeaders())) {
    headers.set(k, v);
  }
  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers,
  });
}

function corsHeaders() {
  return {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type",
  };
}
