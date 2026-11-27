// @ts-check
// Workers start at the release's second address (`/w/<release>/`), so their modules have addresses of their own: in
// WebKit, a page's module a worker has also loaded comes from the network on each reload after (its imports of the
// client, ~0.4 MB, did). A site without the release's import map has no second address: there, the page's own.
// A worker that cannot start or fails says so in the console, by its file's name; its caller decides what to do next.

/** @param {URL} url */
export function startWorker(url) {
  const name = url.pathname.slice(url.pathname.lastIndexOf("/") + 1);
  let address = url.href;
  try {
    address = import.meta.resolve(`worker/${url.pathname.slice(new URL(import.meta.resolve("/")).pathname.length)}`);
  } catch {
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
