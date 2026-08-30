import { QuickMatch } from "../modules/quickmatch-js/0.5.0/src/index.js";

const index = loadIndex();

async function loadIndex() {
  const response = await fetch(
    new URL("../options/search-index.json", import.meta.url),
  );
  if (!response.ok) throw new Error("Failed to load chart search index");
  const packed =
    /** @type {Array<[shared: number, suffix: string, title: string, blank?: 1]>} */ (
      await response.json()
    );
  let previousHref = "";
  const entries = packed.map(([shared, suffix, title, blank]) => {
    const href = previousHref.slice(0, shared) + suffix;
    previousHref = href;
    return /** @type {[string, string, boolean]} */ ([
      title,
      href,
      Boolean(blank),
    ]);
  });
  const haystack = entries.map(([title]) => title.toLowerCase());
  return {
    matcher: new QuickMatch(haystack),
    titleToLink: new Map(
      entries.map(([title, href, blank]) => [
        title.toLowerCase(),
        /** @type {[string, boolean]} */ ([href, blank]),
      ]),
    ),
  };
}

self.addEventListener("message", async (event) => {
  const { id, needle } = /** @type {{ id: number, needle: string }} */ (
    event.data
  );

  try {
    const { matcher, titleToLink } = await index;
    const results = matcher.matches(needle).map((title) => {
      const [href, blank] = titleToLink.get(title) || ["", false];
      return /** @type {[string, string, boolean]} */ ([title, href, blank]);
    });
    self.postMessage({ id, results });
  } catch (error) {
    self.postMessage({
      id,
      error: error instanceof Error ? error.message : String(error),
    });
  }
});
