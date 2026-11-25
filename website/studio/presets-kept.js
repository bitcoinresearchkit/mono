// @ts-check
// Studio's presets as this browser has them, read from the page's first moment: the head runs this before the app,
// so they're made in a worker while the app's own code still downloads, and a preset's address opens without
// waiting for them after it.

import { startWorker } from "/studio/workers.js";

/** @import { Presets } from "./presets.js" */

/** The read the page started with, till it's asked for (see presetsKept): failed before that, let go of (asked, read anew). @type {Promise<Presets> | null} */
let started = read();
started.catch(() => (started = null));

/** The presets, kept or made (see read): the read the page started with, asked for first; any after it (a retry), anew. */
export function presetsKept() {
  const reading = started ?? read();
  started = null;
  return reading;
}

/**
 * The presets and their places' shown names (see studioPresets), made from presets.js and the client by the chain's
 * facts: kept in this browser by what made them (presets.js, facts.js and the modules they import, by the hash
 * seo.mjs writes of them, presets.hash; the client, by the stamp bindgen writes in it, HASH), read back at once on a
 * reload, the chain's facts they were made by kept
 * beside them and looked at again behind (made anew for the next start when they've changed: a pool's first block,
 * a halving, a new year); made in a worker, the facts read there too (off the page's thread: Firefox took a fifth of
 * a second), when either changed. Without the Cache API (Lockdown Mode's Safari: nothing to keep them in), made in
 * the worker at each start all the same; without a worker, made here as ever.
 * @returns {Promise<Presets>}
 */
async function read() {
  try {
    if (typeof caches === "undefined") return JSON.parse((await made()).text);
    const [store, hash] = await Promise.all([caches.open("bitview-studio:presets"), fetch(import.meta.resolve("/studio/presets.hash")).then((response) => (response.ok ? response.text() : Promise.reject(new Error(`/studio/presets.hash: ${response.status}`))))]);
    const key = new URL(`presets-${hash.trim()}.json`, location.origin).href, kept = await store.match(key);
    // (Kept alone, those made from anything else gone, with the facts they were made by; made without them (they
    // didn't come), not kept, nor anything gone: their next start asks for them again.)
    const keep = (/** @type {{ text: string, said: string, client: string }} */ result) => (result.said ? Promise.all([...(store.keys ? [store.keys().then((keys) => Promise.all(keys.filter((request) => request.url !== key).map((request) => store.delete(request))))] : []), store.put(key, new Response(result.text, { headers: { "content-type": "application/json", "x-facts": result.said, "x-client": result.client } }))]) : undefined);
    // (Kept, by the client's stamp they were made by (the worker's, see presets-worker.js): made by another, made anew. Only kept
    // ones ask for the page's client here, so a first visit never loads it early, beside the app's own code.)
    if (kept && kept.headers.get("x-client") === (await import("/modules/bitview-client/index.js")).BitviewClient.HASH) {
      // (Looked at again behind, once the page has started: kept anew if the chain's facts have changed.)
      setTimeout(() => made(kept.headers.get("x-facts") ?? "").then((result) => result && keep(result)).catch(() => {}), 5000);
      return JSON.parse(await kept.text());
    }
    const result = await made();
    void keep(result)?.catch(() => {});
    return JSON.parse(result.text);
  } catch (error) {
    console.error("presets-worker.js: the presets are made on the page instead", error);
    const [{ studioPresets }, { BitviewClient }, { chainFacts }] = await Promise.all([import("/studio/presets.js"), import("/modules/bitview-client/index.js"), import("/studio/facts.js")]);
    return studioPresets(new BitviewClient(location.origin), await chainFacts(location.origin).catch(() => undefined));
  }
}

/**
 * Made in a worker (presets-worker.js), by the chain's facts it reads: as JSON, and those facts as text (see factsKey;
 * none, ""). Given the facts kept ones were made by (`kept`), made only if the chain's now are others: else undefined;
 * given none, always some. With the stamp (HASH) of the client they were made by.
 * @overload @returns {Promise<{ text: string, said: string, client: string }>}
 * @overload @param {string} kept @returns {Promise<{ text: string, said: string, client: string } | undefined>}
 * @param {string | null} [kept]
 */
function made(kept = null) {
  return new Promise((resolve, reject) => {
    const worker = startWorker(new URL("./presets-worker.js", import.meta.url)), done = () => worker.terminate();
    worker.onmessage = ({ data }) => (done(), data.error ? reject(new Error(data.error)) : resolve(data.text === undefined ? undefined : data));
    worker.onerror = (event) => (done(), reject(new Error(event.message || "The presets' worker failed")));
    worker.postMessage(kept);
  });
}
