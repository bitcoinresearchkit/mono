// @ts-check
// Studio's workers start at the release's second address (`/w/<release>/`: crates/bitview_website), so their modules,
// imported relative to them, have addresses of their own. In WebKit, a page's module whose address a worker has also
// loaded comes from the network on each reload after (a worker's imports of the page's own client, ~0.4 MB, did).
// A folder website has no release: there, the page's address.

/** A module worker from its file's URL (beside the caller's: `new URL("./x.js", import.meta.url)`). @param {URL} url */
export function startWorker(url) {
  if (url.pathname.startsWith("/v/")) url.pathname = `/w/${url.pathname.slice(3)}`;
  return new Worker(url, { type: "module" });
}
