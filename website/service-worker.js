// Replaces the retired site's worker at its address: clears caches, unregisters, reloads the pages it held.
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

/** A fragment alone would only move within the page, so the query gains an empty part, which no page reads. */
function reloadable(href) {
  const url = new URL(href);
  if (url.hash) url.search = url.search ? `${url.search}&` : "?";
  return url.href;
}
