// @ts-check
// Bitview's feedback inbox: a Cloudflare Worker on its own hostname, so the servers stay stateless and every node's
// pages send to one place. Its one route is `POST /`. What it keeps is the row in `schema.sql`, and nothing else:
// no address, no user agent, no time of day (a date alone can't be matched against a CDN's logs).
//
// Every failed check answers before the database is touched. Nothing reads the inbox from here: the maintainer's job
// reads it through Cloudflare's own API, so this hostname has no read or admin route to attack.

/** The most a body may weigh, read as it arrives (a chunked request states no length). */
const MAX_BODY = 32_768;
const MAX_MESSAGE = 4_000;
/** Unread messages the inbox holds before it refuses more: the job empties it. */
const MAX_UNREAD = 500;

/** The app's folder alone: `/explorer/address/bc1...` would name an address. */
const PAGE = /^\/(?:[a-z0-9_-]{1,40}\/)?$/;
/** The exact address, masked by the page: on this site only (not `//elsewhere`), printable ASCII. */
const LINK = /^(?:\/(?![/\\])[\x21-\x7e]{0,5999})?$/;
const VERSION = /^[\w-]{0,64}$/;
const BROWSER = /^(?:chromium|firefox|safari|other)-(?:phone|desk)$/;
/** What's sent so a report can be reproduced, each by its exact shape. One that doesn't fit is kept empty, never a reason
 * to lose the note. */
const CONTEXT = {
  build: /^[\w.+-]{0,40}$/,
  os: /^(?:ios|android|macos|windows|linux|other)$/,
  major: /^\d{0,4}$/,
  viewport: /^\d{1,5}x\d{1,5}$/,
  scale: /^\d{1,2}(?:\.\d)?$/,
  theme: /^(?:light|dark)$/,
  mode: /^(?:app|browser)$/,
};
const MAX_ERRORS = 5;
const MAX_ERROR = 300;

/** Everything invisible goes: controls, formats (bidi overrides, zero-widths, Unicode tags), private and unassigned
 * code points, line and paragraph separators and variation selectors, the carriers of text meant for a model and not
 * for the eye. A joined emoji comes apart, harmlessly. */
const INVISIBLE = /[^\P{C}\n\t]|[\p{Zl}\p{Zp}\p{Variation_Selector}]/gu;

/** @param {unknown} text */
const clean = (text) => (typeof text === "string" ? text.toWellFormed().replaceAll("\r\n", "\n").normalize("NFC").replace(INVISIBLE, "").trim() : "");

/** What's counted against an address: its /64 on IPv6, where one sender holds a whole block of them. Here for the moment
 * it's counted, in the edge's memory; this is the one read of it. @param {string} ip */
function sender(ip) {
  if (!ip.includes(":")) return ip;
  const [head, tail = ""] = ip.split("::"), left = head ? head.split(":") : [], right = tail ? tail.split(":") : [];
  return [...left, ...Array(Math.max(0, 8 - left.length - right.length)).fill("0"), ...right].slice(0, 4).join(":");
}

/** @param {number} status */
const answer = (status) => new Response(null, { status, headers: { "access-control-allow-origin": "*" } });

/** @param {Request} request The body as text, or null past the limit. */
async function read(request) {
  if (!request.body) return "";
  const reader = request.body.getReader(), decoder = new TextDecoder();
  let text = "", size = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) return text + decoder.decode();
    size += value.length;
    if (size > MAX_BODY) return reader.cancel(), null;
    text += decoder.decode(value, { stream: true });
  }
}

/** The context kept: only the known fields, each checked, the errors one line each, at most a few. @param {any} raw */
function context(raw) {
  const kept = Object.fromEntries(Object.entries(CONTEXT).map(([name, shape]) => [name, typeof raw?.[name] === "string" && shape.test(raw[name]) ? raw[name] : ""]));
  const errors = (Array.isArray(raw?.errors) ? raw.errors : []).map((/** @type {unknown} */ error) => clean(error).replaceAll(/\s+/g, " ").slice(0, MAX_ERROR)).filter(Boolean).slice(0, MAX_ERRORS);
  return JSON.stringify({ ...kept, errors });
}

/** @param {string} text @returns {any} */
function parse(text) {
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

export default {
  /** @param {Request} request @param {any} env */
  async fetch(request, env) {
    if (request.method !== "POST" || new URL(request.url).pathname !== "/") return answer(404);
    const { success } = await env.LIMIT.limit({ key: sender(request.headers.get("cf-connecting-ip") ?? "") });
    if (!success) return answer(429);

    const text = await read(request);
    if (text === null) return answer(413);
    const body = parse(text);
    const message = clean(body?.message);
    const { page, link, version, browser } = body ?? {};
    if (!message || message.length > MAX_MESSAGE) return answer(400);
    if (typeof page !== "string" || !PAGE.test(page)) return answer(400);
    if (typeof link !== "string" || !LINK.test(link)) return answer(400);
    if (typeof version !== "string" || !VERSION.test(version)) return answer(400);
    if (typeof browser !== "string" || !BROWSER.test(browser)) return answer(400);
    const kept = context(body?.context);

    // (A repeat is looked for first, as an insert that fails still uses up an id. The cap from the ids' two ends, two rows
    // read, not a count of every row on each request: ids are never reused and only ever deleted oldest first, so the
    // span is the rows there are.)
    const { meta } = await env.DB.prepare(
      `INSERT INTO feedback (day, page, link, version, browser, context, message)
       SELECT ?, ?, ?, ?, ?, ?, ? WHERE NOT EXISTS (SELECT 1 FROM feedback WHERE page = ? AND message = ?)
       AND COALESCE((SELECT id FROM feedback ORDER BY id DESC LIMIT 1) - (SELECT id FROM feedback ORDER BY id LIMIT 1) + 1, 0) < ?`,
    ).bind(new Date().toISOString().slice(0, 10), page, link, version, browser, kept, message, page, message, MAX_UNREAD).run();
    if (meta.changes) return answer(204);
    // (An exact repeat of something unread is simply already here; else the inbox is full, and that's said.)
    return answer((await env.DB.prepare("SELECT 1 FROM feedback WHERE page = ? AND message = ?").bind(page, message).first()) ? 204 : 503);
  },
};
