// @ts-check
// The presets: every one, its address, edited where it is.
import { initialsWord } from "/studio/initials.js";
import {
  presetDataset, presetInfo, sectionInfo, STUDIO_DESCRIPTION, STUDIO_ORIGIN, STUDIO_TITLE, studioApplication,
} from "/studio/page-info.js";
import { addressSteps } from "/studio/addresses.js";
import {
  applyPanel, asThen, atWindow, averageFor, chart, chartAgain, chartWindow, commit, discard, dismissPalette,
  drawingsHidden, editChart, edits, folderMark, folderNow, histories, home, homeAt, inLibrary, isDeleted, isPreview,
  keptAsEdited, leaveHome, leftView, library, LIVE, loadPresets, newDraft, nextHalvings, normalize, notice, openChart,
  panelShown, pendingRoute, pickPreset, prefs, presetLabels, presetMarks, presets, presetsFailed, presetsMatching,
  previews, previousChart, readOnly, readPresetList, refresh, rekey, relist, renderCharts, resolved, retryPresets,
  ROUTED, save, saveView, sectionMark, showPalette, started, storage, thumbs, toast, tree, undoToast, viewed, VIEWED,
} from "/studio/app.js";
import { $, all$, escapeHtml } from "/studio/dom.js";
import { inSlices } from "/studio/timing.js";
import { monthYear, plural } from "/studio/format.js";
import { ICONS } from "/studio/icons.js";
import { MODES, newPaneIn, OUTLINES, outlinable, rowsUnfit, STACKS } from "/studio/vocabulary.js";
import { AVERAGES, INTERVAL_LABEL } from "/studio/intervals.js";
import { isNamed } from "/studio/color.js";
import { hashOf, uid } from "/studio/math.js";
import { changeOver, defaultSize, figuresRoom, lastShown } from "/studio/chart.js";
import {
  choicePage, heightRow, layerRows, listPage, palette, plotIn, sizable, slipEdits, typedWords, viewRow,
} from "/studio/command.js";
import {
  childrenOf, folderById, folderPath, go, homeName, isOpen, newFolder, placeNames, placesOf, presetMark, presetMarkKey,
  presetNodeAt, presetSearch, previewIndex, recommendedPresets, saveThumbs, slide, titleOf, treeOf,
} from "/studio/grid.js";
import { goneSaid, hideLayers, keepFocus, layerById, modesFor, movers, paneMode } from "/studio/inspector.js";
import {
  contentOf, exportRows, fromSketch, keepCarriedView, lookFields, menuOf, renameInPlace, validate,
} from "/studio/menus.js";
/** @import { Chart, Layer, Pane } from "/studio/vocabulary.js" */
/** @import { Landing } from "/studio/gestures.js" */
/** @import { Action, Page } from "/studio/command.js" */


/**
 * A preset (presets.js makes them): `p` its place in their tree (its sections, then its own name), `t` its title, `c`
 * its chart as a link writes it, `w` its places when it was one a window (see presets.js).
 * @typedef {{ p: string[], t: string, c: { layers: any[], panes?: any[], index?: string, range?: string }, s?: string, w?: Record<string, string> }} Preset
 */
/** The presets the empty chart offers first, by their place (seo.mjs checks each is one). */
const RECOMMENDED = [
  "Valuation › Price by MVRV zone", "Price › Technical › Halving cycles", "Valuation › On-chain prices", "Valuation › Market and realized cap", "Valuation › MVRV", "Coin age › Age bands › Compare › Supply share",
  "Holders › Compare › Realized price", "Holders › Compare › MVRV", "Price › Returns › Short-term", "Mining › Hashing › Hash price and value",
  "Supply › Inflation", "Valuation › Rarity meter › Full › Bands", "Bedrock › Unweighted › Floors", "Network › Transactions › Count",
];
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
 * The presets' list read, or given up on (readPresetList: nothing opens): a chart written by hand (a link, a file,
 * studio.open) and a series added are read after, as a series' first look comes from them (see firstLook).
 */
const presetsSettled = () => readPresetList().then(() => {}, () => {});
/** The pages the presets fill (each a group of them). @type {WeakSet<Page>} */
const presetPages = new WeakSet();
/**
 * ⌘K, open where the presets show (their page, or everything ⌘K finds), shown again with them: on the row
 * it was on (what ↵ runs doesn't change under you). Elsewhere (a formula, adding a series, a setting), left as it is.
 */
function presetsArrived() {
  if (!isOpen($("palette")) || palette.formula || palette.adding || palette.replacing) return;
  const page = palette.pages.at(-1);
  if (page && !presetPages.has(page)) return;
  // (A preset's copy kept here, Recent's, by its chart till now: its preset's row, by its id (the copy may be let go of by
  // now: one changed since).)
  const item = palette.items[palette.active], at = item?.kind === "chart" ? presetById(item.key) : -1;
  palette.landOn = at >= 0 ? `preset:${at}` : item?.key ?? null;
  showPalette();
}
/**
 * A preset's chart, read as a link is; its panes' and layers' ids as its sketch gives them (p0, l0…: the same each
 * time it's made, so its drawings and what's kept of it follow it).
 * @param {Preset} preset
 */
const fromPreset = (preset) => validate(fromSketch({ ...preset.c, title: preset.t }), true);
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
/** An address that names nothing, said, whatever it holds (a % that isn't one, as written). @param {string} address */
const missedSaid = (address) => `Nothing of the presets' at ${((text) => {
  try {
    return decodeURI(text);
  } catch {
    return text;
  }
})(address)} any more: here they all are`;
/** Where the page is, as an address (its canonical one): a preset's, a place of the presets', else the page's own ("/": your charts are no one else's); null, not known till they're read. */
function addressNow() {
  // (The presets' addresses made only where the page is one of theirs: the page's own, none.)
  if (home ? !("path" in homeAt) : chart.preset === undefined && !isMade(chart)) return "/";
  const book = addresses();
  if (!book) return null;
  if (home) return book.sections.get(JSON.stringify(/** @type {{ path: string[] }} */ (homeAt).path)) ?? "/";
  const i = chartPreset(chart);
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
  const address = addressNow(), i = home ? -1 : chartPreset(chart), path = home && "path" in homeAt ? homeAt.path : null;
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
  const bar = ROUTED && !pendingRoute && !isLinkHash(location.hash) ? (home || (chart.preset === undefined && !isMade(chart)) ? address : presets ? presetLink(chart) ?? "/" : null) : null;
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
/** What a preset opens with, how it's looked at: its own interval (day1 unless it says), its range and its trailing window, if it has one. @param {number} i */
const presetLook = (i) => {
  const preset = /** @type {Preset[]} */ (presets)[i];
  if (!madeLooks.has(preset)) madeLooks.set(preset, ((doc) => ({ index: doc.index, range: doc.range, window: chartWindow(doc)?.window }))(freshPreset(i)));
  return /** @type {{ index: string, range: string, window?: string }} */ (madeLooks.get(preset));
};
/** Each preset's look as made (see presetLook): read once a preset. @type {WeakMap<Preset, { index: string, range: string, window?: string }>} */
const madeLooks = new WeakMap();
/** A chart at its first trailing window (see chartWindow), as its content is told: a window picked isn't an edit. @param {Chart} doc */
const windowless = (doc) => ((w) => (w ? atWindow(structuredClone(doc), w.offered[0]) : doc))(chartWindow(doc));
/**
 * A chart's content as a fingerprint, how it's looked at aside: its view, as a preview's is, its interval, as
 * day1 (every preset opened at day1 before its own interval, an interval changed an edit: a stamp kept from before
 * is the same), and its trailing window (see windowless).
 * @param {Chart} doc
 */
const contentHash = (doc) => hashOf(contentOf(windowless({ ...doc, range: "", index: "day1" })));
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
/** Whether a preset's chart is as it was opened (none of its edits left), as last asked of its stamp. @type {WeakMap<Chart, { at: number, as: boolean }>} */
const pristineSeen = new WeakMap();
function pristine(/** @type {Chart} */ doc) {
  if (!doc.base) return false;
  const seen = pristineSeen.get(doc);
  if (seen?.at === doc.updated) return seen.as;
  const hash = contentHash(doc);
  // (A stamp from before windows could be picked (till 2026-11), its content at the window it was made at: as made
  // all the same, stamped anew, however the copy came (kept here, another tab's, a backup's). Delete in 2027-11.)
  if (hash !== doc.base && doc.base === hashOf(contentOf({ ...doc, range: "", index: "day1" }))) doc.base = hash;
  const as = hash === doc.base;
  pristineSeen.set(doc, { at: doc.updated, as });
  return as;
}
/** Whether a preset's been changed here (it says so, and can be reset). @param {Chart} doc */
const presetEdited = (doc) => doc.preset !== undefined && !pristine(doc);
/** Whether a chart's a preview that is a preset as made (opened at its address, one's own edits of it apart: `made`, its place). @param {Chart} doc */
const isMade = (doc) => doc.made !== undefined && isPreview(doc);
/** The preset a chart is, its own or as made (see isMade): its index; none (or the presets unread), -1. @param {Chart} doc */
const chartPreset = (doc) => presetIndex(isMade(doc) ? { ...doc, preset: doc.made } : doc);
/**
 * A preset's address, with how it's looked at where that isn't how it opens (see linkLookOf): the chart of a preset as
 * made but for its view and its interval; else, or the presets unread, null. @param {Chart} doc
 */
function presetLink(doc) {
  const i = chartPreset(doc);
  if (i < 0 || (isMade(doc) ? contentHash(doc) !== templateHash(i) : presetEdited(doc))) return null;
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
/**
 * A preset opened: itself, to look at and to work on as it is (no preview to keep: its edits stay with it, here, till
 * it's reset). As edited here, else as made, kept once it's edited (left as it is, only while it's among Recent's:
 * see prunePresets).
 */
function openPreset(/** @type {number} */ i) {
  if (!presets?.[i]) return;
  delete document.body.dataset.sheet;
  go(presetChart(i));
}
/**
 * A preset's rows in the sidebar (its own, Recent's: `id`, its chart's), one in sight: none in it, the nearest
 * scrolled to; one in it already, nothing moves. On the next frame, as it's laid out anyway (shown in it all the
 * same): not mid-change, the whole page laid out for it at once; its rows as they are then (written again since,
 * the new ones).
 * @param {number} preset @param {string} id
 */
const revealNearest = (preset, id) => requestAnimationFrame(() => {
  const seen = $("chart-list").getBoundingClientRect();
  const away = (/** @type {HTMLElement} */ row) => ((box) => Math.max(0, seen.top - box.top, box.bottom - seen.bottom))(row.getBoundingClientRect());
  // (Those shown: the rail's, none.)
  const nearest = all$(`[data-open-preset="${preset}"], [data-open-recent="${CSS.escape(id)}"]`, $("chart-list")).filter((row) => row.getClientRects().length).sort((a, b) => away(a) - away(b))[0];
  if (nearest && away(nearest) > 0) nearest.scrollIntoView({ block: "nearest" });
});
/**
 * The presets opened and left as they were, let go of (all but `keep`), but those opened lately: Recent's, drawn at
 * once on a reload, before the presets are read (once they are, one gone since let go of too, one changed since
 * made anew). (Nothing to record: another tab's copy back is as it was, and goes the same way.)
 */
function prunePresets(/** @type {Chart | null} */ keep) {
  const lately = new Set(viewed());
  const left = library.presets.filter((doc) => doc === keep || !pristine(doc) || (lately.has(doc.id) && (!presets || presetIndex(doc) >= 0)));
  // (Recent's, once the presets are read, as they're made now: one changed since, made anew.)
  const stale = presets ? left.filter((doc) => doc !== keep && pristine(doc) && keptPreset(presetIndex(doc), doc) !== doc) : [];
  if (left.length === library.presets.length && !stale.length) return;
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
 * A preset back as made: an edit of its own (see editChart), said with an Undo where it's open. Its copies in other
 * tabs, older, let go of (as a deleted chart's are), even once this one, as made, is let go of here (see prunePresets).
 * @param {Chart} doc
 */
function resetPreset(doc) {
  const i = presetIndex(doc);
  if (i < 0 || !presetEdited(doc)) return;
  // (Before the edit, which stamps it past this; it changes it, edited as it is.)
  library.deleted[doc.id] = Math.max(Date.now(), doc.updated);
  editChart(doc, (draft) => {
    const made = freshPreset(i);
    for (const key of Object.keys(draft)) if (!(key in made) && !["id", "preset", "updated", "folder"].includes(key)) delete /** @type {any} */ (draft)[key];
    Object.assign(draft, made, { id: draft.id, preset: draft.preset, base: contentHash(made) });
  });
  undoToast(`Reset “${titleOf(doc)}”`, doc);
}
/**
 * A preset copied into your Library (`folder`: into it; none, at the top), as it shows here, first there, and opened;
 * said, with an Undo (back to the preset; `quiet`: not, a new folder's own). @returns {Chart}
 */
function copyPreset(/** @type {number} */ i, /** @type {string | undefined} */ folder = undefined, quiet = false) {
  const from = presetDoc(i), copy = /** @type {Chart} */ ({ ...structuredClone(from), id: uid(), updated: Date.now() });
  delete copy.preset;
  delete copy.base;
  // (Where it's looked at, too.)
  if (from === chart) saveView();
  keepCarriedView(copy, { view: leftView(from.id) });
  if (folder && folderById(folder)) copy.folder = folder;
  else delete copy.folder;
  // (Drawn as it is: its picture, its mark.)
  const drawn = previews.get(from.id), mark = thumbs.get(from.id);
  if (drawn) previews.set(copy.id, { ...drawn, key: "" });
  if (mark) (thumbs.set(copy.id, mark), saveThumbs());
  // (Listed first and opened at once (its toast then answers ⌘Z): it's what's worked on now. Undo, back to the preset,
  // the copy gone; Redo, that same copy listed and opened again, so its own steps and every Library step that knew it
  // find it.)
  const preset = presetIdOf(presetKey(i));
  library.charts.unshift(copy);
  const openCopy = () => {
    save();
    // (In Recent, in the preset's stead (by its lasting id: its edits reset, its chart's is another): the same chart,
    // under the same name, the copy what's looked at now; Undo opens the preset again, back in it.)
    storage.set(VIEWED, viewed().filter((id) => id !== preset));
    if (home) leaveHome(() => openChart(copy));
    else openChart(copy);
    renderCharts();
    const same = asThen([copy]);
    if (!quiet) toast(`Saved a copy of “${titleOf(copy)}” to ${copy.folder ? `“${folderPath(copy.folder)}”` : "your Library"}`, { label: "Undo", run: () => {
      if (!inLibrary(copy)) return;
      if (!same()) return keptAsEdited([copy]);
      if (chart === copy) openChart(presets?.[i] ? presetChart(i) : previousChart() ?? newDraft());
      discard(copy);
      save();
      renderCharts();
      // (Back where it went: that folder, made again since, as it is now (see recover).)
      return () => {
        if (inLibrary(copy)) return;
        relist(copy, 0);
        openCopy();
      };
    } });
  };
  openCopy();
  return copy;
}
/** Where a preset's copy can go: your charts, each folder (named with those it's in), a new one. @returns {Action[]} */
function copyChoices(/** @type {number} */ i) {
  return [
    { key: "top", icon: ICONS.grid, label: "Library", run: () => void copyPreset(i) },
    ...treeOf().flatMap(({ item }) => ("layers" in item ? [] : [{ key: item.id, icon: folderMark(item), label: folderPath(item.id), run: () => void copyPreset(i, item.id) }])),
    { key: "new", icon: ICONS.folderAdd, label: "New folder", apart: true, run: () => copyToNewFolder(i) },
  ];
}
/**
 * A preset copied into a new folder of its own, named there: one step, one Undo for both (no Redo: the folder made
 * again would need its name and its place among its own, a path of its own for a rare step).
 * @param {number} i
 */
function copyToNewFolder(i) {
  if (isOpen($("palette"))) dismissPalette();
  const copy = copyPreset(i, undefined, true);
  newFolder([`c:${copy.id}`], undefined, { step: false });
  const folder = library.folders.find((item) => item.id === copy.folder), same = asThen([copy]);
  // (Said, with one Undo: the copy and its new folder gone, back to the preset.)
  toast(`Saved a copy of “${titleOf(copy)}” to a new folder`, { label: "Undo", folder: folder?.id, run: () => {
    if (!same()) return keptAsEdited([copy]);
    const made = folder && folderById(folderNow(folder.id)), listed = library.charts.find((other) => other.id === copy.id);
    if (!listed && !made) return;
    if (chart === listed && presets?.[i]) openChart(presetChart(i));
    if (listed) discard(listed);
    if (made && !childrenOf(made.id).length) (library.folders = library.folders.filter((item) => item !== made)), (library.deleted[made.id] = Date.now());
    save();
    renderCharts();
  } });
}
/**
 * A preset's menu, from its ⋯ (`more`: a row's in the sidebar, a tile's): at it, or where it was asked for (a right
 * click's, a hold's place); on a phone, titled by the preset.
 * @param {HTMLElement} more @param {{ x: number, y: number } | null} [at]
 */
function openPresetMenu(more, at = null) {
  const i = Number(more.dataset.presetMenu);
  menuOf(more, presetActions(i, more.closest("#chart-list, [data-preset-row]") ? "row" : "tile"), at);
}
/**
 * A preset's actions, where they show (`where`, as a chart's): open it, copy it into your charts (or into a folder),
 * reset it (edited); its own, open (its header): renamed in its title, its legend, Export.
 * @param {number} i @param {"row" | "head" | "tile" | "palette"} where @returns {Action[]}
 */
function presetActions(i, where) {
  const doc = editedPreset(i), head = where === "head";
  return [
    ...(where === "tile" || where === "palette" ? [{ key: "open", icon: ICONS.arrowRight, label: "Open", run: () => pickPreset(i) }] : []),
    ...(head ? [{ key: "rename", icon: ICONS.pencil, label: "Rename", run: renameInPlace }] : []),
    { key: "copy", icon: ICONS.duplicate, label: "Save a copy", aside: "Into your Library", run: () => void copyPreset(i) },
    { key: "copy-to", icon: ICONS.folder, label: "Save a copy to…", page: () => choicePage("Save a copy to", () => copyChoices(i)) },
    ...(head && chart.layers.length ? [{ ...legendAction(), apart: true }, { key: "export", icon: ICONS.download, label: "Export", aside: "Image, data, file", page: () => listPage("Export", exportRows) }] : []),
    ...(doc && presetEdited(doc) ? [{ key: "reset", icon: ICONS.undo, label: "Reset", aside: "Discard your edits", apart: !head, run: () => resetPreset(doc) }] : []),
  ];
}
/**
 * The words of the presets' titles and places, each with how many presets have it, made again as the presets come:
 * a slice at a time in idle moments (none of it a long task, a phone without its JIT), the rest at once when a
 * search needs them first.
 */
let presetVocab = { of: /** @type {string[] | null} */ (null), counts: /** @type {Map<string, number>} */ (new Map()), making: /** @type {Generator<void, Map<string, number>> | null} */ (null) };
function* vocabMaking(/** @type {string[]} */ texts) {
  const counts = new Map();
  for (let i = 0; i < texts.length; i++) {
    for (const word of new Set(texts[i].split(/[^a-z0-9]+/))) if (word.length >= 3) counts.set(word, (counts.get(word) ?? 0) + 1);
    if (i % 250 === 249) yield;
  }
  return counts;
}
/** The presets' words (see presetVocab), at once; in an idle moment (`deadline`), as far as it goes, then the next. */
function vocabNow(/** @type {{ timeRemaining: () => number } | undefined} */ deadline = undefined) {
  const words = presetSearch().shown;
  if (presetVocab.of !== words) presetVocab = { of: words, counts: new Map(), making: vocabMaking(words) };
  inSlices(deadline, () => ((step) => (step?.done && ((presetVocab.counts = step.value), (presetVocab.making = null)), !step || Boolean(step.done)))(presetVocab.making?.next()), vocabNow);
  return presetVocab.counts;
}
/**
 * A word typed as no preset has it (nor a renamed one's title, `extra`), the words of theirs it's a slip off (see
 * slipEdits, as fitScore's), the nearest first, then those more presets have; none if it's found as typed: in their words as shown, or, of letters
 * and digits only, two of them joined, in what's searched (`texts`, see searchText: "1week"; "profit," still a slip).
 */
function slipsOf(/** @type {string} */ word, /** @type {string[]} */ texts, /** @type {string[]} */ extra, /** @type {string[]} */ initials) {
  if (word.length < 4 || extra.some((text) => text.includes(word)) || (initialsWord(word) && initials.some((text) => text.includes(word)))) return [];
  const counts = /** @type {Map<string, number>} */ (vocabNow()), most = word.length >= 7 ? 2 : 1, near = /** @type {[string, number, number][]} */ ([]);
  for (const each of counts.keys()) if (each.includes(word)) return [];
  if (/^[a-z0-9]+$/.test(word)) for (let i = 0; i < texts.length; i++) if (texts[i].includes(word)) return [];
  const weigh = (/** @type {string} */ each, /** @type {number} */ count) => ((edits) => edits <= most && near.push([each, edits, count]))(slipEdits(word, each, counts));
  for (const [each, count] of counts) weigh(each, count);
  for (const each of new Set(extra.flatMap((text) => text.split(/[^a-z0-9]+/)))) if (each.length >= 3 && !counts.has(each)) weigh(each, 1);
  return near.sort((a, b) => a[1] - b[1] || b[2] - a[2]).map(([each]) => each);
}
/** ⌘K's heading for presets a search found: those holding only some of its words (`partial`), said so wherever they're listed. */
const presetsGroup = (/** @type {boolean} */ partial) => (partial ? "Presets holding the most words" : "Presets");
/** A preset's row: its title (as edited here, if it is; found by a search, its place too); its preview; ↵ opens it; its actions (⌘K). */
const presetRow = (/** @type {number} */ i, deep = false) => {
  const preset = /** @type {Preset[]} */ (presets)[i], edited = keptPreset(i), title = edited ? titleOf(edited) : preset.t;
  return {
    // (Its mark as the sidebar's, drawn once its row's in sight (see paletteMarksSeen); the presets' icon till then.)
    kind: "row", key: `preset:${i}`, icon: presetMarks.has(presetMarkKey(i)) ? presetMark(i) : ICONS.presets.replace("<svg ", `<svg data-preset-mark="${i}" `), label: title, ...(deep ? { aside: placeNames(preset.p).join(" › "), under: true } : {}), verb: "Open",
    run: () => pickPreset(i),
    actions: () => presetActions(i, "palette"),
    preview: (/** @type {HTMLElement} */ panel) => {
      const doc = presetDoc(i);
      panel.innerHTML = `<h4 class="plain">${escapeHtml(title)}</h4><p class="description">${escapeHtml(placeNames(preset.p.slice(0, -1)).join(" › "))}</p>
              ${plotIn(doc, previewIndex(doc))}<div class="info"><h5>Layers</h5><dl>${layerRows(doc.layers)}</dl></div>`;
    },
  };
};
/**
 * The presets as the sidebar has them, a group at a time: its own page first (View), then its groups (each its
 * count) and charts, in its order, at the top those it recommends first; a search looks through every chart under it. Unread yet, it says so,
 * and fills in once they are.
 * @param {string[]} path @returns {Page}
 */
function presetsPage(path) {
  void loadPresets().catch(() => {});
  /** @type {Page} */
  const page = {
    title: placeNames(path).at(-1) ?? "Presets",
    // (Its rows are its search's, best first. Of no chart: ⌘K comes back to it whichever's open.)
    searches: true,
    global: true,
    placeholder: path.length ? `Search in ${placeNames(path).at(-1)}` : "Search the presets",
    rows: () => {
      if (!presets) return [presetsFailed
        ? { kind: "row", key: "presets-retry", icon: ICONS.recent, label: "The presets couldn't be read", aside: "Try again", verb: "Retry", stay: true, run: retryPresets }
        : { kind: "row", key: "presets-waiting", icon: ICONS.recent, label: "Loading the presets…", dim: true, stay: true }];
      const list = presets, under = (/** @type {number} */ i) => path.every((name, depth) => list[i].p[depth] === name);
      // (At the top, the presets as a whole: searched as such, see presetsMatching.)
      const words = palette.pages.at(-1) === page ? typedWords(palette.query) : [];
      // (Found holding only some of the words, headed as ⌘K's own list heads them: see presetsGroup.)
      if (words.length) {
        const { found, partial } = presetsMatching(words, 300, path.length ? under : null);
        return found.map((i) => ({ ...presetRow(i, true), ...(partial ? { group: presetsGroup(true) } : {}) }));
      }
      // (Its sections, each by its name as shown and how many it holds, and its own presets, as the sidebar has them.)
      const rows = (presetNodeAt(path)?.items ?? []).map((item) => (typeof item === "number" ? presetRow(item) : { kind: "row", key: `group:${item.path.at(-1)}`, icon: sectionMark(item), label: item.name, aside: plural(item.under.length, "chart"), verb: "Open", page: () => presetsPage(item.path) }));
      // (Its page first; at their top, the recommended, a folder of their own before the sections: a row away, not a
      // screen of them.)
      return [viewRow({ path }), ...(path.length ? [] : [{ kind: "row", key: "group:recommended", icon: ICONS.folder, label: "Recommended", aside: plural(recommendedPresets().length, "chart"), verb: "Open", page: recommendedPage }]), ...rows];
    },
  };
  presetPages.add(page);
  return page;
}
/** ⌘K's recommended presets, each with its place. @returns {Page} */
function recommendedPage() {
  return { title: "Recommended", global: true, placeholder: "Search in Recommended", rows: () => (presets ? recommendedPresets().map((i) => ({ ...presetRow(i, true), key: `recommended:${i}` })) : []) };
}

/** After a menu pick re-renders the list, focus goes back to the same menu button. */
const refocus = (selector) => requestAnimationFrame(() => /** @type {HTMLElement | null} */ (document.querySelector(selector))?.focus());
/** A menu item's mark: a check on the current one, the same room on the others. */
const tick = (/** @type {boolean} */ on) => (on ? ICONS.check : '<i class="menu-space"></i>');
/** A pane's chip: its scale, and its stack (a 100% one's scale is its own). @param {Pane} pane */
const paneChip = (pane) => `${pane.stack === "percent" ? "100%" : `${MODES[paneMode(pane)].chip}${pane.stack === "rows" ? " · Heatmap" : pane.stack ? " · Stacked" : ""}`}${pane.reverse ? " · Rev" : ""}${((average) => average.active ? ` · ${average.active}` : "")(averageFor(chart, pane))}`;
/** An edit to one pane, by its id: its scale (a mode), its stack ("" for none). @param {string} id @param {(pane: Pane) => void} edit */
const editPane = (id, edit, look = false) => commit((doc) => {
  const pane = doc.panes.find((item) => item.id === id);
  if (pane) edit(pane);
}, { look });
const setPaneMode = (id, mode) => editPane(id, (pane) => (pane.mode = mode), true);
const setPaneStack = (id, stack) => editPane(id, (pane) => (stack ? (pane.stack = stack) : delete pane.stack));
const setPaneReverse = (id, reverse) => editPane(id, (pane) => (reverse ? (pane.reverse = true) : delete pane.reverse));
const setPaneOutline = (id, on) => editPane(id, (pane) => (on ? (pane.outline = true) : delete pane.outline), true);
const setPaneAverage = (id, average) => editPane(id, (pane) => (average ? (pane.average = average) : delete pane.average));
/** A pane's two orders, named as it draws them: [reverse, its name]. @param {Pane} pane */
const ordersOf = (pane) => /** @type {[boolean, string][]} */ ([[false, pane.stack === "rows" ? "First at the top" : pane.stack ? "First at the foot" : "First in front"], [true, pane.stack === "rows" ? "Last at the top" : pane.stack ? "Last at the foot" : "Last in front"]]);
const toggleLog = (id) => {
  const pane = chart.panes.find((item) => item.id === id);
  if (pane?.stack === "percent") notice("A 100% stack shows shares of the total, so there's no scale to choose");
  else if (pane) setPaneMode(id, paneMode(pane) === 1 ? 0 : 1);
};
/** A pane's scale, stack, order, outline and average, from its chip. */
const scaleMenu = (/** @type {HTMLElement} */ anchor, /** @type {string} */ id) => menuOf(anchor, scaleChoices(id), null, "", { heads: true });
/**
 * A pane's scale, stack, order and average as choices, those in effect ticked (its chip's menu, its ⌘K pages): a stack's
 * Percent and Indexed saying why not; a 100% one, no scale to choose; an order only for several layers.
 * @param {string} id @returns {Action[]}
 */
function scaleChoices(id) {
  const pane = chart.panes.find((item) => item.id === id);
  if (!pane) return [];
  const current = averageFor(chart, pane), ticking = chart.layers.some((layer) => layer.pane === id && !layer.hidden && LIVE.has(layer.series ?? ""));
  return [
    ...(pane.stack === "percent" ? [] : MODES.map((mode, m) => ({ key: `scale:${m}`, group: "Scale", label: mode.name, aside: mode.tip === mode.name ? undefined : mode.tip, current: paneMode(pane) === m, ...(modesFor(pane).includes(m) ? {} : { disabled: `${mode.name} isn't available in a stacked pane (use linear or log)` }), run: () => setPaneMode(id, m) }))),
    ...STACKS.map(([stack, name, says]) => ({ key: `stack:${stack || "off"}`, group: "Stack", label: name, aside: says, current: (pane.stack ?? "") === stack, ...(stack === "rows" && rowsUnfit(chart, pane) ? { disabled: rowsUnfit(chart, pane) } : {}), run: () => setPaneStack(id, stack) })),
    ...(chart.layers.filter((layer) => layer.pane === id).length > 1 ? ordersOf(pane).map(([reverse, name]) => ({ key: `order:${reverse ? "last" : "first"}`, group: "Order", label: name, current: Boolean(pane.reverse) === reverse, run: () => setPaneReverse(id, reverse) })) : []),
    ...(outlinable(chart, pane) ? OUTLINES.map(([on, name]) => ({ key: `outline:${on ? "below" : "none"}`, group: "Outline", label: name, current: Boolean(pane.outline) === on, run: () => setPaneOutline(id, on) })) : []),
    ...["", ...Object.keys(AVERAGES)].map((span) => {
      const choice = averageFor(chart, pane, span);
      return { key: `average:${span || "raw"}`, group: "Average", label: span || "Raw", current: current.active === span,
        aside: span ? `${plural(choice.points, "point")} at ${INTERVAL_LABEL[chart.index]}${ticking ? " · Updates each block" : ""}` : current.span && !current.active ? `Saved ${current.span} resumes when available` : "As measured",
        disabled: choice.disabled ? `${choice.disabled}${span === current.span ? `; saved ${span} resumes when available` : ""}` : undefined, run: () => setPaneAverage(id, span) };
    }),
  ];
}
/** One group of a pane's choices, as its ⌘K page has it (the page its name). */
const scaleGroup = (/** @type {string} */ id, /** @type {string} */ group) => scaleChoices(id).filter((row) => row.group === group).map(({ group: _, ...row }) => row);
/** A pane's menu (its ⋯, a right-click on its head). */
const paneActions = (/** @type {HTMLElement} */ anchor, /** @type {string} */ id, /** @type {{ x: number, y: number } | undefined} */ point = undefined) => menuOf(anchor, paneRows(id), point ?? null);
/**
 * A pane's actions (its menu; ⌘K's page of it, after its settings: `inPalette`, with asides and its eye).
 * @returns {Action[]}
 */
function paneRows(/** @type {string} */ id, inPalette = false) {
  const i = chart.panes.findIndex((pane) => pane.id === id), many = chart.panes.length > 1;
  if (i < 0) return [];
  const layers = chart.layers.filter((layer) => layer.pane === id), shown = layers.some((layer) => !layer.hidden);
  return [
    ...(i > 0 ? [{ key: "up", icon: ICONS.arrowUp, label: "Move up", aside: inPalette ? `Swaps with pane ${i}` : undefined, verb: "Move", stay: true, run: () => movePane(id, -1) }] : []),
    ...(i < chart.panes.length - 1 ? [{ key: "down", icon: ICONS.arrowDown, label: "Move down", aside: inPalette ? `Swaps with pane ${i + 2}` : undefined, verb: "Move", stay: true, run: () => movePane(id, 1) }] : []),
    // (Its share of the height, chosen (a drag's own is hard to get just so by touch): in its menu and on ⌘K's page of it.)
    ...(sizable(i) ? [heightRow(id)] : []),
    ...(inPalette ? [{ key: "eye", icon: shown ? ICONS.eyeOff : ICONS.eye, label: shown ? "Hide every layer" : "Show every layer", aside: shown && many ? "The pane folds to its names" : undefined, verb: shown ? "Hide" : "Show", stay: true, run: () => hideLayers(layers.map((layer) => layer.id)) }] : []),
    { key: "remove", icon: ICONS.trash, label: many ? "Remove the pane" : "Remove every layer", aside: plural(layers.length, "layer"), verb: "Remove", danger: true, run: () => removePane(id) },
  ];
}
/** A pane one place up or down; its menu's button keeps the focus. */
function movePane(id, by) {
  // Where each pane is on screen now (mid-glide, where it shows), for the swap to start from.
  const groups = () => all$("#layer-list .pane-group"), key = (/** @type {HTMLElement} */ group) => group.dataset.pane;
  const was = placesOf(groups(), key);
  commit((doc) => {
    const from = doc.panes.findIndex((item) => item.id === id), to = from + by;
    if (from < 0 || to < 0 || to >= doc.panes.length) return;
    // Each takes its height along (the first is tall by default, by place).
    doc.panes.forEach((pane, k) => (pane.size ??= defaultSize(k)));
    [doc.panes[from], doc.panes[to]] = [doc.panes[to], doc.panes[from]];
  });
  slide(groups(), key, was, { moved: id });
  refocus(`[data-pane-menu="${id}"]`);
}
/** A pane and its layers, undoably (the only one: every layer); the focus goes to the pane that takes its place. */
function removePane(id) {
  const at = chart.panes.findIndex((pane) => pane.id === id), only = chart.panes.length === 1;
  const ids = chart.layers.filter((layer) => layer.pane === id).map((layer) => layer.id), count = ids.length, said = goneSaid(ids);
  commit((doc) => (doc.layers = doc.layers.filter((layer) => layer.pane !== id)));
  undoToast(`${only ? `Removed ${count === 1 ? "the layer" : `all ${count} layers`}` : `Removed a pane and its ${count === 1 ? "layer" : `${count} layers`}`}${said}`);
  requestAnimationFrame(() => /** @type {HTMLElement | null | undefined} */ (only ? document.querySelector("#empty [data-action='add-series']") : all$("#layer-list .pane-scale")[Math.max(0, Math.min(at, chart.panes.length - 1))])?.focus());
}

/**
 * What's marked on the chart besides its layers: the halvings, and the drawings there are, each shown or not (as a
 * layer, by its eye; the row the same).
 */
function markersHtml() {
  // (The past ones on the chart; the next, what it can't say otherwise.)
  const on = Boolean(chart.halvings), [next] = nextHalvings(1), years = next ? `Next ≈ ${monthYear(next.time)}` : "Every 210,000 blocks";
  const count = chart.drawings?.length ?? 0, drawn = !drawingsHidden;
  return `<section class="markers"><div class="markers-head"><span>Markers</span></div>
          <div class="layer-row marker-row ${on ? "" : "hidden"}">
            <button type="button" class="row-main" data-markers="halvings" aria-pressed="${on}" data-tip="${on ? "Hide" : "Show"} the halvings">
              <span class="letter marker-mark">${ICONS.halvings}</span>
              <span class="row-text"><span class="name">Halvings</span><span class="sub">${years}</span></span>
            </button>
            <button type="button" class="cap small" data-markers="halvings-eye" tabindex="-1" aria-hidden="true" data-tip="${on ? "Hide" : "Show"}">${on ? ICONS.eye : ICONS.eyeOff}</button>
          </div>${count ? `
          <div class="layer-row marker-row ${drawn ? "" : "hidden"}">
            <button type="button" class="row-main" data-markers="drawings" aria-pressed="${drawn}" data-tip="${drawn ? "Hide" : "Show"} the drawings" data-kbd="⇧ H">
              <span class="letter marker-mark">${ICONS.draw}</span>
              <span class="row-text"><span class="name">Drawings</span><span class="sub">${plural(count, "drawing")}</span></span>
            </button>
            <button type="button" class="cap small" data-markers="drawings-eye" tabindex="-1" aria-hidden="true" data-tip="${drawn ? "Hide" : "Show"}">${drawn ? ICONS.eye : ICONS.eyeOff}</button>
          </div>` : ""}</section>`;
}
function fullOn() {
  return /** @type {HTMLElement} */ (document.querySelector(".app")).hasAttribute("data-full");
}
/**
 * Full screen, on or off: the chart alone (see .app[data-full]; the panel closed, P opening it for the while), on
 * the screen's whole where the browser gives it (asked in the same click, as it must be; not when the window is the
 * screen already, or says so: Mullvad's does), else the window's. Out by its button, ⇧F or ⌘K, or Escape once
 * there's nothing else it puts away (a tool, a selection, a menu); a browser's own way out leaves it too (Firefox's
 * Escape, a held one in Chromium).
 */
function setFull(on = !fullOn()) {
  if (on === fullOn()) return;
  layOutAlone(on);
  // (The focus in what full screen puts away, the sidebar or the panel: on its way out, then the chart.)
  if (on) keepFocus(document.activeElement?.closest(".sidebar, #panel, .card-head") ?? null, () => $("full"), () => $("plot-keys"));
  if (on) {
    const screenWide = outerWidth >= screen.width && outerHeight >= screen.height;
    // (Chromium gives a page asking for it every Escape but a held one; out again before the browser answered, out.)
    if (!document.fullscreenElement && !screenWide) document.documentElement.requestFullscreen?.({ navigationUI: "hide" }).then(() => (fullOn() ? /** @type {any} */ (navigator).keyboard?.lock?.(["Escape"]) : document.exitFullscreen())).catch(() => {});
  } else {
    /** @type {any} */ (navigator).keyboard?.unlock?.();
    if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
  }
}
/** The chart alone, laid out (full screen's, a picture's) or not: no peeking panel, no sheet; its button said so. */
function layOutAlone(/** @type {boolean} */ on) {
  const button = $("full");
  /** @type {HTMLElement} */ (document.querySelector(".app")).toggleAttribute("data-full", on);
  if (on) {
    document.body.removeAttribute("data-peek");
    delete document.body.dataset.sheet;
  }
  applyPanel(on ? "closed" : prefs.panel);
  button.setAttribute("aria-pressed", String(on));
  button.setAttribute("aria-label", (button.dataset.tip = on ? "Exit full screen" : "Full screen"));
  button.innerHTML = on ? ICONS.shrink : ICONS.expand;
}
const toggleHalvings = () => commit((doc) => (doc.halvings ? delete doc.halvings : (doc.halvings = true)), { look: true });
/** How a chart's legends show: none, or (the default, "") over each pane, naming the line pointed at. */
const LEGENDS = /** @type {const} */ ([["off", "Off", "No legend over the panes", "None"], ["", "On", "A legend over each pane, its values where you point", "Over each pane"]]);
/** The legend's setting, the chart's (its ⋯, ⌘K's page of it). */
const legendAction = (group = "") => ({ key: "legend", icon: ICONS.legend, label: "Legend", aside: LEGENDS.find(([id]) => id === (chart.legend ?? ""))?.[1], group, page: () => choicePage("Legend", () => LEGENDS.map(([id, text, , note]) => ({ key: id || "on", icon: ICONS.legend, label: text, aside: note, current: (chart.legend ?? "") === id, run: () => setLegend(id) }))) });
const setLegend = (/** @type {string} */ mode) => (chart.legend ?? "") !== mode && commit((doc) => (mode === "off" ? (doc.legend = "off") : delete doc.legend), { look: true });
/**
 * The rows' figures' room and their changes, measured ahead while the list is behind (a phone's sheet put away: its
 * opening doesn't wait for them, 60 ms of it on a phone without a JIT): a layer at a time in idle moments, as the
 * rows will ask (see figuresRoom, changeOver; each kept for the data, the view).
 * @param {{ timeRemaining: () => number }} deadline
 */
function roomsAhead(deadline, from = 0) {
  let k = from;
  inSlices(deadline, () => {
    if (!resolved || panelShown() || k >= chart.layers.length) return true;
    const layer = chart.layers[k++], result = resolved.results.get(layer.id);
    if (result) figuresRoom(layer, result), changeOver(layer, { at: lastShown(), rest: true });
    return false;
  }, (next) => roomsAhead(next, k));
}
/** Layers dropped (in their order) before or after a layer, in its pane, or into a new one; everything gliding there, the first from where it was dropped. */
function dropLayers(/** @type {string[]} */ asked, /** @type {Landing} */ at, /** @type {number} */ y) {
  const ids = movers(asked, "item" in at ? layerById(at.item.dataset.layer ?? "")?.pane ?? null : null);
  if (!ids.length) return;
  const toRow = "item" in at ? at.item.dataset.layer : null, after = "item" in at && at.after;
  slideLayers(ids[0], y, () => commit((doc) => {
    const target = toRow ? doc.layers.find((layer) => layer.id === toRow) : null, moving = doc.layers.filter((layer) => ids.includes(layer.id));
    if (toRow && !target) return;
    const pane = target ? target.pane : newPaneIn(doc).id;
    doc.layers = doc.layers.filter((layer) => !ids.includes(layer.id));
    for (const layer of moving) layer.pane = pane;
    if (target) doc.layers.splice(doc.layers.indexOf(target) + (after ? 1 : 0), 0, ...moving);
    else doc.layers.push(...moving);
  }));
}
/**
 * The layer list changed by `change`, its parts gliding from where they were to where they are (`moved`, the one
 * moved, over the others; `y`: where it was let go, a drop's), by a drop or by keys alike.
 * @param {string} moved @param {number | null} y @param {() => void} change
 */
function slideLayers(moved, y, change) {
  const list = $("layer-list"), was = LIST_PARTS.map(([selector, key]) => placesOf(all$(selector, list), key));
  const row = /** @type {HTMLElement | null} */ (list.querySelector(`.layer-row[data-layer="${moved}"]`));
  if (row && y !== null) was[0].set(moved, y - row.offsetHeight / 2);
  change();
  LIST_PARTS.forEach(([selector, key, options], k) => slide(all$(selector, list), key, was[k], { ...options, moved: k === 0 ? moved : null }));
}
/** The parts of the layer list a drop moves: rows, pane heads, the lines between panes, the markers and the adds below. */
const LIST_PARTS = /** @type {[string, (element: HTMLElement) => string | undefined, { fade?: boolean, pseudo?: string }][]} */ ([
  [".layer-row[data-layer]", (row) => row.dataset.layer, {}],
  [".markers", () => "markers", {}],
  [".pane-head", (head) => /** @type {HTMLElement} */ (head.closest("[data-pane]")).dataset.pane, { fade: true }],
  [".pane-group + .pane-group", (group) => group.dataset.pane, { fade: true, pseudo: "::before" }],
  [".layer-adds", () => "adds", {}],
]);
export {
  addressesSoon, adoptPresets, aFrame, BASE, chartPreset, copyPreset, dropLayers, editedPreset, freshPreset,
  fromPreset, fullOn, idsNow, isLinkHash, isMade, keptPreset, layOutAlone, legendAction, LEGENDS, looksOf, markersHtml,
  missedSaid, openPreset, openPresetMenu, ordersOf, paneActions, paneChip, paneRows, presetActions, presetById,
  presetChart, presetDoc, presetDocs, presetEdited, presetIdOf, presetIndex, presetKey, presetLink, presetLook,
  presetPages, presetRow, presetsArrived, presetsGroup, presetShowing, presetsPage, presetsSettled, pristine,
  prunePresets, RECOMMENDED, refocus, resetPreset, revealNearest, roomsAhead, routeOf, scaleGroup, scaleMenu, setFull,
  setLegend, setPaneMode, setPaneOutline, setPaneReverse, setPaneStack, showAddress, slideLayers, slipsOf, tick, toggleHalvings,
  toggleLog, vocabNow,
};
