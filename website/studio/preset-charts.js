// @ts-check
// The presets as charts: each one's chart (as made, as edited here), its address, and their arrival.
/** @import { Preset } from "/studio/presets-edit.js" */
/** @import { PresetSearch } from "/studio/grid.js" */
/** @import { Chart, Layer } from "/studio/vocabulary.js" */
/** @import { Group } from "/studio/app.js" */
/** @import { Folder } from "/studio/vocabulary.js" */
import { ICONS } from "/studio/icons.js";
import { shownStep } from "/studio/words.js";
import { presets, presetLabels, homeAt, isPreview, chart, emptyYours, library, normalize, previews, home, started, ROUTED, tree, pendingRoute, chartWindow, atWindow, viewed, save, isDeleted, readOnly, rekey, chartAgain, histories, edits, refresh, toast, GROUP_OF, drafts, readPresetList } from "/studio/app.js";
import { plural } from "/studio/format.js";
import { hashOf, uid } from "/studio/math.js";
import { titleOf } from "/studio/looks.js";
import { STUDIO_ORIGIN, presetInfo, sectionInfo, STUDIO_TITLE, STUDIO_DESCRIPTION, presetDataset, studioApplication } from "/studio/page-info.js";
import { inSlices } from "/studio/timing.js";
import { withRest } from "/studio/later.js";
import { addressSteps } from "/studio/addresses.js";
import { placeNow } from "/studio/places-were.js";
import { validate, fromSketch, contentOf, lookFields } from "/studio/charts-read.js";
import { isNamed } from "/studio/color.js";

/** @typedef {{ name: string, at: Place }} Step */
/** @typedef {{ path: string[], key: string, name: string, shown: string[], items: (PresetNode | number)[], under: number[], icon?: string }} PresetNode */
let presetRoot = /** @type {{ for: Preset[] | null, node: PresetNode | null, byKey: Map<string, PresetNode>, search: PresetSearch | null, making: Generator<void, PresetSearch> | null }} */ ({ for: null, node: null, byKey: new Map(), search: null, making: null });
function presetTree() {
  if (!presets) return null;
  if (presetRoot.for === presets) return presetRoot.node;
  const root = /** @type {PresetNode} */ ({ path: [], key: "[]", name: "Presets", shown: [], items: [], under: [] }), byKey = new Map([[root.key, root]]);
  presets.forEach((preset, i) => {
    let node = root;
    node.under.push(i);
    for (let depth = 0; depth < preset.p.length - 1; depth++) {
      const path = preset.p.slice(0, depth + 1), key = JSON.stringify(path);
      let next = byKey.get(key);
      if (!next) {
        const name = shownStep(presetLabels, path);
        next = { path, key, name, shown: [...node.shown, name], items: [], under: [], ...(!depth && preset.s && ICONS[preset.s] ? { icon: preset.s } : {}) };
        byKey.set(key, next);
        node.items.push(next);
      }
      node = next;
      node.under.push(i);
    }
    node.items.push(i);
  });
  presetRoot = { for: presets, node: root, byKey, search: null, making: null };
  return root;
}
/** @param {Preset} preset */
const presetName = (preset) => shownStep(presetLabels, preset.p);
function presetNodeAt(/** @type {string[]} */ path) {
  return presetTree() ? presetRoot.byKey.get(JSON.stringify(path)) : undefined;
}
/** @param {string[]} path */
function placeNames(path) {
  return presetNodeAt(path)?.shown ?? path;
}
const homeName = () => ("folder" in homeAt ? folderById(homeAt.folder)?.name ?? "Library" : "previews" in homeAt ? "Previews" : "recent" in homeAt ? "Recent" : "path" in homeAt ? placeNames(homeAt.path).at(-1) ?? "Presets" : "Home");
/** @typedef {{ top: true } | { path: string[] } | { folder: string } | { previews: true } | { recent: true }} Place */
/** @returns {Place} */
const asPlace = (/** @type {any} */ at) => (typeof at?.folder === "string" ? { folder: at.folder } : Array.isArray(at?.path) ? { path: at.path.filter((/** @type {any} */ name) => typeof name === "string") } : at?.previews === true ? { previews: true } : at?.recent === true ? { recent: true } : { top: true });
const backToHome = () => (isPreview(chart) || emptyYours(chart)) && history.state?.fromHome === true && history.state.chart === chart.id;
const folderById = (/** @type {string | undefined} */ id) => library.folders.find((folder) => folder.id === id);
/**
 * A shade (a color not named) is the series' place in a family (a comparison's member, a window, a percentile): a layer
 * in one says nothing of its look, but a baseline's base.
 * @param {Preset[]} list @returns {Map<string, Partial<Layer>>}
 */
function looksOf(list) {
  const counts = new Map();
  for (const preset of list) for (const layer of preset.c.layers) {
    if (typeof layer.series !== "string" || (!isNamed(layer.color) && layer.type !== "baseline")) continue;
    const each = counts.get(layer.series) ?? new Map(), key = `${isNamed(layer.color) ? layer.color : ""}|${layer.type ?? "line"}|${layer.base ?? ""}|${layer.colors ?? ""}`;
    counts.set(layer.series, each.set(key, (each.get(key) ?? 0) + 1));
  }
  return new Map([...counts].map(([name, each]) => {
    const [color, type, base, colors] = [...each].reduce((best, entry) => (entry[1] > best[1] ? entry : best))[0].split("|");
    return [name, { ...(color ? { color } : {}), type, ...(base ? { base: Number(base) } : {}), ...(colors ? { colors: /** @type {1 | 4} */ (Number(colors)) } : {}) }];
  }));
}
// A hidden tab has no frames: it goes on at once, or a tenth of a second on.
function aFrame() {
  return document.hidden ? Promise.resolve() : new Promise((done) => (requestAnimationFrame(() => done(undefined)), setTimeout(done, 100)));
}
/**
 * Ids are as its sketch gives them (p0, l0…): the same each time, so its drawings and what's kept of it follow it.
 * @param {Preset} preset
 */
const fromPreset = (preset) => ((doc) => (normalize(doc), doc))(validate(fromSketch({ ...preset.c, title: preset.t }), true));
/**
 * Made once looked at, its id stable so its drawing is kept; the most recently looked at stay (a gallery's page of
 * them).
 * @type {Map<number, Chart>}
 */
const presetDocs = new Map();
function presetDoc(/** @type {number} */ i) {
  return keptPreset(i) ?? templateDoc(i);
}
function templateDoc(/** @type {number} */ i) {
  let doc = presetDocs.get(i);
  if (doc) presetDocs.delete(i);
  else {
    doc = Object.assign(fromPreset(/** @type {Preset[]} */ (presets)[i]), { id: `preset:${i}` });
    // A drawing kept by its place in the list is this preset's only if it says so: the list may be another now.
    if (previews.get(doc.id)?.of !== presetKey(i)) previews.delete(doc.id);
  }
  presetDocs.set(i, doc);
  for (const [old] of presetDocs) {
    if (presetDocs.size <= 200) break;
    presetDocs.delete(old);
    previews.delete(`preset:${old}`);
  }
  return doc;
}
const BASE = "/studio";
/** @param {string} hash */
const isLinkHash = (hash) => /^#(?:.*&)?(?:chart|image|c)=/.test(hash);
let addressBook = /** @type {{ for: Preset[] | null, steps: ReturnType<typeof addressSteps> | null, book: import("/studio/addresses.js").AddressBook | null }} */ ({ for: null, steps: null, book: null });
const addresses = () => {
  if (!presets) return null;
  if (addressBook.for !== presets) addressBook = { for: presets, steps: addressSteps(presets), book: null };
  while (!addressBook.book) ((step) => step.done && (addressBook.book = step.value))(/** @type {NonNullable<typeof addressBook.steps>} */ (addressBook.steps).next());
  return addressBook.book;
};
/** @param {{ timeRemaining: () => number }} deadline */
const addressesSoon = (deadline) => {
  if (!presets || (addressBook.for === presets && addressBook.book)) return;
  if (addressBook.for !== presets) addressBook = { for: presets, steps: addressSteps(presets), book: null };
  const steps = /** @type {NonNullable<typeof addressBook.steps>} */ (addressBook.steps);
  inSlices(deadline, () => ((step) => (step.done && (addressBook.book = step.value), Boolean(step.done)))(steps.next()), addressesSoon);
};
/**
 * As written: the server sends the other spellings (the slash the other way, an old /chart/ one) to it. A preset once
 * had an address per window (…/sma/200-days): that opens the preset at that window.
 * @param {string} pathname @returns {{ path: string[] } | { preset: number, window?: string } | null}
 */
const routeOf = (pathname) => addresses()?.at.get(pathname.slice(BASE.length)) ?? null;
function addressNow() {
  if (home ? !("path" in homeAt) : chart.preset === undefined) return "/";
  const book = addresses();
  if (!book) return null;
  if (home) return book.sections.get(JSON.stringify(/** @type {{ path: string[] }} */ (homeAt).path)) ?? "/";
  const i = presetIndex(chart);
  return i >= 0 ? book.presets[i] : "/";
}
const CANONICAL_ORIGIN = STUDIO_ORIGIN;
function showAddress() {
  // Once what's being done is done: a step of the history pushed after the page is drawn has the address, not the one
    // before it.
  if (addressDue) return;
  addressDue = true;
  queueMicrotask(() => ((addressDue = false), placeAddress()));
}
let addressDue = false, structuredFor = "";
function placeAddress() {
  if (document.body.hasAttribute("data-picture") || !started) return;
  const address = addressNow(), i = home ? -1 : presetIndex(chart), path = home && "path" in homeAt ? homeAt.path : null;
  const inside = path && presets ? ((items) => [...items.flatMap((item) => (typeof item === "number" ? [] : [item.name])), ...items.flatMap((item) => (typeof item === "number" ? [/** @type {Preset[]} */ (presets)[item].t] : []))])(presetNodeAt(path)?.items ?? []) : [];
  const info = i >= 0 && presets ? presetInfo({ ...presets[i], t: chart.title || presets[i].t }, presetLabels)
    : path && presets ? sectionInfo(path, presetLabels, inside)
    : { title: home && "top" in homeAt ? STUDIO_TITLE : `${home ? homeName() : titleOf(chart)} · Bitview Studio`, description: STUDIO_DESCRIPTION };
  const canonical = `${CANONICAL_ORIGIN}${address ?? (ROUTED ? location.pathname.slice(BASE.length) : "/")}`;
  const structuredKey = `${address}|${i}|${info.title}|${info.description}`;
  if (structuredKey !== structuredFor && i >= 0 && presets && !tree) withRest(placeAddress);
  else if (structuredKey !== structuredFor) {
    const json = document.querySelector('script[type="application/ld+json"]');
    if (json) json.textContent = JSON.stringify(i >= 0 && presets ? presetDataset(presets[i], presetLabels, tree, canonical) : studioApplication(canonical, info.description));
    structuredFor = structuredKey;
  }
  document.title = info.title;
  const said = info.description;
  const meta = /** @type {HTMLMetaElement} */ (document.querySelector('meta[name="description"]'));
  if (meta.content !== said) meta.content = said;
  // Made once it knows where it is: none before, rather than the root's at every address.
  let link = /** @type {HTMLLinkElement | null} */ (document.querySelector('link[rel="canonical"]'));
  if (!link) document.head.append((link = Object.assign(document.createElement("link"), { rel: "canonical" })));
  if (link.href !== canonical) link.href = canonical;
  for (const [property, content] of [["og:title", info.title], ["og:description", said], ["og:url", canonical]]) {
    let tag = /** @type {HTMLMetaElement | null} */ (document.querySelector(`meta[property="${property}"]`));
    if (!tag) {
      tag = document.createElement("meta");
      tag.setAttribute("property", property);
      document.head.append(tag);
    }
    tag.content = content;
  }
  const bar = ROUTED && !pendingRoute && !isLinkHash(location.hash) ? (home || chart.preset === undefined ? address : presets ? presetLink(chart) ?? "/" : null) : null;
  if (bar === null) return;
  const [where, look = ""] = bar.split("#"), url = `${BASE}${where}${location.search}${look && `#${look}`}`;
  // Safari refuses writes past so many in a few seconds: the address stays as it was, written again at the next rest.
  if (`${location.pathname}${location.search}${location.hash}` !== url) {
    try {
      history.replaceState(history.state, "", url);
    } catch {}
  }
}
const presetKey = (/** @type {number} */ i) => JSON.stringify(/** @type {Preset[]} */ (presets)[i].p);
let presetPlaces = /** @type {{ for: Preset[] | null, at: Map<string, number> }} */ ({ for: null, at: new Map() });
const presetIndex = (/** @type {Chart} */ doc) => {
  if (!presets || doc.preset === undefined) return -1;
  if (presetPlaces.for !== presets) presetPlaces = { for: presets, at: new Map(presets.map((preset, i) => [JSON.stringify(preset.p), i])) };
  return presetPlaces.at.get(doc.preset) ?? -1;
};
const editedPreset = (/** @type {number} */ i) => (presets ? library.presets.find((doc) => doc.preset === presetKey(i)) : undefined);
/** @param {number} i */
const presetLook = (i) => {
  const preset = /** @type {Preset[]} */ (presets)[i];
  if (!madeLooks.has(preset)) madeLooks.set(preset, ((doc) => ({ index: doc.index, range: doc.range, window: chartWindow(doc)?.window }))(freshPreset(i)));
  return /** @type {{ index: string, range: string, window?: string }} */ (madeLooks.get(preset));
};
/** @type {WeakMap<Preset, { index: string, range: string, window?: string }>} */
const madeLooks = new WeakMap();
/** @param {Chart} doc */
const windowless = (doc) => ((w) => (w ? atWindow(structuredClone(doc), w.offered[0]) : doc))(chartWindow(doc));
/**
 * Looked at aside: its view, interval (as day1: every preset opened at day1 before its own) and trailing window
 * don't count, nor pane sizes: a separator dragged is a way of looking, not an edit.
 * @param {Chart} doc
 */
const contentHash = (doc) => hashOf(contentOf(windowless({ ...doc, range: "", index: "day1", panes: doc.panes.map(({ size, ...pane }) => pane) })));
const freshPreset = (/** @type {number} */ i) => ((doc) => (normalize(doc), doc))(fromPreset(/** @type {Preset[]} */ (presets)[i]));
/** @type {WeakMap<Preset, string>} */
const templateHashes = new WeakMap();
function templateHash(/** @type {number} */ i) {
  const preset = /** @type {Preset[]} */ (presets)[i];
  let hash = templateHashes.get(preset);
  if (hash === undefined) templateHashes.set(preset, (hash = contentHash(freshPreset(i))));
  return hash;
}
const presetIdOf = (/** @type {string} */ key) => `p${hashOf(key)}`;
/**
 * Made a slice at a time in idle moments (every preset's path is hashed: no long task on a phone without a JIT); the
 * rest at once where one's asked for first.
 */
let presetIds = /** @type {{ for: Preset[] | null, at: Map<string, number>, next: number }} */ ({ for: null, at: new Map(), next: 0 });
function idsNow(/** @type {{ timeRemaining: () => number } | undefined} */ deadline = undefined) {
  const list = presets ?? [];
  if (presetIds.for !== presets) presetIds = { for: presets, at: new Map(), next: 0 };
  inSlices(deadline, () => {
    for (const end = Math.min(list.length, presetIds.next + 250); presetIds.next < end; presetIds.next++) presetIds.at.set(presetIdOf(JSON.stringify(list[presetIds.next].p)), presetIds.next);
    return presetIds.next >= list.length;
  }, idsNow);
}
const presetById = (/** @type {string} */ id) => {
  if (!presets) return -1;
  if (presetIds.for !== presets || presetIds.next < presets.length) idsNow();
  return presetIds.at.get(id) ?? -1;
};
/** @type {WeakMap<Chart, { at: number, as?: boolean, from?: Chart }>} */
const pristineSeen = new WeakMap();
// Before normalize: a newer client moving fields isn't an edit. Told only when asked (it walks the catalog).
const keptAsStored = (/** @type {Chart} */ doc) => void (doc.base && pristineSeen.set(doc, { at: doc.updated, from: structuredClone(doc) }));
function pristine(/** @type {Chart} */ doc) {
  if (!doc.base) return false;
  const seen = pristineSeen.get(doc), current = seen?.at === doc.updated ? seen : undefined;
  if (current?.as !== undefined) return current.as;
  const was = current?.from ?? doc, hash = contentHash(was);
  // Migration (stamps from before windows could be picked): as made all the same, stamped anew. Delete in 2027-11.
  if (hash !== doc.base && doc.base === hashOf(contentOf({ ...was, range: "", index: "day1" }))) doc.base = hash;
  const as = hash === doc.base;
  pristineSeen.set(doc, { at: doc.updated, as });
  return as;
}
/** @param {Chart} doc */
const presetEdited = (doc) => doc.preset !== undefined && !pristine(doc);
/** @param {Chart} doc */
function presetLink(doc) {
  const i = presetIndex(doc);
  if (i < 0 || presetEdited(doc)) return null;
  const look = lookFields(doc, presetLook(i));
  return `${/** @type {NonNullable<ReturnType<typeof addresses>>} */ (addresses()).presets[i]}${look && `#${look}`}`;
}
const keptPreset = (/** @type {number} */ i, had = editedPreset(i)) => (had && (!pristine(had) || had.base === templateHash(i)) ? had : undefined);
// Never edited: stamped 0, so any edit's copy (here, another tab's) wins over it.
function presetChart(/** @type {number} */ i) {
  const key = presetKey(i), had = editedPreset(i), id = presetIdOf(key), kept = keptPreset(i);
  if (kept) return kept;
  const doc = /** @type {Chart} */ ({ ...freshPreset(i), id, preset: key, updated: (library.deleted[id] ?? -1) + 1 });
  doc.base = contentHash(doc);
  library.presets = [...library.presets.filter((other) => other !== had), doc];
  return doc;
}
/** @type {Preset[] | null} */
let restampedFor = null;
function prunePresets(/** @type {Chart | null} */ keep) {
  // Migration for copies stamped while panes' sizes counted as an edit: stamped anew, remove after the release past
    // v0.13.0.
  const resized = presets && restampedFor !== presets ? library.presets.filter((doc) => !pristine(doc) && presetIndex(doc) >= 0 && contentHash(doc) === templateHash(presetIndex(doc))) : [];
  restampedFor = presets;
  for (const doc of resized) (doc.base = templateHash(presetIndex(doc))), pristineSeen.delete(doc);
  const lately = new Set(viewed());
  const left = library.presets.filter((doc) => doc === keep || !pristine(doc) || (lately.has(doc.id) && (!presets || presetIndex(doc) >= 0)));
  const stale = presets ? left.filter((doc) => doc !== keep && pristine(doc) && keptPreset(presetIndex(doc), doc) !== doc) : [];
  if (left.length === library.presets.length && !stale.length && !resized.length) return;
  library.presets = left;
  for (const doc of stale) presetChart(presetIndex(doc));
  save();
}
const presetShows = (/** @type {Chart} */ doc) => JSON.stringify([...new Set(windowless(doc).layers.flatMap((layer) => layer.series ?? layer.formula ?? []))].sort());
/** @returns {string[] | undefined} */
const presetPlace = (/** @type {Chart} */ doc) => {
  try {
    const place = JSON.parse(doc.preset ?? "");
    return Array.isArray(place) ? place : undefined;
  } catch {
    return undefined;
  }
};
/** @param {Chart} doc @returns {((i: number) => boolean)[]} */
const likeness = (doc) => ((title, place = []) => {
  // (A window once beside its chart, as "Supply, all time", is that chart's in the folder's place now, moved or not.)
  const list = /** @type {Preset[]} */ (presets), name = place.at(-1) ?? "", now = placeNow(place)?.join(" › "), folder = place.slice(0, -1), folderNow = (placeNow(folder) ?? folder).join(" › ");
  return [(i) => list[i].p.join(" › ") === now, (i) => Object.hasOwn(list[i].w ?? {}, name) && list[i].p.slice(0, -1).join(" › ") === folderNow, (i) => windowless(freshPreset(i)).title === title, (i) => list[i].p.at(-1) === name, () => true];
})(windowless(doc).title, presetPlace(doc));
/** @param {Chart} doc */
function presetShowing(doc) {
  const set = presetShows(doc), alike = /** @type {Preset[]} */ (presets).flatMap((_, i) => (presetShows(freshPreset(i)) === set ? [i] : []));
  return likeness(doc).map((like) => alike.find(like)).find((i) => i !== undefined) ?? -1;
}
// Nothing lost, nothing hidden: edits move to the preset that now shows the same series, else the chart is kept in the
// Library; said once.
function adoptPresets(/** @type {Chart[]} */ among = library.presets, told = true) {
  // One another tab placed already (deleted here, kept only while it's open) is that tab's.
  const lost = among.filter((doc) => presetIndex(doc) < 0 && !pristine(doc) && !isDeleted(doc));
  if (!lost.length || readOnly) return 0;
  const taken = (/** @type {number} */ i) => ((had) => Boolean(had && (presetEdited(had) || had === chart)))(editedPreset(i));
  /** @type {Map<string, number[]>} */
  const holders = new Map();
  /** @type {Preset[]} */ (presets).forEach((_, i) => {
    if (taken(i)) return;
    const set = presetShows(freshPreset(i));
    holders.set(set, [...(holders.get(set) ?? []), i]);
  });
  for (const doc of lost) normalize(doc);
  /** @type {Map<Chart, number>} */
  const to = new Map(), likes = lost.map(likeness);
  for (let pass = 0; pass < (likes[0]?.length ?? 0); pass++) {
    for (const [k, doc] of lost.entries()) {
      if (to.has(doc)) continue;
      const set = presetShows(doc), at = holders.get(set) ?? [], i = at.find(likes[k][pass]);
      if (i === undefined) continue;
      to.set(doc, i);
      holders.set(set, at.filter((j) => j !== i));
    }
  }
  const stamp = (/** @type {number} */ at) => Math.max(at, Date.now() - 29 * 86400000);
  let kept = 0;
  for (const doc of lost) {
    const i = to.get(doc), old = doc.id, replaced = library.presets.filter((other) => other !== doc && i !== undefined && other.preset === presetKey(i));
    library.presets = library.presets.filter((other) => other !== doc && !replaced.includes(other));
    library.deleted[old] = Math.max(library.deleted[old] ?? 0, stamp(doc.updated));
    // The same id in every tab doing this at once, so its copies are one chart.
    const fixed = i === undefined ? `a${hashOf(old)}`.slice(0, 16) : presetIdOf(presetKey(i));
    const id = i === undefined && library.charts.some((other) => other.id === fixed) ? uid() : fixed;
    rekey(doc, id);
    chartAgain.set(old, id);
    doc.updated = Math.max(Date.now(), doc.updated + 1, (library.deleted[id] ?? 0) + 1, ...replaced.map((other) => other.updated + 1));
    histories.delete(id);
    if (doc === chart) (edits.past = []), (edits.future = []);
    if (i === undefined) {
      delete doc.preset;
      delete doc.base;
      library.charts.unshift(doc);
      kept++;
    } else {
      Object.assign(doc, { preset: presetKey(i), base: templateHash(i) });
      library.presets.push(doc);
    }
  }
  save();
  // A draw of the open one still on its way under its old id gives way; still starting, the start draws it.
  if (started && lost.includes(/** @type {Chart} */ (chart))) refresh();
  const moved = to.size;
  if (!told) return kept;
  toast(`The presets were reorganized: ${[moved ? `${plural(moved, "edited preset")} moved with them` : "", kept ? `${plural(kept, "edited preset")} kept as ${kept === 1 ? "a chart" : "charts"} in your Library` : ""].filter(Boolean).join(", ")}`);
}
/** @param {Place} place @returns {Step[]} */
function stepsTo(place) {
  const group = (/** @type {Group} */ name) => ({ name: GROUP_OF[name][0], at: GROUP_OF[name][1] });
  if ("previews" in place) return [group("previews")];
  if ("recent" in place) return [group("recent")];
  if ("path" in place) return [group("presets"), ...placeNames(place.path).map((name, k) => ({ name, at: { path: place.path.slice(0, k + 1) } }))];
  if (!("folder" in place)) return [];
  const folder = folderById(place.folder);
  return [group("library"), ...(folder ? [...foldersAbove(folder), folder] : []).map((step) => ({ name: step.name, at: { folder: step.id } }))];
}
/** @param {Chart} doc */
function presetPathOf(doc) {
  try {
    const path = doc.preset === undefined ? null : JSON.parse(doc.preset);
    return Array.isArray(path) && path.length && path.every((name) => typeof name === "string") ? /** @type {string[]} */ (path).slice(0, -1) : null;
  } catch {
    return null;
  }
}
/** @returns {Place | null} */
function chartPlace(doc = chart) {
  const i = presetIndex(doc);
  return i >= 0 ? { path: /** @type {Preset[]} */ (presets)[i].p.slice(0, -1) }
    : !presets && presetPathOf(doc) ? { path: /** @type {string[]} */ (presetPathOf(doc)) }
    : isPreview(doc) && !drafts.has(doc) ? { previews: true }
    : library.charts.includes(doc) ? { folder: doc.folder ?? "" }
    : null;
}
/** @param {Folder} folder */
function foldersAbove(folder) {
  const above = [];
  for (let at = folderById(folder.parent), steps = 0; at && steps <= library.folders.length; at = folderById(at.parent), steps++) above.unshift(at);
  return above;
}
const presetsSettled = () => readPresetList().then(() => {}, () => {});
export { presetTree, presetRoot, presetName, presetNodeAt, placeNames, homeName, folderById, backToHome, asPlace, BASE, routeOf, showAddress, presetChart, presetLook, presetEdited, presetIndex, presetDoc, presetIdOf, keptPreset, pristine, looksOf, aFrame, idsNow, adoptPresets, prunePresets, addressesSoon, isLinkHash, presetLink, editedPreset, templateHash, templateHashes, freshPreset, contentHash, windowless, presetKey, presetPlaces, templateDoc, presetDocs, fromPreset, madeLooks, presetShows, likeness, presetPlace, restampedFor, keptAsStored, presetIds, presetById, addressBook, addressDue, structuredFor, placeAddress, addressNow, CANONICAL_ORIGIN, addresses, presetShowing, chartPlace, stepsTo, foldersAbove, presetPathOf, presetsSettled
};
