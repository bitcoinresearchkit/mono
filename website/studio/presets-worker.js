// @ts-check
// Studio's presets made off the page's thread (see presets-kept.js's made), by the chain's facts, read here as the
// modules load. Told the facts kept presets were made by (as text: see factsKey; none, null), posts nothing made if
// the chain's are none or the same; else the presets as JSON, their facts as text and the client's stamp. Its imports
// are relative, so they stay in the release it was served from (a worker doesn't see the page's import map).

const facts = import("./facts.js"), come = facts.then(({ chainFacts }) => chainFacts(location.origin)).catch(() => undefined);

onmessage = async ({ data: kept }) => {
  try {
    const [{ studioPresets }, { BitviewClient }, { factsKey }, read] = await Promise.all([import("./presets.js"), import("../modules/bitview-client/index.js"), facts, come]);
    const said = factsKey(read);
    if (kept !== null && (!said || said === kept)) return postMessage({});
    postMessage({ text: JSON.stringify(studioPresets(new BitviewClient(location.origin), read)), said, client: BitviewClient.HASH });
  } catch (error) {
    postMessage({ error: String(error) });
  }
};
