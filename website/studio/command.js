// @ts-check
// ⌘K: series search, its pages, colors looked at, go to a date.
import { initialsRanges } from "/studio/initials.js";
import { capitalized, holderName, inSentence, isLeaf, windowSaid, windowShort } from "/studio/words.js";
import { startWorker } from "/studio/workers.js";
import {
  acrossUnfit, answers, averageFor, byBlock, byKeys, canLook, canTravel, catalogIndex, changesOf, chart, chartById,
  chooseDrawings, chosen, clearLook, client, closePreview, cohortColor, colorName, commit, comparePeriods, css,
  dateGoes, deleteChart, dismissPalette, drawingsHidden, drawTrail, editChart, engine, EXAMPLE, folderMark, folderNow,
  formatName, formulaLine, fromSpan, goneLayer, goTo, goToDate, goToTime, HEAT_COHORTS, hideBelt, hideDrawings,
  historyFor, home, homeAt, importCharts, indexAt, isOhlc, isPreview, isYours, keepChart, letterChip, library,
  libraryEdits, linkView, listedPreviews, magnifying, markFor, marksOf, measure, mostBars, MOTION, noteUse, notice,
  openPalette, paletteOnField, paletteOpenings, panelShown, paneShowing, pathResume, phone, prefs, presets, previews,
  previousChart, quick, quickHide, quickShow, quickWorks, recentCharts, redoLast, reduced, renderPalette, reshape,
  resolved, revision, say, selected, seriesCount, setSingleKeys, SHARE, sharedTops, SHARES_IMAGES, SHEET_SHARE,
  showGuide, showLook, showPalette, showShortcuts, signalShown, STACKED_ONLY, startTyping, studioVersion, tabletHead,
  takeUp, THEMES, timeAt, togglePanel, toggleSidebar, tokensMedia, tool, toolKept, tree, trendShort, undoLast,
  updatePaletteKeys, used, chartWindow,
} from "/studio/app.js";
import { aside, isSeriesName } from "/studio/catalog-walk.js";
import { $, drawKeys, escapeHtml, keycaps, keyText, proseHtml, TOUCH, viewport } from "/studio/dom.js";
import { inSlices } from "/studio/timing.js";
import {
  ago, dateLabel, format, grouped, humanize, humanizeKey, MONTH_NAMES, monthYear, plural, UNIT_NAMES,
  unitOf,
} from "/studio/format.js";
import { dashGlyph, glyph, glyphSvg, ICONS, markBody, OUTLINE_GLYPH, shapeGlyph } from "/studio/icons.js";
import {
  belowOf, BUBBLE_HIDE, BUBBLE_HIDES, BUBBLE_SIZE, BUBBLE_SIZES, dashOf, defaultOpacity, fittable, follows, FORMULA_MAX,
  GRADIENT_LEAST, GRADIENT_MOST, heatLike, hideText, hollowOf, LINED, makeLayer, MARK_PLACES, markable, MAX_LEVELS,
  drawnInto, gradable, MAX_PANES, measured, MODES, modesOf, nameLayer, paneGradient, newPaneIn, OPACITIES, opacityOf, OUTLINED, OUTLINES, ownGradient, outlinable, placedOn, placeName, resultText,
  seriesLooked, SHAPES, STACKS, stretchResult, summed, typeName, TYPES,
} from "/studio/vocabulary.js";
import {
  BACKS, ENDS, KINDS, labelText, LENS_SAYS, LINES, SIZE_NAMES, SIZES, TEXT_SIZE, THROUGH, TOOLS,
} from "/studio/tools.js";
import { CADENCE, FINE, INTERVAL_DAYS, INTERVAL_LABEL, INTERVAL_ORDER, RANGES } from "/studio/intervals.js";
import { isColor, oklchToHex, SWATCHES } from "/studio/color.js";
import { lowerBound } from "/studio/math.js";
import {
  baseOf, BY_BLOCK, byColor, colorsAt, cssColor, dailyOf, defaultSize, drawnWaiters, failure, fineDrawn, foldedPanes,
  info, ink, known, lastUsed, loads, oklchOf, overviewSource, paintOf, periodOf, pointLabel, presetFits,
  presetShown, rangeAt, resolve, retryInfo, shownIndex, shownRange, shownSpan, unitFor, units, viewName, zoom,
} from "/studio/chart.js";
import { backGlyph, backOf, beltUp, dockChoices, drawingById, drawingNames, endName, endsGlyph, showBelt } from "/studio/drawing.js";
import {
  bandSum, COMPARES, FUNCTIONS, isBandSum, parse, PERIODS_TAKEN, plainOperators, refsOf, seriesNamed, shiftSpan,
  shiftSpanError, unfitHere, VALUES,
} from "/studio/formulas.js";
import {
  areaUnder, batch, chartCounts, chartKeys, chartMark, chartPick, childrenOf, clearPicks, derivedTitle, drawPreview, fadeDef, folderById,
  folderChoices, folderPath, folderRows, giveUpUnframed, go, isOpen, lineOver, moveToFolder, newChart, newFolder, outOf, pickChart,
  picked, placeNames, previewKey, previewSvg, seePaletteMarks, setTheme, stepChart, titleOf, treeOf,
} from "/studio/grid.js";
import {
  clearDrawings, drawingRows, duplicateDrawings, finishPath, flipDrawings, flipsOf, HEAT_ABOUT, HEAT_SHORT,
  heatReading, heatRows, placingPath, PROFILE_ABOUT, PROFILE_SHORT, readingLine, removeDrawings, selectInView,
} from "/studio/heatmaps.js";
import {
  addFormula, addFormulaLayer, addHeatmap, addSeries, addSignal, addSignalLayer, addStop, changeSeries, cloudable,
  colorModeChoices, colorModeOf, colorOf, colorSlots, completeSeries, completion, completionsAt, copyName, dashRow,
  editById, faintOf, fillOf, fitChoices, fitOf, FITS, fitsAt, formatChoices, formulaInputs, fromChoices, fromPage,
  gradientSwatch, isUnfinished, labelOf, layerActions, layerById, layersCommands, levelValue, lineChoices, lineLook,
  lineLookOn, lineStyleGlyph, lineStyleName, modeName, modeSwatch, moveLayer, moveStop, named, overShort, paneChoices,
  paneMode, periodsOf, rangeOf, readNumber, readsOf, readyName, readyPage, recipesFor, refChoices, removeStop,
  replaceSeries, reverseGradient, setColor, setFit, setLineLook, showEvery, slotName, stackedIn, stackedOnly,
  TAG_NEEDS_LINE, usesOf, widthRow, withLevel, withoutLevel,
} from "/studio/inspector.js";
import {
  anythingKept, downloadAll, exportAs, exportRows, fadeOut, hideMenu, NO_INTERVAL, renameInPlace, setRange, showMenu,
  supportedIntervals, switchInterval, switchWindow,
} from "/studio/menus.js";
import {
  copyPreset, fullOn, legendAction, LEGENDS, ordersOf, paneChip, paneRows, presetActions, presetEdited, presetIndex,
  presetPages, presetRow, presetsPage, resetPreset, scaleGroup, setFull, setLegend, setPaneMode, toggleHalvings,
} from "/studio/presets-edit.js";
/** @import { Chart, Layer, Pane } from "/studio/vocabulary.js" */
/** @import { Result } from "/studio/chart.js" */
/** @import { Vector } from "/studio/formulas.js" */
/** @import { Place } from "/studio/grid.js" */
/** @import { Preset } from "/studio/presets-edit.js" */


/**
 * The search's worker: questions wait for its index to be built. Without workers the same engine runs on the page.
 * One question is with it at a time, the others wait their turn, and one given up (typing went on) leaves the line:
 * the latest is never stuck behind stale ones.
 */
const finder = {
  /** Started (once: one that failed isn't again). */
  started: false,
  ready: false,
  worker: /** @type {Worker | null} */ (null),
  /** Questions waiting their turn: a search ({ query, limit }), or what a series is ({ about }). @type {{ message: object, reply: (data: any) => void }[]} */
  line: [],
  /** @type {((data: any) => void) | null} */
  asking: null,
};
function askNext() {
  const next = !finder.asking && finder.worker ? finder.line.shift() : undefined;
  if (!next) return;
  finder.asking = next.reply;
  finder.worker?.postMessage(next.message);
}
/** What the catalog says of a series (as its info does), from this machine's copy; null if it isn't there or can't say. */
const aboutOf = (/** @type {string} */ name) => new Promise((resolve) => {
  if (!finder.worker) return resolve(pageFinder ? pageFinder.then((answer) => answer({ about: name }).about ?? null).catch(() => null) : null);
  if (!finder.ready) return resolve(null);
  finder.line.push({ message: { about: name }, reply: (data) => resolve(data?.about ?? null) });
  askNext();
});
/** The same engine without a worker, built only when workers cannot run. */
let pageFinder = /** @type {ReturnType<typeof import("/studio/series-search.js").seriesSearch> | null} */ (null);
/** What the search is built from, on the page or in its worker: the catalog's URL and the intervals Studio charts. */
const finderFrom = () => ({ catalog: `${client.baseUrl}/api/series`, dated: INTERVAL_ORDER });
function pageSearch() {
  const { catalog, dated } = finderFrom();
  return (pageFinder ??= import("/studio/series-search.js").then(({ seriesSearch }) => seriesSearch(catalog, dated)).catch((error) => { pageFinder = null; throw error; }));
}
function startFinder() {
  if (finder.started) return;
  finder.started = true;
  if (typeof Worker !== "function") return;
  // (Workers turned off for the page: the page's engine answers; its start never fails with it.)
  let worker;
  try {
    worker = finder.worker = startWorker(new URL("./search-worker.js", import.meta.url));
  } catch {
    return; // (startWorker said why.)
  }
  // (Its first message builds the engine; questions sent after it wait for that.)
  worker.postMessage(finderFrom());
  // A worker that fails hands its questions to the same engine on the page.
  const stop = () => {
    worker.terminate();
    const waiting = [finder.asking, ...finder.line.splice(0).map((question) => question.reply)];
    Object.assign(finder, { ready: false, worker: null, asking: null });
    for (const reply of waiting) reply?.(null);
  };
  worker.onerror = stop;
  worker.onmessage = ({ data }) => {
    if (!data || data.broken !== undefined) return console.error("search-worker.js: its index could not be built", data?.broken), stop();
    if (data.ready) return void (finder.ready = true);
    const reply = finder.asking;
    finder.asking = null;
    reply?.(data);
    askNext();
  };
}
/** Answers had already, by limit and query (the catalog stays as it is while the page is open): the oldest go past a few hundred. */
const searched = new Map();
/**
 * Series for a query, best first: a kept answer, else the worker's, or the same browser engine on the page when
 * workers cannot run. More typing drops a queued question before it is searched.
 * (`waiting`: told when the worker cannot run.)
 * @param {string} query @param {number} limit @param {AbortSignal} [signal] @param {() => void} [waiting] @returns {Promise<string[]>}
 */
function findSeries(query, limit, signal, waiting = () => {}) {
  if (signal?.aborted) return Promise.reject(signal.reason);
  const key = `${limit}\n${query}`, kept = searched.get(key);
  if (kept) return Promise.resolve(kept);
  startFinder();
  const asked = () => {
    if (signal?.aborted) return Promise.reject(signal.reason);
    waiting();
    return pageSearch().then((answer) => { signal?.throwIfAborted(); return answer({ query, limit }).names ?? []; });
  };
  let onAbort = () => {};
  return new Promise((resolve, reject) => {
    if (finder.worker) {
      const question = { message: { query, limit }, reply: (/** @type {any} */ data) => {
        if (signal?.aborted) return;
        if (data?.failed) reject(new Error(data.failed));
        else if (data?.names) resolve(data.names);
        else asked().then(resolve, reject);
      } };
      finder.line.push(question);
      onAbort = () => {
        const at = finder.line.indexOf(question);
        if (at >= 0) finder.line.splice(at, 1);
        reject(signal?.reason);
      };
      signal?.addEventListener("abort", onAbort, { once: true });
      return askNext();
    }
    asked().then(resolve, reject);
  }).then((/** @type {string[]} */ names) => {
    searched.set(key, names);
    if (searched.size > 300) searched.delete(/** @type {string} */ (searched.keys().next().value));
    return names;
  }).finally(() => signal?.removeEventListener("abort", onAbort));
}


/**
 * What ⌘K's doing: browsing everything; adding a series (`into` a pane chosen beforehand: its id, or "new", each into a
 * new one); replacing a `layer`'s series; a formula, added (`into` a pane) or a layer's own, `editing`.
 * @typedef {{ kind: "browse" } | { kind: "add", into: string | null, bubbles?: boolean } | { kind: "replace", layer: string } | { kind: "formula", editing: string | null, into: string | null, signal?: boolean, side?: string | null }} PaletteOp
 */
const palette = {
  path: /** @type {string[]} */ ([]),
  items: /** @type {any[]} */ ([]),
  active: 0,
  query: "",
  controller: /** @type {AbortController | null} */ (null),
  /** The rows waiting for this machine's series to be drawn with (↵ draws them first). @type {(() => void) | null} */
  pending: null,
  moved: false,
  /** @type {string | null} the row to land on when going back up (the folder or page just left) */
  landOn: null,
  op: /** @type {PaletteOp} */ ({ kind: "browse" }),
  /** Adding a series: series and the catalog only. */
  get adding() {
    return this.op.kind === "add";
  },
  /** Adding a series as bubbles (on the chart's first layer that can carry them). */
  get bubbles() {
    return this.op.kind === "add" && Boolean(this.op.bubbles);
  },
  /** The layer whose series a pick replaces. */
  get replacing() {
    return this.op.kind === "replace" ? this.op.layer : null;
  },
  /** A formula: the query is the formula, the rows what completes it. */
  get formula() {
    return this.op.kind === "formula";
  },
  /** A signal's formula: marks where it's true, on a layer (else a formula's, drawn as a layer of its own). */
  get signal() {
    return this.op.kind === "formula" && Boolean(this.op.signal);
  },
  /** The formula layer being edited (else one's being added). */
  get editing() {
    return this.op.kind === "formula" ? this.op.editing : null;
  },
  /** The pane added into, chosen beforehand ("new": each into a new one). */
  get into() {
    return this.op.kind === "add" || this.op.kind === "formula" ? this.op.into : null;
  },
  /** How many of its rows are written (the rest as they're scrolled or moved to). */
  drawn: 0,
  /** The pages opened, deepest last (⌫ goes back up). @type {Page[]} */
  pages: [],
  /** The search itself, as left for a page: what was typed, the row that led on. @type {{ query?: string, landOn?: string | null }} */
  root: {},
  /** In the formula page, where a chosen piece goes in: the start of the word it completes, or the caret. */
  from: 0,
};

/** The catalog's node at a path (its own keys only: no "constructor" off a prototype), or undefined. @param {string[]} path */
function nodeAt(path) {
  let node = /** @type {any} */ (tree);
  for (const key of path) {
    if (!node || typeof node !== "object" || !Object.hasOwn(node, key)) return undefined;
    node = node[key];
  }
  return node;
}
/** How many series a catalog folder holds, all the way down; counted once. */
const seriesCounts = new WeakMap();
function seriesUnder(node) {
  if (isLeaf(node)) return 1;
  if (!node || typeof node !== "object") return 0;
  if (!seriesCounts.has(node)) seriesCounts.set(node, Object.keys(node).reduce((sum, key) => sum + seriesUnder(aside(node, key)), 0));
  return seriesCounts.get(node);
}

/** Where a series was last added from in "Add a layer", this session: the next one starts there. */
let addedFrom = /** @type {{ chart: string, path: string[], key: string } | null} */ (null);
/** Whether ⌘K is opening a preset, browsing what's of no chart (the presets' pages, or its search). */
const presetPicked = () => palette.op.kind === "browse" && palette.pages.every((page) => page.global) && ((item) => item?.kind === "row" && /^(?:preset|recommended):/.test(item.key ?? ""))(palette.items[palette.active]);
/** Whether ⌘K is adding a series it's browsing to (its search, a folder). */
const seriesPicked = () => palette.op.kind === "browse" && palette.items[palette.active]?.kind === "series";
/**
 * ⌘K opened by Home's search (a click on it, typing in it; ⌘K itself, the sidebar's Search: ⌘K as everywhere),
 * browsing: on that search, as wide as it, its field over it (see
 * .palette[data-on-home]); kept on it as Home's box, or what of the page shows, changes. Where it can't be (a
 * phone's sheet, the field out of sight, or too low for ⌘K under it), where it always opens.
 */
function placeOnHome() {
  const dialog = $("palette"), at = paletteOnField && palette.op.kind === "browse" && home && "top" in homeAt && !phone.matches ? document.querySelector(".home-search")?.getBoundingClientRect() : undefined;
  // (What of the page shows: a tablet's keyboard covers its foot.)
  const foot = viewport.offsetTop + viewport.height;
  const fits = Boolean(at && at.top >= viewport.offsetTop && foot - at.top - 16 >= 240);
  dialog.toggleAttribute("data-on-home", fits);
  if (at && fits) for (const [name, value] of Object.entries({ x: at.left, y: at.top, w: at.width, room: foot - at.top - 16 })) dialog.style.setProperty(`--on-${name}`, `${value}px`);
}
// (Home's field out of sight while ⌘K's on it, back as it goes (closed, or leaving first: see leave).)
const underHome = (open = isOpen($("palette"))) => ((dialog) => document.querySelector(".home-search")?.toggleAttribute("data-under", open && dialog.hasAttribute("data-on-home") && !dialog.dataset.leaving && !dialog.hasAttribute("data-arriving")))($("palette"));
/**
 * ⌘K's field given the keys (`select`: what's in it selected): by a pointer, always; by touch, only when it has them
 * already (the keyboard up stays up), else ⌘K itself: a tap on the field brings the keyboard, not every tap in ⌘K.
 */
function paletteKeys(select = false) {
  const input = $("palette-input");
  if (TOUCH && !byKeys && document.activeElement !== input) return void (!$("palette").contains(document.activeElement) && $("palette").focus());
  input.focus();
  if (select) input.select();
}
/**
 * A key only a keyboard sends (an arrow, ⇥, esc, one held with ⌘, ⌃ or ⌥), not a screen's (letters, as a phone's
 * types them): a touch device's keyboard (an iPad's, no trackpad) in use, said by the page (`data-keyboard`) till a
 * finger's down again; ⌘K by its keys then, its foot (where it has one: Home's hasn't) and the row they light.
 */
const keyboardKey = (/** @type {KeyboardEvent} */ event) => event.metaKey || event.ctrlKey || event.altKey || /^(Arrow|Tab$|Escape$|Page|Home$|End$|F\d)/.test(event.key);
const keyed = (/** @type {boolean} */ on) => on !== keyboard() && document.documentElement.toggleAttribute("data-keyboard", on);
/** Whether keys a keyboard sends are what's in use (no touch device: always; one, its keyboard's, see keyboardKey). */
const keyboard = () => !TOUCH || document.documentElement.hasAttribute("data-keyboard");
/** A pane's number, as the panel counts them (from 1). */
const paneNumber = (/** @type {string} */ id) => chart.panes.findIndex((pane) => pane.id === id) + 1;
/**
 * The chip and the wording for where ⌘K is: everything, adding a series, or replacing one. The chip says what's
 * made, in a word (a layer's by its letter, a pane after a dot), the field the rest: a phone's width is the field's.
 */
function applyScope() {
  const chip = $("palette-scope"), input = $("palette-input");
  const replacing = palette.replacing && layerById(palette.replacing)?.series;
  chip.hidden = (!palette.adding && !palette.formula && !replacing) || palette.pages.length > 0;
  const into = palette.into === "new" ? " · new pane" : palette.into ? ` · pane ${paneNumber(palette.into)}` : "";
  chip.textContent = replacing ? `Replace ${layerById(palette.replacing)?.letter ?? ""}` : palette.formula ? (palette.editing ? `Edit ${layerById(palette.editing)?.letter ?? "formula"}` : palette.signal ? "Signal" : `Formula${into}`) : palette.bubbles ? "Bubbles" : `Layer${into}`;
  $("palette").dataset.mode = palette.formula ? "formula" : "";
  chip.disabled = Boolean(replacing);
  if (replacing) delete chip.dataset.tip;
  else Object.assign(chip.dataset, { tip: "Back to everything", kbd: "⌫" });
  // As long as the fields it stands in for take: a formula, as long as one read from a link; a search, any.
  if (palette.formula) input.maxLength = FORMULA_MAX;
  else input.removeAttribute("maxlength");
  input.setAttribute("aria-label", replacing ? "Replace series" : palette.formula ? "Formula" : palette.adding ? "Search series" : "Search series, charts and commands");
  input.placeholder = replacing ? "A series to use instead" : palette.signal ? "When it's true, like A > sma(A, 200d)" : palette.formula ? `A formula, like A / B or ${EXAMPLE}` : palette.adding ? (seriesCount ? `Search ${grouped.format(seriesCount)} series` : "Search every series") : "Search series, charts and commands";
  $("palette-count").textContent = seriesCount ? `${grouped.format(seriesCount)} series on Bitview` : "Bitview";
}
/**
 * Up the catalog to `depth`, landing on the folder you came out of (as a tree does), not at the top of the list.
 */
function goUp(depth) {
  palette.landOn = palette.path[depth] ?? null;
  palette.path = palette.path.slice(0, depth);
  showPalette();
}
/** Out of the "Add a layer" or "Add a formula" page, to everything ⌘K does: on the command that stands for the page. */
function leaveScope() {
  palette.landOn = palette.formula ? "Add a formula" : "Add a layer";
  if (palette.formula) palette.query = $("palette-input").value = "";
  palette.op = { kind: "browse" };
  palette.path = [];
  applyScope();
  showPalette();
  paletteKeys();
}
/** The trail under the search, and the wording that goes with where you are. */
function renderPath() {
  if (palette.pages.length) return renderPages();
  const path = palette.path;
  $("palette-path").hidden = !path.length;
  drawTrail([["Catalog", 0], ...path.map((key, i) => [humanizeKey(key), i + 1])], path.length);
  const chip = $("palette-scope");
  if (palette.replacing) chip.disabled = !path.length;
  if (!palette.replacing || path.length) Object.assign(chip.dataset, { tip: path.length ? "Back to the catalog" : "Back to everything", kbd: "⌫" });
  else delete chip.dataset.tip;
  if (path.length) $("palette-input").placeholder = "Search this folder, then every series";
  else applyScope();
}

/** Every pane's height set at once: all alike, or as at first. */
const PANE_HEIGHTS = [
  { key: "equal", label: "Equal pane heights", text: "Makes every pane the same height", verb: "Apply", set: (/** @type {Pane} */ pane) => (pane.size = 1) },
  { key: "reset", label: "Reset pane heights", text: "Restores the default heights: a tall first pane and shorter panes below", verb: "Reset", set: (/** @type {Pane} */ pane) => delete pane.size },
];
/** Pane heights set, as a divider's drag sets them: a layout (no "Saved"; a chart started here not kept by it). */
const setPaneHeights = (/** @type {(pane: Pane) => void} */ set) => commit((doc) => doc.panes.forEach(set), { quiet: true });
/** The pane a scale change acts on: the selected layer's, else the first. */
const scalePane = () => chart.panes.find((item) => item.id === layerById(selected)?.pane) ?? chart.panes[0];

/**
 * @typedef {{ title: string, text: string, icon: string, kbd?: string, run?: () => any, suggested?: boolean,
 *   anywhere?: boolean, keepOpen?: boolean, danger?: boolean, follows?: string, current?: boolean }} Command
 *   `danger` marks it (it's undone as any other). With no `run`, it says how things are (found by typing only):
 *   `current`, the choice in effect, ticked; else not now (dim)
 */
/**
 * Everything ⌘K can do besides adding a series; `suggested` ones show before anything is typed, `anywhere` ones in
 * the grid too (the rest are of the open chart).
 * @returns {Command[]}
 */
function commands() {
  const supported = supportedIntervals();
  const collapsed = document.body.dataset.sidebar === "collapsed";
  const empty = !chart.layers.length;
  return /** @type {Command[]} */ ([
    { title: "Add a layer", text: "Search or browse every series, or add a formula, a cloud, bubbles, a signal, a trend or a heatmap.", icon: "plus", kbd: "A", run: () => openPalette({ add: true }), suggested: true, keepOpen: true },
    { title: "New chart", text: "A blank chart in your Library, to fill with series or a file.", icon: "plus", kbd: "N", run: newChart, suggested: true, anywhere: true },
    ...(isPreview(chart) && !home && chart.layers.length ? [{ title: "Save this chart", text: "Saves this preview to your Library, with your edits.", icon: "keep", run: () => keepChart(), suggested: true }] : []),
    ...(isPreview(chart) && !home && chart.layers.length && previousChart() ? [{ title: "Close this preview", text: `Closes ${titleOf(chart)} without saving it to your Library.`, icon: "close", run: () => closePreview() }] : []),
    { title: "New folder", text: "A folder for charts that belong together, named as you create it.", icon: "folderAdd", run: () => newFolder(), anywhere: true },
    // Each folder, by its name: its page (expand or collapse it, rename it, delete it, its charts).
    ...((counts) => library.folders.map((folder) => ({ title: folderPath(folder.id), text: `A folder: ${plural(counts.get(folder.id) ?? 0, "chart")}.`, icon: folderMark(folder), keepOpen: true, anywhere: true, run: () => openPage(folderOwnPage(folder.id)) })))(chartCounts()),
    // (Home: a place to go, unless it's shown. Recent's, your Library's, the presets' pages: their ⌘K pages' View.)
    ...(home && "top" in homeAt ? [] : [{ title: "Home", text: "Studio's start: its search, Recent, your Library, the presets.", icon: "home", anywhere: true, run: () => goTo({ top: true }) }]),
    // (A cloud, wherever there are layers (its pane's two edges are two); a trend, from layers.)
    ...(chart.layers.length ? [
      { title: "Add a cloud", text: "A fill between two or more layers (each pair of neighbors, in the upper one's color), or out to the pane's top or bottom.", icon: "cloud", keepOpen: true, run: () => pushPage(fromPage("cloud")) },
    ] : []),
    { title: "Add a heatmap", text: HEAT_ABOUT, icon: "heatmap", keepOpen: true, run: () => pushPage(heatAddPage()) },
    { title: "Add a profile", text: PROFILE_ABOUT, icon: "profile", keepOpen: true, run: () => pushPage(profileAddPage()) },
    ...(cloudable() ? [
      // (Not now, it says why: dimmed, nothing to run.)
      { title: "Add a trend", text: trendShort() ? `${trendShort()}.` : "From layers: on the lowest of them until a price touches it, then on the highest until it touches that.", icon: "riding", ...(trendShort() ? {} : { keepOpen: true, run: () => pushPage(fromPage("trend")) }) },
    ] : []),
    ...(empty || chart.panes.length >= MAX_PANES ? [] : [{ title: "Add to a new pane", text: "A series in a new pane, below the others.", icon: "pane", keepOpen: true, run: () => openPalette({ add: true, into: "new" }) }]),
    { title: "Add a formula", text: `A layer computed from others or from any series, like A / B or ${EXAMPLE}.`, icon: "formula", kbd: "F", run: addFormula, suggested: true, keepOpen: true },
    ...(markable(chart).length ? [{ title: "Add bubbles", text: "A series' values as bubbles on a layer, each as big as its value, like a day's realized profit and loss on the price.", icon: glyph("bubbles"), keepOpen: true, run: () => openPalette({ add: true, bubbles: true }) }] : []),
    ...(markable(chart).length ? [{ title: "Add a signal", text: "Marks on a layer where a formula turns up or down, like A > sma(A, 200d), or (A > B) - (A < B).", icon: "signal", run: addSignal, keepOpen: true }] : []),
    ...(empty ? [] : [{ title: SHARE, text: "A link that opens this chart for anyone.", icon: "share", run: () => $("share").click(), suggested: true }]),
    ...(empty ? [] : [{ title: "Open image", text: "The chart's image, in a new tab.", icon: "link", run: () => exportAs("open") }]),
    ...(!isYours(chart) ? [] : [{ title: "Move to folder", text: `${titleOf(chart)} ${chart.folder ? `is now in ${folderPath(chart.folder)}` : "isn't in a folder"}.`, icon: "folder", keepOpen: true, run: () => pushPage(folderPage([chart.id])) }]),
    ...(empty || !SHARES_IMAGES ? [] : [{ title: "Share image", text: "Share the chart's image to Photos, Messages or anywhere else.", icon: "share", run: () => exportAs("share") }]),
    ...(empty ? [] : [{ title: "Copy image", text: "Copies the chart's image, to paste anywhere.", icon: "copy", run: () => exportAs("copy") }]),
    ...(empty ? [] : [{ title: "Download image", text: "A PNG of the chart as shown, with its title (wider on a phone).", icon: "image", run: () => exportAs("png"), suggested: true }]),
    ...(empty ? [] : [{ title: "Download data", text: "A CSV of every layer's values (the hidden ones too), one row per date.", icon: "table", run: () => exportAs("csv") }]),
    { title: "Download chart file", text: "A JSON file you can import again later.", icon: "file", run: () => exportAs("json") },
    ...(anythingKept() ? [{ title: "Download all charts", text: "All charts in one file, with their folders, to import elsewhere.", icon: "download", run: () => downloadAll(), anywhere: true }] : []),
    { title: "Import charts", text: "A chart file, or a file of several charts exported together.", icon: "upload", run: () => importCharts(), anywhere: true },
    // A choice among a few (a theme, an interval, a scale, a legend): each, the one in effect ticked (typed, it says so).
    ...THEMES.map(([id, name, icon, text]) => ({ title: `Theme: ${name}`, text, icon, anywhere: true, ...(id === prefs.theme ? { current: true } : { run: () => setTheme(id) }) })),
    ...(empty ? [] : RANGES.map(([id], i) => ({ id, i })).filter(({ id }) => presetFits(id)).map(({ id, i }) => ({ title: `Show the ${periodOf(id)}`, text: id !== "max" ? "Change the visible range." : FINE.has(chart.index) ? "As far back as fits on the screen (at a day or coarser, all of history), the price scales back on auto." : "All of history, the price scales back on auto.", icon: "charts", kbd: String(i + 1), run: () => setRange(id) }))),
    ...INTERVAL_ORDER.map((id) => ({ follows: `${INTERVAL_LABEL[id]} ${CADENCE[id]}`, title: `Interval: ${CADENCE[id]}`, text: supported(id) ? `Redraw every layer at ${INTERVAL_LABEL[id]}.` : `${NO_INTERVAL}.`, icon: "recent", ...(id === chart.index ? { current: true } : supported(id) ? { run: () => switchInterval(id) } : {}) })),
    ...((found) => (found?.offered ?? []).map((key) => ({ follows: windowShort(key), title: `Window: ${windowSaid(key)}`, text: `Each value its series over the ${windowSaid(key).replace(/^1 /, "")} before it.`, icon: "hourglass", ...(key === found?.window ? { current: true } : { run: () => switchWindow(key) }) })))(empty ? null : chartWindow(chart)),
    ...(empty || summed(scalePane()) ? [] : [scalePane()].flatMap((pane) => /** @type {[number, string][]} */ ([[1, "Logarithmic scale"], [0, "Linear scale"]]).map(([mode, title]) => ({ title, text: `For ${chart.panes.length > 1 ? `pane ${chart.panes.indexOf(pane) + 1}, ${selected ? "the selected layer's" : "the first"}` : "the chart"}.`, icon: "axis", ...(paneMode(pane) === mode ? { current: true } : { kbd: paneMode(pane) < 2 ? "L" : undefined, run: () => setPaneMode(pane.id, mode) }) })))),
    ...(empty ? [] : [
      { title: beltUp() ? "Hide the drawing tools" : "Show the drawing tools", text: beltUp() ? "They show again while drawing, or with a drawing selected." : "The drawing tools, on the chart at all times.", icon: "draw", run: () => (beltUp() ? hideBelt() : showBelt()) },
      ...(tool ? [placingPath() ? { title: "Finish the path", text: "Keeps it as it is (two points at least); the chart pans again.", icon: tool, kbd: "esc", run: () => finishPath() } : { title: "Stop drawing", text: `Stops drawing; the chart pans again.`, icon: tool, kbd: "esc", run: () => takeUp("") }] : []),
      ...(tool && tool !== "text" && !toolKept ? [{ title: "Keep drawing", text: `Keeps the tool after each drawing, to draw several (esc to stop; double-clicking a tool does the same).`, icon: tool, run: () => takeUp(tool, true) }] : []),
      ...KINDS.filter((kind) => kind !== tool).map((kind) => ({ title: TOOLS[kind].verb, text: kind === "text" ? `${TOOLS[kind].tip}.` : `${TOOLS[kind].tip}: one, then the chart pans again (to draw several, ${TOUCH ? "tap its tool twice" : "double-click its tool"}, or Keep drawing).`, icon: kind, kbd: kind === "ruler" ? "M" : `D ${TOOLS[kind].key}`, run: () => takeUp(kind) })),
      { title: "Select drawings", text: `Drag a box around drawings to select them (or ${keyText("⌘")} and drag anytime); esc to stop.`, icon: "select", kbd: "D S", run: () => takeUp("select") },
      { title: magnifying ? "Stop the magnifier" : "Magnifier", text: magnifying ? "The pointer is the pointer again." : `Shows ${LENS_SAYS}${TOUCH ? ", above it as you press" : ""}; esc to stop.`, icon: "magnifier", kbd: "D Z", run: () => takeUp(magnifying ? "" : "magnifier") },
      ...(prefs.belt.on ? [{ title: "Dock the drawing tools", text: `Currently on the ${prefs.belt.dock} of the chart.`, icon: "draw", keepOpen: true, run: () => pushPage(choicePage("Dock the drawing tools", dockChoices)) }] : []),
      ...(chart.drawings?.length ? [{ title: drawingsHidden ? "Show the drawings" : "Hide the drawings", text: drawingsHidden ? "Shows them on the chart again." : "Hides the drawings without deleting them.", icon: drawingsHidden ? "eye" : "eyeOff", kbd: "⇧H", run: () => hideDrawings(!drawingsHidden, true) }] : []),
      ...(chart.drawings?.length ? [{ title: "Remove every drawing", text: `${plural(chart.drawings.length, "drawing")}. You can undo.`, icon: "trash", danger: true, run: clearDrawings }] : []),
    ]),
    ...layersCommands(),
    // The selected drawings, as the picked layers: what the belt does to them.
    ...(((ids) => home || !ids.length ? [] : ((n) => [
      ...flipsOf(ids).map(([across, way, icon, text]) => ({ title: `Flip ${n} ${way}`, text: `${text}, in place${ids.length > 1 ? ", as one" : ""}.`, icon, run: () => flipDrawings(ids, across), suggested: true })),
      { title: `Duplicate ${n}`, text: "Each copy beside its original.", icon: "duplicate", kbd: "⌘D", run: () => duplicateDrawings(ids), suggested: true },
      { title: `Remove ${n}`, text: "You can undo.", icon: "trash", kbd: "⌫", danger: true, run: () => removeDrawings(ids), suggested: true },
      { title: "Clear the selection", text: `${n} selected.`, icon: "close", kbd: "esc", run: () => chooseDrawings([]), suggested: true },
    ])(plural(ids.length, "drawing")))([...chosen])),
    ...(empty ? [] : [{ title: chart.halvings ? "Hide the halvings" : "Show the halvings", text: chart.halvings ? "The lines at each halving, off this chart." : "A line at each halving, and at the next three, estimated.", icon: "halvings", run: toggleHalvings }]),
    ...(empty ? [] : LEGENDS.map(([id, text, tip]) => ({ title: `Legend: ${text}`, text: `${tip}.`, icon: "legend", ...(id === (chart.legend ?? "") ? { current: true } : { run: () => setLegend(id) }) }))),
    ...(empty && !fullOn() ? [] : [{ title: fullOn() ? "Exit full screen" : "Full screen", text: "The chart alone, on the whole screen.", icon: fullOn() ? "shrink" : "expand", kbd: "⇧F", run: () => setFull() }]),
    { title: prefs.linkView ? "Unlink the view" : "Link the view", text: prefs.linkView ? "Each chart goes back to its own dates; this one stays where it is." : "Every chart on the same dates, in every tab: moving the view moves it for all.", icon: "chain", kbd: "V", run: () => linkView(!prefs.linkView) },
    ...(empty ? [] : [{ title: panelShown() ? "Hide the layers panel" : "Show the layers panel", text: "The chart's layers and their settings.", icon: "panel", kbd: "P", run: () => togglePanel() }]),
    ...(phone.matches ? [] : [{ title: collapsed ? "Expand the sidebar" : "Collapse the sidebar", text: "Your Library and the presets, beside the chart.", icon: "sidebar", kbd: "\\", run: toggleSidebar, anywhere: true }]),
    { title: "Studio's version", text: studioVersion ? `${studioVersion}, the same as Bitview's.` : "The server hasn't reported its version yet.", icon: "recent", run: () => notice(studioVersion ? `Bitview Studio ${studioVersion}` : "The server hasn't reported its version yet"), anywhere: true },
    ...(!keyboard() ? [] : [{ title: "Shortcuts", text: "Every keyboard shortcut in Studio.", icon: "keyboard", kbd: "?", run: showShortcuts, anywhere: true }]),
    ...(!keyboard() ? [] : [true, false].map((on) => ({ title: `Single-key shortcuts: ${on ? "On" : "Off"}`, text: on ? "A letter, a digit or a sign alone acts (A adds a series, P the panel…)." : `Only shortcuts with ${keyText("⌘")} or ${keyText("⌥")} act: typing a letter by mistake does nothing.`, icon: "keyboard", anywhere: true, ...(prefs.keys === on ? { current: true } : { run: () => setSingleKeys(on) }) }))),
    { title: "How links work", text: "What a chart link holds, and how to write one or have an assistant write it.", icon: "link", run: showGuide, anywhere: true },
    canTravel(true) ? { title: "Undo", text: placingPath() ? "The path's last point taken back." : historyFor(true) === null ? `${libraryEdits.past.at(-1)?.text}.` : "Back one change.", icon: "undo", kbd: "⌘Z", run: undoLast, anywhere: true } : { title: "Undo", text: "Nothing to undo.", icon: "undo", anywhere: true },
    // The charts picked in the grid: what its bar does.
    ...(home && picked.size ? [
      ...(picked.size === 1 ? [{ title: `Rename “${titleOf(chartById([...picked][0]) ?? { title: "" })}”`, text: "Type a new title.", icon: "pencil", run: () => batch("rename"), anywhere: true }] : []),
      { title: `Move ${plural(picked.size, "chart")} to folder`, text: "Into a folder, a new folder, or out of their folder.", icon: "folder", keepOpen: true, run: () => openPage(folderPage([...picked])), anywhere: true },
      { title: "Clear the selection", text: plural(picked.size, "chart") + " selected.", icon: "close", kbd: "esc", run: () => clearPicks(), anywhere: true },
      { title: `Duplicate ${plural(picked.size, "chart")}`, text: "A copy of each, placed next to it.", icon: "duplicate", run: () => batch("duplicate"), anywhere: true },
      { title: `Download ${plural(picked.size, "chart")}`, text: "Downloads a file you can import later.", icon: "file", run: () => batch("export"), anywhere: true },
      { title: `Delete ${plural(picked.size, "chart")}`, text: "You can undo.", icon: "trash", run: () => batch("delete"), anywhere: true, danger: true },
    ] : []),
    canTravel(false) ? { title: "Redo", text: placingPath() || pathResume ? "The path's point put back." : "Forward again.", icon: "redo", kbd: "⇧⌘Z", run: redoLast, anywhere: true } : { title: "Redo", text: "Nothing to redo.", icon: "redo", anywhere: true },
    ...(empty ? [] : [
      { title: "Zoom in", text: "Zooms in on the middle of the view (on its right edge when at the latest data).", icon: "magnifier", kbd: "+", run: () => zoom(1 / 1.6) },
      { title: "Zoom out", text: "Zooms out from the middle of the view (from its right edge when at the latest data).", icon: "zoomOut", kbd: "−", run: () => zoom(1.6) },
    ]),
    ...(chart.layers.some((layer) => signalShown(layer)) ? [
      { title: "Next signal", text: "The signal's next mark, after the middle of the view.", icon: "signal", kbd: ".", run: () => stepSignal(1) },
      { title: "Previous signal", text: "The signal's previous mark, before the middle of the view.", icon: "signal", kbd: ",", run: () => stepSignal(-1) },
    ] : []),
    ...(empty ? [] : [{ title: "Go to a date", text: "Type a year, month or day to show it.", icon: "calendar", kbd: "G", keepOpen: true, run: () => pushPage(datePage()) }]),
    { title: "Rename this chart", text: "Type a new title.", icon: "pencil", keepOpen: true, run: () => pushPage(renameChartPage(chart)) },
    ...(!isYours(chart) ? [] : [{ title: "Duplicate this chart", text: "A copy, right under it in your list.", icon: "duplicate", run: () => chartPick(chart.id, "duplicate") }]),
    ...(!isYours(chart) ? [] : [{ title: "Delete this chart", text: "You can undo.", icon: "trash", danger: true, run: () => deleteChart(chart) }]),
    ...(chart.preset === undefined || presetIndex(chart) < 0 ? [] : [
      { title: "Save a copy of this preset", text: `Saves a copy of ${titleOf(chart)} to your Library and opens it.`, icon: "duplicate", run: () => copyPreset(presetIndex(chart)) },
      ...(presetEdited(chart) ? [{ title: "Reset this preset", text: "Restores the preset, discarding your edits (you can undo).", icon: "undo", run: () => resetPreset(chart) }] : []),
    ]),
    ...(chart.layers.some((layer) => layer.hidden) ? [{ title: "Show every layer", text: "The hidden ones too.", icon: "eye", run: showEvery }] : []),
    ...(chart.panes.length > 1 ? [
      ...PANE_HEIGHTS.map(({ label, text, set }) => ({ title: label, text: `${text}.`, icon: "pane", run: () => setPaneHeights(set) })),
    ] : []),
  ]).filter((command) => !home || command.anywhere);
}

/**
 * The catalog's top folders in the presets' order: by the first preset charted from a series it alone holds (the
 * most asked for first); none (its series all elsewhere too: the server's plumbing, a copy), last; alike, as the
 * server has them. The presets or the catalog not read yet, as the server has them.
 */
let topRanks = /** @type {{ for: Preset[], ranks: Map<string, number> } | null} */ (null);
function catalogTops(/** @type {any} */ node) {
  const keys = Object.keys(node ?? {});
  if (!rankTops()) return keys;
  const { ranks } = /** @type {NonNullable<typeof topRanks>} */ (topRanks), last = /** @type {Preset[]} */ (presets).length;
  return keys.sort((a, b) => (ranks.get(a) ?? last) - (ranks.get(b) ?? last));
}
/**
 * The catalog's top folders ranked, once both the presets and the catalog are read (made at idle as either comes, a
 * slice at a time, see loadPresets and indexCatalog: not by the first ⌘K, 32 ms at a phone's 4× slower CPU); the rest
 * at once where they're needed first. Whether they are.
 */
function rankTops() {
  if (!presets || !catalogIndex) return false;
  const steps = rankSteps();
  if (steps) while (!steps.next().done);
  return true;
}
/** The ranks made while the page is idle, a slice at a time (all at once, 30 ms of a phone's). @param {{ timeRemaining: () => number }} deadline */
function ranksSoon(deadline) {
  const steps = presets && catalogIndex ? rankSteps() : null;
  if (steps) inSlices(deadline, () => Boolean(steps.next().done), ranksSoon);
}
/** The ranks being made, a few hundred presets a step, for the presets they're of (none: made, or none yet). */
let ranking = /** @type {{ for: Preset[], steps: Generator<void, void> } | null} */ (null);
/** The ranks' steps, those left (null: they're made). */
function rankSteps() {
  const list = /** @type {Preset[]} */ (presets);
  if (topRanks?.for === list) return null;
  if (ranking?.for !== list) ranking = { for: list, steps: (function* () {
    const ranks = new Map();
    for (const [i, preset] of list.entries()) {
      for (const layer of preset.c.layers) for (const name of [layer.series, ...(typeof layer.formula === "string" ? seriesNamed(layer.formula) : [])]) {
        const top = typeof name === "string" && !sharedTops.has(name) ? catalogIndex?.get(name)?.split(".")[0] : undefined;
        if (top && !ranks.has(top)) ranks.set(top, i);
      }
      if ((i + 1) % 200 === 0) yield;
    }
    topRanks = { for: list, ranks };
  })() };
  return ranking.steps;
}
/** A command as ⌘K lists it. @param {any} command */
const commandItem = (command) => ({ kind: "command", key: command.title, command, group: "Commands" });
/**
 * ⌘K's search of every series for `query`, settled into its list as it answers: `compose` makes the rows from the
 * names found (null: the search failed, said under the list), the row the person moved to kept, else the one `lead`
 * picks (the first). The glow says it's searching meanwhile; a search after it lets it go.
 * @param {string} query @param {(names: string[] | null) => any[]} compose @param {(items: any[]) => number} [lead]
 */
function searchSeries(query, compose, lead = () => 0) {
  const controller = (palette.controller = new AbortController());
  const settle = (/** @type {string[] | null} */ names, note = "") => {
    if (controller.signal.aborted) return;
    palette.pending = null;
    const active = palette.items[palette.active];
    palette.items = compose(names);
    palette.active = palette.moved ? Math.max(0, palette.items.indexOf(active)) : lead(palette.items);
    $("palette-glow").hidden = true;
    renderPalette(false, note);
  };
  // (The page's engine is building: draw the other rows meanwhile, its series after.)
  findSeries(query, 80, controller.signal, () => !controller.signal.aborted && palette.pending?.()).then((names) => settle(names), () => settle(null, "Series search is unavailable right now."));
}

/**
 * The formula page: the formula on top, to add; under it what completes the word being typed (leading, so ↵ takes
 * it), or between words what a formula is made of: this chart's layers, then the functions.
 */
function showFormula() {
  palette.controller?.abort();
  const input = $("palette-input"), text = palette.query;
  const caret = input.selectionStart ?? text.length;
  formulaAt = `${text} ${caret} ${input.selectionEnd}`;
  const self = palette.editing ? layerById(palette.editing) : null;
  const { start, token, items } = completionsAt(text, caret, self);
  palette.from = token ? start : caret;
  const top = text.trim() ? [{ kind: "formula", key: "formula" }] : [];
  const pieces = (list) => list.map((item) => ({ kind: "insert", key: item.insert, group: item.recipe ? "Recipes" : item.layer ? "Layers" : item.series ? "Series" : "Functions", ...item }));
  const fits = token ? null : fitsAt(text, caret, self, input.selectionEnd ?? caret);
  const blocks = token ? [] : pieces(fits?.items ?? [
    // (Not while a layer's formula is changed, nor for a signal: a recipe adds a line.)
    ...(text.trim() || self || palette.signal ? [] : recipesFor()),
    ...formulaInputs(self),
    ...Object.entries(FUNCTIONS).map(([name, [, signature, what]]) => ({ insert: `${name}(`, title: signature, text: what })),
  ]);
  palette.items = [...top, ...pieces(items), ...blocks];
  palette.active = items.length ? top.length : 0;
  palette.moved = false;
  renderPalette(false, fits?.more ?? "");
  completeSeries(palette, token, (series) => {
    palette.items.push(...pieces(series.map((item) => ({ ...item, name: item.insert }))));
    if (!palette.moved) palette.active = top.length;
    renderPalette();
  });
}
/** Where ⌘K's formula list was made (the text, the selection): the caret moving elsewhere makes it again. */
let formulaAt = "";
/** A formula that reads, of this chart's layers (every one it names there) or of series by their names (every one known). */
function isOwnFormula(/** @type {string} */ text) {
  if (!/[-+*/()^<>&|=!]/.test(plainOperators(text))) return false;
  let letters, dated;
  try {
    const node = parse(text);
    letters = [...refsOf(node)];
    dated = [...VALUES].some((name) => JSON.stringify(node).includes(`"k":"${name}"`));
  } catch {
    return false;
  }
  if (!letters.every((letter) => chart.layers.some((layer) => layer.letter === letter))) return false;
  // (Of series by their names, every one there, price / realized_price; a condition on them, mvrv > 3.5, or on the
  // date, date >= 2020-05-11, too: a signal of them.)
  // (Series alone, only names the catalog has: before it's indexed, a search, "short-term" isn't short - term.)
  const names = seriesNamed(text), known = (/** @type {string} */ name) => isSeriesName(name) && (!catalogIndex || catalogIndex.has(name));
  return letters.length > 0 || (isCondition(text) ? markable(chart).length > 0 && (dated || names.some(known)) : names.length > 0 && names.every((name) => isSeriesName(name) && Boolean(catalogIndex?.has(name))));
}
/** Whether a formula is a condition (a comparison, or two joined): typed in ⌘K, it's added as a signal. */
const isCondition = (/** @type {string} */ text) => {
  try {
    const node = parse(text);
    return node.k === "bin" && (COMPARES.has(node.op) || node.op === "&" || node.op === "|");
  } catch {
    return false;
  }
};
/** Whether the formula typed is added as a signal: on its page, or a condition typed anywhere else in ⌘K. */
const asSignal = (/** @type {string} */ text) => palette.signal || (!palette.formula && isCondition(text));
/** Why a formula doesn't read, or "" when it does. */
const parseError = (/** @type {string} */ text) => {
  try {
    parse(text);
    return "";
  } catch (error) {
    return /** @type {Error} */ (error).message;
  }
};
/** Why a layer's formula would read itself (its own letter, or through the layers it reads), else "". */
function loopError(/** @type {Layer} */ layer, /** @type {string} */ text) {
  let refs;
  try {
    refs = refsOf(parse(text));
  } catch {
    return "";
  }
  if (refs.has(layer.letter)) return `A formula can't read its own layer, ${layer.letter}`;
  const through = [...refs].some((letter) => ((other) => other && readsOf(other).has(layer.letter))(chart.layers.find((item) => item.letter === letter)));
  return through ? "It would read itself, through the layers it reads" : "";
}
/**
 * A window written that can't be one (a span of 0 or less, a percentile's p past 0 to 100), as computing it would
 * say: found as it's typed, not once it's computed. @returns {string}
 */
function windowError(/** @type {any} */ node) {
  if (!node || typeof node !== "object") return "";
  if (node.k === "call") {
    if (isBandSum(node.f)) {
      try {
        return bandSum(node) && "";
      } catch (error) {
        return /** @type {Error} */ (error).message;
      }
    }
    const [arity, signature, , run] = FUNCTIONS[node.f] ?? [];
    const span = arity >= 2 && run !== null && node.args.length === arity ? node.args[arity - 1] : null, p = node.f === "percentile" ? node.args[1] : null;
    // (A number written, its sign as written: -(-30d) is 30d. Anything else is computing's to say.)
    const written = (/** @type {any} */ arg) => (arg?.k === "num" ? arg.v : arg?.k === "neg" ? -written(arg.a) : NaN);
    if (span && p?.k === "num" && p.unit) return `${signature}: p first, the span last`;
    if (span && !Number.isNaN(written(span)) && !(written(span) > 0)) return `${signature}: the window is a span of time, like 30d, 4w, 6mo or 2y`;
    if (p && !p.unit && !Number.isNaN(written(p)) && !(written(p) >= 0 && written(p) <= 100)) return `${signature}: p is a percent, from 0 to 100`;
    if (node.f === "shift" && shiftSpanError(shiftSpan(node.args[1]))) return shiftSpanError(shiftSpan(node.args[1]));
    if (Object.hasOwn(PERIODS_TAKEN, node.f) && ((least) => node.args.slice(1).some((/** @type {any} */ arg) => ((period) => !Number.isNaN(period) && !(Number.isInteger(period) && period >= least))(written(arg))))(PERIODS_TAKEN[node.f][0])) return `${signature}: ${PERIODS_TAKEN[node.f][1]}`;
    // (Each once: twice, a nest of them doubles at each level.)
    for (const arg of node.args) {
      const error = windowError(arg);
      if (error) return error;
    }
    return "";
  }
  return windowError(node.a) || windowError(node.b);
}
/**
 * Why a formula can't be added to this chart: it doesn't read, a letter names no layer here, or a series it names
 * isn't on Bitview (said as its legend would once drawn, as it's typed: the catalog holds every series, once it's
 * indexed); else "".
 */
function formulaError(/** @type {string} */ text) {
  const problem = parseError(text) || windowError(parse(text));
  if (problem) return problem;
  const missing = [...refsOf(parse(text))].find((letter) => !chart.layers.some((layer) => layer.letter === letter));
  if (missing) return goneLayer(chart, missing) ?? `There is no layer ${missing}`;
  const index = catalogIndex, unknown = index ? [...refsOf(parse(text), "series")].find((name) => !index.has(name)) : undefined;
  return unknown ? `${unknown}: not on Bitview` : "";
}
/** A row of the formula page: the formula to add, or a piece of one (a layer by its color). */
/** The last formula found not to compute, what's wrong with it, and the layer it was for (its row says so while it's the one typed). */
let formulaProblem = { text: "", editing: /** @type {string | null} */ (null), problem: "" };
function formulaRow(item) {
  if (item.kind === "formula") {
    const text = palette.query.trim(), kept = formulaProblem.text === text && formulaProblem.editing === palette.editing ? formulaProblem.problem : "";
    const editing = palette.editing ? layerById(palette.editing) : null, problem = formulaError(text) || kept;
    return `${asSignal(text) || editing?.type === "signal" ? ICONS.signal : ICONS.formula}<span class="item-name">${editing ? "Apply" : "Add"} <code>${escapeHtml(palette.query.trim())}</code></span>${problem ? `<span class="item-meta ${isUnfinished(palette.query.trim()) ? "" : "error"}">${escapeHtml(problem)}</span>` : `<span class="item-meta">${editing ? `To ${escapeHtml(editing.letter)}` : asSignal(text) ? ((on) => (on ? `Signal on ${escapeHtml(on.letter)}` : ""))(markFor(chart, text)) : formulaPlace(text)}</span>`}`;
  }
  const layer = item.layer ? layerById(item.layer) : null;
  const icon = layer ? `<i class="dot" style="--c:${escapeHtml(ink(layer.color))}"></i>` : item.series ? ICONS.series : item.group === "Spans" ? ICONS.recent : item.group === "Percents" ? ICONS.ruler : ICONS.formula;
  return `${icon}<span class="item-name"><code>${escapeHtml(item.title ?? "")}</code></span>${item.text ? `<span class="item-meta">${escapeHtml(item.text)}</span>` : ""}`;
}
/** A piece into the formula: over the word it completes, else at the caret (over what's selected); a recipe, where it goes too. */
function insertPiece(item) {
  const input = $("palette-input");
  // (A recipe's place carried, unless the page is for a pane: then it's that one's already.)
  // (Its new pane said as such: none, the formula's own place would be found for it, see formulaPane.)
  if (item.recipe && !palette.into) Object.assign(palette.op, { into: item.into ?? "new", side: item.side });
  const end = (input.selectionEnd ?? input.value.length) + (/^\w*/.exec(input.value.slice(input.selectionEnd ?? input.value.length))?.[0].length ?? 0);
  input.setRangeText(completion(item.insert, input.value.slice(end)), Math.min(palette.from, end), end, "end");
  input.focus();
  palette.query = input.value;
  showPalette();
}
/**
 * ↵ on the formula, or on a `recipe` (its own, where it goes): if it reads, it becomes a layer (in the pane it's
 * added to, or `pane`; "new": a new one; a signal, marked on `on`, else the layer it reads); if not, its error is in
 * the preview. `keep`: ⌘K stays, the formula selected, to write the next one.
 */
function addFromPalette(pane = palette.into, keep = false, /** @type {string | undefined} */ on = undefined, recipe = /** @type {any} */ (undefined)) {
  const text = recipe?.insert ?? palette.query.trim();
  // (No pane asked for: a pane showing its unit, beside its like there; see formulaPane.)
  const auto = recipe || palette.editing || palette.into != null ? null : formulaPane(text);
  if (pane == null && auto) pane = auto;
  const side = auto && pane === auto ? likeSide(auto) : recipe && pane === recipe.into ? recipe.side : formulaSide(pane), editing = palette.editing ? layerById(palette.editing) : null, problem = formulaError(text) || (editing ? loopError(editing, text) : !asSignal(text) && stackedOnly(pane === "new" ? null : pane) ? STACKED_ONLY : "");
  if (problem) return notice(problem);
  if (keep && !palette.editing) {
    // Added, and the formula stays, selected, to write the next one.
    if (asSignal(text)) addSignalLayer(text, on);
    else addFormulaLayer(text, pane === "new" ? null : pane, side);
    // (Shown again: the recipes on the layer now selected, the one added.)
    showPalette();
    paletteKeys(true);
    return;
  }
  $("palette").hidePopover();
  if (palette.editing) editById(palette.editing, (layer) => (layer.formula = text));
  else if (asSignal(text)) addSignalLayer(text, on);
  else addFormulaLayer(text, pane === "new" ? null : pane, side);
}
/** The side a formula from ⌘K goes on in `pane`: in its layer's pane, that layer's (see derive); else none (a layer's first, the right). */
const formulaSide = (/** @type {string | null | undefined} */ pane) => (palette.op.kind === "formula" && pane === palette.op.into ? palette.op.side ?? null : null);
/** A layer's formula, in ⌘K's formula page: previewed as it's changed, applied with ↵. */
function editFormula(id) {
  const layer = layerById(id);
  if (layer?.formula !== undefined) openPalette({ formula: true, editing: id, query: layer.formula, signal: layer.type === "signal" });
}
let formulaRun = 0;
/** The formula typed's unit, as its preview computed it (on which chart): where it goes, none asked for (see formulaPane). */
let formulaGoes = { text: "", chart: "", unit: "" };
/**
 * Where the formula typed goes when no pane's asked for: a pane showing its unit now (see paneShowing), its unit
 * known (its preview's, this chart's); else none (a new one). @param {string} text
 */
const formulaPane = (text) => (formulaGoes.text === text && formulaGoes.chart === chart.id ? paneShowing(chart, formulaGoes.unit)?.id ?? null : null);
/** The side a formula goes on in a pane showing its unit: its like's there. @param {string} pane */
const likeSide = (pane) => chart.layers.find((layer) => layer.pane === pane && unitFor(layer, resolved?.results.get(layer.id)) === formulaGoes.unit)?.side ?? null;
/** Where a layer added goes, as said: its pane, else a new one (at the most panes, the last; a stack there, nowhere). @param {string | null | undefined} pane */
const goesIn = (pane) => (pane ? `Pane ${paneNumber(pane)}` : chart.panes.length < MAX_PANES ? "New pane" : stackedOnly(null) ? "Nowhere: the last pane is a stack" : `Pane ${chart.panes.length}`);
/** Where the formula typed goes, as its row says: the pane asked for, else one showing its unit (see formulaPane), else a new one. */
const formulaPlace = (/** @type {string} */ text) => goesIn(palette.into && palette.into !== "new" ? palette.into : palette.into == null ? formulaPane(text) : null);
/**
 * The formula computed as it would be added, with this chart's data: its line in its color, its latest value, what it
 * uses, its unit. As it's typed (what's unchanged is reused: the formula alone is computed); a run overtaken by a
 * newer one shows nothing. A `recipe`'s, the same: where it goes, what it is.
 */
function previewFormula(panel, recipe = /** @type {any} */ (undefined)) {
  const text = recipe?.insert ?? palette.query.trim(), into = recipe ? recipe.into : palette.into === "new" ? null : palette.into, side = recipe ? recipe.side : formulaSide(into), key = recipe?.key ?? "formula", run = ++formulaRun;
  void (async () => {
    let node;
    try {
      node = parse(text);
    } catch (caught) {
      const error = /** @type {Error & { unfinished?: true }} */ (caught);
      panel.innerHTML = `<h4><code>${escapeHtml(text)}</code></h4>${error.unfinished ? `<p class="description">${escapeHtml(error.message)}.</p>` : `<div class="error-box">${escapeHtml(error.message)}</div>`}`;
      return;
    }
    const doc = structuredClone(chart);
    // Edited, the layer itself with the new formula; added, a new layer: a signal on the layer it marks, else in the
    // pane chosen (none: a new one).
    let layer = palette.editing ? doc.layers.find((item) => item.id === palette.editing) : null;
    if (layer) layer.formula = text;
    else if (asSignal(text)) doc.layers.push((layer = makeLayer({ formula: text, type: "signal", ref: markFor(doc, text)?.id }, doc)));
    else doc.layers.push((layer = makeLayer({ formula: text, pane: into ?? newPaneIn(doc).id, ...(into && side ? { side } : {}) }, doc)));
    const editing = palette.editing !== null;
    let data = await resolve(doc).catch(() => null);
    if (run !== formulaRun || palette.items[palette.active]?.key !== key) return;
    let result = data?.results.get(layer.id);
    const unit = unitFor(layer, result);
    if (!recipe && !editing) formulaGoes = { text, chart: doc.id, unit };
    const goes = into ?? (recipe || editing || palette.into === "new" || layer.type === "signal" ? null : formulaPane(text));
    if (goes && !editing && layer.type !== "signal") {
      const before = averageFor(doc, doc.panes.find((pane) => pane.id === layer.pane)).active;
      layer.pane = goes;
      layer.side = side ?? likeSide(goes) ?? "right";
      if (before !== averageFor(doc, doc.panes.find((pane) => pane.id === goes)).active) {
        // The discovered destination changes the view. Resolve reuses the raw computations and full history.
        data = await resolve(doc).catch(() => null);
        if (run !== formulaRun || palette.items[palette.active]?.key !== key) return;
        result = data?.results.get(layer.id);
      }
    }
    if (!result || result.error) {
      const problem = result?.detail ?? result?.error ?? "It couldn't be computed";
      panel.innerHTML = `<h4><code>${escapeHtml(text)}</code></h4><div class="error-box">${escapeHtml(problem)}.</div>`;
      if (recipe) return;
      // (Said on its row too, as a mistake in how it's written is, kept while it's the one typed (the row drawn
      // again, a completion offered, still says it): where there's no room for this side, a phone's, the row's the
      // only place it shows. Said aloud once.)
      const fresh = formulaProblem.text !== text || formulaProblem.problem !== problem;
      formulaProblem = { text, editing: palette.editing, problem };
      const meta = $("palette-list").querySelector('[data-kind="formula"] .item-meta');
      if (meta) (meta.textContent = problem), (meta.className = "item-meta error");
      if (fresh) say(problem);
      return;
    }
    if (formulaProblem.text === text) formulaProblem = { text: "", editing: null, problem: "" };
    const uses = usesOf(node), average = averageFor(doc, doc.panes.find((pane) => pane.id === layer.pane));
    const meta = recipe || palette.editing || layer.type === "signal" ? null : $("palette-list").querySelector('[data-kind="formula"] .item-meta:not(.error)');
    if (meta) meta.textContent = formulaPlace(text);
    // (A signal: where its marks are, and how many; its 0s and 1s aren't a line to look at.)
    if (layer.type === "signal") {
      const on = doc.layers.find((item) => item.id === layer?.ref);
      return void (panel.innerHTML = `<h4><code>${escapeHtml(text)}</code></h4>
              <p class="description">${uses.length ? `Uses ${escapeHtml(uses.join(", "))}` : "A constant"}. ${layer.place === "shade" ? "Shaded while it's up or down" : "Marked where it turns up or down"} (above 0, up; below, down).</p>
              <div class="info"><h5>${editing ? "Layer" : "As layer"} ${escapeHtml(layer.letter)}</h5><dl>
                <div><dt>Marked on</dt><dd>${on ? escapeHtml(viewName(doc, on)) : "No layer"}</dd></div>
                <div><dt>Times marked</dt><dd>${grouped.format(marksOf(layer, result.values, data?.results.get(layer.ref ?? "") ?? null).length)}</dd></div>
                <div><dt>Average</dt><dd>${escapeHtml(average.name || "Raw")}</dd></div>
                <div><dt>Now</dt><dd>${escapeHtml(format(Array.prototype.findLast.call(result.values, (value) => Number.isFinite(value)) ?? NaN, unit))}</dd></div>
              </dl></div>`);
    }
    panel.innerHTML = `<h4><code>${escapeHtml(text)}</code></h4>
            <p class="description">${recipe ? escapeHtml(recipe.text) : uses.length ? `Uses ${escapeHtml(uses.join(", "))}` : "A constant"}.</p>
            <svg class="spark" viewBox="0 0 300 100" preserveAspectRatio="none" aria-hidden="true"></svg>
            <div class="info"><h5>${editing ? "Layer" : "As layer"} ${escapeHtml(layer.letter)}</h5><dl>
              <div><dt>Unit</dt><dd>${escapeHtml(UNIT_NAMES[unit] ?? "Number")}</dd></div>
              <div><dt>Average</dt><dd>${escapeHtml(average.name || "Raw")}</dd></div>
              <div><dt>Latest</dt><dd class="spark-caption"></dd></div>
              ${editing ? "" : `<div><dt>Goes in</dt><dd>${goesIn(goes)}</dd></div>`}
            </dl></div>`;
    drawSpark(panel, Array.from(/** @type {Vector} */ (result.values)), unit, ink(layer.color));
  })();
}
/** What a piece of a formula is: a layer of this chart, with its latest value; or a function. */
function previewPiece(item) {
  const layer = item.layer ? layerById(item.layer) : null;
  if (!layer) return `<h4><code>${escapeHtml(item.title ?? "")}</code></h4><p class="description">${escapeHtml(capitalized(item.text ?? ""))}.</p>`;
  const unit = unitFor(layer, resolved?.results.get(layer.id));
  const result = resolved?.results.get(layer.id), values = result && measured(result);
  const value = values?.findLast(Number.isFinite) ?? NaN;
  return `<h4 class="plain">${escapeHtml(labelOf(layer))}</h4><p class="description">Layer ${escapeHtml(layer.letter)} of this chart. Formulas read its raw measurement.</p>
          <div class="info"><dl><div><dt>Unit</dt><dd>${escapeHtml(UNIT_NAMES[unit] ?? "Number")}</dd></div><div><dt>Latest, raw</dt><dd>${escapeHtml(format(value, unit))}</dd></div></dl></div>`;
}
/**
 * A long series drawn small: its lowest and highest points in each of `columns` columns, in their order (its shape,
 * extremes and all, at a fraction of the points); a short one whole. Finite points only, as [index, value].
 * @param {ArrayLike<number>} values @param {number} columns @returns {[number, number][]}
 */
function columnsOf(values, columns) {
  /** @type {[number, number][]} */
  const points = [];
  const per = Math.max(1, values.length / columns);
  for (let start = 0; start < values.length; start += per) {
    let lo = -1, hi = -1;
    // (Finite by a subtraction: no call each value.)
    for (let i = Math.floor(start), end = Math.min(values.length, Math.floor(start + per)); i < end; i++) {
      const value = values[i];
      if (value - value !== 0) continue;
      if (lo < 0 || value < values[lo]) lo = i;
      if (hi < 0 || value > values[hi]) hi = i;
    }
    if (lo < 0) continue;
    for (const i of lo === hi ? [lo] : [Math.min(lo, hi), Math.max(lo, hi)]) points.push([i, values[i]]);
  }
  return points;
}
/**
 * A long series drawn small and smooth: the mean of each of `columns` columns, at its middle; a short one whole.
 * Finite points only, as [index, value]. @param {ArrayLike<number>} values @param {number} columns @returns {[number, number][]}
 */
function meansOf(values, columns) {
  /** @type {[number, number][]} */
  const points = [];
  const per = Math.max(1, values.length / columns);
  for (let start = 0; start < values.length; start += per) {
    let sum = 0, count = 0, at = 0;
    // (Finite by a subtraction: no call each value.)
    for (let i = Math.floor(start), end = Math.min(values.length, Math.floor(start + per)); i < end; i++) {
      const value = values[i];
      if (value - value !== 0) continue;
      sum += value;
      at += i;
      count++;
    }
    if (count) points.push([at / count, sum / count]);
  }
  return points;
}
/** Sparks drawn, each its own fill's id (two panels can show at once: a phone's sheet, the pane behind). */
let sparksDrawn = 0;
/**
 * A panel's mini chart (its .spark, its .spark-caption): a line over the values (on a log scale for a price that
 * stays above zero), its latest under it.
 */
function drawSpark(/** @type {Element} */ panel, values, unit, color = css("--ink"), /** @type {((i: number) => string | null) | null} */ tint = null) {
  const finite = values.filter(Number.isFinite), caption = /** @type {HTMLElement} */ (panel.querySelector(".spark-caption")), spark = /** @type {SVGElement} */ (panel.querySelector(".spark"));
  if (finite.length < 2) return void (caption.textContent = finite.length || values.length < 2 ? "No history yet" : "Every value is infinite or undefined");
  caption.textContent = format(finite.at(-1), unit);
  const logScale = unit === "usd" && finite.every((value) => value > 0);
  const scaled = logScale ? values.map(Math.log10) : values;
  const [lo, hi] = rangeOf(scaled);
  const spots = columnsOf(scaled, 300).map(([i, value]) => ({ i, at: `${((i / (values.length - 1)) * 300).toFixed(1)},${(92 - ((value - lo) / (hi - lo || 1)) * 82).toFixed(1)}` })), points = spots.map((point) => point.at);
  // (Painted date by date: its line in pieces, each in its color.)
  const lines = tint ? byColor(spots, (point) => tint(point.i) ?? color).map((piece) => lineOver(piece.points.map((point) => point.at), piece.color)).join("") : lineOver(points, color);
  const fill = `spark-fill-${++sparksDrawn}`;
  spark.innerHTML = `<defs>${fadeDef(fill, color, ".22")}</defs>${areaUnder(fill, points, 300, 100)}${lines}`;
}

/** highlightMatch's pattern, by what's typed: made once a query, not for each row. */
let highlighting = { query: "", words: /** @type {string[]} */ ([]), pattern: /** @type {RegExp | null} */ (null) };
/**
 * A text with what's typed marked in it (found in the text itself, then escaped around: no mark lands in an entity),
 * a pair typed joined as the search finds it (see writtenPattern): "bip69" marks BIP-69, "1week" 1 week.
 */
function highlightMatch(text, query, initials = false) {
  if (highlighting.query !== query) {
    const words = typedWords(query.replace(/_/g, " "));
    highlighting = { query, words, pattern: words.length ? new RegExp(words.map(writtenPattern).join("|"), "gi") : null };
  }
  if (!highlighting.pattern) return escapeHtml(text);
  const ranges = initials ? initialsRanges(text, highlighting.words) : [];
  for (const match of text.matchAll(highlighting.pattern)) ranges.push([match.index, match.index + match[0].length]);
  ranges.sort((a, b) => a[0] - b[0]);
  const merged = /** @type {[number, number][]} */ ([]);
  for (const range of ranges) {
    const prior = merged.at(-1);
    if (prior && range[0] <= prior[1]) prior[1] = Math.max(prior[1], range[1]);
    else merged.push(range);
  }
  let out = "", last = 0;
  for (const [start, end] of merged) {
    out += `${escapeHtml(text.slice(last, start))}<mark>${escapeHtml(text.slice(start, end))}</mark>`;
    last = end;
  }
  return out + escapeHtml(text.slice(last));
}

/** The row a screen reader's on, as ⌘K's input says it: none when no row is (an empty list). */
const activeOption = () => ($(`option-${palette.active}`) ? $("palette-input").setAttribute("aria-activedescendant", `option-${palette.active}`) : $("palette-input").removeAttribute("aria-activedescendant"));
/** The palette's list, grouped; `pending` adds a searching row, `note` a closing remark. */
/** Whether ⌘K's item does something with ↵ (dimmed, it doesn't: a row not now, a series by block only, a command saying why). @param {any} item */
const actionable = (item) => !(item.kind === "row" && (item.disabled || item.dim || (!item.run && !item.page))) && !(item.kind === "series" && byBlock.has(item.name)) && !(item.kind === "command" && !item.command.run);
/** Whether ⌘K's item can be lit first: it acts, and doesn't destroy (that one's an arrow away: ↵ alone never runs it). @param {any} item */
const litFirst = (item) => actionable(item) && !dangerous(item);
/** How many rows ⌘K writes at a time. */
const PALETTE_PAGE = 24;
/** ⌘K's rows from `from` to `to`, each under its group's name where a group starts. */
function optionsHtml(/** @type {number} */ from, /** @type {number} */ to) {
  const searching = Boolean(palette.query.trim());
  let group = palette.items.slice(0, from).findLast((item) => item.group)?.group ?? "";
  // (Each row described by its group's name, its heading's id: "Move down", in this chart.)
  let start = from;
  while (start > 0 && group && palette.items[start - 1].group === group) start--;
  let groupId = `palette-group-${start}`;
  const shown = new Set(home ? [] : chart.layers.map((layer) => layer.series));
  // (By touch, a row with more than ↵ does: its ⋯ at its end, its actions (as holding it opens them).)
  const more = (/** @type {any} */ item, /** @type {number} */ i, /** @type {string} */ html) => (TOUCH && actionsFor(item).length > 1 ? `${html.slice(0, -5)}<button type="button" class="cap small row-more" data-row-more="${i}" tabindex="-1" aria-label="More it does">${ICONS.more}</button></li>` : html);
  return palette.items.slice(from, to).map((item, at) => more(item, from + at, (() => {
    const i = from + at;
    const header = item.group && item.group !== group ? `<li class="group" role="presentation" id="${(groupId = `palette-group-${i}`)}">${escapeHtml((group = item.group))}</li>` : "";
    const described = item.group ? ` aria-describedby="${groupId}"` : "";
    if (item.kind === "formula" || item.kind === "insert") return `${header}<li role="option" id="option-${i}" aria-posinset="${i + 1}" aria-setsize="${palette.items.length}"${described} data-kind="${item.kind}" data-option="${i}" aria-selected="${i === palette.active}">${formulaRow(item)}</li>`;
    if (item.kind === "row") return `${header}<li role="option" id="option-${i}" aria-posinset="${i + 1}" aria-setsize="${palette.items.length}"${described} data-kind="row" data-option="${i}" aria-selected="${i === palette.active}" class="${item.dim || item.disabled ? "dim" : ""}${item.under ? " under" : ""}"${item.disabled ? ' aria-disabled="true"' : ""}>${rowHtml(item)}</li>`;
    const doc = item.kind === "chart" ? chartById(item.key) : null;
    if (item.kind === "series") {
      const blocked = byBlock.has(item.name), human = humanize(item.name);
      const end = blocked ? `<span class="item-meta">By block only</span>` : `<span class="item-meta"><code>${escapeHtml(item.name)}</code></span>${shown.has(item.name) ? `<span class="item-mark" role="img" aria-label="In this chart">${ICONS.check}</span>` : ""}`;
      return `${header}<li role="option" id="option-${i}" aria-posinset="${i + 1}" aria-setsize="${palette.items.length}"${described} data-kind="series" data-option="${i}" aria-selected="${i === palette.active}" ${blocked ? 'class="dim" aria-disabled="true"' : ""}>${item.group === "Recent series" ? ICONS.recent : ICONS.series}<span class="item-name">${searching ? highlightMatch(human, palette.query, true) : escapeHtml(human)}</span>${end}</li>`;
    }
    // (A command's icon by its name, or drawn already: a folder's mark.)
    const icon = item.kind === "folder" ? ICONS.folder : item.kind === "command" ? ICONS[item.command.icon] ?? (item.command.icon?.startsWith("<") ? item.command.icon : ICONS.series) : doc ? chartMark(doc) : ICONS.series;
    const name = item.kind === "folder" ? escapeHtml(humanizeKey(item.key)) : item.kind === "command" ? (searching ? highlightMatch(item.command.title, palette.query) : escapeHtml(item.command.title)) : doc ? (searching ? highlightMatch(titleOf(doc), palette.query) : escapeHtml(titleOf(doc))) : "";
    const meta = item.kind === "folder" ? `<span class="item-mark"><span class="item-next" aria-hidden="true">${ICONS.next}</span></span>` : item.kind === "command" ? (item.command.current ? `<span class="item-mark" role="img" aria-label="In effect">${ICONS.check}</span>` : `${goesOn(item) ? `<span class="item-mark"><span class="item-next" aria-hidden="true">${ICONS.next}</span></span>` : ""}${item.command.kbd && item.command.run && answers(item.command.kbd) ? keycaps(item.command.kbd) : ""}`) : doc ? `<span class="item-meta">${plural(doc.layers.length, "layer")}</span>` : "";
    const idle = item.kind === "command" && !item.command.run && !item.command.current;
    return `${header}<li role="option" id="option-${i}" aria-posinset="${i + 1}" aria-setsize="${palette.items.length}"${described} data-kind="${item.kind}" data-option="${i}" aria-selected="${i === palette.active}" ${idle ? 'class="dim" aria-disabled="true"' : ""}>${icon}<span class="item-name">${name}</span>${meta}</li>`;
  })())).join("");
}
/** ⌘K's rows written up to `to` (scrolled to, moved to), after those there (and before its closing note). */
function drawOptions(/** @type {number} */ to) {
  to = Math.min(palette.items.length, to);
  if (!(to > (palette.drawn ?? 0))) return;
  // (Their keys drawn as they're made: the rows written before have theirs.)
  const list = $("palette-list"), made = document.createElement("template");
  made.innerHTML = optionsHtml(palette.drawn, to);
  drawKeys(made.content);
  seePaletteMarks(made.content);
  list.insertBefore(made.content, list.querySelector(":scope > [data-tail]"));
  palette.drawn = to;
}

const sparks = new Map();
/** The active row's preview, and the drawing of the chart it showed given up if no other frame shows it. */
function preview() {
  previewActive();
  giveUpUnframed();
}
function previewActive() {
  updatePaletteKeys();
  const item = palette.items[palette.active];
  const panel = $("palette-preview");
  // (Quick Look on: the row's chart; its way, said at the foot where the row is a chart.)
  $("palette-look").hidden = !quickWorks() || !canLook(item);
  if (quick) quickShow();
  if (item?.look) showLook(item.look);
  else clearLook();
  if (!item) return void (panel.innerHTML = "");
  if (item.kind === "formula") return previewFormula(panel);
  if (item.recipe) return previewFormula(panel, item);
  // (A phone's has no room for it (hidden), nor Home's list: nothing drawn there at each letter typed, nor as it
  // unrolls. A formula's is computed above: its row says what it finds.)
  if (phone.matches || $("palette").hasAttribute("data-on-home")) return void (panel.firstChild && (panel.innerHTML = ""));
  if (item.kind === "row") return previewRow(panel, item);
  if (item.kind === "insert" && !item.series) return void (panel.innerHTML = previewPiece(item));
  if (item.kind === "folder") {
    const node = nodeAt(item.path);
    const keys = Object.keys(node ?? {}).filter((key) => node[key] && typeof node[key] === "object");
    const groups = keys.filter((key) => !isLeaf(node[key])).length;
    panel.innerHTML = `<h4 class="plain">${escapeHtml(humanizeKey(item.key))}</h4>
            <p class="description">${[groups ? plural(groups, "group") : "", `${grouped.format(seriesUnder(node))} series`].filter(Boolean).join(" · ")}</p>
            <ul class="contents">${keys.slice(0, 14).map((key) => `<li>${isLeaf(node[key]) ? ICONS.series : ICONS.folder}<span>${escapeHtml(isLeaf(node[key]) ? node[key].name : humanizeKey(key))}</span></li>`).join("")}${keys.length > 14 ? `<li class="rest">and ${grouped.format(keys.length - 14)} more</li>` : ""}</ul>`;
    return;
  }
  if (item.kind === "command") {
    panel.innerHTML = `<h4 class="plain">${escapeHtml(item.command.title)}</h4><p class="description">${escapeHtml(item.command.text)}${item.command.current ? " In effect now." : ""}</p>${item.command.kbd && item.command.run && answers(item.command.kbd) ? `<div class="info"><dl><div><dt>Shortcut</dt><dd>${keycaps(item.command.kbd)}</dd></div></dl></div>` : ""}`;
    return;
  }
  if (item.kind === "chart") return previewChart(panel, chartById(item.key));
  previewSeries(panel, item.name, () => palette.items[palette.active] === item);
}
/**
 * A series in a panel, as ⌘K shows it (its pane; a phone's, the sheet of a row's actions): its name and id, its
 * description, its history and what it is, its data as it comes; `still`, whether the panel still shows it (one arrowed
 * past draws none of it).
 */
function previewSeries(/** @type {HTMLElement} */ panel, /** @type {string} */ name, still = () => true) {
  const head = `<h4 class="plain">${escapeHtml(humanize(name))}</h4><p class="series-id"><code>${escapeHtml(name)}</code>${((layer) => (layer ? ` · in this chart as ${escapeHtml(layer.letter)}` : ""))(home ? null : chart.layers.find((layer) => layer.series === name))}</p>`;
  if (byBlock.has(name)) {
    const day = dailyOf(name);
    panel.innerHTML = `${head}<p class="description">${BY_BLOCK}.${day ? ` By day, there's ${escapeHtml(humanize(day))} (<code>${escapeHtml(day)}</code>).` : ""}</p>`;
    return;
  }
  panel.innerHTML = `${head}<p class="description"><span class="shimmer">Loading description…</span></p>`;
  // What it is, then its data, each as it comes, if it's still the row looked at (one arrowed past draws nothing).
  void (async () => {
    try {
      retryInfo(name);
      const meta = await info(name);
      if (!still()) return;
      const unit = unitOf(meta.type);
      units.set(name, unit);
      panel.innerHTML = `${head}
              ${meta.description ? `<div class="description clamped">${proseHtml(meta.description)}</div>` : ""}
              <svg class="spark" viewBox="0 0 300 100" preserveAspectRatio="none" aria-hidden="true"></svg>
              <div class="info"><h5>Information</h5><dl>
                <div><dt>Unit</dt><dd>${escapeHtml(UNIT_NAMES[unit])}</dd></div>
                <div><dt>Intervals</dt><dd>${meta.indexes.length}</dd></div>
                <div><dt>Latest</dt><dd class="spark-caption"><span class="shimmer">Loading</span></dd></div>
              </dl></div>`;
      const index = meta.indexes.includes("week1") ? "week1" : meta.indexes.includes("day1") ? "day1" : null;
      if (!index) return void (/** @type {HTMLElement} */ (panel.querySelector(".spark-caption")).textContent = "—");
      const key = `${name}@${index}`, count = index === "week1" ? 520 : 1500;
      const tail = () => {
        // (Kept for the last hundred looked at.)
        const kept = sparks.get(key);
        if (kept) return lastUsed(sparks, key, kept);
        const asked = client.seriesEndpoint(name, /** @type {"day1"} */ (index)).last(count).fetch().catch((error) => {
          sparks.delete(key);
          throw error;
        });
        sparks.set(key, asked);
        while (sparks.size > 100) sparks.delete(sparks.keys().next().value);
        return asked;
      };
      // A series a chart loaded at that interval is at hand: its tail, without asking.
      const held = loads.get(key);
      const response = await (held ? lastUsed(loads, key, held).then((data) => ({ data: data.data.slice(-count) }), tail) : tail());
      if (!still()) return;
      drawSpark(panel, response.data.map((value) => (Array.isArray(value) ? value[3] : value)).map((value) => (value == null ? NaN : Number(value))), unit);
    } catch (error) {
      if (still()) panel.innerHTML = `${head}<div class="error-box">${escapeHtml(failure(error))}</div>`;
    }
  })();
}
/** A pane by what it shows: its layers' names, the first two. */
const paneNames = (pane) => chart.layers.filter((layer) => layer.pane === pane.id).slice(0, 2).map(labelOf).join(", ") || "Empty";
/**
 * What can be done with a row, as Raycast's ⌘K: what ↵ does first, then the rest. A series or a formula goes in a
 * new pane or a chosen one, a series replaces the selected layer's or starts a chart; a chart opens, is renamed,
 * duplicated, downloaded or deleted. One action only (a command, a folder) means no menu.
 * @returns {Action[]}
 */
function actionsFor(item) {
  if (!item) return [];
  if (item.kind === "row") return item.actions?.() ?? [];
  const intoPanes = (put) => chart.panes.map((pane, i) => ({ label: `Add to pane ${i + 1}`, icon: ICONS.pane, aside: paneNames(pane), run: () => put(pane.id) }));
  // (A chart with all the panes it can hold: none new offered; ↵ adds to the last one.)
  const room = chart.panes.length < MAX_PANES;
  if (item.kind === "formula" || item.recipe) {
    if (palette.editing) return [];
    // (A recipe: its formula, where it goes, and ⇥ to write it in, to change first.)
    const recipe = item.recipe ? item : undefined, text = recipe?.insert ?? palette.query.trim(), at = recipe ? recipe.into : palette.into ?? formulaPane(text);
    const add = (/** @type {string | null | undefined} */ pane = at, keep = false, /** @type {string | undefined} */ on = undefined) => addFromPalette(pane, keep, on, recipe);
    const found = chart.panes.findIndex((pane) => pane.id === at), into = found < 0 && !room ? chart.panes.length - 1 : found;
    const going = [{ label: "Add and keep going", icon: ICONS.plus, kbd: "⌘↵", run: () => add(at, true) }, ...(recipe ? [{ label: "Change it first", icon: ICONS.pencil, kbd: "⇥", run: () => insertPiece(recipe) }] : [])];
    // (A signal: on a layer, the one it reads first; no pane of its own.)
    if (asSignal(text)) {
      const first = markFor(chart, text), on = [...(first ? [first] : []), ...markable(chart).filter((layer) => layer !== first)].map((layer, i) => ({ label: `Add on ${labelOf(layer)}`, icon: ICONS.signal, aside: layer.letter, ...(i ? {} : { kbd: "↵" }), run: () => add(at, false, layer.id) }));
      return [...on.slice(0, 1), ...going, ...on.slice(1)].map((action) => ({ ...action, stay: true }));
    }
    // Open until the formula is added, as with ↵: an unfinished one stays, to be finished.
    return (into < 0
      ? [{ label: "Add to a new pane", icon: ICONS.pane, kbd: "↵", run: () => add("new") }, ...going, ...intoPanes((pane) => add(pane))]
      : [{ label: `Add to pane ${into + 1}`, icon: ICONS.pane, kbd: "↵", run: () => add() }, ...going, ...(room ? [{ label: "Add to a new pane", icon: ICONS.pane, ...(recipe ? {} : { kbd: "⇧↵" }), run: () => add("new") }] : []), ...intoPanes((pane) => add(pane)).filter((_, i) => i !== into)]
    ).map((action) => ({ ...action, stay: true }));
  }
  if (item.kind === "series" && item.name && home) return [
    { label: "New chart with it", icon: ICONS.plus, kbd: "↵", run: () => choose() },
    { label: "Copy the series name", icon: ICONS.copy, run: () => copyName(item.name) },
  ];
  if (item.kind === "series" && item.name && !palette.replacing) {
    const name = item.name, layer = layerById(selected);
    return [
      { icon: ICONS.plus, label: palette.into === "new" ? "Add to a new pane" : palette.into ? `Add to pane ${paneNumber(palette.into)}` : "Add to chart", kbd: "↵", run: () => choose() },
      { label: "Add and keep going", icon: ICONS.plus, kbd: "⌘↵", stay: true, run: () => choose(false, true) },
      ...(chart.layers.length && !palette.bubbles ? [...(room && palette.into !== "new" ? [{ label: "Add to a new pane", icon: ICONS.pane, kbd: "⇧↵", run: () => choose(true) }] : []), ...intoPanes((pane) => addSeries(name, { pane }))] : []),
      ...(layer?.series && layer.series !== name ? [{ label: `Replace ${layer.letter} with it`, icon: ICONS.swap, aside: labelOf(layer), run: () => replaceSeries(layer.id, name) }] : []),
      { label: "New chart with it", icon: ICONS.plus, run: () => newChartWith(name) },
      { label: "Copy the series name", icon: ICONS.copy, run: () => copyName(name) },
    ];
  }
  if (item.kind === "chart") return chartActions(item.key);
  return [];
}
/**
 * A menu ⌘K opened put away by ⌘K itself (its rows written again, it moved): the focus back in ⌘K, if it was in the
 * menu: its field by keys; by touch, ⌘K (no keyboard but by a tap on its field, as when it opened).
 */
const paletteMenuGone = () => ((had) => (hideMenu(), had && (keyboard() ? $("palette-input") : $("palette")).focus({ preventScroll: true })))($("menu").contains(document.activeElement));
/** ⌘K in ⌘K: the highlighted row's actions, over the footer (on Home, it has none: by the row). */
function showActions(/** @type {HTMLElement | null} */ anchor = null) {
  // (Rows waiting for the series drawn first: the actions are the lit row's of what's typed.)
  palette.pending?.();
  // (On Home, by its row, in sight.)
  if (!anchor && $("palette").hasAttribute("data-on-home")) $(`option-${palette.active}`)?.scrollIntoView({ block: "nearest" });
  anchor ??= $("palette").hasAttribute("data-on-home") ? $(`option-${palette.active}`) : $("palette-actions");
  quickHide();
  const item = palette.items[palette.active], actions = actionsFor(item);
  if (actions.length < 2) return;
  // (A series', on a phone: what it is over them, as the pane beside the list shows it on a computer.)
  const detail = phone.matches && item.kind === "series";
  showMenu(anchor, (detail ? '<div class="palette-preview menu-detail"></div>' : "") + actions.map((action, i) => `${action.danger ? '<div class="menu-rule"></div>' : ""}<button type="button" data-action-index="${i}" ${action.danger ? 'class="danger"' : ""}>${action.icon ?? '<i class="menu-space"></i>'}${escapeHtml(action.label)}${action.kbd ? `<kbd>${escapeHtml(keyText(action.kbd))}</kbd>` : action.aside ? `<span>${escapeHtml(action.aside)}</span>` : ""}${action.page ? `<span class="sub">${ICONS.next}</span>` : ""}</button>`).join(""), ({ actionIndex }) => runAction(actions[Number(actionIndex)], item));
  const box = detail ? /** @type {HTMLElement | null} */ ($("menu").querySelector(".menu-detail")) : null;
  if (box) previewSeries(box, item.name, () => box.isConnected && !$("menu").hidden);
}
/**
 * An action, from ⌘K's menu or a page's row: a page opens; anything else acts, closing ⌘K unless it stays (a
 * toggle, a move: the page shows the new state, on the same row).
 * @param {Action} action
 */
function runAction(action, item) {
  quickHide();
  if (action.disabled) return notice(action.disabled);
  if (action.page) return pushPage(action.page());
  if (!action.stay) $("palette").hidePopover();
  const pages = palette.pages;
  action.run?.();
  // Still here, on the same page (the action didn't open another part of ⌘K): it shows what changed.
  if (!isOpen($("palette")) || palette.pages !== pages) return;
  palette.landOn = item?.key ?? null;
  showPalette();
  paletteKeys();
}

/** What's picked from ⌘K's search, how often and when: what's used most, lately, comes first. */
const USED = "bitview-studio:used";
/** A pick's weight: its count, fading over days (three days on, it counts half). */
const useScore = (key) => (used[key] ? used[key].n / (1 + (Date.now() - used[key].at) / (3 * 86_400_000)) : 0);
/** A list, what's used most lately first; the rest in their order. */
const byUse = (list) => list.map((item, i) => /** @type {[any, number, number]} */ ([item, useScore(item.key), i])).sort((a, b) => b[1] - a[1] || a[2] - b[2]).map(([item]) => item);
/** Picked from the search itself (not a page, nor adding or replacing): what's remembered. */
const fromSearch = () => !palette.pages.length && !palette.adding && !palette.replacing && !palette.formula;

/**
 * An action, a row of a menu or of a ⌘K page: a label, an aside (a current value; `mono` for a name or a formula);
 * a `page` it opens (beside it, in a menu), or what it `run`s (closing ⌘K unless it `stay`s). `current`: the choice
 * in effect (ticked); `disabled`: why it can't act (said when pressed); `danger` marks it (red); `apart` sets it off
 * in a menu (a rule before it), as a new `group` does (in ⌘K, its heading); `text`, what ⌘K says of it as a command.
 * @typedef {{ label: string, aside?: string, text?: string, mono?: boolean, kbd?: string, icon?: string, run?: () => void, page?: () => Page, leads?: boolean, stay?: boolean, danger?: boolean, current?: boolean, marked?: boolean, disabled?: string, apart?: boolean, verb?: string, key?: string, group?: string, preview?: (panel: HTMLElement) => unknown, look?: (doc: Chart) => void }} Action
 * @typedef {{ title: string | (() => string), heading?: boolean, placeholder?: string, rows: () => any[], options?: () => Action[], chooses?: boolean, input?: { value: string, placeholder: string, read: (text: string) => string | null, hint: string | ((text: string) => string), verb: string, icon?: string, apply: (text: string) => void, preview?: (panel: HTMLElement, text: string) => void }, query?: string, landOn?: string | null, searches?: boolean, global?: boolean, left?: () => void }} Page
 */
const pageTitle = (/** @type {Page} */ page) => (typeof page.title === "function" ? page.title() : page.title);
/**
 * Each pair of adjacent words in a text, apart by a space or a hyphen, joined: what a search finds a pair typed as
 * one word by ("bip69": BIP-69; "1week": 1 week, 1-week; "onchain": on-chain).
 */
function joinsOf(/** @type {string} */ text) {
  const words = text.replaceAll("-", " ").split(" ").filter(Boolean), joined = [];
  for (let k = 1; k < words.length; k++) joined.push(words[k - 1] + words[k]);
  return joined;
}
/**
 * A text (lowercased) as a search reads it: its words, and its pieces' pairs of them joined (see joinsOf; its pieces,
 * it alone unless said), each once, in their order (a word typed is found within one: none lost, a title's words
 * and its place's alike).
 */
function searchText(/** @type {string} */ lower, pieces = [lower]) {
  return [...new Set([...lower.split(" "), ...pieces.flatMap(joinsOf)])].join(" ");
}
/**
 * A typed word (see typedWords) as a pattern finding it in a text as written: as typed, or two words joined, spaces
 * and hyphens between them (see joinsOf): "1week", 1 week. Each a literal, none nested: no text takes long to search.
 */
function writtenPattern(/** @type {string} */ word) {
  // (Each dash as any of them: see searchCase.)
  const literal = (/** @type {string} */ text) => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&").replace(/-/g, "[-–−]");
  return [literal(word), ...Array.from({ length: word.length - 1 }, (_, k) => `${literal(word.slice(0, k + 1))}[-–− ]+${literal(word.slice(k + 1))}`)].join("|");
}
/**
 * A text as a search reads it: lowercased, its dashes one (a range typed "1-10" is the "1–10" shown, "-1" the "−1").
 * What's typed and what's searched both go through it.
 */
function searchCase(/** @type {string} */ text) {
  return text.toLowerCase().replace(/[–−]/g, "-");
}
/** What's typed into a search, as the words it looks for (see searchCase), each without the punctuation at its ends ("mvrv:", "(24h)"). */
function typedWords(/** @type {string} */ query) {
  return searchCase(query).split(/\s+/).map((word) => word.replace(/^[("']+|[)"':,;]+$/g, "")).filter(Boolean);
}
/** Texts' words as textWords made them, by text: ⌘K's rows are the same from one keystroke to the next. Let go of past a few thousand. */
const textWordsKept = /** @type {Map<string, { words: readonly string[], joins: readonly string[] }>} */ (new Map());
/**
 * A text's words (lowercased) as fitScore weighs what's typed against them, and its pairs of them joined (`joins`,
 * see joinsOf). Made once a text, kept (see textWordsKept): shared, never changed.
 */
function textWords(/** @type {string} */ lower) {
  const kept = textWordsKept.get(lower);
  if (kept) return kept;
  if (textWordsKept.size >= 4000) textWordsKept.clear();
  const made = { words: lower.split(/[^a-z0-9%×.]+/).filter(Boolean), joins: joinsOf(lower) };
  textWordsKept.set(lower, made);
  return made;
}
/**
 * Edits from one word to another (a letter added, dropped, changed, two swapped), stopping past `most`. (Three rows
 * taken in turn, each one's least kept as it's made: no call, spread or swap by destructuring a letter, as a phone
 * without a JIT pays for them.)
 */
function editsBetween(a, b, most) {
  if (Math.abs(a.length - b.length) > most) return most + 1;
  const n = b.length;
  let before = new Array(n + 1), row = new Array(n + 1), next = new Array(n + 1);
  for (let j = 0; j <= n; j++) row[j] = j;
  for (let i = 1; i <= a.length; i++) {
    let least = (next[0] = i);
    for (let j = 1; j <= n; j++) {
      const kept = row[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1), added = next[j - 1] + 1, dropped = row[j] + 1;
      let edits = kept < added ? kept : added;
      if (dropped < edits) edits = dropped;
      if (i > 1 && j > 1 && a[i - 1] === b[j - 2] && a[i - 2] === b[j - 1] && before[j - 2] + 1 < edits) edits = before[j - 2] + 1;
      next[j] = edits;
      if (edits < least) least = edits;
    }
    if (least > most) return most + 1;
    const spare = before;
    (before = row), (row = next), (next = spare);
  }
  return row[n];
}
/**
 * How far a typed word (four letters or more) is from `other`, one of the words it's checked among (`words`), as a
 * slip in typing: its edits (against as much of `other` as was typed and a letter), a slip at a letter, from seven
 * letters two; more, none. Past a stem the two share (one of `words`, from four letters), one edit at most: an ending
 * put in another's place isn't a slip ("monthly" isn't "months"; "addresess" is "addresses"); nor is the stem itself,
 * where they have it with another ending too ("annexes" is annex; "monthly" isn't month, there being months).
 * @param {string} word @param {string} other @param {{ has(word: string): boolean, keys(): Iterable<string> }} words
 */
function slipEdits(word, other, words) {
  const most = word.length >= 7 ? 2 : 1, edits = editsBetween(word, other.slice(0, word.length + 1), most);
  if (edits > most) return edits;
  let shared = 0;
  while (shared < word.length && shared < other.length && word[shared] === other[shared]) shared++;
  let stem = "";
  for (let k = shared; k >= 4 && !stem; k--) if (words.has(word.slice(0, k))) stem = word.slice(0, k);
  if (!stem) return edits;
  if (other !== stem) return edits > 1 ? most + 1 : edits;
  for (const each of words.keys()) if (each !== stem && each.startsWith(stem)) return most + 1;
  return edits;
}
/**
 * How well what's typed (`words`, see typedWords) finds a text, word by word: a whole word (two joined too, not by a
 * slip: see textWords), a word's start, inside one, or (from four letters)
 * a word of its `name` a letter or two off (a slip in what it's called, not in what's said of it: "light" isn't
 * "right"); 0 when a word finds nothing.
 */
function fitScore(text, words, name = text) {
  const lower = searchCase(text), { words: own, joins } = textWords(lower);
  /** Its name's words, made once a slip's looked for. @type {Set<string> | null} */
  let named = null;
  let total = 0;
  for (const word of words) {
    if (!/[a-z0-9]/.test(word)) {
      if (lower.includes(word)) total += 2;
      continue;
    }
    let best = 0;
    for (const other of own) {
      if (other === word) best = 4;
      else if (other.startsWith(word)) best = Math.max(best, 3);
      else if (word.length >= 3 && other.includes(word)) best = Math.max(best, 2);
      else if (!best && word.length >= 4 && (named ??= new Set(textWords(searchCase(name)).words)).has(other) && slipEdits(word, other, named) <= (word.length >= 7 ? 2 : 1)) best = 1;
      if (best === 4) break;
    }
    // (Two of its words joined: "bip69", BIP-69; found whole, by its start or inside, never by a slip.)
    for (let k = 0; best < 4 && k < joins.length; k++) {
      const other = joins[k];
      if (other === word) best = 4;
      else if (other.startsWith(word)) best = Math.max(best, 3);
      else if (word.length >= 3 && other.includes(word)) best = Math.max(best, 2);
    }
    // (Inside the text, not only a slip off one of its words: "30-day", a slip off its "30day", is in it.)
    if (best < 2 && word.length >= 3 && lower.includes(word)) best = 2;
    if (!best) return 0;
    total += best;
  }
  return total;
}
/** What removes or deletes: found as anything is, never lit for ↵ at once (see litFirst), nor among the recently used. */
const dangerous = (/** @type {any} */ item) => Boolean(item.danger ?? item.command?.danger);
/** What a row of ⌘K is called: a chart's title, a command's, a row's label (where a slip in typing still finds it). */
const nameOf = (/** @type {any} */ item) => (item.kind === "chart" ? titleOf(chartById(item.key) ?? { title: "" }) : item.command?.title ?? item.label ?? "");
/**
 * What's found, best first, then what's used most lately, then as listed (what removes, only when asked for);
 * `named`, what a slip in typing still finds it by (see nameOf).
 */
const rank = (list, textOf, words, named = nameOf) => list.map((item, i) => /** @type {[any, number, number]} */ ([item, fitScore(textOf(item), words, named(item)), i])).filter(([, score]) => score > 0).sort((a, b) => b[1] - a[1] || useScore(b[0].key) - useScore(a[0].key) || a[2] - b[2]).map(([item]) => item);
/** What a row is found by: what it shows (its label, its aside, a layer's letter); a chart, by its title. */
const rowText = (item) => (item.kind === "chart" ? titleOf(chartById(item.key) ?? { title: "" }) : `${item.label ?? ""} ${item.aside ?? ""} ${item.letter ?? ""}`);
/** A way in, by what it is, not what it shows at its end (the chart's title: the chart's own row has it). */
const wayText = (item) => item.label ?? "";
/** Into a page: what was typed, and the row that led there, come back with ⌫. @param {Page} page */
function pushPage(page, turn = true) {
  const from = palette.pages.at(-1) ?? palette.root;
  from.query = palette.query;
  from.landOn = palette.items[palette.active]?.key ?? null;
  palette.pages = [...palette.pages, page];
  // (Its search as it comes, a field's value, or what a search already had: see allOf.)
  palette.query = page.input?.value ?? page.query ?? "";
  $("palette-input").value = palette.query;
  if (turn) turnPage(1);
  else showPalette();
  paletteKeys(true);
}
/**
 * ⌘K's list turned to another page: the page left going as the next comes (deeper on, from the right; back, from
 * the left), no frame of neither. @param {1 | -1} way
 */
function turnPage(way) {
  pageLeaves(way);
  showPalette();
  pageComes(way);
}
/** ⌘K's list as it is, going (see turnPage). @param {1 | -1} way */
const pageLeaves = (way) => isOpen($("palette")) && fadeOut(pageSide(), [{ opacity: 1, transform: "none" }, { opacity: 0, transform: `translateX(${-8 * way}px)` }], MOTION.micro, true);
/** ⌘K's list as it's now, coming (see turnPage). @param {1 | -1} way */
const pageComes = (way) => isOpen($("palette")) && !reduced() && pageSide().animate([{ opacity: 0, transform: `translateX(${8 * way}px)` }, { opacity: 1, transform: "none" }], { duration: MOTION.small, easing: MOTION.ease });
/** A page's side of ⌘K: its path and its list, turned together (the path comes and goes with a page, the list under it). */
const pageSide = () => /** @type {HTMLElement} */ ($("palette-list").parentElement);
/** Back up to `depth` pages (0: the search), on the row that led on. */
function popPage(depth = palette.pages.length - 1) {
  leavePages(Math.max(0, depth));
  const back = palette.pages.at(-1) ?? palette.root;
  palette.query = back.query ?? "";
  palette.landOn = back.landOn ?? null;
  $("palette-input").value = palette.query;
  turnPage(-1);
  paletteKeys();
}
/** The pages past `depth` let go: one left as it was (`left`: a folder just made, its name never given) takes back what it began. */
function leavePages(/** @type {number} */ depth) {
  for (const page of palette.pages.splice(depth)) page.left?.();
}
/** A page: its rows, as filtered by what's typed; a name to type, its one row says what ↵ makes of it. */
function showPage() {
  const page = /** @type {Page} */ (palette.pages.at(-1));
  if (page.input) {
    const text = palette.query.trim(), input = page.input, label = input.read(text);
    // (Nothing typed, nothing it makes: what to type, plain, no row to press.)
    if (!text && !label) {
      palette.items = [];
      renderPalette(false, typeof input.hint === "function" ? input.hint(text) : input.hint);
      // (What it'll show, beside: two columns from the start, as once typed.)
      if (input.preview) $("palette").toggleAttribute("data-none", false), input.preview($("palette-preview"), "");
      return;
    }
    palette.items = [{ kind: "row", key: "apply", icon: input.icon ?? ICONS.pencil, label: label ?? (typeof input.hint === "function" ? input.hint(text) : input.hint), verb: input.verb, dim: !label, stay: true, run: label ? () => input.apply(text) : undefined, ...(input.preview ? { preview: (/** @type {HTMLElement} */ panel) => input.preview?.(panel, text) } : {}) }];
    palette.active = 0;
    return renderPalette();
  }
  const words = typedWords(palette.query);
  palette.items = words.length && !page.searches ? rank(page.rows(), rowText, words) : page.rows();
  // A search here can always widen to everything ⌘K finds.
  const query = palette.query.trim();
  if (query) palette.items.push({ kind: "row", key: "everywhere", icon: ICONS.browse, label: `Search everything for “${query}”`, group: palette.items.length ? "Everywhere" : `Nothing in ${pageTitle(page)} matches`, verb: "Search", stay: true, run: () => {
    leavePages(0);
    // (Everything: out of Add's series alone too, and of the catalog's folder it was in.)
    if (palette.adding) (palette.op = { kind: "browse" }), (palette.path = []);
    palette.query = query;
    $("palette-input").value = query;
    showPalette();
  } });
  // Back on the row that led on; else, in a choice, on the value in effect.
  const back = palette.items.findIndex((item) => item.key === palette.landOn);
  palette.active = Math.max(0, back >= 0 ? back : palette.items.findIndex((item) => item.current));
  palette.landOn = null;
  palette.moved = false;
  renderPalette();
}
/** The pages' trail under the search: ⌘K › Layers › Price › Color; each step goes back there. */
function renderPages() {
  const pages = palette.pages, page = /** @type {Page} */ (pages.at(-1));
  $("palette-path").hidden = false;
  drawTrail([["Search", 0], ...pages.map((item, i) => [pageTitle(item), i + 1])], pages.length);
  applyScope();
  // (A name, as the fields for names take: 120.)
  if (page.input) /** @type {HTMLInputElement} */ ($("palette-input")).maxLength = 120;
  $("palette-input").placeholder = page.input ? page.input.placeholder : page.placeholder ?? `Search ${pageTitle(page).toLowerCase()}`;
  $("palette-input").setAttribute("aria-label", page.input ? pageTitle(page) : `Search ${pageTitle(page).toLowerCase()}`);
}
/** A row: its icon, its label, at its end its value (or a check for the choice in effect), › if it opens a page (not while it can't). */
/** Whether a row of ⌘K's goes on inside it (a folder, a page, another search or an editor): its chevron says so, → takes it. */
function goesOn(/** @type {any} */ item) {
  return item.kind === "folder" || (item.kind === "command" ? Boolean(item.command.keepOpen && item.command.run) : Boolean((item.page || item.leads) && !item.disabled));
}
function rowHtml(item) {
  const query = palette.pages.at(-1)?.input ? "" : palette.query.trim();
  const initials = item.key?.startsWith("preset:");
  const aside = item.aside ? (item.mono ? `<code>${escapeHtml(item.aside)}</code>` : query && item.under ? item.aside.split(" › ").map((piece) => highlightMatch(piece, query, initials)).join(" › ") : escapeHtml(item.aside)) : "";
  const end = aside, marks = [item.current ? `<span role="img" aria-label="In effect">${ICONS.check}</span>` : "", goesOn(item) ? `<span class="item-next" aria-hidden="true">${ICONS.next}</span>` : ""].filter(Boolean).join(" ");
  return `${item.icon ?? (item.current === undefined ? "" : '<i class="menu-space"></i>')}<span class="item-name">${query ? highlightMatch(item.label, query, initials) : escapeHtml(item.label)}</span>${end ? `<span class="item-meta">${end}</span>` : ""}${marks ? `<span class="item-mark">${marks}</span>` : ""}${item.kbd ? `<span class="item-keys">${keycaps(item.kbd)}</span>` : ""}`;
}
/** A chart's layers as ⌘K's previews list them: each by its badge and name, what it draws across. @param {Layer[]} layers */
const layerRows = (layers) => layers.map((layer) => `<div><dt>${letterChip(layer)}<span class="name">${escapeHtml(labelOf(layer))}</span>${layer.hidden ? '<span class="muted">(hidden)</span>' : ""}</dt><dd>${typeName(layer.type)}</dd></div>`).join("");
function previewChart(panel, doc) {
  if (!doc) return void (panel.innerHTML = "");
  // (Its edit's time; a preset not edited, where it is.)
  const at = presetIndex(doc), where = doc.updated ? `edited ${ago(doc.updated)}` : at >= 0 ? `Presets › ${placeNames(/** @type {Preset[]} */ (presets)[at].p.slice(0, -1)).join(" › ")}` : "";
  panel.innerHTML = `<h4 class="plain">${escapeHtml(titleOf(doc))}</h4><p class="description">${[plural(doc.layers.length, "layer"), CADENCE[doc.index], where].filter(Boolean).map(escapeHtml).join(" · ")}</p>
          ${plotIn(doc)}${doc.layers.length ? `<div class="info"><h5>Layers</h5><dl>${layerRows(doc.layers.slice(0, 10))}</dl></div>` : ""}`;
}
/**
 * A chart (or a template's) as ⌘K previews it: drawn as its tile is, at once if it's drawn, else once it is (if
 * the preview still shows it then). @param {Chart} doc
 */
function plotIn(doc, index = doc.index) {
  if (!doc.layers.length) return "";
  const drawn = previews.get(doc.id);
  if (drawn?.key !== previewKey(doc)) void drawPreview(doc, index);
  return `<span class="palette-plot ${drawn ? "" : "pending"}" data-plot="${escapeHtml(doc.id)}">${drawn ? previewSvg(drawn, `palette-plot-${doc.id}`) : ""}</span>`;
}
/** A row's preview: its own; for a page, what's in it; else its name and value. */
function previewRow(panel, item) {
  if (item.preview) return item.preview(panel);
  if (item.page) {
    // (A setting's own page, as it is: the way into it, `page`, goes there, its owner's page under it.)
    const page = (item.of ?? item.page)(), rows = page.input ? [] : page.rows();
    panel.innerHTML = `<h4 class="plain">${escapeHtml(item.label)}</h4><p class="description">${escapeHtml(item.aside ?? "")}</p>${rows.length ? `<ul class="contents">${rows.slice(0, 14).map((row) => {
      const doc = row.kind === "chart" ? chartById(row.key) : null;
      return `<li>${doc ? chartMark(doc) : row.icon ?? ""}<span>${escapeHtml(doc ? titleOf(doc) : row.label)}</span></li>`;
    }).join("")}${rows.length > 14 ? `<li class="rest">and ${grouped.format(rows.length - 14)} more</li>` : ""}</ul>` : ""}`;
    return;
  }
  panel.innerHTML = `<h4 class="plain">${escapeHtml(item.label)}</h4>${item.aside ? `<p class="description">${escapeHtml(item.aside)}</p>` : ""}`;
}
/**
 * A choice among values, the one in effect checked: ↵ sets it and goes back to what it's a setting of. `preview`
 * shows what a value would make of it.
 */
function choicePage(title, options, preview = /** @type {any} */ (null)) {
  // (Its search the choices', whatever its title: "Search move to folder" read as nothing. `chooses`: a setting's
  // choices, what ⌘K's search finds it by too, see settingChoices.)
  return { title, options, chooses: true, placeholder: "Search the choices", rows: () => options().map((option) => ({ kind: "row", verb: "Choose", ...(preview ? { preview: (panel) => preview(panel, option) } : {}), ...option, stay: true, run: () => {
    const page = palette.pages.at(-1);
    option.run();
    // (One that leads on to a page of its own, a new folder's name, leaves it there.)
    if (palette.pages.at(-1) !== page && isOpen($("palette"))) return;
    if (palette.pages.length > 1) popPage();
    else $("palette").hidePopover();
  } })) };
}
/** How many bars a signal's time is shown among, at most: what led to it and what followed, seen (zoomed out further, in to that). */
const AROUND = 240;
/**
 * Each time a signal marked, the latest first: its date, up or down (one with a down state), what the stretch it
 * ends made (its results asked for: where it ends in neither too), its layer's value there; picked, gone to.
 * @param {string} id
 */
function timesPage(id) {
  return listPage("Times marked", () => {
    const layer = layerById(id), values = layer && resolved?.results.get(id)?.values, on = layer?.ref ? layerById(layer.ref) : null, own = on ? resolved?.results.get(on.id) : undefined;
    if (!layer || !values || !resolved) return [];
    const times = resolved.times, measurements = own && measured(own), changes = changesOf(values), signed = changes.some((change) => change.to < 0 || change.was < 0), marked = new Set(marksOf(layer, values));
    return changes.flatMap((change) => {
      const i = change.at, result = layer.results && own ? stretchResult(change, own) : undefined;
      if (!marked.has(i)) return [];
      const said = [change.to < 0 ? "Down" : change.to > 0 ? (signed ? "Up" : "") : "Off", result === undefined ? "" : resultText(result), on ? `${labelOf(on)}${own?.measurement ? " (raw)" : ""} ${format(measurements?.[i] ?? NaN, unitFor(on, own))}` : ""].filter(Boolean);
      return [{ key: `at:${times[i]}`, icon: ICONS.calendar, label: pointLabel(times[i]), aside: said.join(" · ") || undefined, run: () => goToTime(times[i]) }];
    }).reverse();
  }, "Go");
}
/**
 * A shown signal's next mark after the middle of the view (`by` 1), or its last one before it (-1):
 * gone to; none, said.
 */
function stepSignal(/** @type {1 | -1} */ by) {
  const range = shownSpan();
  if (!resolved || !range) return;
  // (From where the view's going, a glide under way: pressed again on the way, the next on, not the same again.)
  const going = engine.controller.target, middle = going ? (going.from + going.to) / 2 : (range.from + range.to) / 2, turns = [...new Set(chart.layers.filter((layer) => signalShown(layer)).flatMap((layer) => marksOf(layer, resolved?.results.get(layer.id)?.values)))].sort((a, b) => a - b);
  const at = by > 0 ? turns.find((i) => i > middle + 0.01) : turns.findLast((i) => i < middle - 0.01);
  // (Minutes and hours: what's loaded only, said.)
  if (at === undefined) return notice(`${by > 0 ? "No signal after this" : "No signal before this"}${fineDrawn() ? " in the loaded range" : ""}`);
  goToTime(resolved.times[at]);
}
/** A page of actions (each one acts and closes ⌘K), its rows as a menu has them. @returns {Page} */
const listPage = (/** @type {string} */ title, /** @type {() => Action[]} */ options, verb = "") => ({ title, options, rows: () => options().map((option) => ({ kind: "row", ...(verb ? { verb } : {}), ...option })) });
/**
 * A value typed in ⌘K's field: its one row says what ↵ makes of it (`read`), or why it can't (`hint`); ↵ sets it and
 * goes back to what it's a setting of.
 * `closes`: once applied, ⌘K closes (what it does is to be seen on the chart).
 * @param {{ title: string, value: string, placeholder: string, read: (text: string) => string | null, hint?: string | ((text: string) => string), verb?: string, icon?: string, closes?: boolean, apply: (text: string) => void, cancel?: () => void, preview?: (panel: HTMLElement, text: string) => void }} spec
 * @returns {Page}
 */
function inputPage({ title, value, placeholder, read, hint = "", verb = "Set", icon, closes = false, apply, cancel, preview }) {
  /** @type {Page} */
  const page = { title, rows: () => [], ...(cancel ? { left: cancel } : {}), input: { value, placeholder, read, hint, verb, icon, preview, apply: (text) => {
    // (Applied: nothing to take back as it goes.)
    delete page.left;
    apply(text);
    if (palette.pages.length > 1 && !closes) popPage();
    else $("palette").hidePopover();
  } } };
  return page;
}
/** A name to type: ↵ sets it, an empty one going back to the default. */
const renamePage = (title, value, fallback, apply) => inputPage({ title, value, placeholder: fallback, verb: "Rename", read: (text) => (text ? `Rename to “${text}”` : `Back to “${fallback}”`), apply });
/**
 * A number to type: an empty one is the `fallback`, when there is one.
 * @param {{ title: string, value: string, fallback?: number, apply: (n: number) => void }} spec
 */
function numberPage({ title, value, fallback, apply }) {
  const read = (text) => {
    const n = text ? readNumber(text) : fallback;
    return n !== undefined && Number.isFinite(n) ? n : null;
  };
  return inputPage({
    title, value, placeholder: fallback === undefined ? "A number" : String(fallback),
    hint: "A number",
    read: (text) => {
      const n = read(text);
      return n === null ? null : `Set to ${n}`;
    },
    apply: (text) => {
      const n = read(text);
      if (n !== null) apply(n);
    },
  });
}
const renameChartPage = (doc) => renamePage("Rename", doc.title ?? "", derivedTitle(doc), (text) => renameChart(chartById(doc.id) ?? doc, text));
/**
 * A folder, as ⌘K's page: expand or collapse it, rename it, move it, delete it (what's in it kept), and what's in it;
 * made again meanwhile (its deletion undone, redone), the folder it is now. @returns {Page}
 */
const folderOwnPage = (/** @type {string} */ given) => ({ title: () => folderById(folderNow(given))?.name ?? "Folder", placeholder: "Search this folder", rows: () => {
  const id = folderNow(given), folder = folderById(id);
  if (!folder) return [];
  return [
    ...folderRows(id, true).map((row) => ({ ...row, group: "Folder" })),
    ...((counts) => childrenOf(id).map((item) => ("layers" in item
      ? { key: `chart:${item.id}`, icon: chartMark(item), label: titleOf(item), verb: "Open", group: "In it", run: () => go(item) }
      : { key: `folder:${item.id}`, icon: folderMark(item), label: item.name, aside: plural(counts.get(item.id) ?? 0, "chart"), verb: "Open", group: "In it", page: () => folderOwnPage(item.id) })))(chartCounts()),
  ].map((row) => ({ kind: "row", ...row }));
} });
/** A chart's title, its own step wherever it's renamed from (see editChart). @param {Chart} doc @param {string} title */
const renameChart = (doc, title) => editChart(doc, (draft) => (draft.title = title));

const PANE_SETTINGS = new Set(["scale", "stack", "order", "average"]);
/**
 * Every setting and action of this chart, its layers and its panes, as found from the search: "Realized price ·
 * Color", its value at its end. Opened, it opens within its owner's page, so ← goes back there.
 */
function settingRows() {
  // (The chart's page adds only its settings, as the chart's: its actions are commands already; its title found
  // too. A layer's adds its own: its pane's are the pane's, once.)
  const shared = new Set(chart.layers.map(labelOf).filter((name, i, names) => names.indexOf(name) !== i));
  /** @type {{ name: string, page: () => Page, only: string[] | null, not: string[] }[]} */
  const owners = [
    { name: "Chart", page: chartPage, only: ["title", "interval", "range"], not: [] },
    ...chart.layers.map((layer) => ({ name: shared.has(labelOf(layer)) ? `${labelOf(layer)} (${layer.letter})` : labelOf(layer), page: () => layerPage(layer.id), only: null, not: [...PANE_SETTINGS] })),
    ...chart.panes.map((pane, i) => ({ name: `Pane ${i + 1}`, page: () => panePage(pane.id), only: null, not: [] })),
    ...((names) => (chart.drawings ?? []).map((drawing) => ({ name: /** @type {string} */ (names.get(drawing.id)), page: () => drawingPage(drawing.id), only: null, not: [] })))(drawingNames()),
  ];
  return owners.flatMap(({ name, page, only, not }) => page().rows().filter((row) => !row.key.startsWith("layer:") && !row.key.startsWith("add-") && (!only || only.includes(row.key)) && !not.includes(row.key)).map((row) => ({
    ...row, key: `setting:${name}:${row.key}`, group: "In this chart", label: `${name} · ${row.label}`, itself: row.label, of: row.page,
    page: row.page && (() => {
      const owner = page();
      pushPage(owner);
      palette.active = Math.max(0, palette.items.findIndex((item) => item.key === row.key));
      return row.page();
    }),
  })));
}
/**
 * The chart's settings as last found (settingRows), and each one's choices as read (see settingChoices): kept while
 * ⌘K stays open (`paletteOpenings`, each time it opens: what changes behind it, a pan, a rename undone in the
 * Library, isn't kept) and the chart, its edits (revision; `updated`, a Library undo's too), its data, its view's
 * width and whether it shows its range (presetShown), and the colors as they show are the same. (Found anew at
 * each keystroke of a search, they were half its time on a phone, 30 layers: ~40 ms a key.)
 */
let settingsKept = { key: /** @type {any[]} */ ([]), rows: /** @type {any[]} */ ([]), choices: /** @type {Map<string, { said: string, choices: any[] }>} */ (new Map()) };
function settingsNow() {
  const key = [paletteOpenings, chart.id, revision, chart.updated, resolved, home, presetShown, engine.time.width, document.documentElement.dataset.theme, tokensMedia.map((media) => media.matches).join()];
  if (key.some((part, i) => part !== settingsKept.key[i])) settingsKept = { key, rows: settingRows(), choices: new Map() };
  return settingsKept;
}
/**
 * A setting's choices (a choice page's, see choicePage; none for another page, a list's), each row's read once
 * while its settings are kept (settingsNow): what else it's found by (see settingRows), `said` their labels.
 */
const settingChoices = () => {
  const read = settingsNow().choices;
  /** @param {any} row @returns {{ said: string, choices: any[] }} */
  return (row) => {
    let said = read.get(row.key);
    if (!said) read.set(row.key, (said = ((page) => ((choices) => ({ said: choices.map((choice) => choice.label ?? "").join(" "), choices }))(page?.chooses ? page.options() : []))(row.of?.())));
    return said;
  };
};
/** Adding: what a layer can be besides a series, first on the page, in sight wherever it opens (and found by its words). */
function otherLayerRows() {
  return [
    { kind: "row", key: "add:formula", icon: ICONS.formula, label: "Formula", aside: "From the layers, by their letters", kbd: "F", preview: (/** @type {HTMLElement} */ panel) => (panel.innerHTML = `<h4 class="plain">Formula</h4><p class="description">A new layer computed from the others by their letters, with + − × ÷ and functions, or from any series by name.</p><ul class="contents">${((letters) => [...(letters.length > 1 ? [`${letters[0]} / ${letters[1]}`] : []), `sma(${letters[0] ?? "A"}, 200d)`, `zscore(${letters[0] ?? "A"}, 1y)`, EXAMPLE])(chart.layers.filter((layer) => layer.type !== "cloud").map((layer) => layer.letter)).map((text) => `<li><code>${escapeHtml(text)}</code></li>`).join("")}</ul>`), group: "Made from your layers", other: true, under: true, verb: "Add", leads: true, run: () => addFormula(palette.into) },
    // (A cloud, wherever there are layers (its pane's two edges are two); a trend, from layers.)
    ...(chart.layers.length ? [
      { kind: "row", key: "add:cloud", icon: ICONS.cloud, label: "Cloud", aside: "Filled between layers", group: "Made from your layers", other: true, under: true, verb: "Add", page: () => fromPage("cloud") },
    ] : []),
    ...(markable(chart).length ? [
      { kind: "row", key: "add:bubbles", icon: glyph("bubbles"), label: "Bubbles", aside: "A series' sizes on a layer", group: "Made from your layers", other: true, under: true, verb: "Add", leads: true, run: () => openPalette({ add: true, bubbles: true }) },
      { kind: "row", key: "add:signal", icon: ICONS.signal, label: "Signal", aside: "Marks where a formula turns up or down", group: "Made from your layers", other: true, under: true, verb: "Add", leads: true, run: () => addSignal() },
    ] : []),
    ...(cloudable() ? [
      { kind: "row", key: "add:trend", icon: ICONS.riding, label: "Trend", aside: trendShort() || "Rides two layers, flipped by a third", group: "Made from your layers", other: true, under: true, verb: "Add", page: () => fromPage("trend"), ...(trendShort() ? { disabled: trendShort() } : {}) },
    ] : []),
    // (A heatmap: a holder group's URPD, its own data, not made from layers.)
    { kind: "row", key: "add:heatmap", icon: ICONS.heatmap, label: "Heatmap", aside: HEAT_SHORT, preview: (/** @type {HTMLElement} */ panel) => (panel.innerHTML = `<h4 class="plain">Heatmap</h4><p class="description">${escapeHtml(HEAT_ABOUT)}</p>`), group: "Distributions", other: true, under: true, verb: "Add", page: heatAddPage },
    { kind: "row", key: "add:profile", icon: ICONS.profile, label: "Profile", aside: PROFILE_SHORT, preview: (/** @type {HTMLElement} */ panel) => (panel.innerHTML = `<h4 class="plain">Profile</h4><p class="description">${escapeHtml(PROFILE_ABOUT)}</p>`), group: "Distributions", other: true, under: true, verb: "Add", page: profileAddPage },
  ];
}
/** Nothing found: the two things searched, to browse instead (every series, by topic; the presets, by section), and other words to try. */
const browseInstead = () =>
  [
    ...(home ? [] : [{ key: "browse:series", icon: ICONS.series, label: "Browse every series", aside: "By topic", text: "Every series, in folders by topic", leads: true, run: () => openPalette({ add: true, path: [] }) }]),
    { key: "browse:presets", icon: ICONS.presets, label: "Browse the presets", aside: presets ? plural(presets.length, "chart") : "Loading…", text: "Ready-made charts, by section", page: () => presetsPage([]) },
  ].map(({ text, ...row }) => ({ kind: "row", group: "Nothing matches. Browse instead", verb: "Open", elsewhere: true, preview: (/** @type {HTMLElement} */ panel) => (panel.innerHTML = `<h4 class="plain">${escapeHtml(row.label)}</h4><p class="description">${text}. Or try other words, describing what the series measures.</p>`), ...row }));
function browseRows() {
  const any = chart.layers.length > 0;
  return [
    // (Its title any length: on a phone, its aside under it, as a found preset's is.)
    { key: "page:chart", icon: chartMark(chart), label: titleOf(chart), aside: "This chart", under: true, page: chartPage },
    ...(recentCharts().length ? [{ key: "page:recent", icon: ICONS.recent, label: "Recent", aside: plural(recentCharts().length, "chart"), page: recentPage }] : []),
    { key: "page:charts", icon: ICONS.keep, label: "Library", aside: plural(library.charts.length, "chart"), page: chartsPage },
    ...(any ? [
      { key: "page:layers", icon: ICONS.series, label: "Layers", aside: plural(chart.layers.length, "layer"), page: layersPage },
      { key: "page:panes", icon: ICONS.pane, label: "Panes", aside: plural(chart.panes.length, "pane"), page: panesPage },
      { key: "page:drawings", icon: ICONS.draw, label: "Drawings", aside: chart.drawings?.length ? plural(chart.drawings.length, "drawing") : "Lines, boxes, text", page: drawingsPage },
    ] : []),
    { key: "page:presets", icon: ICONS.presets, label: "Presets", aside: presets ? plural(presets.length, "chart") : "Loading…", page: () => presetsPage([]) },
  ].filter((row) => !home || row.key === "page:recent" || row.key === "page:charts" || row.key === "page:presets").map((row) => ({ kind: "row", group: "Browse", verb: "Open", ...row }));
}
/**
 * A chart of Recent's as ⌘K's row: a preset as every preset's (as edited here, if it is; `deep`: its place beside it),
 * once the presets are read (before, by its copy kept here, see prunePresets); yours and a preview by their chart.
 * @param {{ id: string, preset: number }} item
 */
const recentRow = ({ id, preset }, deep = false) => (preset >= 0 ? presetRow(preset, deep) : { kind: "chart", key: id });
/** A ⌘K page's first row, where it has a page of its own: that page (as a sidebar row's ⋯ › View). @param {Place} place */
const viewRow = (place) => ({ kind: "row", key: "page", icon: ICONS.arrowRight, label: "View", aside: "Its page", verb: "View", run: () => goTo(place) });
/** Recent, as ⌘K's page: its own page, then the charts last opened, the latest first (as the sidebar has them). */
const recentPage = () => {
  /** @type {Page} */
  const page = { title: "Recent", placeholder: "Search in Recent", rows: () => [viewRow({ recent: true }), ...recentCharts().map((item) => recentRow(item, true))] };
  // (Its presets come as they're read: the page shown again with them, see presetsArrived.)
  presetPages.add(page);
  return page;
};
/** The previews and your charts, as ⌘K's page: their own page first (View), then the previews, yours in no folder, then each folder's under its name and those it's in (the sidebar's order); and making more. */
const chartsPage = () => ({ title: "Library", placeholder: "Search your Library", rows: () => [
  viewRow({ folder: "" }),
  ...listedPreviews().map((doc) => ({ kind: "chart", key: doc.id, group: "Previews" })),
  ...childrenOf().filter((item) => "layers" in item).map((doc) => ({ kind: "chart", key: doc.id, group: "Saved" })),
  ...treeOf().flatMap(({ item }) => ("layers" in item ? [] : childrenOf(item.id).filter((doc) => "layers" in doc).map((doc) => ({ kind: "chart", key: doc.id, group: folderPath(item.id) })))),
  { kind: "row", key: "new", icon: ICONS.plus, label: "New chart", kbd: "N", verb: "Create", group: "Add", run: newChart },
  { kind: "row", key: "new-folder", icon: ICONS.folderAdd, label: "New folder", verb: "Create", group: "Add", run: () => (dismissPalette(), newFolder()) },
  { kind: "row", key: "import", icon: ICONS.upload, label: "Import charts", verb: "Import", group: "Files", run: () => importCharts() },
  ...(anythingKept() ? [{ kind: "row", key: "download-all", icon: ICONS.download, label: "Download all charts", aside: "Folders and all", verb: "Download", group: "Files", run: () => (dismissPalette(), downloadAll()) }] : []),
] });
/** Where charts go, as ⌘K's page. @param {string[]} ids @returns {Page} */
const folderPage = (ids) => choicePage("Move to folder", () => folderChoices(chartKeys(ids)));
/** The header's own buttons, by the ⋯ rows they'd repeat: a preview's Save, a preset's Save a copy and Reset. */
const HEAD_OWN = new Map([["keep", "keep"], ["copy", "preset-copy"], ["reset", "preset-reset"]]);
/**
 * The chart header's ⋯: the tools its header leaves to it first (a tablet's, Add and the drawing tools; a phone's,
 * the drawing tools: its bar has Add), then the chart's, but for those its header shows (a phone's, an edited
 * preset's Save a copy and Reset in it alone).
 */
const headMenu = () => [
  ...(tabletHead.matches && chart.layers.length ? [
    // (Back to ⋯ once the search is put away, as it'd be to Add.)
    { key: "add", icon: ICONS.plus, label: "Add a layer", kbd: "A", group: "tools", run: () => ($("chart-more").focus({ preventScroll: true }), openPalette({ add: true })) },
  ] : []),
  ...((tabletHead.matches || phone.matches) && chart.layers.length ? [
    { key: "draw", icon: ICONS.draw, label: beltUp() ? "Hide the drawing tools" : "Drawing tools", kbd: "D", group: "tools", run: () => (beltUp() ? hideBelt() : showBelt()) },
  ] : []),
  ...chartActions(chart.id, "head").filter((row) => !$(HEAD_OWN.get(row.key ?? "") ?? "")?.checkVisibility()),
];
/**
 * A chart's actions, where they show (`where`): "row", its sidebar row's ⋯ or right-click; "head", the header's ⋯
 * (it's the one open: renamed in its title; the legend and Export too; on a phone, Redo first: no room for it
 * beside Undo); "tile", its card's in the grid
 * (on it alone: the picks as they are); "palette", ⌘K's row of it (opened with ↵, renamed on a page, moved with ⌥↑
 * and ⌥↓). A preview's: kept or let go of, not duplicated or deleted.
 * @param {string} id @param {"row" | "head" | "tile" | "palette"} where @returns {Action[]}
 */
function chartActions(id, where = "palette") {
  const doc = chartById(id);
  if (!doc) return [];
  // (A preset's own.)
  if (doc.preset !== undefined) return presetIndex(doc) >= 0 ? presetActions(presetIndex(doc), where) : [];
  const preview = isPreview(doc), inPalette = where === "palette", tile = where === "tile", head = where === "head", phoneHead = head && phone.matches;
  const leave = preview ? "" : outOf([`c:${id}`]), peers = inPalette && !preview ? childrenOf(doc.folder) : [];
  /** (A card's acts on it alone, the picks as they are; elsewhere, as its row's.) */
  const act = (/** @type {string} */ kind) => () => (tile && !preview ? batch(kind === "download" ? "export" : kind, null, [id]) : chartPick(id, kind));
  return [
    ...(phoneHead && canTravel(false) ? [{ key: "redo", icon: ICONS.redo, label: "Redo", group: "redo", run: redoLast }] : []),
    ...(inPalette || tile ? [{ key: "open", icon: ICONS.arrowRight, label: "Open", kbd: inPalette ? "↵" : undefined, run: () => go(chartById(id)) }] : []),
    ...(tile && !preview && !picked.has(id) ? [{ key: "select", icon: ICONS.checkbox, label: "Select", run: () => pickChart(id) }] : []),
    ...(preview && doc.layers.length ? [{ key: "keep", icon: ICONS.keep, label: "Save", aside: inPalette ? undefined : "Into your Library", run: () => keepChart(doc) }] : []),
    { key: "rename", icon: ICONS.pencil, label: "Rename", ...(inPalette ? { page: () => renameChartPage(doc) } : head ? { run: renameInPlace } : { run: act("rename") }) },
    ...(preview ? [] : [{ key: "duplicate", icon: ICONS.duplicate, label: "Duplicate", run: act("duplicate") }]),
    ...(peers.indexOf(doc) > 0 ? [{ key: "up", icon: ICONS.arrowUp, label: "Move up", kbd: "⌥↑", stay: true, run: () => stepChart(id, -1) }] : []),
    ...(peers.length && peers.indexOf(doc) < peers.length - 1 ? [{ key: "down", icon: ICONS.arrowDown, label: "Move down", kbd: "⌥↓", stay: true, run: () => stepChart(id, 1) }] : []),
    ...(preview ? [] : [{ key: "folder", icon: ICONS.folder, label: "Move to folder", aside: inPalette ? (doc.folder ? folderPath(doc.folder) : "None") : undefined, page: () => choicePage("Move to folder", () => folderChoices([`c:${id}`], inPalette)) }]),
    ...(leave ? [{ key: "out", icon: ICONS.folderOut, label: leave, run: () => moveToFolder([`c:${id}`], null) }] : []),
    ...(head && doc.layers.length ? [{ ...legendAction(), apart: true }, { key: "export", icon: ICONS.download, label: "Export", aside: "Image, data, file", page: () => listPage("Export", exportRows) }] : []),
    ...(head ? [] : [{ key: "download", icon: ICONS.file, label: "Download chart file", aside: "JSON", run: act("download") }]),
    ...(!preview ? [{ key: "delete", icon: ICONS.trash, label: "Delete", danger: true, run: act("delete") }] : doc !== chart || previousChart() ? [{ key: "close", icon: ICONS.close, label: "Close the preview", apart: true, run: () => closePreview(doc) }] : []),
  ];
}
/** @returns {Page} */
function intervalPage() {
  const supported = supportedIntervals();
  return choicePage("Interval", () => INTERVAL_ORDER.map((id) => ({ key: id, icon: ICONS.recent, label: CADENCE[id], aside: INTERVAL_LABEL[id], current: chart.index === id, ...(supported(id) ? { run: () => switchInterval(id) } : { disabled: NO_INTERVAL }) })));
}
/** ⌘K's page of the open chart's windows (see chartWindow). @returns {Page} */
const windowPage = () => choicePage("Window", () => ((found) => (found?.offered ?? []).map((key) => ({ key, icon: ICONS.hourglass, label: capitalized(windowSaid(key)), aside: windowShort(key), current: found?.window === key, run: () => switchWindow(key) })))(chartWindow(chart)));
/** The ways to add a layer, as ⌘K's rows (under `group`): a series, a formula, a cloud, bubbles, a signal, a trend, a heatmap. */
const addRows = (/** @type {string} */ group) => [
  { key: "add-series", icon: ICONS.plus, label: "Add a layer", kbd: "A", verb: "Add", leads: true, stay: true, run: () => openPalette({ add: true }) },
  { key: "add-formula", icon: ICONS.formula, label: "Add a formula", kbd: "F", verb: "Add", leads: true, stay: true, run: addFormula },
  ...(chart.layers.length ? [{ key: "add-cloud", icon: ICONS.cloud, label: "Add a cloud", page: () => fromPage("cloud") }] : []),
  ...(markable(chart).length ? [
    { key: "add-bubbles", icon: glyph("bubbles"), label: "Add bubbles", verb: "Add", leads: true, stay: true, run: () => openPalette({ add: true, bubbles: true }) },
    { key: "add-signal", icon: ICONS.signal, label: "Add a signal", verb: "Add", leads: true, stay: true, run: addSignal },
  ] : []),
  ...(cloudable() ? [{ key: "add-trend", icon: ICONS.riding, label: "Add a trend", page: () => fromPage("trend"), ...(trendShort() ? { aside: trendShort(), disabled: trendShort(), under: true } : {}) }] : []),
  { key: "add-heatmap", icon: ICONS.heatmap, label: "Add a heatmap", aside: HEAT_SHORT, under: true, page: heatAddPage },
  { key: "add-profile", icon: ICONS.profile, label: "Add a profile", aside: PROFILE_SHORT, under: true, page: profileAddPage },
].map((row) => ({ kind: "row", group, ...row }));
/** ⌘K's page for a heatmap: a holder group to add one of (its supply by cost basis; the rest, its own to change). @returns {Page} */
const heatAddPage = () => choicePage("Add a heatmap", () => HEAT_COHORTS.map(({ id: cohort, shown: name }) => ({ key: cohort, icon: ICONS.people, label: name, verb: "Add", run: () => addHeatmap(cohort) })));
/** Adding a profile: long-term and short-term holders together first (the coins' two sides, each its color), then one group; more pressed in its settings. */
const profileAddPage = () => choicePage("Add a profile", () => [
  { key: "lth+sth", icon: ICONS.profile, label: `${holderName("lth").brief} and ${inSentence(holderName("sth").who)}`, aside: "Each its color", verb: "Add", run: () => addHeatmap("lth", ["lth", "sth"]) },
  ...HEAT_COHORTS.map(({ id: cohort, shown: name }) => ({ key: cohort, icon: `<i class="swatch" style="--c: ${escapeHtml(cssColor(cohortColor(cohort)))}"></i>`, label: name, verb: "Add", run: () => addHeatmap(cohort, [cohort]) })),
]);
/** @returns {Page} */
function chartPage() {
  return { title: () => titleOf(chart), placeholder: "Search this chart", rows: () => {
    const any = chart.layers.length > 0;
    return [
      { key: "title", icon: ICONS.pencil, label: "Title", aside: titleOf(chart), group: "Chart", page: () => renameChartPage(chart) },
      ...(any ? [
        { key: "interval", icon: ICONS.recent, label: "Interval", aside: CADENCE[chart.index], group: "Chart", page: intervalPage },
        ...((found) => (found ? [{ key: "window", icon: ICONS.hourglass, label: "Window", aside: capitalized(windowSaid(found.window)), group: "Chart", page: windowPage }] : []))(chartWindow(chart)),
        { key: "go-to", icon: ICONS.calendar, label: "Go to a date", aside: "A year, a month, a day", kbd: "G", group: "Chart", page: datePage },
        { key: "range", icon: ICONS.charts, label: "Range", aside: presetShown ? capitalized(periodOf(shownRange())) : "As you moved it", group: "Chart", page: () => choicePage("Range", () => RANGES.filter(([id]) => presetFits(id)).map(([id, text]) => ({ key: id, icon: ICONS.charts, label: text, aside: capitalized(periodOf(id)), current: presetShown && shownRange() === id, run: () => setRange(id) }))) },
        legendAction("Chart"),
        { key: "layers", icon: ICONS.series, label: "Layers", aside: plural(chart.layers.length, "layer"), group: "In it", page: layersPage },
        { key: "panes", icon: ICONS.pane, label: "Panes", aside: plural(chart.panes.length, "pane"), group: "In it", page: panesPage },
      ] : []),
      ...addRows("In it"),
      ...(any ? [
        { key: "share", icon: ICONS.share, label: SHARE, aside: "Its QR code too", verb: SHEET_SHARE ? "Share" : "Copy", group: "Out of it", run: () => $("share").click() },
        { key: "copy-image", icon: ICONS.copy, label: "Copy image", verb: "Copy", group: "Out of it", run: () => exportAs("copy") },
        { key: "export", icon: ICONS.download, label: "Export", aside: "An image, the data, the chart file", group: "Out of it", page: () => listPage("Export", exportRows) },
      ] : []),
      ...(!isYours(chart) ? [] : [{ key: "folder", icon: ICONS.folder, label: "Folder", aside: chart.folder ? folderPath(chart.folder) : "None", group: "Chart file", page: () => folderPage([chart.id]) }]),
      ...(isPreview(chart) && chart.layers.length ? [{ key: "keep", icon: ICONS.keep, label: "Save", aside: "Save this preview to your Library", verb: "Save", group: "Chart file", run: () => keepChart() }] : []),
      ...(!isYours(chart) ? [] : [{ key: "duplicate", icon: ICONS.duplicate, label: "Duplicate", verb: "Duplicate", group: "Chart file", run: () => chartPick(chart.id, "duplicate") }]),
      ...(!isYours(chart) ? [] : [{ key: "delete", icon: ICONS.trash, label: "Delete", verb: "Delete", group: "Chart file", danger: true, run: () => deleteChart(chart) }]),
      .../** @type {any[]} */ (chart.preset !== undefined && presetIndex(chart) >= 0 ? presetActions(presetIndex(chart), "palette").filter((row) => row.key !== "open").map((row) => ({ ...row, group: "Preset" })) : []),
    ].map((row) => ({ kind: "row", preview: (/** @type {HTMLElement} */ panel) => previewChart(panel, chart), ...row }));
  } };
}
/** @returns {Page} */
const layersPage = () => ({ title: "Layers", placeholder: "Search this chart's layers", rows: () => [
  ...chart.panes.flatMap((pane, i) => chart.layers.filter((layer) => layer.pane === pane.id).map((layer) => ({ ...layerRow(layer), group: `Pane ${i + 1}` }))),
  ...addRows("Add"),
  ...(chart.layers.some((layer) => layer.hidden) ? [{ kind: "row", key: "show-all", icon: ICONS.eye, label: "Show every layer", verb: "Show", group: "Add", stay: true, run: showEvery }] : []),
] });
/** A layer row's ⌘K: what's changed most, the rest a ↵ away on its page. */
const QUICK = new Set(["series", "formula", "between", "name", "type", "color", "pane", "derive", "hide", "solo", "duplicate", "remove"]);
/** A layer, by its letter and name, with what it shows (a series, a formula): ↵ opens all it is, to change. */
function layerRow(layer) {
  const open = () => layerPage(layer.id);
  return {
    kind: "row", key: `layer:${layer.id}`, icon: `${letterChip(layer)}`, label: labelOf(layer),
    aside: layer.from ? `Between ${fromSpan(layer) || "no layers"}` : layer.series !== undefined || heatLike(layer) ? "" : layer.label ? layer.formula : formulaLine(layer), mono: !layer.from && Boolean(layer.label), dim: Boolean(layer.hidden), letter: layer.letter, verb: "Open", page: open,
    actions: () => [{ label: "Open", icon: ICONS.arrowRight, kbd: "↵", page: open }, ...open().rows().filter((row) => QUICK.has(row.key))],
    preview: (/** @type {HTMLElement} */ panel) => previewLayer(panel, layer.id),
  };
}
/** A change to a layer, kept with ↵ (`run`), shown on the chart while highlighted (`look`). @param {string} id */
const bothOf = (id) => (/** @type {(item: Layer, doc?: Chart) => void} */ edit) => ({ run: () => editById(id, edit), look: (/** @type {Chart} */ doc) => {
  const item = doc.layers.find((other) => other.id === id);
  if (item) edit(item, doc);
} });
/**
 * Everything a layer is, each with its value, each to change, grouped as the inspector's sections are: its source
 * and name; its style (lines across included); its layout (pane, scale, axis, order, format); then what's done to
 * it (formulas from it, hiding, the panel, a copy, removing it). A choice or a value typed comes back here.
 * @returns {Page}
 */
function layerPage(id) {
  return { title: () => labelOf(layerById(id) ?? { label: "Layer" }), placeholder: "Search its settings", rows: () => {
    const layer = layerById(id);
    if (!layer) return [];
    const result = resolved?.results.get(layer.id), unit = unitFor(layer, result), at = chart.panes.findIndex((pane) => pane.id === layer.pane);
    const lineish = LINED.has(layer.type);
    const set = (edit) => () => editById(id, edit);
    const both = bothOf(id);
    const now = () => layerById(id) ?? layer;
    // Grouped as the inspector's sections are, named as they are.
    // (Its name with what it is: a heatmap's, a cloud's or a trend's under theirs, not "Formula".)
    const first = layer.series ? "Series" : layer.type === "signal" ? "Signal" : layer.type === "profile" ? "Profile" : heatLike(layer) ? "Heatmap" : layer.from ? (layer.type === "cloud" ? "Cloud" : "Trend") : "Formula";
    const style = dashOf(layer.dash);
    // A choice of its colors (a color, a way of coloring) previewed as it would draw: its look on a copy, its colors date by date.
    const lookOfChoice = (/** @type {HTMLElement} */ panel, /** @type {Action} */ option) => {
      if (!option.look || !resolved) return previewLayer(panel, id);
      const doc = structuredClone(chart);
      option.look(doc);
      const item = doc.layers.find((one) => one.id === id);
      return item ? previewLayer(panel, id, item.color, colorsAt(doc, item, resolved)) : previewLayer(panel, id);
    };
    // A color of it, its page: Bitview's colors, another; a fading one of four, its side's fainter too.
    const ROW_KEYS = /** @type {Record<string, string>} */ ({ color: "color", colorBelow: "color-below", colorFading: "color-fading", colorBelowFading: "color-below-fading" });
    const color = (/** @type {string} */ key, /** @type {string} */ label) => ({ key: ROW_KEYS[key] ?? key, icon: swatchOf(colorOf(layer, key)), label, aside: slotName(layer, key), group: "Style", page: () => choicePage(label, () => [
      ...colorRows(key === "colorFading" ? now().colorFading ?? "" : key === "colorBelowFading" ? now().colorBelowFading ?? "" : colorOf(now(), key), (name) => set((item) => setColor(item, key, name))(), (doc, name) => {
        const item = doc.layers.find((other) => other.id === id);
        if (item) setColor(item, key, name);
      }),
      // (A gradient's color: its place, typed; it removed (two left at least).)
      ...(key.startsWith("stop") ? ((k, stops) => [
        { key: "place", icon: ICONS.ruler, label: "Its place", aside: `${stops[k]?.[0] ?? 0}%`, page: () => numberPage({ title: `${label}, its place`, value: String(stops[k]?.[0] ?? 0), apply: (n) => editById(id, (item) => moveStop(item, k, n)) }) },
        { key: "remove", icon: ICONS.trash, label: "Remove it", danger: true, ...(stops.length > GRADIENT_LEAST ? {} : { disabled: `A gradient has ${GRADIENT_LEAST} colors at least` }), run: () => editById(id, (item) => removeStop(item, k)) },
      ])(Number(key.slice(4)), now().gradient ?? []) : []),
      ...(key === "colorFading" || key === "colorBelowFading" ? [((side) => ({ key: "paler", icon: swatchOf(faintOf(side)), label: `Fainter ${colorName(side).toLowerCase()}`, aside: "The growing one's", current: (key === "colorFading" ? now().colorFading : now().colorBelowFading) === undefined, run: set((item) => setColor(item, key, "")), look: (/** @type {Chart} */ doc) => ((item) => item && setColor(item, key, ""))(doc.layers.find((one) => one.id === id)) }))(key === "colorFading" ? now().color : belowOf(now()))] : []),
    ], lookOfChoice) });
    const source = layerById(layer.colorFrom ?? "");
    const rows = [
      ...(heatLike(layer) ? heatRows(id) : layer.from ? [{ key: "between", icon: layer.type === "cloud" ? ICONS.cloud : ICONS.riding, label: "Between", aside: fromSpan(layer) || "None", group: layer.type === "cloud" ? "Cloud" : "Trend", leads: true, page: () => fromPage(/** @type {"cloud" | "trend"} */ (layer.type), id) }]
        : layer.series ? [{ key: "series", icon: ICONS.swap, label: "Replace series", aside: layer.series, mono: true, verb: "Replace", group: first, leads: true, stay: true, run: () => changeSeries(id) }]
        : [{ key: "formula", icon: ICONS.formula, label: "Formula", aside: layer.formula, mono: true, verb: "Edit", group: first, leads: true, stay: true, run: () => editFormula(id) }]),
      ...(layer.type === "trend" ? [{ key: "ref", icon: ICONS.riding, label: "Flips when it touches", aside: ((ref) => (ref ? labelOf(ref) : "None"))(layerById(layer.ref ?? "")), group: "Trend", page: () => choicePage("Flips when it touches", () => refChoices(id)) }] : []),
      ...(layer.type === "signal" ? [
        { key: "ref", icon: ICONS.signal, label: "Marked on", aside: ((ref) => (ref ? labelOf(ref) : "None"))(layerById(layer.ref ?? "")), group: first, page: () => choicePage("Marked on", () => refChoices(id)) },
        { key: "place", icon: glyphSvg(markBody(layer.place, "currentColor")), label: "Shown as", aside: placeName(layer.place ?? "above"), group: first, page: () => choicePage("Shown as", () => MARK_PLACES.map(([place]) => ({ key: place, icon: glyphSvg(markBody(place, "currentColor")), label: placeName(place), current: (now().place ?? "above") === place, run: set((item) => (item.place = place === "above" ? undefined : place)) }))) },
        { key: "results", icon: ICONS.signal, label: "Results", aside: layer.results ? "On" : "Off", verb: layer.results ? "Turn off" : "Turn on", group: first, stay: true, run: set((item) => (item.results = !item.results || undefined)) },
        { key: "times", icon: ICONS.calendar, label: "Times marked", aside: ((values) => (values ? plural(marksOf(layer, values).length, "time") : undefined))(result?.values), group: first, page: () => timesPage(id) },
      ] : []),
      ...(layer.series ? [
        { key: "copy-name", icon: ICONS.copy, label: "Copy the series name", verb: "Copy", group: first, run: () => copyName(/** @type {string} */ (layer.series)) },
      ] : []),
      ...(!layer.from && layer.type !== "signal" ? /** @type {const} */ ([["cycles", false, "Compare halving cycles"], ["cycles", true, "Overlay halving cycles"], ["years", false, "Compare years"], ["years", true, "Overlay years"]]).map(([which, over, label]) => ({ key: over ? `${which}-over` : which, icon: ICONS.pane, label, verb: over ? "Overlay" : "Compare", group: first, ...((why) => (why ? { aside: why, disabled: why } : { aside: over ? "Each over the ones before" : "Each from its start" }))(((periods) => (periods.length ? unfitHere([periods[0].formula]) : overShort(which)))(periodsOf(layer, which, over))), run: () => comparePeriods(id, which, over) })) : []),
      { key: "name", icon: ICONS.pencil, label: "Name", aside: labelOf(layer), group: first, page: () => renamePage("Name", layer.label ?? "", labelOf({ ...layer, label: undefined }), (text) => editById(id, (item) => nameLayer(item, text))) },
      // (A bubbles layer's own, after its series' or formula's, as its settings have them: one group each.)
      ...(layer.type === "bubbles" ? [
        { key: "ref", icon: glyph("bubbles"), label: "On", aside: ((ref) => (ref ? labelOf(ref) : "Its pane's middle"))(layerById(layer.ref ?? "")), group: "Bubbles", page: () => choicePage("On", () => refChoices(id)) },
        { key: "size", icon: glyph("bubbles"), label: "Size", aside: `${layer.size ?? BUBBLE_SIZE} pixels`, group: "Bubbles", page: () => choicePage("Size", () => BUBBLE_SIZES.map((n) => ({ key: String(n), label: `${n} pixels`, aside: n === BUBBLE_SIZE ? "Its own" : undefined, current: (now().size ?? BUBBLE_SIZE) === n, ...both((item) => (item.size = n === BUBBLE_SIZE ? undefined : n)) }))) },
        { key: "hide", icon: glyph("bubbles"), label: "Hide below", aside: ((n) => (n ? `${hideText(n)} of the largest` : "None"))(layer.hide ?? BUBBLE_HIDE), group: "Bubbles", page: () => choicePage("Hide below", () => BUBBLE_HIDES.map((n) => ({ key: String(n), label: hideText(n), aside: n === BUBBLE_HIDE ? "Its own" : n ? undefined : "Every one, however small", current: (now().hide ?? BUBBLE_HIDE) === n, ...both((item) => (item.hide = n === BUBBLE_HIDE ? undefined : n)) }))) },
        { key: "sized", icon: glyph("bubbles"), label: "Sized to", aside: layer.scaleTo === "all" ? "All time" : "In view", verb: layer.scaleTo === "all" ? "Size to what's in view" : "Size to all time", group: "Bubbles", stay: true, run: set((item) => (item.scaleTo = item.scaleTo === "all" ? undefined : "all")) },
      ] : []),
      ...(seriesLooked(layer) ? [{ key: "type", icon: glyph(layer.type), label: "Type", aside: TYPES.find(([type]) => type === layer.type)?.[1], group: "Style", page: () => choicePage("Type", () => TYPES.filter(([type]) => type !== "candles" || result?.opens || (layer.series && isOhlc(layer.series))).map(([type, text]) => ({ key: type, icon: glyph(type), label: text, current: now().type === type, ...both((item) => reshape(item, (one) => (one.type = type))) }))) }] : []),
      ...(layer.type === "baseline" || layer.type === "histogram" ? [{ key: "base", icon: ICONS.ruler, label: "Base", aside: format(baseOf(layer), unit), group: "Style", page: () => numberPage({ title: "Base", value: String(baseOf(layer)), fallback: 0, apply: (n) => editById(id, (item) => (item.base = n || undefined)) }) }] : []),
      ...(layer.type === "baseline" ? [lineRow(id, "base", "Base line", "Style")] : []),
      ...(modesOf(layer, chart).length > 1 ? [{ key: "colors", icon: modeSwatch(layer, colorModeOf(layer)), label: "Colors", aside: modeName(colorModeOf(layer), layer), group: "Style", page: () => choicePage("Colors", () => colorModeChoices(id), lookOfChoice) }] : []),
      ...colorSlots(layer).map(([key, label]) => (key === "from"
        ? { key: "color-from", icon: source ? letterChip(source) : ICONS.series, label: "Colors from", aside: source ? labelOf(source) : "", group: "Style", page: () => choicePage("Colors from", () => fromChoices(id), lookOfChoice) }
        : color(key, colorSlots(layer).length > 1 || colorModeOf(layer) === "levels" ? `Color ${inSentence(label)}` : "Color"))),
      // (A gradient's: another color, and what its lowest and highest are of.)
      ...((stops) => !stops ? [] : [
        // (The gradients ready made, as its editor's row of them: each a small bar of itself.)
        { key: "gradient-ready", icon: gradientSwatch(stops), label: "Ready made", aside: readyName(stops), group: "Style", page: () => readyPage(id, lookOfChoice) },
        { key: "gradient-reverse", icon: ICONS.swap, label: "Reverse the gradient", aside: "Its lowest's color its highest's", group: "Style", run: () => reverseGradient(id) },
        { key: "gradient-add", icon: ICONS.plus, label: "Add a color", aside: "In the widest gap", group: "Style", ...(stops.length < GRADIENT_MOST ? { run: set((item) => addStop(item)) } : { disabled: `A gradient holds up to ${GRADIENT_MOST} colors` }) },
        { key: "relative", icon: ICONS.calendar, label: "Relative to", aside: layer.upToDate ? "Up to each date" : "All of history", group: "Style", page: () => choicePage("Relative to", () => /** @type {[boolean, string, string][]} */ ([[false, "All of history", "Its lowest and highest of all its history"], [true, "Up to each date", "What each day showed then, no hindsight"]]).map(([on, text, says]) => ({ key: String(on), label: text, aside: says, current: Boolean(now().upToDate) === on, ...both((item) => (item.upToDate = on || undefined)) }))) },
      ])(ownGradient(chart, layer)),
      { key: "opacity", icon: ICONS.sliders, label: "Opacity", aside: `${Math.round(opacityOf(layer) * 100)}%`, group: "Style", page: () => choicePage("Opacity", () => OPACITIES.map((opacity) => ({ key: String(opacity), icon: ICONS.sliders, label: `${Math.round(opacity * 100)}%`, current: opacityOf(now()) === opacity, ...both((item) => (item.opacity = opacity === defaultOpacity(item) ? undefined : opacity)) }))) },
      ...(layer.type !== "histogram" && !placedOn(layer) ? [widthRow(layer.width, now, both)] : []),
      ...(lineish ? [
        dashRow(style, now, both),
        { key: "shape", icon: shapeGlyph(layer.shape ?? "straight"), label: "Shape", aside: SHAPES.find(([shape]) => shape === (layer.shape ?? "straight"))?.[1], group: "Style", page: () => choicePage("Shape", () => SHAPES.map(([shape, text]) => ({ key: shape, icon: shapeGlyph(shape), label: text, aside: shape === "steps" ? "Each value held until the next" : undefined, current: (now().shape ?? "straight") === shape, ...both((item) => (item.shape = shape === "straight" ? undefined : shape)) }))) },
      ] : []),
      ...(layer.type === "area" ? [
        { key: "fill", icon: glyph("area"), label: "Fill", aside: layer.fill === "above" ? "From the line up" : "From the line down", verb: layer.fill === "above" ? "Fill down" : "Fill up", group: "Style", stay: true, run: set((item) => (item.fill = item.fill === "above" ? undefined : "above")) },
        { key: "fill-opacity", icon: ICONS.sliders, label: "Fill opacity", aside: `${Math.round(fillOf(layer, stackedIn(layer))[0] * 100)}%`, group: "Style", page: () => choicePage("Fill opacity", () => OPACITIES.map((fill) => ({ key: String(fill), icon: ICONS.sliders, label: `${Math.round(fill * 100)}%`, current: fillOf(now(), stackedIn(now()))[0] === fill, ...both((item) => (item.fillOpacity = fill === fillOf({ ...item, fillOpacity: undefined }, stackedIn(item))[0] ? undefined : fill)) }))) },
        // (Its style where it fades: filled above or stacked, it's even already.)
        ...(layer.fill !== "above" && !stackedIn(layer) ? [{ key: "fill-style", icon: glyph("area"), label: "Fill style", aside: layer.solid ? "Solid" : "Gradient", verb: layer.solid ? "Make it a gradient" : "Make it solid", group: "Style", stay: true, run: set((item) => (item.solid = !item.solid || undefined)) }] : []),
      ] : []),
      ...(layer.type === "candles" ? ((hollow) => [{ key: "hollow", icon: glyph("candles"), label: "Candles", aside: hollow ? "Hollow" : "Filled", verb: hollow ? "Fill them" : "Make rising ones hollow", group: "Style", stay: true, run: set((item) => (item.hollow = hollowOf(item) ? "none" : undefined)) }])(hollowOf(layer)) : []),
      ...(OUTLINED.has(layer.type) ? [{ key: "outline", icon: OUTLINE_GLYPH, label: "Outline", aside: layer.outline ? "On" : "Off", verb: layer.outline ? "Turn off" : "Turn on", group: "Style", stay: true, run: set((item) => (item.outline = !item.outline || undefined)) }] : []),
      ...(layer.type === "bubbles" ? [] : [
        { key: "price-line", icon: dashGlyph(2, 2), label: "Price line", aside: layer.priceLine ? ((into) => (layer.type !== "signal" && into ? `On, not drawn in ${into}` : "On"))(drawnInto(chart, chart.panes.find((pane) => pane.id === layer.pane))) : "Off", verb: layer.priceLine ? "Turn off" : "Turn on", group: "Style", stay: true, run: set((item) => (item.priceLine = !item.priceLine || undefined)) },
        { key: "levels", icon: dashGlyph(5, 3), label: "Levels", aside: layer.levels?.length ? `${plural(layer.levels.length, "level")}${((why) => (why ? `, ${why.toLowerCase()}` : ""))(acrossUnfit(chart, layer))}` : "None", group: "Style", page: () => levelsPage(id) },
      ]),
      { key: "pane", icon: ICONS.pane, label: "Pane", aside: `Pane ${at + 1}`, group: "Layout", page: () => choicePage("Pane", () => paneChoices([id])) },
      { key: "side", icon: ICONS.axis, label: "Axis", aside: layer.side === "left" ? "Left" : "Right", verb: `To the ${layer.side === "left" ? "right" : "left"}`, group: "Layout", stay: true, run: set((item) => (item.side = item.side === "left" ? "right" : "left")) },
      ...(fittable(layer) ? [{ key: "fit", icon: ICONS.axis, label: "Fit the axis", aside: FITS.find(([key]) => key === fitOf(layer))?.[1] ?? "On", group: "Layout", page: () => choicePage("Fit the axis", () => fitChoices([now()]).map(([key, label, aside]) => ({ key, label, aside, current: fitOf(now()) === key, run: set((item) => setFit(item, key)) }))) }] : []),
      ...panePage(layer.pane).rows().filter((row) => PANE_SETTINGS.has(row.key)).map((row) => ({ ...row, group: "Layout" })),
      ...((mates) => [
        ...(mates.indexOf(layer) > 0 ? [{ key: "up", icon: ICONS.arrowUp, label: "Move up", aside: `Swaps with ${labelOf(mates[mates.indexOf(layer) - 1])}`, verb: "Move", group: "Layout", stay: true, run: () => moveLayer(id, -1) }] : []),
        ...(mates.indexOf(layer) < mates.length - 1 ? [{ key: "down", icon: ICONS.arrowDown, label: "Move down", aside: `Swaps with ${labelOf(mates[mates.indexOf(layer) + 1])}`, verb: "Move", group: "Layout", stay: true, run: () => moveLayer(id, 1) }] : []),
      ])(chart.layers.filter((item) => item.pane === layer.pane)),
      { key: "format", icon: ICONS.table, label: "Format", aside: formatName(layer, result), group: "Layout", page: () => choicePage("Format", () => formatChoices(id)) },
      ...layerActions(id, "palette").map((row) => ({ ...row, group: "Layer" })),
    ];
    // (One made from layers: what's a series' or a formula's isn't its; a cloud's, what's a line's either.)
    const NOT_ITS = new Set(["type", "fill", "base", "levels", "to-formula", "to-series", "shape", ...(follows(layer) ? ["pane", "side"] : []), ...(layer.type === "cloud" ? ["width", "dash", "price-line", "derive"] : [])]);
    // (A cloud's or a trend's place its layers'; a signal: its formula, where its marks are, its color; no line's
    // look, nor a place of its own: its layer's.)
    const NOT_A_SIGNAL = new Set(["type", "fill", "base", "levels", "to-series", "shape", "colors", "color-from", "width", "dash", "price-line", "pane", "side", "scale", "stack", "order", "up", "down", "format"]);
    // (A heatmap: its days, its cohort's: nothing of a line's look or a series' (its colors and opacity its own).)
    const NOT_A_HEATMAP = new Set(["cycles", "years", "type", "base", "color-from", "width", "dash", "shape", "fill", "fill-opacity", "fill-style", "price-line", "levels", "stack", "format"]);
    return (layer.type === "signal" ? rows.filter((row) => !NOT_A_SIGNAL.has(row.key)) : heatLike(layer) ? rows.filter((row) => !NOT_A_HEATMAP.has(row.key)) : layer.from ? rows.filter((row) => !NOT_ITS.has(row.key)).map((row) => (row.key === "name" ? { ...row, group: typeName(layer.type) } : row)) : rows).map((row) => ({ kind: "row", preview: (/** @type {HTMLElement} */ panel) => previewLayer(panel, id), ...row }));
  } };
}
const swatchOf = (color) => `<i class="swatch" style="--c:${escapeHtml(cssColor(color))}"></i>`;
/** A color typed: a hex (#, or not; three digits or six), an OKLCH, or any color CSS reads, as a hex (opaque only). */
function readColor(/** @type {string} */ text) {
  const typed = text.trim().toLowerCase(), hex = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/.exec(typed);
  // (A person's own, kept in hex: shown as typed in either theme, see shownOklch.)
  if (hex) return `#${hex[1].length === 3 ? [...hex[1]].map((digit) => digit + digit).join("") : hex[1]}`;
  if (isColor(typed)) return typed.startsWith("oklch(") ? oklchToHex(...oklchOf(typed)) : typed;
  if (!typed || !CSS.supports("color", typed)) return null;
  measure.fillStyle = "#000000";
  measure.fillStyle = typed;
  return /^#[0-9a-f]{6}$/.test(measure.fillStyle) ? measure.fillStyle : null;
}
/**
 * ⌘K's colors: Bitview's, each a row (`look`: shown on the chart while highlighted), and any other, typed (as the
 * surface picks one).
 */
function colorRows(/** @type {string} */ current, /** @type {(color: string) => void} */ apply, /** @type {((doc: Chart, color: string) => void) | undefined} */ look = undefined) {
  const own = Boolean(current) && !SWATCHES.includes(current);
  return [
    ...SWATCHES.map((name) => ({ key: name, icon: swatchOf(name), label: colorName(name), current: current === name, run: () => apply(name), ...(look ? { look: (/** @type {Chart} */ doc) => look(doc, name) } : {}) })),
    { key: "other", icon: own ? swatchOf(current) : ICONS.pencil, label: "Another color", aside: own ? ink(current) : "A hex, OKLCH or CSS color", current: own, page: () => inputPage({ title: "Another color", value: own ? ink(current) : "", placeholder: "#f7931a, oklch(0.7 0.18 50), teal…", hint: "A hex, OKLCH or CSS color", verb: "Set", read: (text) => (readColor(text) ? `Set to ${readColor(text)}` : null), apply: (text) => {
      const color = readColor(text);
      if (color) apply(color);
    } }) },
  ];
}
/** A drawing's row's ⌘K: what's changed most, the rest a ↵ away on its page. */
const DRAWING_QUICK = new Set(["flip-x", "flip-y", "duplicate", "remove"]);
/** The chart's drawings, each to open; the tools, to draw another; removing them all. @returns {Page} */
const drawingsPage = () => ({ title: "Drawings", placeholder: "Search the drawings", rows: () => {
  const names = drawingNames();
  return [
    // (↵ selects one on the chart, ⌘K closed: the arrows move it there, ⌘ the arrows its end; all it is, in its actions.)
    ...(chart.drawings ?? []).map((drawing) => ({ kind: "row", key: drawing.id, icon: TOOLS[drawing.kind].icon, label: /** @type {string} */ (names.get(drawing.id)), aside: `On ${labelOf(layerById(drawing.layer) ?? { label: "" })}`, group: "On the chart", verb: "Select", run: () => selectInView(drawing.id), actions: () => [
      { label: "Settings", icon: ICONS.sliders, page: () => drawingPage(drawing.id) },
      ...drawingPage(drawing.id).rows().filter((row) => DRAWING_QUICK.has(String(row.key))),
    ] })),
    ...KINDS.map((kind) => ({ kind: "row", key: `tool:${kind}`, icon: TOOLS[kind].icon, label: TOOLS[kind].verb, aside: TOOLS[kind].tip, under: true, kbd: kind === "ruler" ? "M" : `D ${TOOLS[kind].key}`, verb: "Draw", group: "Draw", run: () => takeUp(kind) })),
    { kind: "row", key: "tool:magnifier", icon: ICONS.magnifier, label: "Magnifier", aside: capitalized(LENS_SAYS), under: true, kbd: "D Z", verb: magnifying ? "Stop" : "Use", group: "Draw", run: () => takeUp(magnifying ? "" : "magnifier") },
    ...(chart.drawings?.length ? [
      { kind: "row", key: "select-box", icon: ICONS.select, label: "Select drawings with a box", aside: "Drag a box around them", kbd: "D S", verb: "Select", group: "Select", run: () => takeUp("select") },
      { kind: "row", key: "select-all", icon: ICONS.select, label: "Select every drawing", aside: plural(chart.drawings.length, "drawing"), verb: "Select", group: "Select", run: () => chooseDrawings((chart.drawings ?? []).map((drawing) => drawing.id)) },
      { kind: "row", key: "clear", icon: ICONS.trash, label: "Remove every drawing", verb: "Remove", group: "Draw", danger: true, run: clearDrawings },
    ] : []),
  ];
} });
/** A drawing, all it is: its color, its line (a text's words and size), a trend line's reach; showing it; removing it. @param {string} id @returns {Page} */
function drawingPage(id) {
  return { title: () => drawingNames().get(id) ?? "Drawing", placeholder: "Search its settings", rows: () => {
    const drawing = drawingById(id);
    if (!drawing) return [];
    const now = () => drawingById(id) ?? drawing, style = dashOf(drawing.dash);
    /** A change, kept with ↵, shown on the chart while highlighted. */
    const both = (edit) => ({ run: () => editById(id, edit), look: (/** @type {Chart} */ doc) => {
      const item = doc.drawings?.find((other) => other.id === id);
      if (item) edit(item);
    } });
    return [
      ...(drawing.kind === "text" ? [{ key: "text", icon: ICONS.pencil, label: "Text", aside: drawing.text, verb: "Edit", group: "Style", run: () => {
        $("palette").hidePopover();
        const now = drawingById(id);
        if (now) startTyping(now, $("plot-keys"));
      } }] : []),
      // (A ruler's look is its reading's: up or down.)
      ...(drawing.kind === "ruler" ? [] : [
        { key: "color", icon: swatchOf(drawing.color), label: "Color", aside: colorName(drawing.color), group: "Style", page: () => choicePage("Color", () => colorRows(now().color, (name) => editById(id, (item) => (item.color = name)), (doc, name) => {
          const item = doc.drawings?.find((other) => other.id === id);
          if (item) item.color = name;
        })) },
        { key: "opacity", icon: ICONS.opacity, label: "Opacity", aside: `${Math.round((drawing.opacity ?? 1) * 100)}%`, group: "Style", page: () => choicePage("Opacity", () => OPACITIES.map((opacity) => ({ key: String(opacity), icon: ICONS.opacity, label: `${Math.round(opacity * 100)}%`, current: (now().opacity ?? 1) === opacity, ...both((item) => (item.opacity = opacity === 1 ? undefined : opacity)) }))) },
      ]),
      ...(drawing.kind === "ruler" ? [] : drawing.kind === "text"
        ? []
        : [
          widthRow(drawing.width, now, both),
          dashRow(style, now, both),
        ]),
      ...(LINES.has(drawing.kind) ? [{ key: "label", icon: ICONS.label, label: "Label", aside: drawing.label ?? "None", group: "Style", page: () => inputPage({ title: "Label", value: drawing.label ?? "", placeholder: "Label text (leave empty for none)", hint: "No label", verb: "Set", read: (text) => (labelText(text) ? `Label it “${labelText(text)}”` : drawing.label ? "Remove its label" : null), apply: (text) => editById(id, (item) => {
        if (labelText(text)) item.label = labelText(text);
        else {
          delete item.label;
          delete item.at;
        }
      }) }) }] : []),
      ...(drawing.kind === "text" || drawing.label ? [
        { key: "size", icon: ICONS.text, label: "Size", aside: SIZE_NAMES[drawing.size ?? TEXT_SIZE], group: "Style", page: () => choicePage("Size", () => SIZES.map((size) => ({ key: String(size), icon: ICONS.text, label: SIZE_NAMES[size], aside: `${size}px`, current: (now().size ?? TEXT_SIZE) === size, ...both((item) => (item.size = size)) }))) },
        { key: "back", icon: backGlyph(drawing.back ?? ""), label: "Background", aside: backOf(drawing.back)[1], group: "Style", page: () => choicePage("Background", () => BACKS.map(([back, text, tip]) => ({ key: back || "none", icon: backGlyph(back), label: text, aside: tip, current: (now().back ?? "") === back, ...both((item) => (item.back = back || undefined)) }))) },
      ] : []),
      // (Each end its own: plain, an arrow, on past its point.)
      ...(THROUGH.has(drawing.kind) ? /** @type {const} */ (["start", "end"]).map((side) => ({ key: side, icon: side === "start" ? endsGlyph(drawing.start, "") : endsGlyph("", drawing.end), label: side === "start" ? "Start" : "End", aside: endName(drawing[side]), group: "Style", page: () => choicePage(side === "start" ? "Start" : "End", () => ENDS.map(([end, text, tip]) => ({ key: end || "plain", icon: side === "start" ? endsGlyph(end, "") : endsGlyph("", end), label: text, aside: tip, current: (now()[side] ?? "") === end, ...both((item) => {
        if (end) item[side] = end;
        else delete item[side];
      }) }))) })) : []),
      { key: "select", icon: TOOLS[drawing.kind].icon, label: "Select it", aside: "On the chart: the arrows move it", verb: "Select", group: "Drawing", run: () => selectInView(id) },
      ...drawingRows([id]).filter((row) => DRAWING_QUICK.has(String(row.key))).map((row) => ({ ...row, group: "Drawing" })),
    ].map((row) => ({ kind: "row", ...row }));
  } };
}
/** A line across, as a row: its color and style, opening them (and a level's value) to change. */
function lineRow(id, key, label, group) {
  const layer = /** @type {Layer} */ (layerById(id)), look = lineLook(layer, key);
  return { key: `line:${key}`, icon: swatchOf(look.color), label: look.name ? `${label}: ${look.name}` : label, aside: `${colorName(look.color)}, ${lineStyleName(look).toLowerCase()}`, group, page: () => linePage(id, key, label) };
}
/** A line across (a level, by index, or the base): a level's value; its color, its style; removing a level. @returns {Page} */
function linePage(id, key, title) {
  return { title, placeholder: "Search its settings", rows: () => {
    const layer = layerById(id), level = key === "base" ? null : layer?.levels?.[Number(key)];
    if (!layer || (key !== "base" && !level)) return [];
    const now = () => lineLook(layerById(id) ?? layer, key), look = now();
    return [
      ...(level ? [{ key: "name", icon: ICONS.label, label: "Name", aside: level.name ?? "None: its value shows", page: () => inputPage({ title: "Name", value: level.name ?? "", placeholder: "Bull, bear, overheated…", hint: "What its values from here up read as, in its color", verb: "Name", read: (text) => (text.trim() ? `Name it “${text.trim().slice(0, 40)}”` : "No name: its value shows"), apply: (text) => editById(id, (item) => {
        const line = item.levels?.[Number(key)];
        if (!line) return;
        if (text.trim()) line.name = text.trim().slice(0, 40);
        else delete line.name;
      }) }) }] : []),
      ...(level ? [{ key: "value", icon: ICONS.pencil, label: "Value", aside: format(level.value, unitFor(layer, resolved?.results.get(layer.id))), page: () => numberPage({ title: "Value", value: String(level.value), apply: (n) => editById(id, (item) => item.levels?.[Number(key)] && (item.levels[Number(key)].value = n)) }) }] : []),
      { key: "color", icon: swatchOf(look.color), label: "Color", aside: colorName(look.color), page: () => choicePage("Color", () => colorRows(now().color, (name) => setLineLook(id, key, "color", name), (doc, name) => lineLookOn(doc, id, key, "color", name))) },
      { key: "dash", icon: lineStyleGlyph(look), label: "Line style", aside: lineStyleName(look), page: () => choicePage("Line style", () => lineChoices(id, key)) },
      { key: "tag", icon: ICONS.label, label: "Value on the axis", aside: look.tag && look.line !== false ? "On" : "Off", verb: look.tag ? "Turn off" : "Turn on", stay: true, ...(look.line === false ? { disabled: TAG_NEEDS_LINE } : {}), run: () => setLineLook(id, key, "tag", now().tag ? undefined : true) },
      ...(level ? [{ key: "remove", icon: ICONS.trash, label: "Remove the level", verb: "Remove", danger: true, stay: true, run: () => {
        editById(id, (item) => withoutLevel(item, Number(key)));
        popPage();
      } }] : []),
    ].map((row) => ({ kind: "row", preview: (/** @type {HTMLElement} */ panel) => previewLayer(panel, id), ...row }));
  } };
}
/** A layer's levels, each with its color and style, and a new one at a value typed (the panel's round one over the latest, to start). @returns {Page} */
function levelsPage(id) {
  return { title: "Levels", placeholder: "Search its levels", rows: () => {
    const layer = layerById(id);
    if (!layer) return [];
    const unit = unitFor(layer, resolved?.results.get(layer.id));
    return [
      // (Highest first, as the panel has them.)
      ...(layer.levels ?? []).map((level, i) => ({ level, i })).sort((a, b) => b.level.value - a.level.value || a.i - b.i).map(({ level, i }) => ({ ...lineRow(id, String(i), format(level.value, unit), ""), key: `level:${i}`, page: () => linePage(id, String(i), `Level at ${format(level.value, unit)}`) })),
      ...(layer.levels?.length && (layer.byLevels || named(layer)) ? [{ key: "under", icon: swatchOf(layer.color), label: `Under ${format(Math.min(...layer.levels.map((level) => level.value)), unit)}`, aside: layer.underName ? `Named “${layer.underName}”` : "Its name: none, its value shows", page: () => inputPage({ title: "Under its levels", value: layer.underName ?? "", placeholder: "Bear, cold, oversold…", hint: "What its values under the lowest level read as", verb: "Name", read: (text) => (text.trim() ? `Name it “${text.trim().slice(0, 40)}”` : "No name: its value shows"), apply: (text) => editById(id, (item) => (text.trim() ? (item.underName = text.trim().slice(0, 40)) : delete item.underName)) }) }] : []),
      { key: "add", icon: ICONS.plus, label: "Add a level", aside: "At a value you type", ...((layer.levels?.length ?? 0) < MAX_LEVELS ? {} : { disabled: `A layer holds up to ${MAX_LEVELS} levels` }), page: () => numberPage({ title: "New level", value: String(levelValue(layer)), apply: (n) => {
        const index = layerById(id)?.levels?.length ?? 0;
        editById(id, (item) => withLevel(item, n));
        // Back on the levels, on the new one.
        const levels = palette.pages.at(-2);
        if (levels) levels.landOn = `level:${index}`;
      } }) },
    ].map((row) => ({ kind: "row", preview: (/** @type {HTMLElement} */ panel) => previewLayer(panel, id), ...row }));
  } };
}
/**
 * A layer's latest, said, where its line's caption doesn't say it: a cloud's range, a heatmap's cell at the price
 * (its readout's at rest); else "—" till its line's drawn.
 * @param {Layer} layer @param {Result | undefined} result @param {string} unit
 */
function latestSaid(layer, result, unit) {
  const last = (resolved?.times.length ?? 0) - 1;
  if (layer.type === "cloud" && result?.values) return rangeAt(layer, { at: result.values.length - 1, rest: true }, unit);
  if (heatLike(layer) && last >= 0) return ((reading) => (reading ? readingLine(reading) : "—"))(heatReading(layer, { at: last, rest: true }));
  return "—";
}
/** A layer: what it shows, its line as drawn now (in its color, or `color`), its type, unit, place, latest value. */
function previewLayer(panel, id, color = /** @type {string | null} */ (null), /** @type {((i: number) => string | null) | null | undefined} */ tint = undefined) {
  const layer = layerById(id);
  if (!layer) return void (panel.innerHTML = "");
  const result = resolved?.results.get(layer.id), unit = unitFor(layer, result), at = chart.panes.findIndex((pane) => pane.id === layer.pane);
  const about = layer.series ? known.get(layer.series)?.description : null;
  // (What it is, in its own words: its formula, what it is between; a series or a heatmap by its name and description, its id is the layer page's.)
  const source = layer.from ? escapeHtml(`Between ${fromSpan(layer) || "no layers"}`) : layer.formula !== undefined ? `<code>${escapeHtml(layer.formula)}</code>` : "";
  panel.innerHTML = `<h4 class="plain">${escapeHtml(viewName(chart, layer))}</h4>
          ${source && `<p class="description">${source}</p>`}
          ${about ? `<div class="description clamped">${proseHtml(about)}</div>` : ""}
          ${result?.error ? `<div class="error-box">${escapeHtml(result.detail ?? result.error)}.</div>` : layer.type === "cloud" || heatLike(layer) ? "" : '<svg class="spark" viewBox="0 0 300 100" preserveAspectRatio="none" aria-hidden="true"></svg>'}
          <div class="info"><h5>Layer ${escapeHtml(layer.letter)}</h5><dl>
            <div><dt>Type</dt><dd>${escapeHtml(typeName(layer.type))}</dd></div>
            <div><dt>Unit</dt><dd>${escapeHtml(UNIT_NAMES[unit] ?? "Number")}</dd></div>
            <div><dt>Layout</dt><dd>Pane ${at + 1}, ${layer.side === "left" ? "left" : "right"} axis</dd></div>
            <div><dt>Latest</dt><dd class="spark-caption"${layer.type === "cloud" || heatLike(layer) ? ` data-latest="${escapeHtml(layer.id)}"` : ""}>${escapeHtml(latestSaid(layer, result, unit))}</dd></div>
          </dl></div>`;
  // (Its colors date by date, where they vary: as given, else as it's drawn; a color given, that one alone.)
  const dated = tint !== undefined ? tint : color ? null : resolved ? paintOf(chart, layer, resolved) : null;
  if (result?.values && !result.error && layer.type !== "cloud") drawSpark(panel, Array.from(result.values), unit, ink(color ?? layer.color), dated && ((i) => ((name) => (name ? ink(name) : null))(dated(i))));
}
/** @returns {Page} */
const panesPage = () => ({ title: "Panes", placeholder: "Search this chart's panes", rows: () => [
  ...chart.panes.map((pane, i) => paneRow(pane.id, i)),
  ...(chart.panes.length > 1 ? [
    ...PANE_HEIGHTS.map(({ key, label, text, verb, set }) => ({ kind: "row", key, icon: ICONS.pane, label, aside: text, verb, group: "Heights", stay: true, run: () => setPaneHeights(set) })),
  ] : []),
] });
/** A pane's share of the chart's height, in percent, among those that show (folded, none: see foldedPanes). */
function shareOf(i) {
  const folded = foldedPanes(chart), sizes = chart.panes.map((pane, k) => (folded.has(pane.id) ? 0 : pane.size ?? defaultSize(k)));
  return Math.round((sizes[i] / sizes.reduce((sum, size) => sum + size, 0)) * 100);
}
/** Whether a pane's height can be chosen: it shows, and so does another. */
const sizable = (/** @type {number} */ i) => ((folded) => !folded.has(chart.panes[i].id) && chart.panes.filter((pane) => !folded.has(pane.id)).length > 1)(foldedPanes(chart));
/** A pane at a share of the chart's height, the others keeping theirs between them. */
const setShare = (id, share) => commit((doc) => shareOn(doc, id, share));
/** The same, on a doc as it is (a copy, to look at). @param {Chart} doc */
function shareOn(doc, id, share) {
  // (Among the panes that show: a folded one keeps its own, for when it opens.)
  const folded = foldedPanes(doc), i = doc.panes.findIndex((pane) => pane.id === id), open = doc.panes.map((pane) => !folded.has(pane.id));
  const sizes = doc.panes.map((pane, k) => pane.size ?? defaultSize(k)), others = sizes.reduce((sum, size, k) => sum + (k !== i && open[k] ? size : 0), 0);
  if (i < 0 || !open[i] || !others) return;
  sizes[i] = (share * others) / (1 - share);
  // Scaled to sum to the number of panes that show: choices made one after another don't grow them past what's kept.
  const total = sizes.reduce((sum, size, k) => sum + (open[k] ? size : 0), 0), count = open.filter(Boolean).length;
  doc.panes.forEach((pane, k) => open[k] && (pane.size = Math.max(0.01, Math.round((sizes[k] / total) * count * 100) / 100)));
}
const SHARES = [10, 25, 33, 50, 67, 75, 90];
/** A pane's Height: its share of the chart, chosen among SHARES (seen as it would be, on ⌘K's page). @param {string} id */
function heightRow(id) {
  const at = () => chart.panes.findIndex((pane) => pane.id === id);
  return { key: "height", icon: ICONS.sliders, label: "Height", aside: `${shareOf(at())}% of the chart`, page: () => choicePage("Height", () => SHARES.map((share) => ({ key: String(share), icon: ICONS.sliders, label: `${share}% of the chart`, current: shareOf(at()) === share, run: () => setShare(id, share / 100), look: (/** @type {Chart} */ doc) => shareOn(doc, id, share / 100) })), (panel, option) => previewPane(panel, id, Number(option.key) / 100)) };
}
/** A pane, by its layers, scale and height: ↵ opens all it is. */
function paneRow(id, i) {
  const pane = chart.panes[i], open = () => panePage(id);
  return {
    kind: "row", key: `pane:${id}`, icon: ICONS.pane, label: `Pane ${i + 1}`, aside: `${paneNames(pane)} · ${paneChip(pane)}${foldedPanes(chart).has(pane.id) ? " · Folded" : sizable(i) ? ` · ${shareOf(i)}%` : ""}`, verb: "Open", page: open,
    actions: () => [{ label: "Open", icon: ICONS.arrowRight, kbd: "↵", page: open }, ...open().rows().filter((row) => !row.key.startsWith("layer:"))],
    preview: (/** @type {HTMLElement} */ panel) => previewPane(panel, id),
  };
}
/** Everything a pane is: its scale, its height, its place (swapped with a neighbor), removing it; its layers. @returns {Page} */
function panePage(id) {
  const at = () => chart.panes.findIndex((pane) => pane.id === id);
  return { title: () => `Pane ${at() + 1}`, placeholder: "Search this pane", rows: () => {
    const i = at();
    if (i < 0) return [];
    const pane = chart.panes[i], layers = chart.layers.filter((layer) => layer.pane === id);
    return [
      ...(summed(pane) ? [] : [{ key: "scale", icon: ICONS.axis, label: "Scale", aside: MODES[paneMode(pane)].name, group: "Pane", page: () => choicePage("Scale", () => scaleGroup(id, "Scale"), (panel) => previewPane(panel, id)) }]),
      { key: "stack", icon: ICONS.charts, label: "Stack", aside: STACKS.find(([stack]) => stack === (pane.stack ?? ""))?.[1], group: "Pane", page: () => choicePage("Stack", () => scaleGroup(id, "Stack")) },
      ...(layers.length > 1 ? [{ key: "order", icon: ICONS.charts, label: "Order", aside: ordersOf(pane)[pane.reverse ? 1 : 0][1], group: "Pane", page: () => choicePage("Order", () => scaleGroup(id, "Order")) }] : []),
      ...(gradable(chart, pane) ? [{ key: "gradient", icon: ICONS.charts, label: "Gradient", aside: paneGradient(pane)[1], group: "Pane", page: () => choicePage("Gradient", () => scaleGroup(id, "Gradient")) }] : []),
      ...(outlinable(chart, pane) ? [{ key: "outline", icon: ICONS.charts, label: "Outline", aside: OUTLINES[pane.outline ? 1 : 0][1], group: "Pane", page: () => choicePage("Outline", () => scaleGroup(id, "Outline")) }] : []),
      { key: "average", icon: ICONS.sliders, label: "Average", aside: averageFor(chart, pane).active || "Raw", group: "Pane", page: () => choicePage("Average", () => scaleGroup(id, "Average")) },
      ...paneRows(id, true).map((row) => ({ ...row, group: "Pane" })),
      ...layers.map((layer) => ({ ...layerRow(layer), group: "Layers" })),
      { key: "add-series", icon: ICONS.plus, label: "Add a layer here", verb: "Add", group: "Layers", leads: true, stay: true, run: () => openPalette({ add: true, into: id }) },
      { key: "add-formula", icon: ICONS.formula, label: "Add a formula here", verb: "Add", group: "Layers", leads: true, stay: true, run: () => addFormula(id) },
    ].map((row) => ({ kind: "row", preview: (/** @type {HTMLElement} */ panel) => previewPane(panel, id), ...row }));
  } };
}
/** A pane: where it sits among the others (a map of their heights; at `share` of the chart, if given), its scale, its layers. */
function previewPane(panel, id, share = /** @type {number | null} */ (null)) {
  const i = chart.panes.findIndex((pane) => pane.id === id), pane = chart.panes[i];
  if (!pane) return void (panel.innerHTML = "");
  const layers = chart.layers.filter((layer) => layer.pane === id);
  // (A folded pane, a sliver: it shows only its legend.)
  const folded = foldedPanes(chart), sizes = chart.panes.map((other, k) => (folded.has(other.id) ? 0 : other.size ?? defaultSize(k))), others = sizes.reduce((sum, size, k) => sum + (k === i ? 0 : size), 0);
  if (share !== null) sizes[i] = (share * others) / (1 - share);
  panel.innerHTML = `<h4 class="plain">Pane ${i + 1}</h4>
          <p class="description">${escapeHtml(`${pane.stack === "percent" ? "100% stacked" : `${MODES[paneMode(pane)].name} scale${pane.stack === "rows" ? ", as a heatmap" : pane.stack ? ", stacked" : ""}`}${pane.reverse ? `, ${ordersOf(pane)[1][1].toLowerCase()}` : ""}${((average) => average.name ? `, ${average.name}` : "")(averageFor(chart, pane))}`)}${folded.has(id) ? ", folded: every layer in it hidden" : sizable(i) ? `, ${share !== null ? Math.round(share * 100) : shareOf(i)}% of the chart's height` : ""}.</p>
          <div class="pane-map" aria-hidden="true">${sizes.map((size, k) => `<i class="${k === i ? "on" : ""}" style="flex:${size ? size : "0 0 4px"}">${size ? k + 1 : ""}</i>`).join("")}</div>
          <div class="info"><h5>Layers</h5><dl>${layerRows(layers) || "<div><dt>None yet</dt><dd></dd></div>"}</dl></div>`;
}
/**
 * A date as typed (2021, 2021-11, nov 2021, 2021-11-10, 10 nov 2021): the span it names, a year, a month or a day,
 * in seconds, and how to say it; null if it isn't one.
 * @returns {{ from: number, to: number, day: boolean, label: string } | null}
 */
function readDate(text) {
  const words = text.trim().toLowerCase().replace(/,/g, " ").replace(/\s+/g, " ");
  const month = (word) => (word.length >= 3 ? MONTH_NAMES.findIndex((name) => name.startsWith(word)) : -1);
  /** @type {number | undefined} */
  let at, day;
  let match, year = 0;
  // (Today and yesterday by name.)
  if (words === "today" || words === "yesterday") {
    const now = new Date(), time = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate()) - (words === "yesterday" ? 86400000 : 0);
    return { from: time / 1000, to: time / 1000 + 86399, day: true, label: dateLabel(time / 1000, false) };
  }
  if ((match = /^(\d{4})$/.exec(words))) year = Number(match[1]);
  else if ((match = /^(\d{4})[-/.](\d{1,2})$/.exec(words))) [year, at] = [Number(match[1]), Number(match[2]) - 1];
  else if ((match = /^(\d{4})[-/.](\d{1,2})[-/.](\d{1,2})$/.exec(words))) [year, at, day] = [Number(match[1]), Number(match[2]) - 1, Number(match[3])];
  else if ((match = /^([a-z]+) (\d{4})$/.exec(words)) && month(match[1]) >= 0) [year, at] = [Number(match[2]), month(match[1])];
  else if ((match = /^(\d{1,2}) ([a-z]+) (\d{4})$/.exec(words)) && month(match[2]) >= 0) [year, at, day] = [Number(match[3]), month(match[2]), Number(match[1])];
  else if ((match = /^([a-z]+) (\d{1,2}) (\d{4})$/.exec(words)) && month(match[1]) >= 0) [year, at, day] = [Number(match[3]), month(match[1]), Number(match[2])];
  else return null;
  if (year < 1900 || year > 2100 || (at !== undefined && (at < 0 || at > 11)) || (day !== undefined && day < 1)) return null;
  if (day !== undefined) {
    const time = Date.UTC(year, /** @type {number} */ (at), day);
    // No 31 November.
    if (new Date(time).getUTCDate() !== day) return null;
    return { from: time / 1000, to: time / 1000 + 86399, day: true, label: dateLabel(time / 1000, false) };
  }
  if (at !== undefined) return { from: Date.UTC(year, at, 1) / 1000, to: Date.UTC(year, at + 1, 1) / 1000 - 1, day: false, label: monthYear(Date.UTC(year, at, 1) / 1000, true) };
  return { from: Date.UTC(year, 0, 1) / 1000, to: Date.UTC(year + 1, 0, 1) / 1000 - 1, day: false, label: String(year) };
}
/** Where a span falls in this chart's points: the first in it and the last (a day: the point that holds it). */
function pointsOf(span) {
  const times = resolved?.times ?? [];
  const last = Math.max(0, lowerBound(times, span.to + 1) - 1);
  return { first: Math.min(lowerBound(times, span.from), last), last };
}
/** The stretch a date shows (its middle, as wide as the chart shows, when it's longer), and whether it's all loaded. */
function stretchOf(span) {
  // (A span running past now, this month or year, ends now: its middle is of what there is.)
  const step = INTERVAL_DAYS[shownIndex()] * 86400, range = shownSpan(), end = Math.min(span.to, Date.now() / 1000), middle = (span.from + end) / 2;
  // (A day shows at the zoom the chart has, centered: that's its stretch.)
  const width = span.day && range ? (range.to - range.from) * step : Math.min(end - span.from, mostBars() * step);
  const from = middle - width / 2, to = Math.min(middle + width / 2, end), times = resolved?.times ?? [];
  // (Loaded up to the end of the last point's own interval.)
  return { from, to, loaded: times.length > 1 && from >= times[0] && to <= /** @type {number} */ (times.at(-1)) + step };
}
/** Why a span can't be gone to on this chart (none of its data there), or nothing. */
function outOfData(span) {
  const times = resolved?.times ?? [];
  if (times.length < 2) return "This chart has no data to go to yet";
  // At a fine interval, all of its history (what isn't loaded loads); at a coarse one, the last point stands for its
  // whole interval (a yearly chart's 2026 takes October). (By the data on screen: see shownIndex.)
  const index = shownIndex(), fine = FINE.has(index), start = fine ? timeAt(index, 0) : times[0];
  const end = timeAt(index, indexAt(index, fine ? Date.now() / 1000 : /** @type {number} */ (times.at(-1))) + 1);
  if (span.to >= start && span.from < end) return "";
  return `This chart runs from ${monthYear(start)} to ${monthYear(end - 1)}`;
}
/**
 * A Go on once the chart's latest draw is on screen (see settled), if it's still the latest Go, on this chart: a task
 * on, after what that draw's caller does once it's drawn (a range asked for, set then).
 */
const afterDrawn = (/** @type {number} */ go, /** @type {() => void} */ then) => {
  const id = chart.id, wake = () => (drawnWaiters.delete(wake), setTimeout(() => chart.id === id && go === dateGoes && then()));
  drawnWaiters.add(wake);
};
/** Where ↵ would take the view: the chart's overview, the span lit on it; when it starts and ends, how many points. */
function previewDate(panel, span) {
  const why = span ? outOfData(span) : "";
  // (At a fine interval, a stretch not loaded yet loads first: nothing to light or count until then.)
  const away = Boolean(span && !why && fineDrawn() && !stretchOf(span).loaded);
  const points = span && !why && !away ? pointsOf(span) : null, wide = points && points.last - points.first + 1 > mostBars();
  panel.innerHTML = `<h4 class="plain">${span ? escapeHtml(capitalized(span.label)) : "Go to a date"}</h4>
          <p class="description">${escapeHtml(!span ? "A year or a month is shown whole; a day, centered at the zoom the chart has." : why || (away ? "Loads first, then shows it." : span.day ? "Centered, at the zoom the chart has." : wide ? "Its middle: all of it takes a longer interval." : "Shown whole, edge to edge."))}</p>
          <svg class="spark" viewBox="0 0 300 100" preserveAspectRatio="none" aria-hidden="true"></svg><span class="spark-caption" hidden></span>
          ${span && !why && !away ? `<div class="info"><dl id="date-facts"></dl></div>` : ""}`;
  const source = overviewSource();
  if (!source || !resolved) return void panel.querySelector(".spark")?.remove();
  const values = Array.from(source.values), n = values.length;
  drawSpark(panel, values, unitFor(source.layer, resolved.results.get(source.layer.id)), ink(source.layer.color));
  if (!span || why || away) return;
  const { first, last } = pointsOf(span), x = (i) => ((i / Math.max(1, n - 1)) * 300).toFixed(1);
  panel.querySelector(".spark")?.insertAdjacentHTML("afterbegin", `<rect x="${x(first)}" y="0" width="${Math.max(1.5, Number(x(last)) - Number(x(first)))}" height="100" style="fill: var(--line-strong)" />`);
  $("date-facts").innerHTML = `<div><dt>From</dt><dd>${escapeHtml(pointLabel(resolved.times[first]))}</dd></div><div><dt>To</dt><dd>${escapeHtml(pointLabel(resolved.times[last]))}</dd></div><div><dt>Points</dt><dd>${grouped.format(last - first + 1)}</dd></div>`;
}
/** Go to a date, typed in ⌘K's field: the span it names lit on the overview; ↵ takes the view there and closes ⌘K. */
const datePage = () => inputPage({
  title: "Go to a date", value: "", placeholder: "2021, 2021-11, 10 nov 2021…", verb: "Go", icon: ICONS.calendar, closes: true,
  hint: (text) => (readDate(text) ? outOfData(/** @type {any} */ (readDate(text))) : "Type a year, a month or a day"),
  read: (text) => {
    const span = readDate(text);
    return span && !outOfData(span) ? `Go to ${span.label}` : null;
  },
  // (Then the navigator has the keys, on from there: ← → ↑ ↓; by touch, none.)
  apply: (text) => (goToDate(readDate(text)), !TOUCH && requestAnimationFrame(() => $("navigator").focus({ preventScroll: true }))),
  preview: (panel, text) => previewDate(panel, readDate(text)),
});
/** ⌘K, straight on a page. @param {Page} page */
function openPage(page) {
  // (Opened on it: ⌘K's own way in, no page turned; open already, the page it shows leaves.)
  const was = isOpen($("palette"));
  if (was) pageLeaves(1);
  openPalette();
  pushPage(page, false);
  if (was) pageComes(1);
}

/** A new chart that starts with a series. */
async function newChartWith(name) {
  newChart();
  await addSeries(name);
}

/** ↵ on the row: what it does; ⇧↵, a series in a new pane; ⌘↵, a series or a formula added with ⌘K left open. */
function choose(newPane = false, keep = false) {
  const item = palette.items[palette.active];
  if (!item) return;
  // (Quick Look put away first: what's done is done to the chart open, not the one looked at.)
  quickHide();
  if (item.recipe) return addFromPalette(item.into, keep, undefined, item);
  if (item.kind === "insert") return insertPiece(item);
  // (⇧↵: in a new pane, as a series is (every pane taken: said, as a series's is); else where its row says.)
  if (item.kind === "formula") {
    if (newPane && !palette.editing && !asSignal(palette.query.trim()) && chart.layers.length && chart.panes.length >= MAX_PANES) return void notice(`A chart holds up to ${MAX_PANES} panes`);
    return addFromPalette(newPane ? "new" : undefined, keep);
  }
  if (item.kind === "folder") {
    palette.path = item.path;
    showPalette();
    return;
  }
  if (item.kind === "row") {
    if (fromSearch() && item.key !== "everywhere") noteUse(item.key);
    // (⇧↵ or ⌘↵ on a row: its action with that key, where it has one.)
    const keyed = newPane || keep ? item.actions?.().find((action) => action.kbd === (newPane ? "⇧↵" : "⌘↵")) : undefined;
    return runAction(keyed ?? item, item);
  }
  if (item.kind === "command") {
    // (One saying how things are: nothing to do, ⌘K closes.)
    if (!item.command.run) return void $("palette").hidePopover();
    if (fromSearch()) noteUse(item.key);
    if (!item.command.keepOpen) $("palette").hidePopover();
    item.command.run();
    return;
  }
  if (item.kind === "chart") {
    $("palette").hidePopover();
    go(chartById(item.key));
    return;
  }
  if (home) {
    $("palette").hidePopover();
    return void newChartWith(item.name);
  }
  const replacing = palette.replacing;
  // (Adding to a new pane, chosen beforehand: each one added goes in one.)
  newPane ||= palette.into === "new";
  if (newPane && !replacing && chart.layers.length && chart.panes.length >= MAX_PANES) return void notice(`A chart holds up to ${MAX_PANES} panes`);
  // Related series sit together: adding another to this chart reopens here, on this one.
  if (!replacing) addedFrom = palette.adding && palette.path.length ? { chart: chart.id, path: palette.path.slice(), key: item.key } : null;
  if (keep && !replacing) {
    // Added, and ⌘K stays on these results: the next one is a ⌘↵ away.
    addSeries(item.name, { newPane, pane: newPane ? null : palette.into, bubbles: palette.bubbles }).then(() => {
      renderPalette();
      paletteKeys();
    });
    return;
  }
  $("palette").hidePopover();
  if (replacing) replaceSeries(replacing, item.name);
  else addSeries(item.name, { newPane, pane: newPane ? null : palette.into, bubbles: palette.bubbles });
}
export {
  aboutOf, actionable, actionsFor, activeOption, addedFrom, afterDrawn, applyScope, AROUND, bothOf, browseInstead,
  browseRows, byUse, catalogTops, chartActions, chartsPage, choicePage, choose, columnsOf, commandItem, commands,
  dangerous, datePage, drawOptions, finder, findSeries, fitScore, formulaAt, formulaError, formulaSide, goesOn, goUp,
  headMenu, heightRow, inputPage, insertPiece, isOwnFormula, keyboard, keyboardKey, keyed, latestSaid, layerRow,
  layerRows, leavePages, leaveScope, listPage, litFirst, loopError, meansOf, nodeAt, openPage, optionsHtml,
  otherLayerRows, outOfData, pageTitle, palette, PALETTE_PAGE, paletteKeys, paletteMenuGone, paneNames, paneRow,
  placeOnHome, plotIn, pointsOf, popPage, presetPicked, preview, previewDate, previewLayer, pushPage, rank, ranksSoon,
  readColor, readDate, recentRow, renameChart, renameChartPage, renderPath, rowText, scalePane, searchCase,
  searchSeries, searchText, seriesPicked, settingChoices, settingsNow, showActions, showFormula, showPage, sizable,
  slipEdits, startFinder, stepSignal, stretchOf, textWords, timesPage, typedWords, underHome, USED, useScore, viewRow,
  wayText,
};
