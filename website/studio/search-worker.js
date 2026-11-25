// @ts-check
// The series search's worker (see command.js's finder): its first message, what the search is built from ({ catalog,
// dated }), builds it, posting { ready } or { broken }; each one after is a question, answered once it's built. Its
// imports are relative, so they stay in the release it was served from (a worker doesn't see the page's import map).
import { seriesSearch } from "./series-search.js";

/** @type {ReturnType<typeof seriesSearch> | null} */
let engine = null;
onmessage = async ({ data }) => {
  if (!engine) {
    engine = seriesSearch(data.catalog, data.dated);
    return engine.then(() => postMessage({ ready: true }), (error) => postMessage({ broken: String(error) }));
  }
  try {
    postMessage((await engine)(data));
  } catch (error) {
    postMessage({ failed: String(error) });
  }
};
