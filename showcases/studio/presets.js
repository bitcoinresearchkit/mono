// @ts-check
/**
 * The website's charts as the Studio would start them: its options tree (the local website, or the live one the build
 * fetches) run where there's no page (a worker, Node), each chart turned into the Studio's panes and layers. Shared by
 * the dev page (presets-worker.js, from the local website) and the build (build.mjs, from the live one).
 *
 * A preset: `p` its place in the website's tree (its groups, then its own name), `t` its title, `a` its top pane's
 * layers (under the price, added by the Studio as the website adds it), `b` its bottom pane's (the website's first unit).
 * A layer: `s` its series' name, `k` its type, `c` its color (`c2` under its base), `l` its label, `h` hidden (the website
 * shows it off), `d` its dash, `z` its base, `v` its levels (the website's constant lines in its pane).
 * @typedef {{ s: string, k?: string, c?: string, c2?: string, l?: string, h?: 1, d?: number, z?: number, v?: number[] }} PresetLayer
 * @typedef {{ p: string[], t: string, a: PresetLayer[], b: PresetLayer[] }} Preset
 */

/** The browser globals the website's modules read as they load, stubbed where there are none. */
function stubPage() {
  const noop = () => {};
  const g = /** @type {any} */ (globalThis);
  const element = () => ({ style: {}, dataset: {}, classList: { add: noop, remove: noop, toggle: noop, contains: () => false }, append: noop, setAttribute: noop, addEventListener: noop });
  g.window ??= g;
  g.matchMedia ??= () => ({ matches: true, addEventListener: noop, removeEventListener: noop, addListener: noop });
  g.localStorage ??= { getItem: () => null, setItem: noop, removeItem: noop };
  g.document ??= { getElementById: () => null, querySelector: () => null, querySelectorAll: () => [], createElement: element, documentElement: element(), head: element(), body: element(), addEventListener: noop };
  g.history ??= { replaceState: noop, pushState: noop };
  // (Its client is made against the page's address; nothing is asked of it.)
  g.location ??= { href: "https://bitview.space/", origin: "https://bitview.space", pathname: "/", search: "", hash: "" };
  // A color reads its CSS variable: here, the variable's own name ("--red"), which the website passes through as is.
  g.getComputedStyle ??= () => ({ getPropertyValue: (/** @type {string} */ name) => name });
  g.requestAnimationFrame ??= (/** @type {() => void} */ run) => setTimeout(run, 0);
}

/** The website's series types, as the Studio's. */
const TYPES = { Line: "line", Baseline: "baseline", Histogram: "histogram", Dots: "dots", DotsBaseline: "dots", Candlestick: "candles", Price: "candles" };

/**
 * Every chart of the website whose root is at `base` (a URL ending in "/", where its `scripts/` are), as presets.
 * @param {string} base @returns {Promise<Preset[]>}
 */
export async function websitePresets(base) {
  stubPage();
  const { createPartialOptions } = await import(`${base}scripts/options/partial.js`);
  /** A website color's name in the shared palette ("ink" for the page's own), or none. */
  const colorOf = (/** @type {any} */ color) => {
    const name = typeof color === "function" ? String(color()).replace(/^--/, "") : "";
    return name === "color" ? "ink" : /^[a-z]+$/.test(name) && !["transparent", "background-color", "border-color", "off-border-color"].includes(name) ? name : undefined;
  };
  /** A blueprint's series: the usd one of a price pattern (the website's default unit up top). */
  const seriesOf = (/** @type {any} */ series) => (series?.name ? series : series?.usd);
  /** A constant line's value (the website draws these as series of `constant_` paths). */
  const constantOf = (/** @type {any} */ blueprint) => {
    const by = blueprint.series?.by;
    for (const key in by) return by[key]?.path?.includes("constant_") ? Number.parseFloat(blueprint.title) : NaN;
    return NaN;
  };
  /** A pane's blueprints as layers: its constant lines as the levels of its first layer. */
  const layersOf = (/** @type {any[]} */ blueprints) => {
    /** @type {PresetLayer[]} */
    const layers = [];
    /** @type {number[]} */
    const levels = [];
    for (const blueprint of blueprints) {
      const level = constantOf(blueprint);
      if (Number.isFinite(level)) {
        levels.push(level);
        continue;
      }
      // (A series without days, by block only, the Studio can't show.)
      const series = seriesOf(blueprint.series), s = series?.name;
      if (!s || (series.by && !("day1" in series.by))) continue;
      const [c, c2] = Array.isArray(blueprint.colors) ? blueprint.colors.map(colorOf) : Array.isArray(blueprint.color) ? blueprint.color.map(colorOf) : [colorOf(blueprint.color)];
      /** @type {PresetLayer} */
      const layer = { s };
      const k = TYPES[/** @type {keyof typeof TYPES} */ (blueprint.type ?? "Line")];
      if (k && k !== "line") layer.k = k;
      if (c) layer.c = c;
      if (c2) layer.c2 = c2;
      if (blueprint.title) layer.l = blueprint.title;
      if (blueprint.defaultActive === false) layer.h = 1;
      if (Number.isFinite(blueprint.options?.lineStyle) && blueprint.options.lineStyle) layer.d = blueprint.options.lineStyle;
      if (Number.isFinite(blueprint.options?.baseValue?.price) && blueprint.options.baseValue.price) layer.z = blueprint.options.baseValue.price;
      layers.push(layer);
    }
    if (levels.length && layers[0]) layers[0].v = [...new Set(levels)];
    return layers;
  };
  /** @type {Preset[]} */
  const presets = [];
  const walk = (/** @type {any[]} */ nodes, /** @type {string[]} */ path) => {
    for (const node of nodes) {
      if (node.tree) walk(node.tree, [...path, node.name]);
      else if ((node.kind ?? "chart") === "chart") {
        // The bottom pane in its first unit, as the website opens it; up top, its usd series (the price is the Studio's).
        const top = (node.top ?? []).filter((/** @type {any} */ blueprint) => blueprint.type !== "Price");
        const firstUnit = (node.bottom ?? []).find((/** @type {any} */ blueprint) => blueprint.unit)?.unit;
        const bottom = (node.bottom ?? []).filter((/** @type {any} */ blueprint) => blueprint.unit === firstUnit);
        const a = layersOf(top), b = layersOf(bottom);
        // (One whose series the Studio can't show at all: none, not the price alone.)
        if (!a.length && !b.length && (top.length || bottom.length)) continue;
        presets.push({ p: [...path, node.name], t: node.title ?? node.name, a, b });
      }
    }
  };
  // (Its root's "Charts" group is where they all are.)
  const root = createPartialOptions();
  walk(root.find((/** @type {any} */ node) => node.name === "Charts")?.tree ?? root, []);
  return presets;
}
