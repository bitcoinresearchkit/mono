const CACHE = "v1";
const ROOT = "/";
const API = "/api";

// Match hashed filenames: name.abc12345.js/mjs/css
const HASHED_RE = /\.[0-9a-f]{8}\.(js|mjs|css)$/;

const sw = /** @type {ServiceWorkerGlobalScope} */ (/** @type {unknown} */ (self));

const offline = () =>
  new Response("Offline", {
    status: 503,
    headers: { "Content-Type": "text/plain" },
  });

/**
 * @param {Request | string} req
 */
function fetchAndCache(req) {
  return fetch(req).then((res) => {
    if (res.ok) {
      const clone = res.clone();
      caches.open(CACHE).then((c) => c.put(req, clone));
    }
    return res;
  });
}

sw.addEventListener("install", (e) => {
  e.waitUntil(
    caches
      .open(CACHE)
      .then((c) => c.addAll([ROOT]))
      .then(() => sw.skipWaiting()),
  );
});

sw.addEventListener("activate", (e) => {
  e.waitUntil(
    Promise.all([
      sw.clients.claim(),
      caches
        .keys()
        .then((keys) =>
          Promise.all(
            keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)),
          ),
        ),
    ]),
  );
});

sw.addEventListener("fetch", (event) => {
  const req = event.request;
  const url = new URL(req.url);

  // Only handle same-origin GET requests
  if (req.method !== "GET" || url.origin !== location.origin) return;

  const path = url.pathname;

  // Bypass API and redirects
  if (path.startsWith(API)) return;

  // Navigation: network-first for shell
  if (req.mode === "navigate") {
    event.respondWith(
      fetchAndCache(ROOT).catch(() =>
        caches.match(ROOT).then((c) => c || offline()),
      ),
    );
    return;
  }

  // Hashed assets: cache-first (immutable)
  if (HASHED_RE.test(path)) {
    event.respondWith(
      caches
        .match(req)
        .then((cached) => cached || fetchAndCache(req))
        .catch(() => offline()),
    );
    return;
  }

  // Other: network-first with cache fallback
  // SPA routes (no extension) fall back to ROOT, static assets get 503
  const isStatic = path.includes(".") && !path.endsWith(".html");
  event.respondWith(
    fetchAndCache(req).catch(() =>
      caches
        .match(req)
        .then(
          (cached) =>
            cached ||
            (isStatic
              ? offline()
              : caches.match(ROOT).then((c) => c || offline())),
        ),
    ),
  );
});
