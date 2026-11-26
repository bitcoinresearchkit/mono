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
import { showView } from "/studio/vocabulary.js";
import { titleOf } from "/studio/looks.js";
import { STUDIO_ORIGIN, presetInfo, sectionInfo, STUDIO_TITLE, STUDIO_DESCRIPTION, presetDataset, studioApplication } from "/studio/page-info.js";
import { inSlices } from "/studio/timing.js";
import { addressSteps } from "/studio/addresses.js";
import { validate, fromSketch, viewShown, contentOf, lookFields } from "/studio/charts-read.js";
import { isNamed } from "/studio/color.js";

/** @typedef {{ name: string, at: Place }} Step */
/**
 * The presets' sections as a tree, made once they're read: each with what's in it as made (its sections and its
 * own presets, in the presets' order), every preset it holds at any depth (`under`, in their order), its key (its path, as kept), its name as
 * shown (see shownStep) and its path as shown (`shown`); and, once a search needs them, each preset's words (see presetSearch).
 * @typedef {{ path: string[], key: string, name: string, shown: string[], items: (PresetNode | number)[], under: number[], icon?: string }} PresetNode
 */
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
        // (A top section's icon, on its first preset: see presets.js.)
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
/** A preset's name as shown (see shownStep). @param {Preset} preset */
const presetName = (preset) => shownStep(presetLabels, preset.p);
/** A section of the presets as the sidebar has it (see presetTree), by its path; none, gone. */
function presetNodeAt(/** @type {string[]} */ path) {
  return presetTree() ? presetRoot.byKey.get(JSON.stringify(path)) : undefined;
}
/** A section's steps as shown (see shownStep); their names till the presets are read. @param {string[]} path */
function placeNames(path) {
  return presetNodeAt(path)?.shown ?? path;
}
/** The page's name, where it is: a folder's, the Library's, the previews', Recent's, a section's, the presets'; at the top, Home. */
const homeName = () => ("folder" in homeAt ? folderById(homeAt.folder)?.name ?? "Library" : "previews" in homeAt ? "Previews" : "recent" in homeAt ? "Recent" : "path" in homeAt ? placeNames(homeAt.path).at(-1) ?? "Presets" : "Home");
/**
 * Where the page is: at the top, Charts (`top`); the presets (`path` [], their page) or a section of them (`path`,
 * as their tree names it); your Library (`folder` "") or one of its folders (`folder`); the previews (`previews`); the
 * charts last opened (`recent`). Each a step of the history (Back goes back up).
 * @typedef {{ top: true } | { path: string[] } | { folder: string } | { previews: true } | { recent: true }} Place
 */
/** A place as kept in a step of the history, as it can be (none: the top). @returns {Place} */
const asPlace = (/** @type {any} */ at) => (typeof at?.folder === "string" ? { folder: at.folder } : Array.isArray(at?.path) ? { path: at.path.filter((/** @type {any} */ name) => typeof name === "string") } : at?.previews === true ? { previews: true } : at?.recent === true ? { recent: true } : { top: true });
/** A new chart, untouched, opened from the grid: leaving it goes back there. */
const backToHome = () => (isPreview(chart) || emptyYours(chart)) && history.state?.fromHome === true && history.state.chart === chart.id;
/** A folder of yours, by its id (none: gone, or the Library's top). */
const folderById = (/** @type {string | undefined} */ id) => library.folders.find((folder) => folder.id === id);
/**
 * Each series' look as the presets draw it (its color, its type, a baseline's base): the one they give it most, for a
 * series added on its own to start as they draw it. A shade (a color not named) is the series' place in a family (a
 * comparison's member, a window, a percentile): a layer in one says nothing of its look, but a baseline's base.
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
/**
 * The next frame, for work split across frames; a hidden tab (no frames: a tab opened behind, a prerender) or one
 * hidden meanwhile goes on at once, or a tenth of a second on.
 */
function aFrame() {
  return document.hidden ? Promise.resolve() : new Promise((done) => (requestAnimationFrame(() => done(undefined)), setTimeout(done, 100)));
}
/**
 * A preset's chart, read as a link is; its panes' and layers' ids as its sketch gives them (p0, l0…: the same each
 * time it's made, so its drawings and what's kept of it follow it).
 * @param {Preset} preset
 */
const fromPreset = (preset) => ((doc) => (normalize(doc), doc))(validate(fromSketch({ ...preset.c, title: preset.t }), true));
/**
 * Each preset as the chart it makes, made once it's looked at (its id stable: its drawing kept), the last
 * two hundred looked at kept (with their drawings: a gallery's page of them). @type {Map<number, Chart>}
 */
const presetDocs = new Map();
/** A preset's chart as it shows: as edited here, else as made (a copy left as it was, of a preset since changed, too). */
function presetDoc(/** @type {number} */ i) {
  return keptPreset(i) ?? templateDoc(i);
}
/** A preset's chart as made (made once looked at, kept a while). */
function templateDoc(/** @type {number} */ i) {
  let doc = presetDocs.get(i);
  if (doc) presetDocs.delete(i);
  else {
    doc = Object.assign(fromPreset(/** @type {Preset[]} */ (presets)[i]), { id: `preset:${i}` });
    // (A drawing kept by its place in the list is this preset's only if it says so: the list may be another now.)
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
/** Where Studio's served on a domain: DOMAIN/studio/, its own addresses (the presets') under it. */
const BASE = "/studio";
/** Whether an address's hash holds a link (a chart's, compressed or written, or a picture's), not how to look at one. @param {string} hash */
const isLinkHash = (hash) => /^#(?:.*&)?(?:chart|image|c)=/.test(hash);
/** The presets' addresses, made once they're read. */
let addressBook = /** @type {{ for: Preset[] | null, steps: ReturnType<typeof addressSteps> | null, book: import("/studio/addresses.js").AddressBook | null }} */ ({ for: null, steps: null, book: null });
/** The presets' addresses as they are now: those made already, else the rest made at once. */
const addresses = () => {
  if (!presets) return null;
  if (addressBook.for !== presets) addressBook = { for: presets, steps: addressSteps(presets), book: null };
  while (!addressBook.book) ((step) => step.done && (addressBook.book = step.value))(/** @type {NonNullable<typeof addressBook.steps>} */ (addressBook.steps).next());
  return addressBook.book;
};
/** The presets' addresses made while the page is idle, a slice at a time (all at once, 27 ms of a phone's). @param {{ timeRemaining: () => number }} deadline */
const addressesSoon = (deadline) => {
  if (!presets || (addressBook.for === presets && addressBook.book)) return;
  if (addressBook.for !== presets) addressBook = { for: presets, steps: addressSteps(presets), book: null };
  const steps = /** @type {NonNullable<typeof addressBook.steps>} */ (addressBook.steps);
  inSlices(deadline, () => ((step) => (step.done && (addressBook.book = step.value), Boolean(step.done)))(steps.next()), addressesSoon);
};
/**
 * What an address opens: a preset, a place of the presets'; none, nothing of theirs. As written: its server sends the
 * other spellings of one (the slash the other way, an old /chart/ one) to it. A preset's once made one for each window
 * (…/realized-profit-share-24-hours, …/sma/200-days): the preset at that window (see addresses.js; its address then
 * written as it is now, see presetLink).
 * @param {string} pathname @returns {{ path: string[] } | { preset: number, window?: string } | null}
 */
const routeOf = (pathname) => addresses()?.at.get(pathname.slice(BASE.length)) ?? null;
/** Where the page is, as an address (its canonical one): a preset's, a place of the presets', else the page's own ("/": your charts are no one else's); null, not known till they're read. */
function addressNow() {
  // (The presets' addresses made only where the page is one of theirs: the page's own, none.)
  if (home ? !("path" in homeAt) : chart.preset === undefined) return "/";
  const book = addresses();
  if (!book) return null;
  if (home) return book.sections.get(JSON.stringify(/** @type {{ path: string[] }} */ (homeAt).path)) ?? "/";
  const i = presetIndex(chart);
  return i >= 0 ? book.presets[i] : "/";
}
/** Search names the live Studio on every origin. */
const CANONICAL_ORIGIN = STUDIO_ORIGIN;
/**
 * The page as it's found, as it is now: its title, its description and its canonical address (each place's own): a
 * preset's, named by what it charts and where it's kept; a section's, by what's in it; the presets' page; your own,
 * the studio's. The address bar shows the same: a preset's address, with how it's looked at where that's not as it
 * opens (see linkLookOf), Share's link; a chart of yours (edited, a preview), the Studio's own, its link alone Share's.
 */
function showAddress() {
  // (Once what's being done is done: a step of the history pushed after the page's drawn has the address, not the
  // one before it.)
  if (addressDue) return;
  addressDue = true;
  queueMicrotask(() => ((addressDue = false), placeAddress()));
}
let addressDue = false, structuredFor = "";
function placeAddress() {
  // (A link's picture being made: the page is its, not a place's; the page still starting, as it was asked for.)
  if (document.body.hasAttribute("data-picture") || !started) return;
  const address = addressNow(), i = home ? -1 : presetIndex(chart), path = home && "path" in homeAt ? homeAt.path : null;
  const inside = path && presets ? ((items) => [...items.flatMap((item) => (typeof item === "number" ? [] : [item.name])), ...items.flatMap((item) => (typeof item === "number" ? [/** @type {Preset[]} */ (presets)[item].t] : []))])(presetNodeAt(path)?.items ?? []) : [];
  // (A preset's words, its title as the chart says it: at another window, that window's, see atWindow.)
  const info = i >= 0 && presets ? presetInfo({ ...presets[i], t: chart.title || presets[i].t }, presetLabels)
    : path && presets ? sectionInfo(path, presetLabels, inside)
    : { title: home && "top" in homeAt ? STUDIO_TITLE : `${home ? homeName() : titleOf(chart)} · Bitview Studio`, description: STUDIO_DESCRIPTION };
  const canonical = `${CANONICAL_ORIGIN}${address ?? (ROUTED ? location.pathname.slice(BASE.length) : "/")}`;
  const structuredKey = `${address}|${i}|${info.title}|${info.description}`;
  if (structuredKey !== structuredFor) {
    const json = document.querySelector('script[type="application/ld+json"]');
    if (json) json.textContent = JSON.stringify(i >= 0 && presets ? presetDataset(presets[i], presetLabels, tree, canonical) : studioApplication(canonical, info.description));
    structuredFor = structuredKey;
  }
  document.title = info.title;
  const said = info.description;
  const meta = /** @type {HTMLMetaElement} */ (document.querySelector('meta[name="description"]'));
  if (meta.content !== said) meta.content = said;
  // (Its canonical address made once it knows where it is: none before, rather than the root's at every address.)
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
  // (Where the page's Studio's own: a link still being read, a picture's, or broken, left as it is till it's dealt with.)
  // (An address waiting for the presets to be read, kept.)
  const bar = ROUTED && !pendingRoute && !isLinkHash(location.hash) ? (home || chart.preset === undefined ? address : presets ? presetLink(chart) ?? "/" : null) : null;
  if (bar === null) return;
  const [where, look = ""] = bar.split("#"), url = `${BASE}${where}${location.search}${look && `#${look}`}`;
  // (Safari refuses writes past so many in a few seconds: the address stays as it was, written again at the next rest.)
  if (`${location.pathname}${location.search}${location.hash}` !== url) {
    try {
      history.replaceState(history.state, "", url);
    } catch {}
  }
}
/** Which preset a chart of one is: its place in the presets (its sections and its own name). */
const presetKey = (/** @type {number} */ i) => JSON.stringify(/** @type {Preset[]} */ (presets)[i].p);
/** Each preset's place, by its key (made once they're read). */
let presetPlaces = /** @type {{ for: Preset[] | null, at: Map<string, number> }} */ ({ for: null, at: new Map() });
const presetIndex = (/** @type {Chart} */ doc) => {
  if (!presets || doc.preset === undefined) return -1;
  if (presetPlaces.for !== presets) presetPlaces = { for: presets, at: new Map(presets.map((preset, i) => [JSON.stringify(preset.p), i])) };
  return presetPlaces.at.get(doc.preset) ?? -1;
};
/** A preset as edited here, if it is. */
const editedPreset = (/** @type {number} */ i) => (presets ? library.presets.find((doc) => doc.preset === presetKey(i)) : undefined);
/** What a preset opens with, how it's looked at: its own interval (day1 unless it says), its range, its trailing window, if it has one, and its views (see viewShown). @param {number} i */
const presetLook = (i) => {
  const preset = /** @type {Preset[]} */ (presets)[i];
  if (!madeLooks.has(preset)) madeLooks.set(preset, ((doc) => ({ index: doc.index, range: doc.range, window: chartWindow(doc)?.window, show: viewShown(doc) }))(freshPreset(i)));
  return /** @type {{ index: string, range: string, window?: string, show: string }} */ (madeLooks.get(preset));
};
/** Each preset's look as made (see presetLook): read once a preset. @type {WeakMap<Preset, { index: string, range: string, window?: string, show: string }>} */
const madeLooks = new WeakMap();
/** A chart at its first trailing window (see chartWindow), as its content is told: a window picked isn't an edit. @param {Chart} doc */
const windowless = (doc) => ((w) => (w ? atWindow(structuredClone(doc), w.offered[0]) : doc))(chartWindow(doc));
/** A chart at each pane's first view (see Lens), as its content is told: a view shown isn't an edit. @param {Chart} doc */
const firstViews = (doc) => (doc.panes.some((pane) => pane.lens?.at) ? ((copy) => (copy.panes.forEach((pane) => showView(copy, pane.id, 0)), copy))(structuredClone(doc)) : doc);
/**
 * A chart's content as a fingerprint, how it's looked at aside: its view, as a preview's is, its interval, as
 * day1 (every preset opened at day1 before its own interval, an interval changed an edit: a stamp kept from before
 * is the same), its trailing window (see windowless), its views (see firstViews), and its panes without their sizes: a
 * separator dragged is a way of looking, not an edit (the sizes stay with the copy, kept, but never tell it from its
 * preset).
 * @param {Chart} doc
 */
const contentHash = (doc) => hashOf(contentOf(windowless(firstViews({ ...doc, range: "", index: "day1", panes: doc.panes.map(({ size, ...pane }) => pane) }))));
/** A preset's chart as made now, as an edit leaves a chart (normalized: nothing for an edit to change). */
const freshPreset = (/** @type {number} */ i) => ((doc) => (normalize(doc), doc))(fromPreset(/** @type {Preset[]} */ (presets)[i]));
/**
 * Each preset's fingerprint as made now, by the preset as read: made once (a preset makes the same chart wherever
 * it's made, as a copy's stamp, kept, has it), not for each time it's asked (each of its rows, its search, its
 * opening). @type {WeakMap<Preset, string>}
 */
const templateHashes = new WeakMap();
/** A preset's fingerprint as made now. */
function templateHash(/** @type {number} */ i) {
  const preset = /** @type {Preset[]} */ (presets)[i];
  let hash = templateHashes.get(preset);
  if (hash === undefined) templateHashes.set(preset, (hash = contentHash(freshPreset(i))));
  return hash;
}
/** A preset's chart's id: the same wherever it's opened (another tab's, a step of the history's), by which it is. */
const presetIdOf = (/** @type {string} */ key) => `p${hashOf(key)}`;
/**
 * Each preset by its chart's id, made as they come: a slice at a time in idle moments (none of it a long task, a
 * phone's engine without a JIT: every preset's path hashed), the rest at once where one's asked for first.
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
/**
 * Whether a preset's chart is as it was opened (none of its edits left), as last asked of its stamp; or, for a copy
 * just read, its stored form, to be told from when it's first asked (see keptAsStored). @type {WeakMap<Chart, { at: number, as?: boolean, from?: Chart }>}
 */
const pristineSeen = new WeakMap();
/** A copy as stored, before normalize: a newer client moving fields isn't an edit. Told only when asked (it walks the catalog). */
const keptAsStored = (/** @type {Chart} */ doc) => void (doc.base && pristineSeen.set(doc, { at: doc.updated, from: structuredClone(doc) }));
function pristine(/** @type {Chart} */ doc) {
  if (!doc.base) return false;
  const seen = pristineSeen.get(doc), current = seen?.at === doc.updated ? seen : undefined;
  if (current?.as !== undefined) return current.as;
  const was = current?.from ?? doc, hash = contentHash(was);
  // (A stamp from before windows could be picked (till 2026-11), its content at the window it was made at: as made
  // all the same, stamped anew, however the copy came (kept here, another tab's, a backup's). Delete in 2027-11.)
  if (hash !== doc.base && doc.base === hashOf(contentOf({ ...was, range: "", index: "day1" }))) doc.base = hash;
  const as = hash === doc.base;
  pristineSeen.set(doc, { at: doc.updated, as });
  return as;
}
/** Whether a preset's been changed here (it says so, and can be reset). @param {Chart} doc */
const presetEdited = (doc) => doc.preset !== undefined && !pristine(doc);
/**
 * A preset's address, with how it's looked at where that isn't how it opens (see linkLookOf): the chart of a preset as
 * made but for its view and its interval; else, or the presets unread, null. @param {Chart} doc
 */
function presetLink(doc) {
  const i = presetIndex(doc);
  if (i < 0 || presetEdited(doc)) return null;
  const look = lookFields(doc, presetLook(i));
  return `${/** @type {NonNullable<ReturnType<typeof addresses>>} */ (addresses()).presets[i]}${look && `#${look}`}`;
}
/**
 * A preset's copy kept here that still stands: edited, or left as it was of the preset as made now (`had`: its copy,
 * found already; null, none).
 */
const keptPreset = (/** @type {number} */ i, had = editedPreset(i)) => (had && (!pristine(had) || had.base === templateHash(i)) ? had : undefined);
/**
 * A preset's chart: as edited here, else as made now (one left as it was, made anew: the presets may have changed
 * since). Never edited: stamped 0 (past a reset, just after it), any edit's copy (here,
 * another tab's) wins over it. Its steps here stay with it (a reset let go of with it, undone once it's back).
 */
function presetChart(/** @type {number} */ i) {
  const key = presetKey(i), had = editedPreset(i), id = presetIdOf(key), kept = keptPreset(i);
  if (kept) return kept;
  const doc = /** @type {Chart} */ ({ ...freshPreset(i), id, preset: key, updated: (library.deleted[id] ?? -1) + 1 });
  doc.base = contentHash(doc);
  library.presets = [...library.presets.filter((other) => other !== had), doc];
  return doc;
}
/** The presets read when the copies were last restamped (see prunePresets). @type {Preset[] | null} */
let restampedFor = null;
/**
 * The presets opened and left as they were, let go of (all but `keep`), but those opened lately: Recent's, drawn at
 * once on a reload, before the presets are read (once they are, one gone since let go of too, one changed since
 * made anew). (Nothing to record: another tab's copy back is as it was, and goes the same way.)
 */
function prunePresets(/** @type {Chart | null} */ keep) {
  // (Migration, remove after the release following 2026-11-26 (the next past v0.13.0): a copy stamped while its panes' sizes counted as an edit is as made where they were its only change, stamped anew.)
  const resized = presets && restampedFor !== presets ? library.presets.filter((doc) => !pristine(doc) && presetIndex(doc) >= 0 && contentHash(doc) === templateHash(presetIndex(doc))) : [];
  restampedFor = presets;
  for (const doc of resized) (doc.base = templateHash(presetIndex(doc))), pristineSeen.delete(doc);
  const lately = new Set(viewed());
  const left = library.presets.filter((doc) => doc === keep || !pristine(doc) || (lately.has(doc.id) && (!presets || presetIndex(doc) >= 0)));
  // (Recent's, once the presets are read, as they're made now: one changed since, made anew.)
  const stale = presets ? left.filter((doc) => doc !== keep && pristine(doc) && keptPreset(presetIndex(doc), doc) !== doc) : [];
  if (left.length === library.presets.length && !stale.length && !resized.length) return;
  library.presets = left;
  for (const doc of stale) presetChart(presetIndex(doc));
  save();
}
/** What a chart shows, its looks aside (its trailing window too, see windowless): its series and formulas, as a set; what's made from them (a cloud, a trend) an edit of it. */
const presetShows = (/** @type {Chart} */ doc) => JSON.stringify([...new Set(windowless(doc).layers.flatMap((layer) => layer.series ?? layer.formula ?? []))].sort());
/** A preset's chart's place, as stored: a key that isn't one, none. @returns {string[] | undefined} */
const presetPlace = (/** @type {Chart} */ doc) => {
  try {
    const place = JSON.parse(doc.preset ?? "");
    return Array.isArray(place) ? place : undefined;
  } catch {
    return undefined;
  }
};
/**
 * Which of the presets showing what a chart whose place is gone shows is it, first to last: the one that had its place
 * (one a window once, see addresses.js), of its title (at their first window, see windowless: an amount and its share
 * show the same series), of its place's name, any.
 * @param {Chart} doc @returns {((i: number) => boolean)[]}
 */
const likeness = (doc) => ((title, place = []) => {
  const list = /** @type {Preset[]} */ (presets), name = place.at(-1) ?? "";
  return [(i) => Object.hasOwn(list[i].w ?? {}, name) && list[i].p.length === place.length && place.slice(0, -1).every((step, k) => list[i].p[k] === step), (i) => windowless(freshPreset(i)).title === title, (i) => list[i].p.at(-1) === name, () => true];
})(windowless(doc).title, presetPlace(doc));
/**
 * The preset that shows what a preset's chart whose place is gone shows (a preset renamed since, moved), the likest
 * (see likeness); none, -1. @param {Chart} doc
 */
function presetShowing(doc) {
  const set = presetShows(doc), alike = /** @type {Preset[]} */ (presets).flatMap((_, i) => (presetShows(freshPreset(i)) === set ? [i] : []));
  return likeness(doc).map((like) => alike.find(like)).find((i) => i !== undefined) ?? -1;
}
/**
 * Presets edited here whose place is gone (the presets arranged anew since, by a newer Studio): each moved to the
 * preset that now shows the same series, its edits there (edited: Reset brings that one back as made); none does,
 * kept as a chart of the Library, its title and edits as they were (a series gone since says so there, and offers
 * the closest). Nothing lost, nothing hidden; said once (`told`: else by the caller, an import, which it tells how
 * many it kept as charts).
 */
function adoptPresets(/** @type {Chart[]} */ among = library.presets, told = true) {
  // (Those left as they were opened aren't edits: prunePresets lets go of them, after (the open one, the preset that
  // shows it now opened in its place, see presetShowing). One another tab placed already (deleted here, kept only
  // while it's open) is that tab's.)
  const lost = among.filter((doc) => presetIndex(doc) < 0 && !pristine(doc) && !isDeleted(doc));
  if (!lost.length || readOnly) return 0;
  // (A preset taken already: edited here, or the one open.)
  const taken = (/** @type {number} */ i) => ((had) => Boolean(had && (presetEdited(had) || had === chart)))(editedPreset(i));
  /** Each set's presets free to take one. @type {Map<string, number[]>} */
  const holders = new Map();
  /** @type {Preset[]} */ (presets).forEach((_, i) => {
    if (taken(i)) return;
    const set = presetShows(freshPreset(i));
    holders.set(set, [...(holders.get(set) ?? []), i]);
  });
  // (Series by their paths first: renamed since, by their names now.)
  for (const doc of lost) normalize(doc);
  // Each to a preset of its set, the likest first (see likeness), one each.
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
    // (Its copy as stored let go of, here and in any tab's next save, by its id as of its stamp, a month kept.)
    library.deleted[old] = Math.max(library.deleted[old] ?? 0, stamp(doc.updated));
    // (The same id in every tab doing this at once: its copies one chart; that id another chart's (another version of
    // it, imported), one of its own.)
    const fixed = i === undefined ? `a${hashOf(old)}`.slice(0, 16) : presetIdOf(presetKey(i));
    const id = i === undefined && library.charts.some((other) => other.id === fixed) ? uid() : fixed;
    rekey(doc, id);
    chartAgain.set(old, id);
    // (Newer than its old copy's end, than any reset of where it goes, and than the copy there it replaces.)
    doc.updated = Math.max(Date.now(), doc.updated + 1, (library.deleted[id] ?? 0) + 1, ...replaced.map((other) => other.updated + 1));
    // (Its steps back were of its old place: none.)
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
  // (The open one drawn again under its new id: a draw of it still on its way, under its old one, gives way. Still
  // starting, the start draws it.)
  if (started && lost.includes(/** @type {Chart} */ (chart))) refresh();
  const moved = to.size;
  if (!told) return kept;
  toast(`The presets were reorganized: ${[moved ? `${plural(moved, "edited preset")} moved with them` : "", kept ? `${plural(kept, "edited preset")} kept as ${kept === 1 ? "a chart" : "charts"} in your Library` : ""].filter(Boolean).join(", ")}`);
}
/**
 * The way to a place, the place last, from its group (none for the top): a section's, the Presets and the sections
 * it's in; a folder's, the Library and the folders it's in (one gone, the Library's).
 * @param {Place} place @returns {Step[]}
 */
function stepsTo(place) {
  const group = (/** @type {Group} */ name) => ({ name: GROUP_OF[name][0], at: GROUP_OF[name][1] });
  if ("previews" in place) return [group("previews")];
  if ("recent" in place) return [group("recent")];
  if ("path" in place) return [group("presets"), ...placeNames(place.path).map((name, k) => ({ name, at: { path: place.path.slice(0, k + 1) } }))];
  if (!("folder" in place)) return [];
  const folder = folderById(place.folder);
  return [group("library"), ...(folder ? [...foldersAbove(folder), folder] : []).map((step) => ({ name: step.name, at: { folder: step.id } }))];
}
/** A preset chart's sections, by its own key (read as stored: anything but a list of names, none). @param {Chart} doc */
function presetPathOf(doc) {
  try {
    const path = doc.preset === undefined ? null : JSON.parse(doc.preset);
    return Array.isArray(path) && path.length && path.every((name) => typeof name === "string") ? /** @type {string[]} */ (path).slice(0, -1) : null;
  } catch {
    return null;
  }
}
/** Where a chart is (the open one's, else): a preset's section, your folder (or the Library's top), the previews; a new chart, not yet anywhere, nowhere. @returns {Place | null} */
function chartPlace(doc = chart) {
  const i = presetIndex(doc);
  // (A preset's, before the presets are read: its own key says where it is (its sections, then its name).)
  return i >= 0 ? { path: /** @type {Preset[]} */ (presets)[i].p.slice(0, -1) }
    : !presets && presetPathOf(doc) ? { path: /** @type {string[]} */ (presetPathOf(doc)) }
    : isPreview(doc) && !drafts.has(doc) ? { previews: true }
    : library.charts.includes(doc) ? { folder: doc.folder ?? "" }
    : null;
}
/** The folders a folder's in, the outermost first. @param {Folder} folder */
function foldersAbove(folder) {
  const above = [];
  for (let at = folderById(folder.parent), steps = 0; at && steps <= library.folders.length; at = folderById(at.parent), steps++) above.unshift(at);
  return above;
}
/**
 * The presets' list read, or given up on (readPresetList: nothing opens): a chart written by hand (a link, a file,
 * studio.open) and a series added are read after, as a series' first look comes from them (see firstLook).
 */
const presetsSettled = () => readPresetList().then(() => {}, () => {});
export { presetTree, presetRoot, presetName, presetNodeAt, placeNames, homeName, folderById, backToHome, asPlace, BASE, routeOf, showAddress, presetChart, presetLook, presetEdited, presetIndex, presetDoc, presetIdOf, keptPreset, pristine, looksOf, aFrame, idsNow, adoptPresets, prunePresets, addressesSoon, isLinkHash, presetLink, editedPreset, templateHash, templateHashes, freshPreset, contentHash, windowless, firstViews, presetKey, presetPlaces, templateDoc, presetDocs, fromPreset, madeLooks, presetShows, likeness, presetPlace, restampedFor, keptAsStored, presetIds, presetById, addressBook, addressDue, structuredFor, placeAddress, addressNow, CANONICAL_ORIGIN, addresses, presetShowing, chartPlace, stepsTo, foldersAbove, presetPathOf, presetsSettled
};
