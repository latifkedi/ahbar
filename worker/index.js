// Ahbar — Cloudflare Worker + D1 Edge API for Passwordless Subscriptions & Signal/Noise Votes
// Mirrors crates/api (Axum) so you can run either self-hosted Rust or zero-cost Cloudflare Workers.

const CORS_HEADERS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, Authorization"
};

async function signToken(email, action, secret) {
  const enc = new TextEncoder();
  const key = await crypto.subtle.importKey(
    "raw",
    enc.encode(secret || "ahbar-default-dev-secret"),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"]
  );
  const sig = await crypto.subtle.sign("HMAC", key, enc.encode(`${email.trim().toLowerCase()}|${action}`));
  return Array.from(new Uint8Array(sig))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

export default {
  async fetch(request, env) {
    if (request.method === "OPTIONS") {
      return new Response(null, { headers: CORS_HEADERS });
    }

    const url = new URL(request.url);

    // 1. POST /api/subscribe (Create or update passwordless subscription)
    if (request.method === "POST" && url.pathname === "/api/subscribe") {
      const body = await request.json();
      const email = (body.email || "").trim().toLowerCase();
      if (!email || !email.includes("@")) {
        return Response.json({ ok: false, message: "Invalid email address" }, { status: 400, headers: CORS_HEADERS });
      }

      const mode = body.mode || "hybrid";
      const density = body.density || "compact";
      const frequency = body.frequency || "daily";
      const contentFilter = body.content_filter || "all";
      const areasJson = JSON.stringify(body.areas || ["cs", "ai", "math", "physics", "systems"]);
      const tagsJson = JSON.stringify(body.tags || []);

      if (env.DB) {
        await env.DB.prepare(
          `INSERT INTO subscribers (email, mode, density, frequency, content_filter, areas_json, tags_json, verified, updated_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, datetime('now'))
           ON CONFLICT(email) DO UPDATE SET
             mode = excluded.mode,
             density = excluded.density,
             frequency = excluded.frequency,
             content_filter = excluded.content_filter,
             areas_json = excluded.areas_json,
             tags_json = excluded.tags_json,
             updated_at = datetime('now')`
        )
          .bind(email, mode, density, frequency, contentFilter, areasJson, tagsJson)
          .run();
      }

      const prefsToken = await signToken(email, "prefs", env.AHBAR_HMAC_SECRET);
      return Response.json(
        {
          ok: true,
          message: "Preferences saved. Your passwordless daily dispatch is active.",
          prefs_token: prefsToken
        },
        { headers: CORS_HEADERS }
      );
    }

    // 2. GET /api/unsubscribe (1-click RFC 8058 unsubscribe)
    if (url.pathname === "/api/unsubscribe") {
      const email = (url.searchParams.get("email") || "").trim().toLowerCase();
      const token = url.searchParams.get("token") || "";
      const expected = await signToken(email, "unsub", env.AHBAR_HMAC_SECRET);
      if (!email || token !== expected) {
        return Response.json({ ok: false, message: "Invalid unsubscribe signature" }, { status: 403, headers: CORS_HEADERS });
      }
      if (env.DB) {
        await env.DB.prepare("DELETE FROM subscribers WHERE email = ?1").bind(email).run();
      }
      return Response.json({ ok: true, message: `Unsubscribed ${email} from Ahbar.` }, { headers: CORS_HEADERS });
    }

    // 3. POST /api/vote (Signal / Noise item feedback)
    if (request.method === "POST" && url.pathname === "/api/vote") {
      const body = await request.json();
      const itemId = (body.item_id || "").trim();
      const vote = body.vote === "noise" ? "noise" : "signal";
      const ip = request.headers.get("CF-Connecting-IP") || "anon";
      const voterHash = await signToken(ip, itemId, env.AHBAR_HMAC_SECRET);

      if (env.DB && itemId) {
        await env.DB.prepare(
          `INSERT INTO item_votes (item_id, vote, voter_hash)
           VALUES (?1, ?2, ?3)
           ON CONFLICT(item_id, voter_hash) DO UPDATE SET vote = excluded.vote`
        )
          .bind(itemId, vote, voterHash)
          .run();
      }
      return Response.json({ ok: true, item_id: itemId, vote }, { headers: CORS_HEADERS });
    }

    return Response.json({ service: "ahbar-edge-api", status: "ok" }, { headers: CORS_HEADERS });
  }
};
