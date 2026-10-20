// The retired website's service worker, at its address: a browser that installed it updates to this one, which clears
// its caches, unregisters itself and reloads the pages it held, so they reach the showcases (it sent every page to `/`).
self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (event) =>
  event.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.map((key) => caches.delete(key))))
      .then(() => self.registration.unregister())
      .then(() => self.clients.matchAll({ type: "window" }))
      .then((clients) => clients.forEach((client) => client.navigate(reloadable(client.url)))),
  ),
);

/** A page's address to load it anew: one with a fragment (a Studio link's chart) would only move within the page, so
 * its query gains an empty part, which no page reads. */
function reloadable(href) {
  const url = new URL(href);
  if (url.hash) url.search = url.search ? `${url.search}&` : "?";
  return url.href;
}
