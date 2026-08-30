import { writeFile } from "node:fs/promises";

const style = {
  getPropertyValue: () => "",
  setProperty: () => {},
  removeProperty: () => {},
};
const element = () => ({
  style,
  dataset: {},
  classList: { add() {}, remove() {}, toggle() {} },
  append() {},
  appendChild() {},
  addEventListener() {},
  removeEventListener() {},
  setAttribute() {},
  removeAttribute() {},
  querySelector: () => null,
  querySelectorAll: () => [],
  children: [],
});

Object.assign(globalThis, {
  location: { href: "http://localhost/" },
  window: globalThis,
  matchMedia: () => ({ matches: false, addEventListener() {} }),
  getComputedStyle: () => style,
  document: {
    documentElement: element(),
    body: element(),
    createElement: element,
    getElementById: element,
    querySelector: () => element(),
    querySelectorAll: () => [],
  },
  localStorage: { getItem: () => null, setItem() {}, removeItem() {} },
});

const [{ createPartialOptions }, { stringToId }] = await Promise.all([
  import("../website/scripts/options/partial.js"),
  import("../website/scripts/utils/format.js"),
]);

/** @type {Array<[title: string, href: string, blank: boolean]>} */
const entries = [];

/**
 * @param {ReturnType<typeof createPartialOptions>} tree
 * @param {string[]} parentPath
 */
function visit(tree, parentPath = []) {
  for (const option of tree) {
    const path = [...parentPath, stringToId(option.name)];
    if ("tree" in option) {
      visit(option.tree, path);
    } else {
      entries.push([
        option.title || option.name,
        "url" in option
          ? option.url()
          : "kind" in option && option.kind === "explorer"
            ? "/"
            : `/${path.join("/")}`,
        "url" in option,
      ]);
    }
  }
}

visit(createPartialOptions());

let previousHref = "";
const packed = entries.map(([title, href, blank]) => {
  let shared = 0;
  while (
    shared < previousHref.length &&
    shared < href.length &&
    previousHref[shared] === href[shared]
  ) {
    shared++;
  }
  previousHref = href;
  return blank
    ? /** @type {const} */ ([shared, href.slice(shared), title, 1])
    : /** @type {const} */ ([shared, href.slice(shared), title]);
});

const target = new URL(
  "../website/scripts/options/search-index.json",
  import.meta.url,
);
await writeFile(target, `${JSON.stringify(packed)}\n`);
console.log(
  `Generated ${entries.length.toLocaleString("en-US")} search entries`,
);
