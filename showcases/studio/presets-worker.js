// @ts-check
// The dev page's presets: the local website's charts, made off the page (its modules read a page as they load: here,
// a stub of one), posted back once. The built page has them in it instead, from the live website.
import { websitePresets } from "./presets.js";

websitePresets(new URL("../../website/", import.meta.url).href).then(
  (presets) => postMessage({ presets }),
  (error) => postMessage({ error: String(error?.message ?? error) }),
);
