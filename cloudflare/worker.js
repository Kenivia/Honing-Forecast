import { unflatten } from "devalue";

const MARKET_URL =
  "https://marketdata-api.yrzhao1068589.workers.dev/v1/prices/latest";
const BIBLE_URL = "https://lostark.bible";
const CACHE_TTL = 3600;
const MAX_UPLOAD = 1_000_000;
// free plan: 50 subrequests per invocation
const MAX_CHARS = 40;
const BIBLE_REGIONS = { nae: "NA", euc: "CE" };
const WEEK_MS = 7 * 24 * 3600 * 1000;
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
        const [, region, char_name, suffix = ""] = url.pathname.match(
          /^\/character\/([^/]+)\/([^/]+)(?:\/([^/]*))?$/,
        );
        if (region && char_name) {
          return handleCharacterProxy(region, char_name, suffix, env, ctx);
        }
      }

      if (request.method === "POST" && url.pathname === "/snapshots") {
        // awaited so a failure gets the catch's CORS headers
        return await handleSnapshots(request, env);
      }

      if (request.method === "POST" && url.pathname === "/") {
        // console.log(request, env, ctx);
        return handleMarketProxy(request, env, ctx);
      }

      return corsResponse(new Response("Not Found", { status: 404 }));
    } catch (e) {
      // console.log(`Worker error: ${e.message}`);
      return corsResponse(
        new Response(`Worker error: ${e.message}`, { status: 500 }),
      );
    }
  },
};

async function handleCharacterProxy(region, charName, suffix, env, ctx) {
  const upstreamUrl =
    `${BIBLE_URL}/character/${region}/${charName}` +
    (suffix ? `/${suffix}` : "");
  const cacheKey = `character:${region}:${charName}:${suffix}`;

  const cached = await env.CACHE_KV.get(cacheKey);
  if (cached) {
    return corsResponse(
      new Response(cached, {
        status: 200,
        headers: { "Content-Type": "text/html", "X-Cache": "HIT" },
      }),
    );
  }

  const upstream = await fetch(upstreamUrl, {
    method: "GET",
    headers: UWUOWO_HEADERS,
  });

  if (!upstream.ok) {
    return corsResponse(
      new Response(
        `Upstream error: ${upstream.status}, response: ${await upstream.text()}`,
        { status: 502 },
      ),
    );
  }

  const data = await upstream.text();
  if (data == "") {
    return corsResponse(
      new Response(
        `Empty response: ${upstream.status}, response: ${await upstream.text()}`,
        { status: 502 },
      ),
    );
  }

  ctx.waitUntil(env.CACHE_KV.put(cacheKey, data, { expirationTtl: CACHE_TTL }));

  return corsResponse(
    new Response(data, {
      status: 200,
      headers: { "Content-Type": "text/html", "X-Cache": "MISS" },
    }),
  );
}

async function handleMarketProxy(request, env, ctx) {
  const body = await request.text();
  const cacheKey = await hashBody(body);

  const cached = await env.CACHE_KV.get(cacheKey);
  if (cached) {
    return corsResponse(
      new Response(cached, {
        status: 200,
        headers: { "Content-Type": "application/json", "X-Cache": "HIT" },
      }),
    );
  }

  const upstream = await fetch(MARKET_URL, {
    method: "POST",
    headers: {
      Accept: "*/*",
      "Accept-Language": "en-US,en;q=0.9",
      "Content-Type": "application/json",
      Priority: "u=4",
    },
    body,
  });

  if (!upstream.ok) {
    return corsResponse(
      new Response(`Upstream error: ${upstream.status}`, { status: 502 }),
    );
  }

  const data = await upstream.text();

  ctx.waitUntil(env.CACHE_KV.put(cacheKey, data, { expirationTtl: CACHE_TTL }));

  return corsResponse(
    new Response(data, {
      status: 200,
      headers: { "Content-Type": "application/json", "X-Cache": "MISS" },
    }),
  );
}

// One roster's snapshots, stored with each character's recent raids from lostark.bible and
// without anything that names the character.
async function handleSnapshots(request, env) {
  const body = await request.text();
  if (body.length > MAX_UPLOAD) {
    return corsResponse(new Response("Too large", { status: 413 }));
  }
  const { roster, region, version, snapshots } = JSON.parse(body);
  if (typeof roster !== "string" || !Array.isArray(snapshots)) {
    return corsResponse(new Response("Bad upload", { status: 400 }));
  }

  // the latest name of each character
  const names = {};
  for (const snapshot of snapshots) {
    for (const [id, char] of Object.entries(snapshot.chars ?? {})) {
      names[id] = char.name;
      delete char.name;
    }
  }
  const raids = {};
  await Promise.all(
    Object.entries(names)
      .slice(0, MAX_CHARS)
      .map(async ([id, name]) => {
        raids[id] = await fetchRaids(BIBLE_REGIONS[region], name);
      }),
  );

  await env.DB.prepare(
    "INSERT INTO uploads (received, roster, region, version, snapshots, raids) VALUES (?, ?, ?, ?, ?, ?)",
  )
    .bind(
      Date.now(),
      roster,
      String(region),
      String(version),
      JSON.stringify(snapshots),
      JSON.stringify(raids),
    )
    .run();
  return corsResponse(new Response("ok", { status: 200 }));
}

// The character's recent logs, reduced to what does not identify it, or why there are none.
async function fetchRaids(region, name) {
  if (!region) return "no region";
  if (!name || /^Newchar\d*$/.test(name)) return "no name";
  try {
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

async function hashBody(text) {
  const encoded = new TextEncoder().encode(text);
  const digest = await crypto.subtle.digest("SHA-256", encoded);
  return Array.from(new Uint8Array(digest))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
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
