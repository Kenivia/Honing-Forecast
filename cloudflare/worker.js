const MARKET_URL =
  "https://marketdata-api.yrzhao1068589.workers.dev/v1/prices/latest";
const BIBLE_URL = "https://lostark.bible";
const CACHE_TTL = 3600;

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
    `${BIBLE_URL}/character/${region}/${charName}` + suffix ? `/${suffix}` : "";
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
        `Upstream error: ${upstream.status}, ${await upstream.text()}`,
        { status: 502 },
      ),
    );
  }

  const data = await upstream.text();

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
