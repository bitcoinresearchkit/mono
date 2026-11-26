// @ts-check
// The presets: every one, its address, edited where it is.
import { initialsWord } from "/studio/initials.js";
import {
  applyPanel, asThen, averageFor, chart, commit, discard, dismissPalette, drawingsHidden, editChart, folderMark,
  folderNow, home, inLibrary, keptAsEdited, leaveHome, leftView, library, LIVE, loadPresets, newDraft, nextHalvings,
  notice, openChart, panelShown, pickPreset, prefs, presetMarks, presets, presetsFailed, presetsMatching, previews,
  previousChart, relist, renderCharts, resolved, retryPresets, save, saveView, sectionMark, showPalette, storage,
  thumbs, toast, undoToast, viewed, VIEWED, keepFocus, fullOn,
} from "/studio/app.js";
import { $, all$, escapeHtml, isOpen } from "/studio/dom.js";
import { inSlices } from "/studio/timing.js";
import { monthYear, plural } from "/studio/format.js";
import { ICONS } from "/studio/icons.js";
import { gradable, HEAT_GRADIENTS, MODES, newPaneIn, OUTLINES, outlinable, paneGradient, rowsUnfit, STACKS, summed } from "/studio/vocabulary.js";
import { AVERAGES, INTERVAL_LABEL } from "/studio/intervals.js";
import { uid } from "/studio/math.js";
import { changeOver, defaultSize, figuresRoom, lastShown, paneShows } from "/studio/chart.js";
import {
  choicePage, heightRow, layerRows, listPage, palette, plotIn, sizable, slipEdits, typedWords, viewRow,
} from "/studio/command.js";
import {
  childrenOf, folderPath, go, newFolder, placesOf, presetMark, presetMarkKey,
  presetSearch, previewIndex, recommendedPresets, saveThumbs, slide, treeOf,
} from "/studio/grid.js";
import { goneSaid, hideLayers, modesFor, movers, } from "/studio/inspector.js";
import {
  exportRows, menuOf, renameInPlace, } from "/studio/menus.js";
import { layerById, paneMode, titleOf } from "/studio/looks.js";
import { keepCarriedView } from "/studio/charts-read.js";
import { placeNames, presetNodeAt, folderById, presetChart, presetEdited, presetIndex, presetDoc, presetIdOf, keptPreset, editedPreset, freshPreset, contentHash, presetKey, presetById } from "/studio/preset-charts.js";
/** @import { Chart, Pane } from "/studio/vocabulary.js" */
/** @import { Landing } from "/studio/gestures.js" */
/** @import { Action, Page } from "/studio/command.js" */

/**
 * `p`: its place in the tree (sections, then its own name); `t`: title; `c`: chart as a link writes it; `w`: its places
 * when it was one a window (see presets.js).
 * @typedef {{ p: string[], t: string, c: { layers: any[], panes?: any[], index?: string, range?: string }, s?: string, w?: Record<string, string> }} Preset
 */
// seo.mjs checks each is a preset.
const RECOMMENDED = [
  "Valuation › Price by MVRV zone", "Price › Technical › Halving cycles", "Valuation › On-chain prices › Compare", "Valuation › Market and realized cap", "Valuation › MVRV", "Holders › Age bands › Compare › Supply share",
  "Holders › Compare › Realized price", "Holders › Compare › MVRV", "Price › Returns › Short-term", "Mining › Hashing › Hash price and value",
  "Supply › Inflation", "Valuation › Rarity meter › Full, bands", "Valuation › Bedrock › Unweighted › Floors", "Network › Transactions › Count",
];

/** @type {WeakSet<Page>} */
const presetPages = new WeakSet();
// Shown again on the row it was on: what ↵ runs doesn't change under you.
function presetsArrived() {
  if (!isOpen($("palette")) || palette.formula || palette.adding || palette.replacing) return;
  const page = palette.pages.at(-1);
  if (page && !presetPages.has(page)) return;
  const item = palette.items[palette.active], at = item?.kind === "chart" ? presetById(item.key) : -1;
  palette.landOn = at >= 0 ? `preset:${at}` : item?.key ?? null;
  showPalette();
}

/** @param {string} address */
const missedSaid = (address) => `Nothing of the presets' at ${((text) => {
  try {
    return decodeURI(text);
  } catch {
    return text;
  }
})(address)} any more: here they all are`;

function openPreset(/** @type {number} */ i) {
  if (!presets?.[i]) return;
  delete document.body.dataset.sheet;
  go(presetChart(i));
}
/**
 * On the next frame, as it's laid out anyway: not mid-change, which would lay the whole page out for it at once.
 * @param {number} preset @param {string} id
 */
const revealNearest = (preset, id) => requestAnimationFrame(() => {
  const seen = $("chart-list").getBoundingClientRect();
  const away = (/** @type {HTMLElement} */ row) => ((box) => Math.max(0, seen.top - box.top, box.bottom - seen.bottom))(row.getBoundingClientRect());
  const nearest = all$(`[data-open-preset="${preset}"], [data-open-recent="${CSS.escape(id)}"]`, $("chart-list")).filter((row) => row.getClientRects().length).sort((a, b) => away(a) - away(b))[0];
  if (nearest && away(nearest) > 0) nearest.scrollIntoView({ block: "nearest" });
});

/**
 * Its copies in other tabs, older, are let go of as a deleted chart's are, even once this one, as made, is let go of
 * here (see prunePresets).
 * @param {Chart} doc
 */
function resetPreset(doc) {
  const i = presetIndex(doc);
  if (i < 0 || !presetEdited(doc)) return;
  library.deleted[doc.id] = Math.max(Date.now(), doc.updated);
  editChart(doc, (draft) => {
    const made = freshPreset(i);
    for (const key of Object.keys(draft)) if (!(key in made) && !["id", "preset", "updated", "folder"].includes(key)) delete /** @type {any} */ (draft)[key];
    Object.assign(draft, made, { id: draft.id, preset: draft.preset, base: contentHash(made) });
  });
  undoToast(`Reset “${titleOf(doc)}”`, doc);
}
/** @returns {Chart} */
function copyPreset(/** @type {number} */ i, /** @type {string | undefined} */ folder = undefined, quiet = false) {
  const from = presetDoc(i), copy = /** @type {Chart} */ ({ ...structuredClone(from), id: uid(), updated: Date.now() });
  delete copy.preset;
  delete copy.base;
  if (from === chart) saveView();
  keepCarriedView(copy, { view: leftView(from.id) });
  if (folder && folderById(folder)) copy.folder = folder;
  else delete copy.folder;
  const drawn = previews.get(from.id), mark = thumbs.get(from.id);
  if (drawn) previews.set(copy.id, { ...drawn, key: "" });
  if (mark) (thumbs.set(copy.id, mark), saveThumbs());
  // Listed first and opened at once (its toast then answers ⌘Z): Undo goes back to the preset, the copy gone; Redo
    // lists and opens that same copy again, so its own steps and every Library step that knew it find it.
  const preset = presetIdOf(presetKey(i));
  library.charts.unshift(copy);
  const openCopy = () => {
    save();
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
/** @returns {Action[]} */
function copyChoices(/** @type {number} */ i) {
  return [
    { key: "top", icon: ICONS.grid, label: "Library", run: () => void copyPreset(i) },
    ...treeOf().flatMap(({ item }) => ("layers" in item ? [] : [{ key: item.id, icon: folderMark(item), label: folderPath(item.id), run: () => void copyPreset(i, item.id) }])),
    { key: "new", icon: ICONS.folderAdd, label: "New folder", apart: true, run: () => copyToNewFolder(i) },
  ];
}
/**
 * One step, one Undo for both; no Redo (the folder made again would need its name and place: a path of its own for a
 * rare step).
 * @param {number} i
 */
function copyToNewFolder(i) {
  if (isOpen($("palette"))) dismissPalette();
  const copy = copyPreset(i, undefined, true);
  newFolder([`c:${copy.id}`], undefined, { step: false });
  const folder = library.folders.find((item) => item.id === copy.folder), same = asThen([copy]);
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
/** @param {HTMLElement} more @param {{ x: number, y: number } | null} [at] */
function openPresetMenu(more, at = null) {
  const i = Number(more.dataset.presetMenu);
  menuOf(more, presetActions(i, more.closest("#chart-list, [data-preset-row]") ? "row" : "tile"), at);
}
/** @param {number} i @param {"row" | "head" | "tile" | "palette"} where @returns {Action[]} */
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
// Made a slice at a time in idle moments (no long task on a phone without a JIT), the rest at once when a search needs
// them first.
let presetVocab = { of: /** @type {string[] | null} */ (null), counts: /** @type {Map<string, number>} */ (new Map()), making: /** @type {Generator<void, Map<string, number>> | null} */ (null) };
function* vocabMaking(/** @type {string[]} */ texts) {
  const counts = new Map();
  for (let i = 0; i < texts.length; i++) {
    for (const word of new Set(texts[i].split(/[^a-z0-9]+/))) if (word.length >= 3) counts.set(word, (counts.get(word) ?? 0) + 1);
    if (i % 250 === 249) yield;
  }
  return counts;
}
function vocabNow(/** @type {{ timeRemaining: () => number } | undefined} */ deadline = undefined) {
  const words = presetSearch().shown;
  if (presetVocab.of !== words) presetVocab = { of: words, counts: new Map(), making: vocabMaking(words) };
  inSlices(deadline, () => ((step) => (step?.done && ((presetVocab.counts = step.value), (presetVocab.making = null)), !step || Boolean(step.done)))(presetVocab.making?.next()), vocabNow);
  return presetVocab.counts;
}
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
const presetsGroup = (/** @type {boolean} */ partial) => (partial ? "Presets holding the most words" : "Presets");
const presetRow = (/** @type {number} */ i, deep = false) => {
  const preset = /** @type {Preset[]} */ (presets)[i], edited = keptPreset(i), title = edited ? titleOf(edited) : preset.t;
  return {
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
/** @param {string[]} path @returns {Page} */
function presetsPage(path) {
  void loadPresets().catch(() => {});
  /** @type {Page} */
  const page = {
    title: placeNames(path).at(-1) ?? "Presets",
    searches: true,
    global: true,
    placeholder: path.length ? `Search in ${placeNames(path).at(-1)}` : "Search the presets",
    rows: () => {
      if (!presets) return [presetsFailed
        ? { kind: "row", key: "presets-retry", icon: ICONS.recent, label: "The presets couldn't be read", aside: "Try again", verb: "Retry", stay: true, run: retryPresets }
        : { kind: "row", key: "presets-waiting", icon: ICONS.recent, label: "Loading the presets…", dim: true, stay: true }];
      const list = presets, under = (/** @type {number} */ i) => path.every((name, depth) => list[i].p[depth] === name);
      const words = palette.pages.at(-1) === page ? typedWords(palette.query) : [];
      if (words.length) {
        const { found, partial } = presetsMatching(words, 300, path.length ? under : null);
        return found.map((i) => ({ ...presetRow(i, true), ...(partial ? { group: presetsGroup(true) } : {}) }));
      }
      const rows = (presetNodeAt(path)?.items ?? []).map((item) => (typeof item === "number" ? presetRow(item) : { kind: "row", key: `group:${item.path.at(-1)}`, icon: sectionMark(item), label: item.name, aside: plural(item.under.length, "chart"), verb: "Open", page: () => presetsPage(item.path) }));
      // At the top, the recommended in a folder of their own before the sections: a row away, not a screen of them.
      return [viewRow({ path }), ...(path.length ? [] : [{ kind: "row", key: "group:recommended", icon: ICONS.folder, label: "Recommended", aside: plural(recommendedPresets().length, "chart"), verb: "Open", page: recommendedPage }]), ...rows];
    },
  };
  presetPages.add(page);
  return page;
}
/** @returns {Page} */
function recommendedPage() {
  return { title: "Recommended", global: true, placeholder: "Search in Recommended", rows: () => (presets ? recommendedPresets().map((i) => ({ ...presetRow(i, true), key: `recommended:${i}` })) : []) };
}

const refocus = (selector) => requestAnimationFrame(() => /** @type {HTMLElement | null} */ (document.querySelector(selector))?.focus());
const tick = (/** @type {boolean} */ on) => (on ? ICONS.check : '<i class="menu-space"></i>');
/** @param {Pane} pane */
const paneChip = (pane) => `${pane.stack === "percent" ? "100%" : pane.stack === "stack" ? "Stacked" : `${MODES[paneMode(pane)].chip}${pane.stack === "rows" ? " · Heatmap" : ""}`}${pane.reverse ? " · Rev" : ""}${((average) => average.active ? ` · ${average.active}` : "")(averageFor(chart, pane))}`;
/** @param {string} id @param {(pane: Pane) => void} edit */
const editPane = (id, edit, look = false) => commit((doc) => {
  const pane = doc.panes.find((item) => item.id === id);
  if (pane) edit(pane);
}, { look });
const setPaneMode = (id, mode) => editPane(id, (pane) => (pane.mode = mode), true);
const setPaneStack = (id, stack) => editPane(id, (pane) => (stack ? (pane.stack = stack) : delete pane.stack));
const setPaneReverse = (id, reverse) => editPane(id, (pane) => (reverse ? (pane.reverse = true) : delete pane.reverse));
const setPaneOutline = (id, on) => editPane(id, (pane) => (on ? (pane.outline = true) : delete pane.outline), true);
/** @param {string} id @param {string} gradient */
const setPaneGradient = (id, gradient) => editPane(id, (pane) => (gradient !== HEAT_GRADIENTS[0][0] ? (pane.gradient = gradient) : delete pane.gradient), true);
const setPaneAverage = (id, average) => editPane(id, (pane) => (average ? (pane.average = average) : delete pane.average));
/** @param {Pane} pane */
const ordersOf = (pane) => /** @type {[boolean, string][]} */ ([[false, pane.stack === "rows" ? "First at the top" : pane.stack ? "First at the foot" : "First in front"], [true, pane.stack === "rows" ? "Last at the top" : pane.stack ? "Last at the foot" : "Last in front"]]);
const toggleLog = (id) => {
  const pane = chart.panes.find((item) => item.id === id);
  if (summed(pane)) notice("A stack draws linear, so there's no scale to choose");
  else if (pane) setPaneMode(id, paneMode(pane) === 1 ? 0 : 1);
};
const scaleMenu = (/** @type {HTMLElement} */ anchor, /** @type {string} */ id) => menuOf(anchor, scaleChoices(id), null, "", { heads: true });
/** @param {string} id @returns {Action[]} */
function scaleChoices(id) {
  const pane = chart.panes.find((item) => item.id === id);
  if (!pane) return [];
  const current = averageFor(chart, pane), ticking = chart.layers.some((layer) => layer.pane === id && !layer.hidden && LIVE.has(layer.series ?? ""));
  return [
    ...(summed(pane) ? [] : MODES.map((mode, m) => ({ key: `scale:${m}`, group: "Scale", label: mode.name, aside: mode.tip === mode.name ? undefined : mode.tip, current: paneMode(pane) === m, ...(modesFor(pane).includes(m) ? {} : { disabled: `${mode.name} isn't available in a heatmap (use linear or log)` }), run: () => setPaneMode(id, m) }))),
    ...STACKS.map(([stack, name, says]) => ({ key: `stack:${stack || "off"}`, group: "Stack", label: name, aside: says, current: (pane.stack ?? "") === stack, ...(stack === "rows" && rowsUnfit(chart, pane) ? { disabled: rowsUnfit(chart, pane) } : {}), run: () => setPaneStack(id, stack) })),
    ...(chart.layers.filter((layer) => layer.pane === id).length > 1 ? ordersOf(pane).map(([reverse, name]) => ({ key: `order:${reverse ? "last" : "first"}`, group: "Order", label: name, current: Boolean(pane.reverse) === reverse, run: () => setPaneReverse(id, reverse) })) : []),
    ...(gradable(chart, pane) ? HEAT_GRADIENTS.map(([gradient, name]) => ({ key: `gradient:${gradient}`, group: "Gradient", label: name, current: paneGradient(pane)[0] === gradient, run: () => setPaneGradient(id, gradient) })) : []),
    ...(outlinable(chart, pane) ? OUTLINES.map(([on, name]) => ({ key: `outline:${on ? "below" : "none"}`, group: "Outline", label: name, current: Boolean(pane.outline) === on, run: () => setPaneOutline(id, on) })) : []),
    ...["", ...Object.keys(AVERAGES)].map((span) => {
      const choice = averageFor(chart, pane, span);
      return { key: `average:${span || "raw"}`, group: "Average", label: span || "Raw", current: current.active === span,
        aside: span ? `${plural(choice.points, "point")} at ${INTERVAL_LABEL[chart.index]}${ticking ? " · Updates each block" : ""}` : current.span && !current.active ? `Saved ${current.span} resumes when available` : "As measured",
        disabled: choice.disabled ? `${choice.disabled}${span === current.span ? `; saved ${span} resumes when available` : ""}` : undefined, run: () => setPaneAverage(id, span) };
    }),
  ];
}
const scaleGroup = (/** @type {string} */ id, /** @type {string} */ group) => scaleChoices(id).filter((row) => row.group === group).map(({ group: _, ...row }) => row);
const paneActions = (/** @type {HTMLElement} */ anchor, /** @type {string} */ id, /** @type {{ x: number, y: number } | undefined} */ point = undefined) => menuOf(anchor, paneRows(id), point ?? null);
/** @returns {Action[]} */
function paneRows(/** @type {string} */ id, inPalette = false) {
  const i = chart.panes.findIndex((pane) => pane.id === id), many = chart.panes.length > 1;
  if (i < 0) return [];
  const layers = chart.layers.filter((layer) => layer.pane === id), shown = paneShows(chart, id);
  return [
    ...(i > 0 ? [{ key: "up", icon: ICONS.arrowUp, label: "Move up", aside: inPalette ? `Swaps with pane ${i}` : undefined, verb: "Move", stay: true, run: () => movePane(id, -1) }] : []),
    ...(i < chart.panes.length - 1 ? [{ key: "down", icon: ICONS.arrowDown, label: "Move down", aside: inPalette ? `Swaps with pane ${i + 2}` : undefined, verb: "Move", stay: true, run: () => movePane(id, 1) }] : []),
    // A chosen height: a drag's is hard to get just so by touch.
    ...(sizable(i) ? [heightRow(id)] : []),
    { key: "eye", icon: shown ? ICONS.eyeOff : ICONS.eye, label: shown ? "Hide pane" : "Show pane", verb: shown ? "Hide" : "Show", stay: true, run: () => hideLayers(layers.map((layer) => layer.id), shown) },
    { key: "remove", icon: ICONS.trash, label: many ? "Remove the pane" : "Remove every layer", aside: plural(layers.length, "layer"), verb: "Remove", danger: true, run: () => removePane(id) },
  ];
}
function movePane(id, by) {
  // Where each pane is on screen now (mid-glide, where it shows), for the swap to start from.
  const groups = () => all$("#layer-list .pane-group"), key = (/** @type {HTMLElement} */ group) => group.dataset.pane;
  const was = placesOf(groups(), key);
  commit((doc) => {
    const from = doc.panes.findIndex((item) => item.id === id), to = from + by;
    if (from < 0 || to < 0 || to >= doc.panes.length) return;
    doc.panes.forEach((pane, k) => (pane.size ??= defaultSize(k)));
    [doc.panes[from], doc.panes[to]] = [doc.panes[to], doc.panes[from]];
  });
  slide(groups(), key, was, { moved: id });
  refocus(`[data-pane-menu="${id}"]`);
}
function removePane(id) {
  const at = chart.panes.findIndex((pane) => pane.id === id), only = chart.panes.length === 1;
  const ids = chart.layers.filter((layer) => layer.pane === id).map((layer) => layer.id), count = ids.length, said = goneSaid(ids);
  commit((doc) => (doc.layers = doc.layers.filter((layer) => layer.pane !== id)));
  undoToast(`${only ? `Removed ${count === 1 ? "the layer" : `all ${count} layers`}` : `Removed a pane and its ${count === 1 ? "layer" : `${count} layers`}`}${said}`);
  requestAnimationFrame(() => /** @type {HTMLElement | null | undefined} */ (only ? document.querySelector("#empty [data-action='add-series']") : all$("#layer-list .pane-scale")[Math.max(0, Math.min(at, chart.panes.length - 1))])?.focus());
}

function markersHtml() {
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

// Whole screen where the browser gives it, asked in the same click (it must be); not when the window is the screen
// already or says so (Mullvad's does). A browser's own way out (Firefox's Escape, a held one in Chromium) leaves it
// too.
function setFull(on = !fullOn()) {
  if (on === fullOn()) return;
  layOutAlone(on);
  if (on) keepFocus(document.activeElement?.closest(".sidebar, #panel, .card-head") ?? null, () => $("full"), () => $("plot-keys"));
  if (on) {
    const screenWide = outerWidth >= screen.width && outerHeight >= screen.height;
    // Chromium gives a page asking for it every Escape but a held one; out again before the browser answered, out.
    if (!document.fullscreenElement && !screenWide) document.documentElement.requestFullscreen?.({ navigationUI: "hide" }).then(() => (fullOn() ? /** @type {any} */ (navigator).keyboard?.lock?.(["Escape"]) : document.exitFullscreen())).catch(() => {});
  } else {
    /** @type {any} */ (navigator).keyboard?.unlock?.();
    if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
  }
}
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
const LEGENDS = /** @type {const} */ ([["off", "Off", "No legend over the panes", "None"], ["", "On", "A legend over each pane, its values where you point", "Over each pane"]]);
const legendAction = (group = "") => ({ key: "legend", icon: ICONS.legend, label: "Legend", aside: LEGENDS.find(([id]) => id === (chart.legend ?? ""))?.[1], group, page: () => choicePage("Legend", () => LEGENDS.map(([id, text, , note]) => ({ key: id || "on", icon: ICONS.legend, label: text, aside: note, current: (chart.legend ?? "") === id, run: () => setLegend(id) }))) });
const setLegend = (/** @type {string} */ mode) => (chart.legend ?? "") !== mode && commit((doc) => (mode === "off" ? (doc.legend = "off") : delete doc.legend), { look: true });
/**
 * Measured ahead while the list is behind (a phone's sheet put away): its opening doesn't wait for them (slow on a
 * phone without a JIT), a layer at a time in idle moments.
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
/** @param {string} moved @param {number | null} y @param {() => void} change */
function slideLayers(moved, y, change) {
  const list = $("layer-list"), was = LIST_PARTS.map(([selector, key]) => placesOf(all$(selector, list), key));
  const row = /** @type {HTMLElement | null} */ (list.querySelector(`.layer-row[data-layer="${moved}"]`));
  if (row && y !== null) was[0].set(moved, y - row.offsetHeight / 2);
  change();
  LIST_PARTS.forEach(([selector, key, options], k) => slide(all$(selector, list), key, was[k], { ...options, moved: k === 0 ? moved : null }));
}
const LIST_PARTS = /** @type {[string, (element: HTMLElement) => string | undefined, { fade?: boolean, pseudo?: string }][]} */ ([
  [".layer-row[data-layer]", (row) => row.dataset.layer, {}],
  [".markers", () => "markers", {}],
  [".pane-head", (head) => /** @type {HTMLElement} */ (head.closest("[data-pane]")).dataset.pane, { fade: true }],
  [".pane-group + .pane-group", (group) => group.dataset.pane, { fade: true, pseudo: "::before" }],
  [".layer-adds", () => "adds", {}],
]);
export {
  copyPreset, dropLayers, 
  layOutAlone, legendAction, LEGENDS, markersHtml,
  missedSaid, openPreset, openPresetMenu, ordersOf, paneActions, paneChip, paneRows, presetActions, 
  
  presetPages, presetRow, presetsArrived, presetsGroup, presetsPage, 
  RECOMMENDED, refocus, resetPreset, revealNearest, roomsAhead, scaleGroup, scaleMenu, setFull,
  setLegend, setPaneGradient, setPaneMode, setPaneOutline, setPaneReverse, setPaneStack, slideLayers, slipsOf, tick, toggleHalvings,
  toggleLog, vocabNow,
};
