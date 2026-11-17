// @ts-check
// The catalog walked off the page's thread (see app.js's indexCatalog): told the intervals Studio charts, posts what
// it found as JSON (read back at once: 3 ms of a phone's thread, not the walk's 180). Its imports are relative, so
// they stay in the release it was served from (a worker doesn't see the page's import map).
import { BitviewClient } from "../modules/bitview-client/index.js";
import { catalogWalk } from "./catalog-walk.js";

onmessage = ({ data: dated }) => {
  try {
    const steps = catalogWalk(new BitviewClient(location.origin).series, dated);
    let step = steps.next();
    while (!step.done) step = steps.next();
    const { index, byBlock, sharedTops } = step.value;
    postMessage({ text: JSON.stringify([[...index], [...byBlock], [...sharedTops]]) });
  } catch (error) {
    postMessage({ error: String(error) });
  }
};
