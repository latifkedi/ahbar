// Ahbar — Cloudflare Worker + D1 Edge API for Passwordless Subscriptions & Signal/Noise Votes
// Mirrors crates/api so you can run either self-hosted Rust or zero-cost Cloudflare Workers.

const CORS_HEADERS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, Authorization",
  "X-Content-Type-Options": "nosniff"
};

function isValidEmail(raw) {
  const s = (raw || "").trim();
  if (s.length < 5 || s.length > 254 || /\s/.test(s)) return false;
  const parts = s.split("@");
  if (parts.length !== 2) return false;
  const [local, domain] = parts;
  if (!local || local.length > 64 || local.startsWith(".") || local.endsWith(".")) return false;
  if (domain.length < 3 || !domain.includes(".") || domain.startsWith(".") || domain.endsWith(".")) return false;
  return /^[A-Za-z0-9.-]+$/.test(domain);
}

function constantTimeEqual(a, b) {
  if (typeof a !== "string" || typeof b !== "string" || a.length !== b.length) {
    return false;
  }
  let diff = 0;
  for (let i = 0; i < a.length; i++) {
    diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return diff === 0;
}

// Identical to crates/shared/src/auth.rs: HMAC-SHA256("ahbar:v1:" + lowercase_email) truncated to 16 bytes (32 hex chars)
async function signSubscriberToken(email, secret) {
  const enc = new TextEncoder();
  const key = await crypto.subtle.importKey(
    "raw",
    enc.encode(secret || "ahbar-default-dev-hmac-secret-key"),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"]
  );
  const msg = `ahbar:v1:${(email || "").trim().toLowerCase()}`;
  const sig = await crypto.subtle.sign("HMAC", key, enc.encode(msg));
  return Array.from(new Uint8Array(sig).slice(0, 16))
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
      let body;
      try {
        body = await request.json();
      } catch {
        return Response.json({ ok: false, error: "Invalid JSON" }, { status: 400, headers: CORS_HEADERS });
      }
      const email = (body.email || "").trim().toLowerCase();
      if (!isValidEmail(email)) {
        return Response.json({ ok: false, message: "Invalid email address" }, { status: 400, headers: CORS_HEADERS });
      }

      const expectedToken = await signSubscriberToken(email, env.AHBAR_HMAC_SECRET);
      const hasValidToken = typeof body.token === "string" && constantTimeEqual(body.token.trim().toLowerCase(), expectedToken);

      const mode = ["standard", "personalized", "hybrid"].includes(body.mode) ? body.mode : "standard";
      const density = ["three-per-area", "three_per_area", "compact", "comprehensive"].includes(body.density) ? body.density : "three-per-area";
      const frequency = ["daily", "weekdays_only", "weekend_only"].includes(body.frequency) ? body.frequency : "daily";
      const contentFilter = ["all", "papers_only", "code_only"].includes(body.content_filter) ? body.content_filter : "all";
      const safeAreas = Array.isArray(body.areas) ? body.areas.slice(0, 10) : ["cs", "ai", "math", "physics", "systems", "bio", "crypto", "econ", "geopolitics", "philosophy"];
      const safeTags = Array.isArray(body.tags)
        ? body.tags.filter((t) => typeof t === "string" && /^[A-Za-z0-9_-]{1,40}$/.test(t)).slice(0, 30)
        : [];
      const areasJson = JSON.stringify(safeAreas);
      const tagsJson = JSON.stringify(safeTags);

      if (env.DB) {
        const existing = await env.DB.prepare("SELECT email FROM subscribers WHERE email = ?1").bind(email).first();
        if (existing && !hasValidToken) {
          return Response.json(
            {
              ok: true,
              email,
              requires_magic_link: true,
              message: "This address is already subscribed. Use the magic link in your email to modify preferences."
            },
            { headers: CORS_HEADERS }
          );
        }

        await env.DB.prepare(
          `INSERT INTO subscribers (email, mode, density, frequency, content_filter, areas_json, tags_json, verified, updated_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))
           ON CONFLICT(email) DO UPDATE SET
             mode = excluded.mode,
             density = excluded.density,
             frequency = excluded.frequency,
             content_filter = excluded.content_filter,
             areas_json = excluded.areas_json,
             tags_json = excluded.tags_json,
             verified = excluded.verified,
             updated_at = datetime('now')`
        )
          .bind(email, mode, density, frequency, contentFilter, areasJson, tagsJson, hasValidToken ? 1 : 0)
          .run();
      }

      return Response.json(
        {
          ok: true,
          email,
          message: "Preferences saved. Your passwordless daily dispatch is active."
        },
        { headers: CORS_HEADERS }
      );
    }

    // 2. GET or POST /api/unsubscribe (1-click RFC 8058 unsubscribe)
    if (url.pathname === "/api/unsubscribe" && (request.method === "GET" || request.method === "POST")) {
      let email = (url.searchParams.get("email") || "").trim().toLowerCase();
      let token = (url.searchParams.get("token") || "").trim().toLowerCase();
      if (request.method === "POST" && (!email || !token)) {
        try {
          const body = await request.json();
          email = (body.email || email).trim().toLowerCase();
          token = (body.token || token).trim().toLowerCase();
        } catch {
          // ignore non-JSON POST body when query params are used
        }
      }
      const expected = await signSubscriberToken(email, env.AHBAR_HMAC_SECRET);
      if (!isValidEmail(email) || !constantTimeEqual(token, expected)) {
        return Response.json({ ok: false, message: "Invalid unsubscribe signature" }, { status: 403, headers: CORS_HEADERS });
      }
      if (env.DB) {
        await env.DB.prepare("DELETE FROM subscribers WHERE email = ?1").bind(email).run();
      }
      return Response.json({ ok: true, unsubscribed: true, message: `Unsubscribed ${email} from Ahbar.` }, { headers: CORS_HEADERS });
    }

    // 3. POST /api/vote (Signal / Noise item feedback)
    if (request.method === "POST" && url.pathname === "/api/vote") {
      let body;
      try {
        body = await request.json();
      } catch {
        return Response.json({ ok: false, error: "Invalid JSON" }, { status: 400, headers: CORS_HEADERS });
      }
      const itemId = typeof body.item_id === "string" ? body.item_id.trim() : "";
      if (!/^[A-Za-z0-9_-]{1,120}$/.test(itemId)) {
        return Response.json({ ok: false, error: "Invalid item_id" }, { status: 400, headers: CORS_HEADERS });
      }
      const vote = body.vote === "noise" ? "noise" : "signal";
      const ip = request.headers.get("CF-Connecting-IP") || "anon";
      const voterHash = await signSubscriberToken(`${ip}|${itemId}`, env.AHBAR_HMAC_SECRET);

      if (env.DB) {
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

    // 4. GET /feed/custom (Dynamic user-filtered Atom 1.0 feed)
    if (request.method === "GET" && url.pathname === "/feed/custom") {
      const areaFilter = url.searchParams.get("area");
      const tagFilter = url.searchParams.get("tag");
      const minScore = parseFloat(url.searchParams.get("min_score") || "0");
      const lang = url.searchParams.get("lang") === "tr" ? "tr" : "en";

      const dataUrl = `https://latifkedi.github.io/ahbar/data/latest.${lang}.json`;
      let issueData;
      try {
        const resp = await fetch(dataUrl, { cf: { cacheTtl: 300, cacheEverything: true } });
        if (!resp.ok) throw new Error("Failed to fetch latest issue data");
        issueData = await resp.json();
      } catch {
        return new Response(
          `<?xml version="1.0" encoding="utf-8"?><feed xmlns="http://www.w3.org/2005/Atom"><title>Ahbar Error</title></feed>`,
          {
            status: 502,
            headers: { "Content-Type": "application/atom+xml; charset=utf-8", ...CORS_HEADERS }
          }
        );
      }

      let items = issueData.items || [];
      if (areaFilter) {
        items = items.filter((i) => i.area === areaFilter);
      }
      if (tagFilter) {
        const tags = tagFilter.split(",").map((t) => t.trim().toLowerCase());
        items = items.filter((i) => (i.tags || []).some((t) => tags.includes(t.toLowerCase())));
      }
      if (minScore > 0) {
        items = items.filter((i) => (i.metrics?.final_score || 0) >= minScore);
      }

      function escapeXml(str) {
        return (str || "")
          .replace(/&/g, "&amp;")
          .replace(/</g, "&lt;")
          .replace(/>/g, "&gt;")
          .replace(/"/g, "&quot;")
          .replace(/'/g, "&apos;");
      }

      const feedId = `tag:latifkedi.github.io,2026:ahbar:custom:${encodeURIComponent(url.search)}`;
      const feedTitle = `Ahbar — Custom Filtered Feed (${areaFilter || "All"} / ${tagFilter || "All Tags"})`;
      const updated = issueData.generated_at || new Date().toISOString();

      let entriesXml = "";
      for (const item of items) {
        const title = lang === "tr" && item.headline_tr ? item.headline_tr : item.headline;
        const summary = lang === "tr" && item.summary_tr ? item.summary_tr : item.summary;
        const itemUrl = item.url || `https://latifkedi.github.io/ahbar/?item=${item.id}`;
        entriesXml += `  <entry>\n    <title>${escapeXml(title)}</title>\n    <link href="${escapeXml(itemUrl)}"/>\n    <id>tag:latifkedi.github.io,2026:ahbar:item:${item.id}</id>\n    <updated>${updated}</updated>\n    <summary>${escapeXml(summary)}</summary>\n    <author><name>${escapeXml(item.source_name || "Ahbar")}</name></author>\n    <category term="${escapeXml(item.area)}"/>\n  </entry>\n`;
      }

      const atomXml = `<?xml version="1.0" encoding="utf-8"?>\n<feed xmlns="http://www.w3.org/2005/Atom">\n  <title>${escapeXml(feedTitle)}</title>\n  <link href="https://latifkedi.github.io/ahbar"/>\n  <link rel="self" href="${escapeXml(url.href)}"/>\n  <id>${escapeXml(feedId)}</id>\n  <updated>${updated}</updated>\n  <author><name>Ahbar Autonomous Synthesizer</name></author>\n${entriesXml}</feed>`;

      return new Response(atomXml, {
        headers: {
          "Content-Type": "application/atom+xml; charset=utf-8",
          "Cache-Control": "public, max-age=300, s-maxage=300",
          ...CORS_HEADERS
        }
      });
    }

    return Response.json({ service: "ahbar-edge-api", status: "ok" }, { headers: CORS_HEADERS });
  }
};
