import { unflatten } from "devalue";

const MARKET_URL =
  "https://marketdata-api.yrzhao1068589.workers.dev/v1/prices/latest";
const BIBLE_URL = "https://lostark.bible";
// Workers Cache keeps 200s for an hour; a hit never runs the Worker
const CACHE = "public, max-age=3600";
// upstream fetches per minute, across all of Cloudflare
const LIMITS = { market: 10, bible: 200 };
const MAX_UPLOAD = 300_000;
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
        const bible = url.pathname.match(/^\/bible\/([^/]+)\/([^/]+)$/);
        if (bible) {
          return await handleBible(bible[1], decodeURIComponent(bible[2]), env);
        }
      }

      if (request.method === "POST" && url.pathname === "/snapshots") {
        // awaited so a failure gets the catch's CORS headers
        return await handleSnapshots(request, env, ctx);
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
  const upstream = await fetch(MARKET_URL, {
    method: "POST",
    headers: {
      Accept: "*/*",
      "Accept-Language": "en-US,en;q=0.9",
      "Content-Type": "application/json",
      Priority: "u=4",
    },
    body: JSON.stringify({ region_slug: region, item_slugs: items.split(",") }),
  });
  if (!upstream.ok) return reply(`Upstream error: ${upstream.status}`, 502);
  return reply(await upstream.text(), 200, "application/json");
}

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
        "Cache-Control": cache ? CACHE : "no-store",
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
