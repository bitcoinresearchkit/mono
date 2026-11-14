// @ts-check
// Studio's presets as this browser has them, read from the page's first moment: the head runs this before the app,
// so they're made in a worker while the app's own code still downloads, and a preset's address opens without
// waiting for them after it.

/** @import { Presets } from "./presets.js" */

const [presetsUrl, clientUrl, factsUrl] = ["/studio/presets.js", "/modules/bitview-client/index.js", "/studio/facts.js"].map((path) => new URL(path, location.origin).href);

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
 * facts: kept in this browser by what made them (presets.js and the modules it imports, by the hash seo.mjs writes
 * of them, presets.hash; the client), read back at once on a reload, the chain's facts they were made by kept
 * beside them and looked at again behind (made anew for the next start when they've changed: a pool's first block,
 * a halving, a new year); made in a worker, the facts read there too (off the page's thread: Firefox took a fifth of
 * a second), when either changed. Without the Cache API (Lockdown Mode's Safari: nothing to keep them in), made in
 * the worker at each start all the same; without a worker, made here as ever.
 * @returns {Promise<Presets>}
 */
async function read() {
  try {
    if (typeof caches === "undefined") return JSON.parse((await made()).text);
    const [store, texts] = await Promise.all([caches.open("bitview-studio:presets"), Promise.all([new URL("/studio/presets.hash", location.origin).href, clientUrl].map((url) => fetch(url).then((response) => (response.ok ? response.text() : Promise.reject(new Error(`${url}: ${response.status}`))))))]);
    const hash = [...new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(texts.join("\0"))))].map((byte) => byte.toString(16).padStart(2, "0")).join("");
    const key = new URL(`presets-${hash}.json`, location.origin).href, kept = await store.match(key);
    // (Kept alone, those made from anything else gone, with the facts they were made by; made without them (they
    // didn't come), not kept, nor anything gone: their next start asks for them again.)
    const keep = (/** @type {{ text: string, said: string }} */ result) => (result.said ? Promise.all([...(store.keys ? [store.keys().then((keys) => Promise.all(keys.filter((request) => request.url !== key).map((request) => store.delete(request))))] : []), store.put(key, new Response(result.text, { headers: { "content-type": "application/json", "x-facts": result.said } }))]) : undefined);
    if (kept) {
      // (Looked at again behind, once the page has started: kept anew if the chain's facts have changed.)
      setTimeout(() => made(kept.headers.get("x-facts") ?? "").then((result) => result && keep(result)).catch(() => {}), 5000);
      return JSON.parse(await kept.text());
    }
    const result = await made();
    void keep(result)?.catch(() => {});
    return JSON.parse(result.text);
  } catch {
    const [{ studioPresets }, { BitviewClient }, { chainFacts }] = await Promise.all([import(presetsUrl), import(clientUrl), import(factsUrl)]);
    return studioPresets(new BitviewClient({ baseUrl: location.origin }), await chainFacts(location.origin).catch(() => undefined));
  }
}

/**
 * Made in a worker, by the chain's facts it reads: as JSON, and those facts as text (see factsKey; none, ""). Given
 * the facts kept ones were made by (`kept`), made only if the chain's now are others: else undefined; given none,
 * always some.
 * @overload @returns {Promise<{ text: string, said: string }>}
 * @overload @param {string} kept @returns {Promise<{ text: string, said: string } | undefined>}
 * @param {string | null} [kept]
 */
function made(kept = null) {
  return new Promise((resolve, reject) => {
    const source = URL.createObjectURL(new Blob([`(${maker})(...${JSON.stringify([presetsUrl, clientUrl, factsUrl, location.origin, kept])});`], { type: "text/javascript" }));
    let worker;
    try {
      worker = new Worker(source, { type: "module" });
    } catch (error) {
      URL.revokeObjectURL(source);
      return reject(error);
    }
    const done = () => (worker.terminate(), URL.revokeObjectURL(source));
    worker.onmessage = ({ data }) => (done(), data.error ? reject(new Error(data.error)) : resolve(data.text === undefined ? undefined : data));
    worker.onerror = (event) => (done(), reject(new Error(event.message || "The presets' worker failed")));
  });
}

/**
 * The presets, made in a worker (see made): its whole source, its arguments written in. Reads the chain's facts (none,
 * if they don't come) as its modules load; given kept presets' facts (`kept`, as text: see factsKey), posts nothing
 * made if the facts are none or the same. Else posts them as JSON, and their facts as text.
 * @param {string} presetsUrl @param {string} clientUrl @param {string} factsUrl @param {string} origin @param {string | null} kept
 */
async function maker(presetsUrl, clientUrl, factsUrl, origin, kept) {
  try {
    const factsModule = import(factsUrl), facts = factsModule.then(({ chainFacts }) => chainFacts(origin)).catch(() => undefined);
    const [{ studioPresets }, { BitviewClient }, { factsKey }, come] = await Promise.all([import(presetsUrl), import(clientUrl), factsModule, facts]);
    const said = factsKey(come);
    if (kept !== null && (!said || said === kept)) return postMessage({});
    postMessage({ text: JSON.stringify(studioPresets(new BitviewClient({ baseUrl: origin }), come)), said });
  } catch (error) {
    postMessage({ error: String(error) });
  }
}
