// @ts-check
// Studio's workers start at the release's second address (`/w/<release>/`: crates/bitview_website), so their modules,
// imported relative to them, have addresses of their own. In WebKit, a page's module whose address a worker has also
// loaded comes from the network on each reload after (a worker's imports of the page's own client, ~0.4 MB, did).
// A site served without the release's import map (any plain folder server) has no second address: there, the page's own.
// A worker that cannot start or fails says so in the console, by its file's name; its caller decides what to do next.

/** A module worker from its file's URL (beside the caller's: `new URL("./x.js", import.meta.url)`). @param {URL} url */
export function startWorker(url) {
  const name = url.pathname.slice(url.pathname.lastIndexOf("/") + 1);
  let address = url.href;
  try {
    address = import.meta.resolve(`worker/${url.pathname.slice(new URL(import.meta.resolve("/")).pathname.length)}`);
  } catch {
    // (No `worker/` in an import map: the page's own address, set above.)
  }
  let worker;
  try {
    worker = new Worker(address, { type: "module" });
  } catch (error) {
    console.error(`${name}: its worker could not start`, error);
    throw error;
  }
  worker.addEventListener("error", (event) => console.error(`${name}: its worker failed: ${event.message || "its script did not load"}`, event.filename ? `${event.filename}:${event.lineno}` : ""));
  worker.addEventListener("messageerror", () => console.error(`${name}: a message from its worker could not be read`));
  return worker;
}
