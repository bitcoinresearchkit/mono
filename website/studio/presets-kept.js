// @ts-check
// Studio's presets as this browser has them, read from the page's first moment: the head runs this before the app,
// so they're made in a worker while the app's own code still downloads, and a preset's address opens without
// waiting for them after it.

import { startWorker } from "/studio/workers.js";

/** @import { Presets } from "./presets.js" */

/**
 * Failed before it is asked for: let go of, read anew.
 * @type {Promise<Presets> | null}
 */
let started = read();
started.catch(() => (started = null));

export function presetsKept() {
  const reading = started ?? read();
  started = null;
  return reading;
}

/**
 * Kept in this browser by what made them (presets.js, facts.js and their imports by presets.hash; the client by its
 * HASH stamp), the
 * chain's facts kept beside them and looked at again behind. Made in a worker so the page's thread is free (Firefox
 * took a fifth of a second).
 * Without the Cache API (Lockdown Mode's Safari), made at each start.
 * @returns {Promise<Presets>}
 */
async function read() {
  try {
    if (typeof caches === "undefined") return JSON.parse((await made()).text);
    const [store, hash] = await Promise.all([caches.open("bitview-studio:presets"), fetch(import.meta.resolve("/studio/presets.hash")).then((response) => (response.ok ? response.text() : Promise.reject(new Error(`/studio/presets.hash: ${response.status}`))))]);
    const key = new URL(`presets-${hash.trim()}.json`, location.origin).href, kept = await store.match(key);
    // Kept alone, with the facts they were made by; made without them (they didn't come) they aren't kept.
    const keep = (/** @type {{ text: string, said: string, client: string }} */ result) => (result.said ? Promise.all([...(store.keys ? [store.keys().then((keys) => Promise.all(keys.filter((request) => request.url !== key).map((request) => store.delete(request))))] : []), store.put(key, new Response(result.text, { headers: { "content-type": "application/json", "x-facts": result.said, "x-client": result.client } }))]) : undefined);
    // Only a kept result asks for the page's client, so a first visit never loads it early.
    if (kept && kept.headers.get("x-client") === (await import("/modules/bitview-client/index.js")).BitviewClient.HASH) {
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
 * Only made if the chain's facts now differ from `kept`'s, else undefined; given none, always made.
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
