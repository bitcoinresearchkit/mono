// @ts-check
// A layer's inspector: picking layers, their settings, gradients, adding layers.
import { press, stack as stackColumns } from "/modules/bitview-charts/index.js";
import { capitalized } from "/studio/words.js";
import {
  $, acrossUnfit, afterHands, all$, axisLabel, BELOW, belowOf, byBlock, byTouch, chart, CHEVRON, chooseDrawings,
  chosen, clamp, clearLayerPicks, client, closest, cohortName, colorName, colorPane, commit, css, DASHES, dashGlyph,
  dashOf, DEFAULTS, dismissPalette, EDGE_NAMES, edgeIcon, EDGES, edgesOf, escape, EXAMPLE, fieldUnit, FINE, firstLook,
  fittable, flowFits, follows, format, FORMAT_GROUPS, formatName, FORMATS, formulaFit, fromLetters, fromOf, fromShort,
  fromUnfit, gamma, glyph, goesWith, GRADIENT_LEAST, GRADIENT_MOST, GRADIENT_PRESETS, gradientWidth, grouped,
  handsHeld, HEAT_GRADIENT, heatLike, hexToOklch, holding, hollowOf, home, humanize, iconNames, ICONS, indexCatalog,
  inside, inSlices, INTERVAL_DAYS, INTERVAL_LABEL, INTERVAL_ORDER, isOhlc, isSeriesName, kept, keyText, kindLook,
  layerPicks, layersBatch, layersColors, letterChip, limit, LINE_STYLE, LINED, lowerBound, makeLayer, markable,
  markFor, markPicks, MAX_LEVELS, MAX_PANES, maxChroma, memberable, membersOf, menuOf, minedHalvings,
  modesOf, MOTION, multiEdit, newPaneIn, nextColor, nextLetter, NO_LINE, notice, oklchToHex, oklchToLinear, ONE_SCALE,
  oneScale, openPalette, OUTLINED, ownerOf, PAIRS, paneShowing, phone, placedBy, placedOn, plural, profileColor,
  recent, reduced, refFor, refsFor, remember, removedNames, renderBelt, renderInspector, renderLayers, reshape,
  resolved, secondColor, select, selected, seriesCount, seriesLooked, shapeGlyph, SHAPES, showList, showMenu,
  shownOklch, signedSignal, SLIDERS, SLOP, spanDays, SPANS, STACKED_ONLY, STARTERS, SWATCHES, toast, toggleMember,
  track, tree, TREND_FLIP, TWO_KEPT, TWO_OR_ONE, TYPES, uid, undoToast, UNIT_NAMES, unitOf, VALUES, viewport,
  VIVID_ROWS, vividGrid, vividWork, whenIdle, widest, WIDTHS, windowPoints,
} from "/studio/app.js";
import {
  BY_BLOCK, cssColor, dailyOf, datedIntervals, foldedInto, gone, idOf, info, ink, lastShown, noValues, oklchOf, respot,
  retryInfo, seriesIn, SIDES, unitFor, units, updateLegends, valueAt,
} from "/studio/chart.js";
import {
  bothOf, choicePage, findSeries, formulaError, formulaSide, listPage, loopError, nodeAt, palette, paneNames,
  previewLayer, pushPage,
} from "/studio/command.js";
import { beltSide, drawingById, lookFor, setLook } from "/studio/drawing.js";
import {
  argNames, FUNCTIONS, GENESIS, hasFunction, highlightFormula, parse, PERIODS_TAKEN, refsOf, turnsInto, unfitHere,
} from "/studio/formulas.js";
import { fillSlot, isOpen, recommendedCards } from "/studio/grid.js";
import { HEAT_FADE, heatAmount, heatFelt, heatGrid, heatLooks, heatName, heatPath } from "/studio/heatmaps.js";
import { exportData, scrolled } from "/studio/menus.js";
import { presetsSettled, refocus, slideLayers, tick } from "/studio/presets-edit.js";
/** @import { Chart, Layer, Pane } from "/studio/app.js" */
/** @import { Action, Command, Page } from "/studio/command.js" */
/** @import { Node, Vector } from "/studio/formulas.js" */


/** A stacked area's band, its tint: even, from its line to the one under it (a cloud's bands alike, each its own):
 * the band read as an area, its edges a fine line (STACK_WIDTH). */
const STACK_FILL = 0.35, STACK_WIDTH = 1;
/**
 * An area's fill, its top's opacity and its foot's (before its own opacity): its own (`fillOpacity`, `solid`), else its
 * kind's. Stacked or filled above, even (a band fading by price would read lighter the lower it sits; above a line
 * that runs high, it would all but vanish); else fading to none at its foot, unless solid. Colored by date (from
 * another layer, by its levels), stronger, its foot faintly so. @param {Layer} layer @param {boolean} stacked
 * @returns {[number, number]}
 */
function fillOf(layer, stacked) {
  const regime = Boolean(layer.colorFrom || layer.byLevels), above = layer.fill === "above";
  const top = layer.fillOpacity ?? (stacked || above ? (regime ? 0.3 : above ? 0.1 : STACK_FILL) : regime ? 0.5 : 0.2);
  return [top, stacked || above || layer.solid ? top : regime ? top * 0.08 : 0];
}
/** Whether a layer's pane stacks it. @param {Layer} layer */
const stackedIn = (layer, doc = chart) => Boolean(doc.panes.find((pane) => pane.id === layer.pane)?.stack);
/** The open chart's stacks as last made, and what they were made of (see stackOf). */
let stacksMade = { key: "", stacked: /** @type {Map<string, { values: Float64Array, foot: Float64Array, scale: Float64Array | null, group: string, rank: number }>} */ (new Map()), times: /** @type {unknown} */ (null) };
/**
 * What stacked panes draw: each layer shown on a side at its own value plus those of the layers before it in the
 * stack (the panel's order, its first at the foot: A, A+B, A+B+C; reversed, its last); in a 100% one, that as a
 * share of them all. Each is drawn as it is (its type, its look): only its values change. A layer without a value at
 * a point adds none there and shows none (it starts where its data does); in a 100% stack, a negative one counts as
 * none (a share of a whole). The library's stack makes them: `foot`, the stack under it (a share, in a 100% one),
 * `scale` the factor a 100% one's sums took, for its own values there (a candle's open, high, low and close);
 * `group` names its stack (its pane and side), `rank` its place there (its foot 0).
 * @param {Chart} doc @param {{ results: Map<string, any>, times?: number[] }} data
 * @returns {Map<string, { values: Float64Array, foot: Float64Array, scale: Float64Array | null, group: string, rank: number }>} by layer
 */
function stackOf(doc, data, kept = true) {
  // The open chart's, kept while what they're made of is the same (the stacked panes, their layers, places and values):
  // an edit elsewhere (a color, a width) finds the same totals, and their series aren't set again. (Its marks' own,
  // from other data, aren't kept: `kept` false.)
  const key = kept && doc.id === chart?.id ? doc.panes.filter((pane) => pane.stack).map((pane) => `${pane.id}:${pane.stack}:${pane.reverse ? 1 : 0}:${doc.layers.filter((layer) => layer.pane === pane.id).map((layer) => {
    const shown = data.results.get(layer.id)?.values;
    return `${layer.id}.${layer.side}.${layer.hidden ? 0 : 1}.${layer.from ?? ""}.${placedOn(layer) ? 1 : 0}.${shown ? idOf(shown) : "-"}`;
  }).join()}`).join("|") : "";
  if (key && key === stacksMade.key) return stacksMade.stacked;
  const stacked = new Map();
  for (const pane of doc.panes) {
    if (!pane.stack) continue;
    const shares = pane.stack === "percent";
    for (const side of SIDES) {
      const layers = doc.layers.filter((layer) => layer.pane === pane.id && layer.side === side && !layer.hidden && !layer.from && !placedOn(layer) && data.results.get(layer.id)?.values);
      if (!layers.length) continue;
      const group = `${pane.id}:${side}`, ordered = pane.reverse ? [...layers].reverse() : layers;
      const { tops, feet, scale } = stackColumns(ordered.map((layer) => data.results.get(layer.id).values), { percent: shares });
      ordered.forEach((layer, rank) => stacked.set(layer.id, { values: tops[rank], foot: feet[rank], scale, group, rank }));
    }
  }
  if (!key) return stacked;
  // (One that comes out as it was (a layer over it hidden, shown, another's moved), on the same dates, the one it
  // was: its series not set again, see build.)
  for (const [id, item] of stacksMade.times === data.times ? stacked : []) {
    const was = stacksMade.stacked.get(id);
    if (was && was.group === item.group && was.rank === item.rank && sameVector(was.values, item.values) && sameVector(was.foot, item.foot) && (was.scale === item.scale || (was.scale && item.scale && sameVector(was.scale, item.scale)))) stacked.set(id, was);
  }
  stacksMade = { key, stacked, times: data.times };
  return stacked;
}
/** Whether two runs of values are the same, a missing one where the other's is (NaN as NaN). @param {ArrayLike<number>} a @param {ArrayLike<number>} b */
function sameVector(a, b) {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) if (a[i] !== b[i] && (a[i] === a[i] || b[i] === b[i])) return false;
  return true;
}
/** The scale of the pane a layer is on. @param {Chart} doc */
const modeOf = (doc, layer) => paneMode(doc.panes.find((item) => item.id === layer.pane));
/**
 * A pane's scale as drawn: a stack's is linear or log (percent change and indexed would start each band from its
 * own first value), a 100% stack's linear (shares of a whole), whatever the pane keeps for later. @param {Pane} [pane]
 */
const paneMode = (pane) => (!pane ? 0 : pane.stack === "percent" || (pane.stack && pane.mode > 1) ? 0 : pane.mode);
/** The scales a pane can take now (a stack: linear and log; a 100% one: none to choose). @param {Pane} pane */
const modesFor = (pane) => (pane.stack === "percent" ? [] : pane.stack ? [0, 1] : [0, 1, 2, 3]);
const labelOf = (layer) => layer.label || (layer.series ? humanize(layer.series) : layer.from ? fromName(layer) : heatLike(layer) ? heatName(layer) : layer.formula || "Formula");
/**
 * A layer made from layers, named, unnamed: what it is, then its layers' letters (three at most; past that, how
 * many), then a cloud's edge (both: the pane). @param {{ from?: string[], type?: string }} layer @param {Chart | null} [doc]
 */
const fromName = (layer, doc = ownerOf(/** @type {Layer} */ (layer))) => ((noun, letters, edges) => (edges.length > 1 && !letters.length ? `${noun} of the pane` : `${!letters.length ? noun : letters.length > 3 ? `${noun} of ${letters.length} layers` : `${noun} of ${letters.join(", ")}`}${edges.length ? ` to the ${edges.join(" and ")}` : ""}`))(layer.type === "trend" ? "Trend" : "Cloud", fromLetters(layer, doc), edgesOf(layer));
const layerById = (id) => chart.layers.find((layer) => layer.id === id);

/**
 * A view's content written anew, scrolled as it was: Safari on iOS sends some views whose content is replaced back
 * to their top (a layer's settings, at each edit, on iOS 26.5).
 * @param {HTMLElement} container @param {string} html
 */
function rewrite(container, html) {
  // (Never scrolled (see scrolled), at its top: nothing read, nothing put back. Either lays the page out at once,
  // the Charts page's whole grid with it while a chart opens over it.)
  const top = scrolled.has(container) ? container.scrollTop : 0;
  container.innerHTML = html;
  if (top) container.scrollTop = top;
}
/** Rebuilding a panel's HTML would drop focus: these find the same control again afterwards. */
function focusKey(container) {
  const active = document.activeElement;
  return active instanceof HTMLElement && container.contains(active) ? keyOf(active) : null;
}
/**
 * What finds an element's like once it's drawn again: its id, or its tag and data (not what marks it for a moment:
 * a tip, a key; a legend item named (pointed), a profile's group lit). @param {HTMLElement} element
 */
const keyOf = (element) => ({ tag: element.tagName, id: element.id, data: Object.entries(element.dataset).filter(([name]) => !["tip", "kbd", "pointed", "lit"].includes(name)) });
function restoreFocus(container, key) {
  if (!key || container.contains(document.activeElement)) return;
  const find = () => (key.id ? container.querySelector(`#${CSS.escape(key.id)}`) : [...container.querySelectorAll(key.tag)].find((element) => element instanceof HTMLElement && key.data.length > 0 && key.data.every(([name, value]) => element.dataset[name] === value)));
  let match = find();
  // (Its like on a preset's tile written as a slot yet: the tile written for it, see fillSlot.)
  const preset = match ? undefined : key.data.find(([name]) => name === "openPreset" || name === "presetMenu")?.[1], slot = preset === undefined ? null : container.querySelector(`.tile-slot[data-slot="${CSS.escape(preset)}"]`);
  if (slot) (fillSlot(/** @type {HTMLElement} */ (slot)), (match = find()));
  /** @type {HTMLElement | null | undefined} */ (match)?.focus();
}
/**
 * The focus, where what had it goes (`away`: put away, the panel closed, the sidebar slid off; else removed, once
 * the page is drawn again), on the first of `next` that shows (a row's successor, the button that brings it back):
 * never lost to the page. A focus nowhere (after a click on the chart) stays nowhere.
 * @param {Element | null} away @param {...(() => Element | null | undefined)} next
 */
function keepFocus(away, ...next) {
  const had = document.activeElement;
  if (!had || had === document.body) return;
  const go = () => {
    for (const pick of next) {
      const element = pick();
      if (element instanceof HTMLElement && element.isConnected && !away?.contains(element) && element.checkVisibility({ checkVisibilityCSS: true })) return (element.focus({ preventScroll: true }), true);
    }
    return false;
  };
  if (away?.contains(had)) return void go();
  // (Gone: once the page is drawn again; what takes its place still coming in, once it's in.)
  const lost = () => !had.isConnected || document.activeElement === document.body || !had.checkVisibility();
  requestAnimationFrame(() => lost() && !go() && setTimeout(() => lost() && go(), MOTION.page + 40));
}
/** A pane's layers in the panel's list (its legend's chip pressed): the list shown, that pane's at its top. */
function paneLayers(/** @type {string} */ id) {
  showList();
  // (A focus on its legend, hidden under the list now: to that pane's first row.)
  keepFocus($("chart"), () => $("layer-list").querySelector(`.pane-group[data-pane="${CSS.escape(id)}"] [data-select]`));
  requestAnimationFrame(() => /** @type {HTMLElement | null} */ ($("layer-list").querySelector(`.pane-group[data-pane="${CSS.escape(id)}"]`))?.scrollIntoView({ block: "start", behavior: reduced.matches ? "auto" : "smooth" }));
}
/** The picks shown: the rows, the legends, the bar (drawings let go: one selection at a time). */
function showPicks() {
  if (layerPicks.size) {
    showList();
    if (chosen.size) chooseDrawings([]);
  }
  markPicks();
}
const pickedLayers = () => chart.layers.filter((layer) => layerPicks.has(layer.id));
/** The bar over the list's foot: how many are picked, and what can be done to them. */
function renderLayersBar() {
  const layers = pickedLayers(), hide = layers.some((layer) => !layer.hidden), bar = $("layers-bar");
  bar.hidden = !layers.length;
  document.body.toggleAttribute("data-layers-picked", layers.length > 0);
  if (!layers.length) return;
  $("layers-picked").textContent = plural(layers.length, "layer");
  const toggle = /** @type {HTMLElement} */ (bar.querySelector('[data-layers-batch="hide"]'));
  toggle.innerHTML = `${hide ? ICONS.eyeOff : ICONS.eye}<span class="label">${hide ? "Hide" : "Show"}</span>`;
  toggle.setAttribute("aria-label", hide ? "Hide" : "Show");
  toggle.dataset.tip = hide ? "Hide" : "Show";
  // (A cloud of two or more, on one scale: else a tap says why.)
  const cloud = /** @type {HTMLElement} */ (bar.querySelector('[data-layers-batch="cloud"]')), bases = layers.filter((layer) => !layer.from && layer.type !== "signal"), apart = bases.length >= 2 && fromShort({ type: "cloud", from: bases.map((layer) => layer.id) });
  cloud.setAttribute("aria-disabled", String(bases.length < 2 || Boolean(apart)));
  cloud.dataset.tip = bases.length < 2 ? "A cloud is between two layers or more" : apart || "A cloud between them";
}
/** Hides layers, in one step; all hidden already, shows them. @param {string[]} ids */
function hideLayers(ids) {
  const hide = chart.layers.some((layer) => ids.includes(layer.id) && !layer.hidden);
  commit((doc) => doc.layers.forEach((layer) => ids.includes(layer.id) && (layer.hidden = hide || undefined)), { look: true });
}
/** A layer alone in its pane (its signals with it): shown, the others there hidden; alone already, every one there shown again. */
function soloLayer(/** @type {string} */ id) {
  const layer = layerById(id);
  if (!layer) return;
  // (The only one in its pane, or a signal (alone, its layer hidden, it shows nothing): shown or hidden, as a click
  // does.)
  if (layer.type === "signal" || !othersInPane(layer)) return toggleHidden(id);
  const alone = aloneInPane(layer);
  commit((doc) => doc.layers.forEach((other) => other.pane === layer.pane && (other.hidden = alone || itsOwn(other, layer) ? undefined : true)), { look: true });
}
/** Whether a layer is the only one shown in its pane, its signals with it (beside others hidden). */
const aloneInPane = (/** @type {Layer} */ layer) => !layer.hidden && othersInPane(layer) && chart.layers.every((other) => other.pane !== layer.pane || itsOwn(other, layer) || other.hidden);
/** Whether a layer's pane holds others than it and its signals. @param {Layer} layer */
const othersInPane = (layer) => chart.layers.some((other) => other.pane === layer.pane && !itsOwn(other, layer));
/** A layer, or a signal marked on it: shown alone with it. @param {Layer} other @param {Layer} layer */
const itsOwn = (other, layer) => other.id === layer.id || (other.type === "signal" && other.ref === layer.id) || foldedInto(chart).get(other.id) === layer.id;
/**
 * Of layers to move to a pane (`into`; null: a new one), those that move themselves: a cloud or a trend goes where
 * its layers go, a signal where its layer goes (in its own pane, it's put in its place as any). One asked to without
 * them, said.
 */
function movers(/** @type {string[]} */ ids, /** @type {string | null} */ into) {
  const going = ids.filter((id) => ((layer) => !follows(layer) || layer?.pane === into)(layerById(id)));
  const left = ids.map(layerById).find((layer) => layer && !going.includes(layer.id) && goesWith(layer).some((id) => !ids.includes(id)));
  const its = left?.type === "signal" ? ["its layer", "goes", "it"] : ["its layers", "go", "them"];
  if (left) notice(going.length ? `${labelOf(left)} stays with ${its[0]}` : `${labelOf(left)} goes where ${its[0]} ${its[1]}: move ${its[2]}`);
  return going;
}
/** Moves layers to a pane (`into`), or to a new one, at its end in their order. @param {string[]} asked @param {string | null} into */
function moveLayers(asked, into) {
  const ids = movers(asked, into);
  if (!ids.length) return;
  commit((doc) => {
    const pane = into ? doc.panes.find((item) => item.id === into) : newPaneIn(doc);
    if (!pane) return;
    // (Those that go where these go (bubbles on one, a cloud between them), along with them, where they were among
    // them: what's drawn over what, as it was.)
    const along = new Set(ids);
    // (A rider of a rider too, wherever it's listed: gathered till none's left.)
    for (let more = true; more; ) {
      more = false;
      for (const layer of doc.layers) if (!along.has(layer.id) && follows(layer) && goesWith(layer).every((id) => along.has(id))) (along.add(layer.id), (more = true));
    }
    const moving = doc.layers.filter((layer) => along.has(layer.id));
    for (const layer of moving) layer.pane = pane.id;
    doc.layers = [...doc.layers.filter((layer) => !along.has(layer.id)), ...moving];
  });
}
/**
 * Where layers can go: each pane (theirs ticked, when they're in one), or a new one; moved there at its end, in their
 * order (`after`: then, the focus back where it was).
 * @param {HTMLElement} anchor @param {string[]} ids
 */
function paneMenu(anchor, ids, after = () => {}) {
  menuOf(anchor, paneChoices(ids, after));
}
/**
 * Where layers go, as choices: each pane (theirs ticked), a new one.
 * @param {string[]} ids
 * @returns {Action[]}
 */
function paneChoices(ids, after = () => {}) {
  const inPanes = new Set(chart.layers.filter((layer) => ids.includes(layer.id)).map((layer) => layer.pane)), theirs = (/** @type {string} */ id) => inPanes.size === 1 && inPanes.has(id);
  return [
    ...chart.panes.map((pane, i) => ({ key: pane.id, icon: ICONS.pane, label: `Pane ${i + 1}`, aside: paneNames(pane), current: theirs(pane.id), run: () => (theirs(pane.id) || moveLayers(ids, pane.id), after()) })),
    { key: "new", icon: ICONS.plus, label: "New pane", apart: true, ...(chart.panes.length < MAX_PANES ? {} : { disabled: `A chart holds up to ${MAX_PANES} panes` }), run: () => (moveLayers(ids, null), after()) },
  ];
}
/** The picked rows' menu, for a right-click on one: as the bar. @param {HTMLElement} anchor */
function layersMenu(anchor, point) {
  menuOf(anchor, layersActions(), point);
}
/** What the picked layers do together (a right-click on one of them; ⌘K's commands, see layersCommands). @returns {Action[]} */
function layersActions() {
  const layers = pickedLayers(), n = plural(layers.length, "layer"), hide = layers.some((layer) => !layer.hidden);
  const bases = layers.filter((layer) => !layer.from && layer.type !== "signal"), ids = bases.map((layer) => layer.id);
  return [
    { key: "hide", icon: hide ? ICONS.eyeOff : ICONS.eye, label: `${hide ? "Hide" : "Show"} ${n}`, text: layers.map(labelOf).join(", "), kbd: "H", run: () => layersBatch("hide") },
    { key: "duplicate", icon: ICONS.duplicate, label: `Duplicate ${n}`, text: "Each copy under its original.", kbd: "⌘D", run: () => layersBatch("duplicate") },
    { key: "move", icon: ICONS.pane, label: "Move to pane", text: "One of the others, or a new one.", page: () => choicePage("Move to pane", () => paneChoices(layers.map((layer) => layer.id))) },
    { key: "cloud", icon: ICONS.cloud, label: "A cloud between them", text: "A fill between each pair of neighbors, in the upper one's color.", ...(bases.length < 2 ? { disabled: "Needs two or more layers that aren't clouds, trends or signals" } : fromShort({ type: "cloud", from: ids }) ? { disabled: ONE_SCALE } : {}), run: () => layersBatch("cloud") },
    { key: "trend", icon: ICONS.riding, label: "A trend between them", text: "Follows the lowest until a price touches it, then the highest until the price touches that, and so on.", ...(bases.length < 2 ? { disabled: "Needs two or more layers that aren't clouds, trends or signals" } : ((why) => (why ? { disabled: why } : {}))(fromShort({ type: "trend", from: ids }))), run: () => layersBatch("trend") },
    { key: "remove", icon: ICONS.trash, label: `Remove ${n}`, text: "You can undo.", kbd: "⌫", danger: true, run: () => layersBatch("remove") },
  ];
}
/** The picked layers' actions as ⌘K's commands (one that can't act says why, found, not run), and the selection let go of. @returns {Command[]} */
function layersCommands() {
  if (!layerPicks.size) return [];
  const commands = layersActions().map((row) => ({ title: row.label, text: row.disabled ? `${row.disabled}.` : row.text ?? "", icon: /** @type {string} */ (iconNames.get(row.icon ?? "")), kbd: row.kbd, danger: row.danger, suggested: true, ...(row.disabled ? {} : row.page ? { keepOpen: true, run: () => pushPage(/** @type {() => Page} */ (row.page)()) } : { run: row.run }) }));
  return [...commands, { title: "Clear the selection", text: `${plural(pickedLayers().length, "layer")} selected.`, icon: "close", kbd: "esc", run: () => clearLayerPicks(), suggested: true }];
}

/** On a phone, a layer just added: its settings a tap away (the chart, meanwhile, in sight). @param {Layer | undefined} layer */
const editAdded = (layer) => (phone.matches && layer ? { label: "Edit", run: () => {
  if (isOpen($("palette"))) dismissPalette();
  if (layerById(layer.id)) select(layer.id);
} } : undefined);

/** What shows which layer's selected: the list, the settings, the legends, the belt (nothing else drawn again). */
function showSelection() {
  renderLayers();
  renderInspector();
  // (The one open, in the spotlight while the panel shows it.)
  respot();
  updateLegends();
  renderBelt();
}
const editingMany = () => multiEdit && !selected && pickedLayers().length > 1;
/** The layers the inspector edits: the one selected, or those picked, edited together (see multiEdit). */
const inHand = () => (editingMany() ? pickedLayers() : [layerById(selected)].filter((layer) => layer !== undefined));
/** What every one of some layers has, by `of`: that, or undefined where they differ (said "Mixed"). @template T @param {Layer[]} layers @param {(layer: Layer) => T} of */
const sharedBy = (layers, of) => (layers.length && layers.every((layer) => of(layer) === of(layers[0])) ? of(layers[0]) : undefined);
/** The fields a layer's panel has by index, their count: its levels', its gradient's colors'. @param {Layer} layer */
const fieldsShape = (layer) => `${layer.levels?.length ?? 0}:${layer.byGradient ? (layer.gradient?.length ?? 0) : 0}`;
/**
 * A layer's actions, at the inspector's foot: as the layers panel's adds, a row each (each by its key's id: the focus
 * back on it once redrawn). @param {Layer} layer
 */
/** The layer's eye, by its name in its settings. */
const eyeOf = (/** @type {Layer} */ layer) => `<button type="button" id="inspector-eye" data-hidden="${Boolean(layer.hidden)}" class="cap small" data-toggle="${layer.id}" aria-label="${layer.hidden ? "Show" : "Hide"} ${escape(labelOf(layer))}" data-tip="${layer.hidden ? "Show" : "Hide"}${layer.type === "signal" ? "" : `\n${keyText("⌥-click: alone in its pane")}`}" data-kbd="H">${layer.hidden ? ICONS.eyeOff : ICONS.eye}</button>`;
const actionsOf = (/** @type {Layer} */ layer) => `<section class="layer-actions">${layerActions(layer.id, "panel").map((row, i) => `<button type="button" id="act-${row.key}" class="nav-item${row.danger ? " danger" : ""}" data-act="${i}"${row.page ? ' aria-haspopup="menu"' : ""}${row.disabled ? ` aria-disabled="true" data-tip="${escape(row.disabled)}"` : ""}${row.kbd?.startsWith("⌘") ? ` data-kbd="${[...row.kbd].join(" ")}"` : ""}>${row.icon ?? ""}<span class="label">${escape(row.label)}</span>${row.kbd ? `<kbd${row.kbd.startsWith("⌘") ? ' class="mod"' : ""}>${escape(keyText(row.kbd))}</kbd>` : ""}</button>`).join("")}</section>`;
/** The layer a layer's on (a signal's) or flipped by (a trend's), as its picker shows it. @param {Layer} layer */
const refPicked = (layer) => ((ref) => (ref ? `${letterChip(ref)}<span class="name">${escape(labelOf(ref))}</span>` : `<span class="name">Pick a layer</span>`))(layerById(layer.ref ?? ""));

/**
 * The inspector's fields for the layers in hand (see inHand: one, or those picked, edited together): each shown where
 * it applies to every one of them, its choice pressed where they all have it, none where they differ ("Mixed").
 * One layer's fields are these with it alone.
 */
const typeField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every(seriesLooked)) return "";
  const now = sharedBy(layers, (layer) => layer.type), ohlc = layers.every((layer) => resolved?.results.get(layer.id)?.opens || (layer.series && isOhlc(layer.series)));
  return `<div class="field"><span>Type<span class="color-now">${escape(now === undefined ? "Mixed" : TYPES.find(([type]) => type === now)?.[1] ?? "")}</span></span><div class="tabs glyphs" role="group" aria-label="Chart type">${TYPES.map(([type, text]) => `<button type="button" data-type="${type}" aria-pressed="${now === type}" aria-label="${text}" ${type === "candles" && !ohlc ? "aria-disabled=\"true\" data-tip=\"Candles: only for open-high-low-close series\"" : `data-tip="${text}"`}>${glyph(type)}<span>${text}</span></button>`).join("")}</div></div>`;
};
/**
 * Several layers' color, as one's picker is (colorPicker: Palette, or Custom, its surface and its hex): where each is
 * one color all along, the one they all have shown ("Mixed" where they differ: the surface starts at the first's),
 * picked for all of them; else none (a way of coloring of their own, which one color can't say).
 */
const colorField = (/** @type {Layer[]} */ layers) => {
  // (A profile's colors are its groups', each its own: set in its own settings.)
  if (!layers.every((layer) => colorModeOf(layer) === "one" && layer.type !== "profile")) return "";
  const now = sharedBy(layers, (layer) => layer.color), last = colorSource.get(`${chart.id}:${layers.map((layer) => layer.id).join()}:color`);
  const from = last && last.color === now ? last.source : now === undefined || SWATCHES.includes(now) ? "presets" : "custom";
  const tabs = colorSourceTabs("Color", [["presets", "Palette"], ["custom", "Custom"]], from);
  const body = from === "custom" ? `<div class="color-box">${planeHtml(now ?? layers[0].color, "layer:color", "Color")}${hexField("layer:color", now ?? "")}</div>` : swatchGrid(now && SWATCHES.includes(now) ? now : "");
  return `<div class="field"><span>Color<span class="color-now">${escape(now === undefined ? "Mixed" : colorName(now))}</span></span></div><div class="color-editor" data-palette="color">${tabs}<div class="color-pick">${body}</div></div>`;
};
/** Where a color's from, as tabs: `label`'s (the color's name), each source by its name, `from` the one shown. @param {string} label @param {[string, string][]} sources @param {string} from */
const colorSourceTabs = (label, sources, from) => `<div class="tabs color-source" role="group" aria-label="${escape(label)}, its color from">${sources.map(([source, name]) => `<button type="button" data-color-source="${source}" aria-pressed="${source === from}">${name}</button>`).join("")}</div>`;
const opacityField = (/** @type {Layer[]} */ layers) => `<div class="field"><span>Opacity${resetFor("opacity", layers)}</span>${slider("opacity", layers)}</div>`;
const widthField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every((layer) => layer.type !== "histogram" && layer.type !== "cloud" && !placedOn(layer) && !heatLike(layer))) return "";
  const type = sharedBy(layers, (layer) => layer.type);
  return `<div class="field"><span>Width${resetFor("width", layers)}</span>${slider("width", layers, type === "candles" ? "The line candles turn into, zoomed out" : type === "dots" ? "Dot size, which also follows the zoom" : "")}</div>`;
};
const lineField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every((layer) => LINED.has(layer.type))) return "";
  const dash = sharedBy(layers, (layer) => layer.dash), shape = sharedBy(layers, (layer) => layer.shape ?? "straight");
  return `<div class="field"><span>Line</span><div class="pair">
              <div class="tabs glyphs" role="group" aria-label="Line style" style="flex: ${DASHES.length}">${DASHES.map(([value, text, length, gap]) => `<button type="button" data-dash="${value}" aria-pressed="${dash === value}" aria-label="${text}" data-tip="${text}">${dashGlyph(length, gap)}</button>`).join("")}</div>
              ${layers.some((layer) => layer.type === "trend") ? "" : `<div class="tabs glyphs" role="group" aria-label="Line shape" style="flex: ${SHAPES.length}">${SHAPES.map(([value, text]) => `<button type="button" data-shape="${value}" aria-pressed="${shape === value}" aria-label="${text}" data-tip="${value === "steps" ? "Steps: each value held until the next" : text}">${shapeGlyph(value)}</button>`).join("")}</div>`}
            </div></div>`;
};
const fillField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every((layer) => layer.type === "area")) return "";
  const now = sharedBy(layers, (layer) => layer.fill ?? "below"), solid = sharedBy(layers, (layer) => Boolean(layer.solid));
  // (Its style where it fades: filled above or stacked, it's even already.)
  const fades = layers.every((layer) => layer.fill !== "above" && !stackedIn(layer));
  return `<div class="field"><span>Fill</span><div class="tabs" role="group" aria-label="Fill">${[["below", "Below", "Filled from the line down"], ["above", "Above", "Filled from the line up"]].map(([fill, text, tip]) => `<button type="button" data-fill="${fill}" aria-pressed="${now === fill}" data-tip="${tip}">${text}</button>`).join("")}</div></div>
          <div class="field"><span>Fill opacity${resetFor("fill-opacity", layers)}</span>${slider("fill-opacity", layers)}</div>
          ${fades ? `<div class="field"><span>Fill style</span><div class="tabs" role="group" aria-label="Fill style">${[[false, "Gradient", "Fading to nothing at its foot"], [true, "Solid", "Even all the way down"]].map(([on, text, tip]) => `<button type="button" data-solid="${on ? 1 : 0}" aria-pressed="${solid === on}" data-tip="${tip}">${text}</button>`).join("")}</div></div>` : ""}`;
};
/** Candles' bodies: hollow (when rising, filled when falling, as usual) or all filled. */
const candlesField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every((layer) => layer.type === "candles")) return "";
  const now = sharedBy(layers, hollowOf);
  return `<div class="field"><span>Candles</span><div class="tabs" role="group" aria-label="Candles">${[[true, "Hollow", "Hollow when rising, filled when falling"], [false, "Filled", "Filled, rising or falling"]].map(([on, text, tip]) => `<button type="button" data-hollow="${on ? 1 : 0}" aria-pressed="${now === on}" data-tip="${tip}">${text}</button>`).join("")}</div></div>`;
};
const priceLineField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every((layer) => layer.type !== "cloud" && !placedOn(layer) && !heatLike(layer))) return "";
  const now = sharedBy(layers, (layer) => Boolean(layer.priceLine)), stacked = layers.some((layer) => layer.priceLine && chart.panes.find((pane) => pane.id === layer.pane)?.stack);
  return `<div class="field switch"><span>Price line</span><div class="tabs" role="group" aria-label="Price line">${[[false, "Off", "No line"], [true, "On", "A line across the chart at its latest value"]].map(([on, text, tip]) => `<button type="button" data-price-line="${on ? 1 : 0}" aria-pressed="${now === on}" data-tip="${tip}">${text}</button>`).join("")}</div></div>${stacked ? `<p class="from-status" role="status">Not drawn in a stack.</p>` : ""}`;
};
const outlineField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every((layer) => OUTLINED.has(layer.type))) return "";
  const now = sharedBy(layers, (layer) => Boolean(layer.outline));
  return `<div class="field switch"><span>Outline</span><div class="tabs" role="group" aria-label="Outline">${[[false, "Off", "No edge"], [true, "On", "Edged in the chart's background, to stand out over what's under it"]].map(([on, text, tip]) => `<button type="button" data-outline="${on ? 1 : 0}" aria-pressed="${now === on}" data-tip="${tip}">${text}</button>`).join("")}</div></div>`;
};
/**
 * Bubbles' section in the inspector: the layer they're on (a menu of those that can carry them), their largest's
 * size, what's too small to draw, what they're sized against. @param {Layer} layer
 */
const bubbleSection = (layer) => `<section>
              <h3>Bubbles</h3>
              <div class="field"><span>On</span><button type="button" class="preset" id="bubble-ref" aria-haspopup="menu" data-tip="The layer they're on">${((ref) => (ref ? `${letterChip(ref)}<span class="name">${escape(labelOf(ref))}</span>` : `<span class="name">Its pane's middle</span>`))(layerById(layer.ref ?? ""))}${CHEVRON}</button></div>
              <div class="field"><span>Size${resetFor("bubble-size", layer)}</span>${slider("bubble-size", layer, "The largest's size across")}</div>
              <div class="field"><span>Hide below${resetFor("bubble-hide", layer)}</span>${slider("bubble-hide", layer, "None drawn under this share of the largest")}</div>
              <div class="field"><span>Sized to</span><div class="tabs" role="group" aria-label="Sized to">${[["", "In view", "The largest in view full size: zoomed in, as readable"], ["all", "All time", "The largest of all its history full size: sizes compare across periods"]].map(([value, text, says]) => `<button type="button" data-bubbles-scale="${value}" aria-pressed="${(layer.scaleTo ?? "") === value}" data-tip="${says}">${text}</button>`).join("")}</div></div>
            </section>`;
const paneField = (/** @type {Layer[]} */ layers) => {
  if (layers.some(follows)) return "";
  const at = sharedBy(layers, (layer) => chart.panes.findIndex((pane) => pane.id === layer.pane)), name = at === undefined ? "Mixed" : `Pane ${at + 1}`;
  return `<div class="field"><span>Pane</span><button type="button" id="pane" class="preset" data-action="pane-menu" aria-haspopup="menu" aria-label="Pane: ${at === undefined ? "mixed" : at + 1}"><span class="name">${name}</span>${CHEVRON}</button></div>`;
};
const FITS = [["on", "On", "The axis takes in its values (a running total alone on a linear axis, from zero)"], ["zero", "From zero", "A linear axis includes zero and its values"], ["off", "Off", "The axis framed without it: drawn where it is, past the edge if it's far"]];
const fitChoices = (/** @type {Layer[]} */ layers) => FITS.filter(([key]) => key !== "zero" || layers.every((layer) => modeOf(chart, layer) === 0));
const fitOf = (/** @type {Layer} */ layer) => layer.fit === false ? "off" : layer.fit === "zero" && modeOf(chart, layer) === 0 ? "zero" : "on";
const setFit = (/** @type {Layer} */ layer, /** @type {string} */ fit) => { if (fit === "on") delete layer.fit; else layer.fit = fit === "zero" ? "zero" : false; };
const fitField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every(fittable)) return "";
  const now = sharedBy(layers, fitOf);
  return `<div class="field switch"><span>Fit the axis</span><div class="tabs" role="group" aria-label="Fit the axis">${fitChoices(layers).map(([on, text, tip]) => `<button type="button" data-fit="${on}" aria-pressed="${now === on}" data-tip="${tip}">${text}</button>`).join("")}</div></div>`;
};
const axisField = (/** @type {Layer[]} */ layers) => {
  if (layers.some(follows)) return "";
  const now = sharedBy(layers, (layer) => layer.side);
  return `<div class="field"><span>Axis</span><div class="tabs" role="group" aria-label="Axis side">${[["left", "Left"], ["right", "Right"]].map(([side, text]) => `<button type="button" data-side="${side}" aria-pressed="${now === side}">${text}</button>`).join("")}</div></div>`;
};
const resetFor = (key, /** @type {Layer | Layer[]} */ target) => {
  const layers = [target].flat(), spec = DEFAULTS[key];
  if (layers.every(spec.is)) return "";
  // (Several, their defaults apart (an area's opacity isn't a line's): to each one's.)
  const text = sharedBy(layers, (layer) => (typeof spec.text === "function" ? spec.text(layer) : spec.text)) ?? "each one's default";
  return `<button type="button" class="reset" data-reset="${key}" aria-label="Reset to ${text}" data-tip="Reset to ${text}">${ICONS.undo}</button>`;
};
/** The inspector's sliders gliding, by id: until when. */
const glidesUntil = new Map();
/** Where a slider's handle sits for a layer, from 0 to 1 (its stop's place among its stops). */
const sliderAt = (/** @type {keyof typeof SLIDERS} */ id, /** @type {Layer} */ layer) => Math.max(0, SLIDERS[id].stops.indexOf(SLIDERS[id].of(layer))) / (SLIDERS[id].stops.length - 1);
const sliderTrack = (/** @type {HTMLElement} */ element) => /** @type {HTMLElement} */ (element.firstElementChild);
/** A slider: its stops, a handle at the value, and the value written out at the end away from the handle. */
function slider(id, /** @type {any} */ target, tip = "") {
  // (Several layers at different stops: "Mixed", no handle, no fill, till one's set for all.)
  const spec = SLIDERS[id], list = [target].flat(), value = sharedBy(list, spec.of), mixed = value === undefined, index = spec.stops.indexOf(value), n = spec.stops.length - 1;
  const at = mixed ? 0 : sliderAt(id, list[0]);
  return `<div id="${id}" class="stepper${mixed ? " mixed" : ""}" role="slider" tabindex="0" aria-label="${spec.label}" aria-valuemin="0" aria-valuemax="${n}" aria-valuenow="${Math.max(0, index)}" aria-valuetext="${mixed ? "Mixed" : escape(spec.say(value))}" data-value-side="${at > 0.5 ? "start" : "end"}" style="--at: ${at}" ${tip ? `data-tip="${escape(tip)}"` : ""}>
          <span class="stepper-track">${spec.stops.map((_, i) => `<i ${(at > 0.5 ? i / n <= 0.25 : i / n >= 0.75) ? 'class="under"' : ""} style="--k: ${i / n}"></i>`).join("")}<span class="value">${mixed ? "Mixed" : escape(spec.text(value))}</span></span></div>`;
}
/** A value, as a slider sets it: one undo step per drag or run of keys, the chart rebuilt at each stop. */
function setSlider(id, value) {
  const layers = inHand();
  if (!layers.length || layers.every((layer) => SLIDERS[id].of(layer) === value)) return;
  editLayer((target) => SLIDERS[id].set(target, value), { coalesce: `${id}:${layers.map((layer) => layer.id).join()}` });
  // (Dragged, the page isn't drawn again till it ends: the slider shows its new value itself, as the belt's do.)
  const element = $(id), now = inHand();
  if (holding && element && now.length) {
    element.outerHTML = slider(id, now, element.dataset.tip ?? "");
    $(id)?.focus({ preventScroll: true });
  }
}
/** The stop nearest a point across a slider (found again each time: the inspector redraws under the drag). */
function stopAt(id, clientX) {
  const element = $(id);
  if (!element) return undefined;
  const box = element.getBoundingClientRect(), stops = SLIDERS[id].stops;
  if (!box.width) return undefined;
  return stops[clamp(Math.round(((clientX - box.left - 3) / (box.width - 6)) * (stops.length - 1)), 0, stops.length - 1)];
}
/**
 * A press that picks by `move`, as `track` has it; by touch (in what scrolls, a sheet), as a finger on a page means:
 * a tap picks where it's lifted, a drag clearly across picks along it, a drag up or down scrolls (the browser's) and
 * picks nothing (a slider scrolled past keeps its value).
 */
function pickPress(event, element, cursor, move) {
  if (holding) return;
  if (event.pointerType !== "touch") return track(event, element, cursor, move);
  const stop = press(event, {
    move: (next) => {
      const dx = next.clientX - event.clientX, dy = next.clientY - event.clientY;
      if (Math.hypot(dx, dy) <= SLOP) return;
      stop();
      // (Clearly across: an arc as a scroll begins, or a slant, is a scroll.)
      if (Math.abs(dx) > 2 * Math.abs(dy)) track(next, element, cursor, move);
    },
    // (A tap: picked where it's lifted, unless a drag's held meanwhile (one at a time, see track).)
    end: (release) => release && !holding && move(release),
  });
}
/**
 * Sliders' pointer and keys, wherever they are (the panel's, the belt's popover's): a press scrubs from stop to stop
 * (one undo step a drag; by touch, as pickPress has it), the arrows and Page Up and Down step, Home and End go to the ends; off its stops (a value
 * set otherwise), a step goes to the nearest stop that way. `valueOf` reads a slider's value now; `set` sets it.
 * @param {HTMLElement} host @param {{ valueOf: (id: string) => number | undefined, set: (id: string, value: number) => void }} how
 */
function sliders(host, { valueOf, set }) {
  host.addEventListener("pointerdown", (event) => {
    const element = closest(event, ".stepper");
    if (event.button !== 0 || !element || !SLIDERS[element.id]) return;
    const id = element.id;
    pickPress(event, element, "scrubbing", (point) => {
      const value = stopAt(id, point.clientX);
      if (value !== undefined) set(id, value);
    });
  });
  host.addEventListener("keydown", (event) => {
    const element = /** @type {HTMLElement} */ (event.target), spec = SLIDERS[element.id], value = spec && element.classList.contains("stepper") ? valueOf(element.id) : undefined;
    const up = ["ArrowRight", "ArrowUp", "PageUp"].includes(event.key), down = ["ArrowLeft", "ArrowDown", "PageDown"].includes(event.key);
    if (value === undefined || !(up || down || event.key === "Home" || event.key === "End")) return;
    event.preventDefault();
    event.stopPropagation();
    const stops = /** @type {number[]} */ (spec.stops), at = stops.indexOf(value);
    const to = event.key === "Home" ? stops[0] : event.key === "End" ? stops.at(-1)
      : at >= 0 ? stops[clamp(at + (up ? 1 : -1), 0, stops.length - 1)]
      : up ? stops.find((stop) => stop > value) ?? stops.at(-1) : stops.findLast((stop) => stop < value) ?? stops[0];
    set(element.id, /** @type {number} */ (to));
  });
}
/**
 * A gradient's edit (`edit` gives back the color then to be set, its index), the one in hand's; `again`, the
 * inspector drawn again with that color the one set, its cursor focused.
 * @param {(item: Layer) => number} edit
 */
function gradientEdit(edit, again = false) {
  let k = -1;
  editLayer((item) => (k = item.gradient ? edit(item) : k));
  if (k < 0) return;
  colorSlot.set(`${chart.id}:${selected}`, `stop${k}`);
  if (again) renderInspector(), refocus(`#inspector .gradient-stop[data-stop="${k}"]`);
}
/**
 * A gradient's cursor (its place, its color), its bar and its field (not one being typed in: `field` false), as its
 * color `k` is now (drawn while a drag holds the inspector still).
 */
function showStop(/** @type {number} */ k, field = true) {
  const layer = layerById(selected), stops = layer?.gradient, at = stops?.[k]?.[0];
  if (!stops || at === undefined) return;
  const cursor = /** @type {HTMLElement | null} */ ($("inspector").querySelector(`.gradient-stop[data-stop="${k}"]`)), bar = /** @type {HTMLElement | null} */ ($("inspector").querySelector("[data-gradient-bar]"));
  if (cursor) {
    // (What it says too: its tip's keys after its first line, as written.)
    const said = stopSaid(stops[k]), tip = cursor.dataset.tip ?? "";
    cursor.style.setProperty("left", `${at}%`);
    cursor.style.setProperty("--c", cssColor(stops[k][1]));
    cursor.setAttribute("aria-label", said);
    cursor.dataset.tip = said + tip.slice(tip.indexOf("\n") < 0 ? tip.length : tip.indexOf("\n"));
  }
  bar?.style.setProperty("--g", gradientCss(stops));
  // (Another color moved than the one in hand (a drag held the panel still): it, in hand, its row and ring.)
  const hand = $("inspector").querySelector(".gradient-hand");
  if (hand && !hand.querySelector(`[data-stop-at="${k}"]`)) {
    hand.outerHTML = gradientHand(layer, k);
    for (const other of all$(".gradient-stop", $("inspector"))) other.setAttribute("aria-pressed", String(other.dataset.stop === String(k)));
  }
  const input = /** @type {HTMLInputElement | null} */ ($("inspector").querySelector(`[data-stop-at="${k}"]`));
  if (input && field) input.value = String(at);
  // (The color in hand's worth, under its knob where it is now; the gradient ready made it is now, if one.)
  const ready = /** @type {HTMLElement | null} */ ($("inspector").querySelector('[data-action="gradient-ready"]'));
  showScale();
  const named = hand?.isConnected ? hand.querySelector("[data-stop-name]") : $("inspector").querySelector("[data-stop-name]");
  if (named) named.innerHTML = `<i class="swatch" style="background: ${escape(cssColor(stops[k][1]))}"></i>${escape(colorName(stops[k][1]))}`;
  if (ready) ready.innerHTML = `${gradientSwatch(stops)}<span class="name">${escape(readyName(stops))}</span>${CHEVRON}`;
}
/** A gradient's color, said: its name, its place. @param {[number, string]} stop */
const stopSaid = ([at, color]) => `${colorName(color)}, at ${at}%`;
/** Where a point is across a gradient's bar, 0 to 100. */
const barAt = (/** @type {HTMLElement} */ bar, /** @type {number} */ clientX) => ((box) => (box.width ? ((clientX - box.left) / box.width) * 100 : 0))(bar.getBoundingClientRect());

/** How a layer's colors are chosen now (see COLOR_MODES). @param {Layer} layer */
function colorModeOf(layer) {
  if (layer.byGradient && layer.gradient) return "gradient";
  if (layer.colorFrom) return "from";
  if (layer.byLayers && modesOf(layer).includes("layers")) return "layers";
  if (layer.byLevels) return "levels";
  if (layer.type === "histogram") return layer.colors === 1 ? "one" : layer.colors === 4 ? "momentum" : "sides";
  if (TWO_OR_ONE.has(layer.type)) return layer.colors === 1 ? "one" : "sides";
  if (layer.type === "baseline" || (TWO_KEPT.has(layer.type) && layer.colors !== 1 && layer.colorBelow !== undefined && modesOf(layer).includes("sides"))) return "sides";
  return "one";
}
/** One way of coloring that's two colors (see colorSlots): a P&L heatmap's, its profit's and its loss's; a signal's that marks both ways, up and down. */
const pairedOne = (/** @type {Layer} */ layer) => (layer.type === "heatmap" && layer.measure === "pnl") || signedSignal(layer);
/** A way of coloring, named for a layer of its type (a pair, by what its two are for). */
const modeName = (/** @type {string} */ mode, /** @type {Layer} */ layer) => (mode === "one" ? (pairedOne(layer) ? "Two" : "One") : mode === "sides" ? "Two" : mode === "momentum" ? "Four" : mode === "levels" ? "By its levels" : mode === "layers" ? "By its layers" : mode === "gradient" ? "Gradient" : "From a layer");
/**
 * A layer's colors in its way of coloring, each with what it's for: [its key (a Layer's field; "from", the layer it
 * takes them from), its label].
 * @param {Layer} layer @returns {[string, string][]}
 */
function colorSlots(layer) {
  // (A gradient's: its stops, each by where it is.)
  if (layer.byGradient && layer.gradient) return layer.gradient.map(([at], k) => [`stop${k}`, `At ${at}%`]);
  // (A profile's: each holder group's. A heatmap's: one; its unrealized P&L's, its profit's and its loss's.)
  if (layer.type === "profile") return (layer.profile ?? []).map((cohort) => [`cohort:${cohort}`, cohortName(cohort)]);
  if (layer.type === "heatmap") return layer.measure === "pnl" ? [["color", "In profit"], ["colorBelow", "In loss"]] : [["color", "Color"]];
  const mode = colorModeOf(layer), pair = layer.type === "cloud" ? ((letters) => (letters.length === 2 ? [`${letters[0]} over ${letters[1]}`, `${letters[1]} over ${letters[0]}`] : ["First over", "Second over"]))(fromLetters(layer)) : PAIRS[layer.type];
  return mode === "from" ? [["from", "Layer"]] : mode === "levels" ? [["color", "Under its levels"]] : mode === "layers" ? []
    : mode === "sides" || signedSignal(layer) ? [["color", pair[0]], ["colorBelow", pair[1]]]
    : mode === "momentum" ? [["color", "Above, growing"], ["colorFading", "Above, fading"], ["colorBelow", "Below, growing"], ["colorBelowFading", "Below, fading"]]
    : [["color", "Color"]];
}
/** A color fainter, as the card shows it (MACD's bars falling back): halfway to the card, its chroma halved. */
function faintOf(/** @type {string} */ color) {
  const [l, c, h] = shownOklch(color), [card] = oklchOf(css("--card"));
  return kept(oklchToHex(l + (card - l) * 0.5, c * 0.5, h));
}
/** Bars' color where they fall back toward their base, above it or below (four colors): theirs, else their side's fainter. @param {Layer} layer */
const fadingOf = (layer, /** @type {boolean} */ below) => (below ? layer.colorBelowFading : layer.colorFading) ?? faintOf(below ? belowOf(layer) : layer.color);
const colorOf = (/** @type {Layer} */ layer, /** @type {string} */ key) => (key.startsWith("cohort:") ? profileColor(layer, key.slice(7)) : key.startsWith("stop") ? layer.gradient?.[Number(key.slice(4))]?.[1] ?? layer.color : key === "colorBelow" ? belowOf(layer) : key === "colorFading" ? fadingOf(layer, false) : key === "colorBelowFading" ? fadingOf(layer, true) : layer.color);
/**
 * One of a layer's colors, set (its way of coloring as it is): its own; its second, kept only where it isn't its
 * default (red below a baseline's base, bars' too; a trend's or a cloud's, as picked); a fading one of four (empty:
 * its side's fainter again).
 */
function setColor(/** @type {Layer} */ layer, /** @type {string} */ key, /** @type {string} */ color) {
  if (key.startsWith("stop")) ((stop) => stop && color && (stop[1] = color))(layer.gradient?.[Number(key.slice(4))]);
  // (A profile's holder group's: its own, see normalize.)
  else if (key.startsWith("cohort:")) color && (layer.cohortColors = { ...layer.cohortColors, [key.slice(7)]: color });
  else if (key === "colorFading") layer.colorFading = color || undefined;
  else if (key === "colorBelowFading") layer.colorBelowFading = color || undefined;
  // (Red, the default, left unset, but candles': their two colors said, as unset reads as one, see normalize.)
  else if (key === "colorBelow") layer.colorBelow = !TWO_KEPT.has(layer.type) && layer.type !== "candles" && color === "red" ? undefined : color;
  else layer.color = color;
}
/**
 * A layer's way of coloring, set (`from`: the layer it takes its colors from); the colors picked kept, for when it's
 * colored that way again (a trend's or a cloud's second, given one the first time it gets two: see secondColor).
 */
function setColorMode(/** @type {Layer} */ layer, /** @type {string} */ mode, from = "") {
  delete layer.colorFrom;
  delete layer.byLevels;
  delete layer.byLayers;
  delete layer.colors;
  // (Its gradient kept, as its other colors are: colored by it again, as it was.)
  delete layer.byGradient;
  if (mode === "gradient") (layer.byGradient = true), (layer.gradient ??= gradientFor(layer));
  else if (mode === "levels") (layer.byLevels = true), colorLevels(layer);
  else if (mode === "layers") (layer.byLayers = true), layersColors(layer, ownerOf(layer));
  else if (mode === "from") layer.colorFrom = from;
  else if ((layer.type === "histogram" && (mode === "one" || mode === "momentum")) || (TWO_OR_ONE.has(layer.type) && mode === "one")) layer.colors = mode === "one" ? 1 : 4;
  // (Two the same, as its layers' may be: a second apart.)
  else if (TWO_KEPT.has(layer.type) && mode === "sides") layer.colorBelow = layer.colorBelow === undefined || layer.colorBelow === layer.color ? secondColor(layer) : layer.colorBelow;
  else if (TWO_KEPT.has(layer.type) && mode === "one" && layer.colorBelow !== undefined) layer.colors = 1;
  // (Two colors picked: their second said (red unset), as one unset reads as one, see normalize.)
  else if (TWO_OR_ONE.has(layer.type) && mode === "sides") layer.colorBelow ??= secondColor(layer);
}

/**
 * A gradient a layer starts with: a heatmap's unrealized P&L through gray at no gain from loss to profit; another
 * heatmap's from indigo through rose to yellow (dark to bright, as heat); anything else's from blue to red.
 * @param {Layer} layer @returns {[number, string][]}
 */
const gradientFor = (layer) => (layer.type === "heatmap" ? (layer.measure === "pnl" ? [[0, "red"], [50, "gray"], [100, "green"]] : structuredClone(HEAT_GRADIENT)) : [[0, "blue"], [100, "red"]]);
/**
 * A gradient as CSS draws it, across: its own colors (see gradientTable), each as shown, a sample of them (CSS's own
 * mixing, between its stops, would be another: through gray where they're mixed in sRGB).
 * @param {[number, string][]} stops
 */
const gradientCss = (stops) => ((table) => `linear-gradient(to right, ${Array.from({ length: 17 }, (_, k) => `${cssColor(table[Math.round((k / 16) * (table.length - 1))])} ${((k / 16) * 100).toFixed(2)}%`).join(", ")})`)(gradientTable(stops));
/** Each gradient's colors, by its stops: see gradientTable. @type {Map<string, string[]>} */
const gradientTables = new Map();
/** The page's ink as the dark shows it (`--ink`'s dark side): as colors are kept, see gradientTable. */
const INK_KEPT = "#f7f8f8";
/** How many colors a gradient is drawn in (each a value's place, rounded to the nearest of so many). */
const GRADIENT_STEPS = 64;
/**
 * A gradient's colors as kept (each `oklch(…)`, as a color picked is: ink shows it in the theme), `GRADIENT_STEPS`
 * of them from 0% to 100%: between two stops, mixed in OKLCH, lightness and chroma straight and hue the shorter
 * way round (a gray one taking its neighbor's: no hue of its own), so two far apart stay vivid between (mixed in
 * OKLab, blue to yellow went through gray); past its outer ones, theirs.
 * @param {[number, string][]} stops @returns {string[]}
 */
function gradientTable(stops) {
  const key = JSON.stringify(stops), had = gradientTables.get(key);
  if (had) return had;
  // (Ink mixed as it's kept, the dark's: the light shows it dark as any other color, see ink.)
  const lchs = stops.map(([, color]) => (color === "ink" ? hexToOklch(INK_KEPT) : oklchOf(color))), GRAY = 0.02;
  const table = Array.from({ length: GRADIENT_STEPS }, (_, k) => {
    const at = (k / (GRADIENT_STEPS - 1)) * 100, next = stops.findIndex(([where]) => where >= at);
    const [a, b, t] = next <= 0 ? [next < 0 ? stops.length - 1 : 0, next < 0 ? stops.length - 1 : 0, 0] : [next - 1, next, (at - stops[next - 1][0]) / Math.max(1e-9, stops[next][0] - stops[next - 1][0])];
    const [l0, c0, h0] = lchs[a], [l1, c1, h1] = lchs[b];
    const from = c0 < GRAY ? h1 : h0, to = c1 < GRAY ? h0 : h1, turn = ((((to - from) % 360) + 540) % 360) - 180;
    return `oklch(${+(l0 + (l1 - l0) * t).toFixed(4)} ${+(c0 + (c1 - c0) * t).toFixed(4)} ${+((((from + turn * t) % 360) + 360) % 360).toFixed(1)})`;
  });
  if (gradientTables.size > 200) gradientTables.clear();
  gradientTables.set(key, table);
  return table;
}
/** A layer's lowest and highest values (none: Infinity and -Infinity). @param {Vector | number[]} values @returns {[number, number]} */
function rangeOf(values) {
  let low = Infinity, high = -Infinity;
  // (By index, finite by a subtraction: a gradient's layer's every value at each live price; zeros by
  // Math's own, a negative one kept as it was.)
  for (let i = 0; i < values.length; i++) {
    const value = values[i];
    if (value === 0) (low = Math.min(low, value)), (high = Math.max(high, value));
    else if (value - value === 0) (low = value < low ? value : low), (high = value > high ? value : high);
  }
  return [low, high];
}
/**
 * Where each value is between its lowest and its highest (0 to 1): of all its history, or (`upToDate`) of its
 * history up to it (no hindsight: what that day would have shown). None where it has none.
 * @param {Vector} values @param {boolean} upToDate @returns {Float64Array}
 */
function gradientPlaces(values, upToDate) {
  const places = new Float64Array(values.length).fill(NaN);
  let [low, high] = upToDate ? [Infinity, -Infinity] : rangeOf(values);
  for (let i = 0; i < values.length; i++) {
    const value = values[i];
    if (!Number.isFinite(value)) continue;
    if (upToDate) (low = Math.min(low, value)), (high = Math.max(high, value));
    places[i] = high > low ? (value - low) / (high - low) : 0.5;
  }
  return places;
}
/**
 * The surface, one plane, every point (u across, t down, 0 to 1) a color. Down, lightness, white at the top to
 * black at the bottom, each lightness given its room (`spaceAt`): all of it from the dark tones still seen to the
 * palest tints, little at the very ends, where everything reads as black or white. One smooth curve, no seams.
 * Across, the hues over HUES of the width; past them their chroma eases away, into the grays at the right.
 */
const HUES = 0.86;
const smoothstep = (from, to, x) => {
  const s = clamp((x - from) / (to - from), 0, 1);
  return s * s * (3 - 2 * s);
};
const spaceAt = (l) => 0.05 + 0.95 * smoothstep(0.05, 0.22, l) * (1 - smoothstep(0.95, 1, l));
/** The height of a lightness (0 at the top: white). */
function heightOf(l) {
  const i = clamp(l, 0, 1) * 1000, k = Math.min(999, Math.floor(i));
  return 1 - (BELOW[k] + (BELOW[k + 1] - BELOW[k]) * (i - k));
}
/** The lightness at a height: the curve read back, by halving through its table. */
function lightAt(t) {
  const below = 1 - clamp(t, 0, 1);
  let low = 0, high = 1000;
  while (high - low > 1) {
    const mid = (low + high) >> 1;
    if (BELOW[mid] <= below) low = mid;
    else high = mid;
  }
  const span = BELOW[high] - BELOW[low];
  return (low + (span ? (below - BELOW[low]) / span : 0)) / 1000;
}
/** The hue across (the last kept short of 360°, which is 0°, on the left), and how much chroma a point keeps there. */
const hueAt = (u) => Math.min(u / HUES, 1) * 359.9;
const keepAt = (u) => (u <= HUES ? 1 : 0.5 + 0.5 * Math.cos(Math.PI * Math.min(1, (u - HUES) / ((1 - HUES) * 0.85))));
/** The surface's chroma, across every hue: OKLCH's even saturation, less only where sRGB can't hold it. */
const CHROMA = 0.18;

/** Bitview's colors, a dot each (its name on hover), the one in use ringed. */
/**
 * A custom color's hex, typed or followed (`target`: what it sets), its name after it in the field (none: an empty
 * field, a color another's); joined under its surface, the two one control. @param {string} target @param {string} color
 */
const hexField = (target, color) => `<label class="hex-field"><span class="hex-box"><input class="hex" data-hex="${escape(target)}" value="${color ? escape(ink(color)) : ""}" maxlength="40" spellcheck="false" autocomplete="off" aria-label="Hex color" /><span class="color-now">${color ? escape(colorName(color)) : ""}</span></span></label>`;
const swatchGrid = (/** @type {string} */ current) => `<div class="swatch-grid" role="radiogroup" aria-label="Bitview's colors">${SWATCHES.map((color) => `<button type="button" role="radio" data-pick-color="${escape(color)}" aria-checked="${current === color}" aria-label="${escape(colorName(color))}" data-tip="${escape(colorName(color))}"><i class="swatch" style="--c:${escape(cssColor(color))}"></i></button>`).join("")}</div>`;
/** The surface: every hue across, lightness down, the color's place ringed (`target`: what it picks for, see planeTarget). */
function planeHtml(/** @type {string} */ color, /** @type {string} */ target, name = "Color") {
  const { u, t } = placeOf(color), said = planeSays(color, u);
  return `<div class="plane" data-plane="${escape(target)}" role="slider" tabindex="0" aria-label="${escape(name)}: hue across, then grays; lightness down" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${said.now}" aria-valuetext="${said.text}" style="--x: ${u}; --y: ${t}"><canvas></canvas><i class="ring" style="--c:${escape(cssColor(color))}"></i></div>`;
}
/** What a surface says of its color, at its place across: its lightness, and its hue (amid the grays, a gray). */
function planeSays(/** @type {string} */ color, /** @type {number} */ u) {
  const [l, c, h] = shownOklch(color), grayish = u > HUES + (1 - HUES) * 0.85 - 1e-9;
  return { now: Math.round(l * 100), text: `${grayish || c < 0.02 ? "Gray" : `Hue ${Math.round(h)} degrees`}, lightness ${Math.round(l * 100)} percent` };
}
// A color to pick, the same wherever one is (a layer's, a drawing's, a line's): one of Bitview's, a click; any other,
// on the surface under them, every hue across and lightness down, at one chroma (a quieter color is a matter of
// opacity).

/** A layer's levels as its bands, lowest first: each one's value and color (of two at one value, the first's, as its line across is). @param {Layer} layer */
function levelSteps(layer) {
  const steps = (layer.levels ?? []).map((level, k) => ({ value: level.value, color: lineLook(layer, k).color, k })).sort((a, b) => a.value - b.value || a.k - b.k);
  return steps.filter((step, i) => !i || step.value !== steps[i - 1].value);
}
/** Colors for levels that color a layer, in order up: warmer as they go, none its own or one taken. */
const LEVEL_COLORS = ["yellow", "orange", "red", "pink", "purple", "violet", "blue", "sky", "teal", "green", "lime", "amber"];
/** A level added to a layer (in its edit): its own color where the layer's colored by its levels. @param {Layer} layer @param {number} value */
function withLevel(layer, value) {
  layer.levels = [...(layer.levels ?? []), { value }];
  if (layer.byLevels) colorLevels(layer);
}
/** A layer's level taken off (in its edit): none left, none. @param {Layer} layer @param {number} i */
function withoutLevel(layer, i) {
  const others = layer.levels?.filter((_, k) => k !== i);
  layer.levels = others?.length ? others : undefined;
  // (No levels, no band under them.)
  if (!layer.levels) delete layer.underName;
}
/**
 * A layer's levels made to color it: each without a color of its own given one (by value, lowest first, from
 * LEVEL_COLORS), so its bands show at once and don't follow its own color under them. @param {Layer} layer
 */
function colorLevels(layer) {
  const taken = new Set([layer.color, ...(layer.levels ?? []).map((level) => level.color).filter(Boolean)]);
  const free = LEVEL_COLORS.filter((color) => !taken.has(color));
  for (const level of [...(layer.levels ?? [])].sort((a, b) => a.value - b.value)) if (!level.color) level.color = free.shift() ?? layer.color;
}
/** A layer's colors by its levels, as a swatch: its own under them, each level's above it, the highest on top. @param {Layer} layer */
function bandsSwatch(layer) {
  const colors = [layer.color, ...levelSteps(layer).map((step) => step.color)];
  const share = 100 / colors.length;
  return `<i class="swatch" style="background: linear-gradient(to top, ${colors.map((color, k) => `${escape(cssColor(color))} ${(k * share).toFixed(1)}% ${((k + 1) * share).toFixed(1)}%`).join(", ")})"></i>`;
}
/** A way of coloring a layer, as a swatch: its colors top to bottom (by its levels, its bands; from a layer, its letter). */
function modeSwatch(/** @type {Layer} */ layer, /** @type {string} */ mode) {
  if (mode === "levels") return bandsSwatch(layer);
  // (A gradient's, across, as its bar is.)
  if (mode === "gradient") return gradientSwatch(layer.gradient ?? gradientFor(layer));
  // (From a layer: which, once there is one; none chosen yet, no letter that would say one was.)
  if (mode === "from") return ((source) => (source ? letterChip(source) : ICONS.series))(layerById(layer.colorFrom ?? ""));
  const colors = mode === "momentum" ? [layer.color, fadingOf(layer, false), fadingOf(layer, true), belowOf(layer)] : mode === "sides" || (mode === "one" && pairedOne(layer)) || (mode === "layers" && !edgesOf(layer).length) ? [layer.color, belowOf(layer)] : [layer.color];
  const share = 100 / colors.length;
  return `<i class="swatch" style="background: linear-gradient(to bottom, ${colors.map((color, k) => `${escape(cssColor(color))} ${(k * share).toFixed(1)}% ${((k + 1) * share).toFixed(1)}%`).join(", ")})"></i>`;
}
/** A layer's color in one of its places, named (a fading one of four not set: its side's, paler). */
const slotName = (/** @type {Layer} */ layer, /** @type {string} */ key) => ((own) => (own === undefined && (key === "colorFading" || key === "colorBelowFading") ? `Fainter ${colorName(key === "colorFading" ? layer.color : belowOf(layer)).toLowerCase()}` : colorName(colorOf(layer, key))))(key === "colorFading" ? layer.colorFading : key === "colorBelowFading" ? layer.colorBelowFading : "");
/**
 * A layer's colors: how they're chosen (those its type has; one way only, none to choose), then the colors that way
 * takes, each on its line, said by what it's for (from a layer: which).
 */
function colorPicker(/** @type {Layer} */ layer) {
  // (A profile of no groups: none to color.)
  if (layer.type === "profile" && !layer.profile?.length) return "";
  const mode = colorModeOf(layer), name = modeName(mode, layer), source = layerById(layer.colorFrom ?? ""), ways = modesOf(layer).length > 1;
  // The way: a choice, as every other, on its line's right (none to choose: its label alone).
  const head = `<div class="field"><span>${mode === "one" && !pairedOne(layer) && !(layer.type === "profile" && (layer.profile?.length ?? 0) > 1) ? "Color" : "Colors"}</span>${ways ? `<button type="button" id="color-mode" class="preset" data-action="color-mode" aria-haspopup="menu" aria-label="Colors: ${escape(name)}"><span class="name">${escape(name)}</span>${CHEVRON}</button>` : ""}</div>`;
  if (mode === "from") return `${head}<div class="field color-slot"><span>Layer</span><button type="button" id="color-from" class="preset" data-action="color-from" aria-haspopup="menu" aria-label="Colors from: ${escape(source ? labelOf(source) : "")}">${source ? letterChip(source) : ""}<span class="name">${escape(source ? labelOf(source) : "")}</span>${CHEVRON}</button></div>`;
  // By its levels, by its layers: its colors theirs (none here).
  if (mode === "levels" || mode === "layers") return head;
  const places = colorSlots(layer), key = slotNow(layer), label = places.find(([slot]) => slot === key)?.[1] ?? "Color";
  const own = key === "colorFading" ? layer.colorFading : key === "colorBelowFading" ? layer.colorBelowFading : colorOf(layer, key), from = sourceNow(layer, key);
  // Which one (two, four): the colors themselves, under what they're for (four: its sides down, what the bar does
  // across), the one being set ringed.
  const chip = (/** @type {string} */ slot, /** @type {string} */ name) => `<button type="button" class="slot-chip" data-color-slot="${slot}" aria-pressed="${slot === key}" aria-label="${escape(name)}: ${escape(slotName(layer, slot))}" data-tip="${escape(slotName(layer, slot))}" style="--c:${escape(cssColor(colorOf(layer, slot)))}"></button>`;
  // (A profile's groups: each its name over its color, as many as it has.)
  const which = mode === "gradient" ? gradientEditor(layer, key) : layer.type === "profile" ? (places.length > 1 ? `<div class="color-slots cohorts" role="group" aria-label="Which color">${places.map(([slot, name]) => `<span class="cohort-slot"><span class="head">${escape(name)}</span>${chip(slot, name)}</span>`).join("")}</div>` : "") : places.length > 2
    ? `<div class="color-slots four" role="group" aria-label="Which color"><span></span><span class="head">Growing</span><span class="head">Fading</span><span>Above</span>${chip("color", "Above, growing")}${chip("colorFading", "Above, fading")}<span>Below</span>${chip("colorBelow", "Below, growing")}${chip("colorBelowFading", "Below, fading")}</div>`
    : places.length > 1 ? `<div class="color-slots" role="group" aria-label="Which color">${places.map(([, name]) => `<span class="head">${escape(name)}</span>`).join("")}${places.map(([slot, name]) => chip(slot, name)).join("")}</div>` : "";
  // Where it's from: Bitview's colors; any other (the surface, its hex); a fading one's own, its growing one fainter.
  const sources = /** @type {[string, string][]} */ ([["presets", "Palette"], ["custom", "Custom"], ...(fadingSlot(key) ? [["auto", "Auto"]] : [])]);
  const tabs = colorSourceTabs(label, sources, from);
  const target = `layer:${key}`, side = key === "colorFading" ? layer.color : belowOf(layer);
  const body = from === "custom" ? `<div class="color-box">${planeHtml(colorOf(layer, key), target, label)}${hexField(target, colorOf(layer, key))}</div>`
    : from === "auto" ? `<p class="color-auto"><i class="swatch" style="--c:${escape(cssColor(faintOf(side)))}"></i>Fainter ${escape(colorName(side).toLowerCase())}</p>`
    : swatchGrid(own && SWATCHES.includes(own) ? own : "");
  // (Relative to all of its history, or up to each date: not a heatmap's, each day its own.)
  const relative = mode === "gradient" && !heatLike(layer) ? `<div class="field"><span>Relative to</span><div class="tabs" role="group" aria-label="Relative to">${/** @type {[boolean, string, string][]} */ ([[false, "All of history", "Its lowest and highest of all its history"], [true, "Up to each date", "Its lowest and highest up to each date: what that day showed, no hindsight"]]).map(([on, text, tip]) => `<button type="button" data-gradient-to="${on ? 1 : ""}" aria-pressed="${Boolean(layer.upToDate) === on}" data-tip="${escape(tip)}">${text}</button>`).join("")}</div></div>` : "";
  return `${head}<div class="color-editor" data-palette="${key}">${which}${tabs}<div class="color-pick">${body}</div></div>${relative}`;
}
/** The gradient ready made a gradient is, if it's one (its stops, every one the same). @param {[number, string][] | undefined} stops */
const readyGradient = (stops) => GRADIENT_PRESETS.find(([, , preset]) => JSON.stringify(preset) === JSON.stringify(stops))?.[0] ?? null;
/** Its name: the gradient ready made it is, else its own. @param {[number, string][] | undefined} stops */
const readyName = (stops) => GRADIENT_PRESETS.find(([id]) => id === readyGradient(stops))?.[1] ?? "Your own";
/** A gradient drawn small, as a swatch is (a row's icon, a select's). @param {[number, string][] | undefined} stops */
const gradientSwatch = (stops) => `<i class="swatch gradient-swatch" style="background: ${escape(gradientCss(stops ?? []))}"></i>`;
/**
 * Each layer's own gradient, as it was when one ready made took its place (to go back to: see readyPage), this
 * session (not kept with it), by its chart's id and its own.
 * @type {Map<string, [number, string][]>}
 */
const ownGradients = new Map();
/**
 * A layer's gradients ready made, to take one: the one menu of them, whatever opens it (its editor's select, ⌘K's
 * row, there with `preview`, its colors' look at each), each shown on the chart as it's highlighted; its own first,
 * once it has one (the one it has, else the one a ready made took the place of: never lost to one tried).
 * @param {string} id
 */
const readyPage = (id, /** @type {any} */ preview = null) => ((both) => choicePage("Ready made", () => {
  const key = `${chart.id}:${id}`, stops = layerById(id)?.gradient, own = readyGradient(stops) ? ownGradients.get(key) : stops;
  return [...(own ? /** @type {const} */ ([["own", "Your own", own]]) : []), ...GRADIENT_PRESETS].map(([choice, name, ready]) => ({ key: choice, icon: gradientSwatch(ready), label: name, current: choice === "own" ? !readyGradient(stops) : readyGradient(stops) === choice, ...both((item) => {
    if (item.gradient && !readyGradient(item.gradient)) ownGradients.set(key, structuredClone(item.gradient));
    item.gradient = structuredClone(ready);
  }) }));
}, preview))(bothOf(id));
/**
 * A layer's gradient the other way around, each color where its mirror is (its lowest's color its highest's): the
 * color in hand still in hand, where it went. @param {string} id
 */
function reverseGradient(id) {
  // (Drawn again whole, as after a press in it: a place's field in focus (⌘K's, closed, gives it back) is no place
  // being typed.)
  const pressed = () => ($("inspector").dataset.pressed = "1");
  pressed();
  editById(id, (item) => {
    if (!item.gradient) return;
    // (Before it's drawn again.)
    colorSlot.set(`${chart.id}:${id}`, `stop${item.gradient.length - 1 - (Number(slotNow(item).slice(4)) || 0)}`);
    item.gradient = item.gradient.map(([at, color]) => /** @type {[number, string]} */ ([100 - at, color])).reverse();
  });
  // (The same either way (nothing changed, nothing drawn again): its color in hand, where it went.)
  if (id === selected) pressed(), renderInspector();
}
/**
 * Where a layer's values fall along its gradient, as an axis's ticks are written (see axisLabel): a heatmap's at each
 * power of ten from its faintest cell seen to its reference (unrealized P&L's, a loss's and a gain's either side
 * of none, at 50%, kept or not together); else round steps from its lowest to its highest; each its place (0 to
 * 100) and its label's left on the bar as laid out (`width` across: under its place, inside it at its ends), the
 * strongest kept where two would come closer than a gap. First of all, the color in hand's own worth under its
 * knob (at `hand`: `own`), those near it giving way. None yet (or the bar not laid out), none.
 * @param {Layer} layer @param {number} width @param {number} [hand] @returns {[number, string, number, boolean][]}
 */
function gradientTicks(layer, width, hand) {
  if (!width) return [];
  // (Each kept with the others of its group, or none of them.)
  const result = resolved?.results.get(layer.id), ticks = /** @type {[number, number][][]} */ ([]), worth = hand === undefined ? null : gradientWorth(layer, hand);
  // (None to write (no reference yet, values all the same): the color in hand's alone.)
  if (layer.type === "heatmap") {
    const most = heatLooks.get(layer.id)?.reference ?? 0, pnl = layer.measure === "pnl";
    for (let power = Math.floor(Math.log10(most)); most && 10 ** power >= heatAmount(0, most); power--) {
      const felt = heatFelt(10 ** power, most);
      ticks.push(pnl ? [[50 + felt * 50, 10 ** power], [50 - felt * 50, -(10 ** power)]] : [[felt * 100, 10 ** power]]);
    }
    if (pnl && most) ticks.unshift([[50, 0]]);
  } else {
    const [low, high] = rangeOf(result?.values ?? []), rough = (high - low) / 4, step = ((ten) => [1, 2, 5, 10].map((k) => k * ten).find((size) => size >= rough) ?? 10 * ten)(10 ** Math.floor(Math.log10(rough)));
    // (Eleven at most, counted apart from the values: a step too small to move one (a range of a few ulps) still
    // ends, its ticks as one, the overlaps dropped below.)
    for (let i = 0, first = Math.ceil(low / step); high > low && i <= 10 && first + i <= Math.floor(high / step); i++) ticks.push([[(((first + i) * step - low) / (high - low)) * 100, (first + i) * step]]);
    ticks.sort((a, b) => Math.abs(b[0][1]) - Math.abs(a[0][1]));
  }
  // (Kept, the first first, while its label is a gap clear of those kept: each as wide as it's set, see widest. Those
  // under the color in hand's worth give way alone, the others of their group kept.)
  const GAP = 12, label = (/** @type {number} */ at, /** @type {string} */ text, own = false) => ((size) => /** @type {[number, string, number, number, boolean]} */ ([at, text, clamp((at / 100) * width - size / 2, 0, width - size), size, own]))(widest([text], own ? 510 : 400, 11));
  const clear = (/** @type {[number, string, number, number, boolean]} */ [, , left, size], /** @type {[number, string, number, number, boolean][]} */ others) => others.every(([, , other, room]) => left >= other + room + GAP || left + size <= other - GAP);
  const own = worth && hand !== undefined ? [label(hand, worth, true)] : [], kept = [...own];
  for (const group of ticks) {
    const shown = group.map(([at, value]) => label(at, axisLabel(value))).filter((tick) => clear(tick, own));
    if (shown.every((tick) => clear(tick, kept))) kept.push(...shown);
  }
  return kept.sort((a, b) => a[0] - b[0]).map(([at, text, left, , own]) => /** @type {[number, string, number, boolean]} */ ([at, text, left, own]));
}
/**
 * How a gradient heatmap's weakest cells fade into the chart (see heatColor), as a mask across its bar: an amount's
 * from its lowest; unrealized P&L's either side of none. None else. @param {Layer} layer
 */
const gradientFade = (layer) => (layer.type !== "heatmap" ? "" : layer.measure === "pnl" ? `linear-gradient(to right, #000 ${50 - 50 * HEAT_FADE}%, transparent 50%, #000 ${50 + 50 * HEAT_FADE}%)` : `linear-gradient(to right, transparent, #000 ${100 * HEAT_FADE}%)`);
/**
 * A gradient's editor, at the top of its colors' (under its colors' line): the gradient ready made it is (a select of
 * them, see readyPage), across from it ways to reverse it and to add a color; its bar, as the chart draws it (a
 * heatmap's weakest faded), its colors each a cursor through it where it is, its knob under it (pressed, its color
 * the one set; dragged, or moved by the arrows, between its neighbors; ⌫, gone, two left at least), a press on the
 * bar or under it a color added there; where its values fall along it (see gradientTicks). Then the color in hand,
 * heading the picker that sets it: a row as every field's (its color, named; what its place is worth, its place,
 * typed, a way to remove it).
 * @param {Layer} layer @param {string} key the color being set
 */
function gradientEditor(layer, key) {
  const stops = layer.gradient ?? [], few = stops.length <= GRADIENT_LEAST, k = clamp(Number(key.slice(4)) || 0, 0, stops.length - 1);
  const stop = (/** @type {[number, string]} */ [place, tint], /** @type {number} */ i) => `<button type="button" class="gradient-stop" data-color-slot="stop${i}" data-stop="${i}" aria-pressed="${i === k}" style="left: ${place}%; --c: ${escape(cssColor(tint))}" aria-label="${escape(stopSaid([place, tint]))}" data-tip="${escape(stopSaid([place, tint]))}${escape(`\n${keyText(`Drag, or ← → to move it${few ? "" : "; ⌫ to remove it"}`)}`)}"></button>`;
  const fade = gradientFade(layer);
  const head = `<div class="gradient-head"><button type="button" class="preset" data-action="gradient-ready" aria-haspopup="menu" aria-label="Ready made: ${escape(readyName(stops))}" data-tip="Gradients ready made">${gradientSwatch(stops)}<span class="name">${escape(readyName(stops))}</span>${CHEVRON}</button><button type="button" class="cap" data-action="gradient-reverse" aria-label="Reverse the gradient" data-tip="Reverse the gradient">${ICONS.swap}</button><button type="button" class="cap" data-action="gradient-add" aria-label="Add a color" ${stops.length >= GRADIENT_MOST ? `aria-disabled="true" data-tip="A gradient holds up to ${GRADIENT_MOST} colors"` : 'data-tip="Add a color, in the widest gap"'}>${ICONS.plus}</button></div>`;
  return `<div class="gradient-edit" role="group" aria-label="Its gradient">${head}<div class="gradient-scale" data-gradient-bar style="--g: ${escape(gradientCss(stops))}${fade ? `; --mask: ${fade}` : ""}"${stops.length < GRADIENT_MOST ? ' data-tip="A press adds a color here"' : ""}><span class="gradient-fill"></span>${stops.map(stop).join("")}</div>${gradientScale(layer, stops[k]?.[0])}</div>${gradientHand(layer, k)}`;
}
/**
 * Where a gradient's values fall along it, under its bar, the color in hand's (at `hand`) under its knob (see
 * gradientTicks; none yet, nothing). @param {Layer} layer @param {number} [hand]
 */
function gradientScale(layer, hand) {
  const ticks = gradientTicks(layer, gradientWidth, hand), ends = [gradientWorth(layer, 0), gradientWorth(layer, 100)], own = ticks.find((tick) => tick[3]);
  return ticks.length ? `<div class="gradient-ticks" role="img" aria-label="${escape(`${ends[0] && ends[1] ? `From ${ends[0]} to ${ends[1]}` : ticks.map(([, text]) => text).join(", ")}${own ? `; the color in hand at ${own[1]}` : ""}`)}">${ticks.map(([, text, left, mine]) => `<span${mine ? ' class="own"' : ""} style="left: ${left.toFixed(1)}px">${escape(text)}</span>`).join("")}</div>` : "";
}
/**
 * The inspector's gradient scale, the color in hand's worth on it, as its layer's values are now and that color is
 * (a heatmap's reference come or changed, see viewHeat; a color moved, see showStop; the bar's width, see
 * gradientWidth).
 */
function showScale() {
  const layer = layerById(selected), bar = $("inspector").querySelector("[data-gradient-bar]");
  if (!layer || !bar) return;
  const input = /** @type {HTMLInputElement | null} */ ($("inspector").querySelector("[data-stop-at]"));
  $("inspector").querySelector(".gradient-ticks")?.remove();
  bar.insertAdjacentHTML("afterend", gradientScale(layer, input ? layer.gradient?.[Number(input.dataset.stopAt)]?.[0] : undefined));
}
/**
 * A gradient's color in hand (its `k`th), a row as every field's: it, named; its place, typed, a way to remove it
 * (two left at least). (What its place is worth: under its knob, see gradientTicks.) @param {Layer} layer @param {number} k
 */
function gradientHand(layer, k) {
  const stops = layer.gradient ?? [], few = stops.length <= GRADIENT_LEAST, at = stops[k]?.[0] ?? 0, color = stops[k]?.[1] ?? layer.color;
  return `<div class="field gradient-hand"><span class="gradient-named" data-stop-name><i class="swatch" style="background: ${escape(cssColor(color))}"></i>${escape(colorName(color))}</span><span class="gradient-at"><label class="number-field"><input class="number-input" type="text" inputmode="numeric" autocomplete="off" spellcheck="false" value="${at}" data-stop-at="${k}" aria-label="${escape(colorName(color))}, its place" enterkeyhint="done" /><span class="unit">%</span></label><button type="button" class="cap small" data-stop-remove="${k}" aria-label="Remove ${escape(colorName(color))}" data-tip="${few ? `A gradient has ${GRADIENT_LEAST} colors at least` : "Remove"}"${few ? ' aria-disabled="true"' : ""}>${ICONS.trash}</button></span></div>`;
}
/**
 * What a place along a gradient (0 to 100) is worth, as its values are written: between its lowest and highest of
 * all its history (up to each date, the same range in all); a heatmap's, on its log scale from its faintest cell
 * seen to its reference (see heatColor), its unrealized P&L's from its largest loss through none to its largest
 * gain. None yet, none. @param {Layer} layer @param {number} at @returns {string | null}
 */
function gradientWorth(layer, at) {
  const result = resolved?.results.get(layer.id), unit = unitFor(layer, result), place = at / 100;
  if (layer.type === "heatmap") {
    // (A place is how strong it reads, see heatColor.)
    const most = heatLooks.get(layer.id)?.reference ?? 0;
    if (!most) return null;
    if (layer.measure !== "pnl") return format(heatAmount(place, most), unit);
    // (Its middle, none; either side, a loss or a gain as strong as it's far from it.)
    return place === 0.5 ? format(0, unit) : format(Math.sign(place - 0.5) * heatAmount(Math.abs(place - 0.5) * 2, most), unit);
  }
  const [low, high] = rangeOf(result?.values ?? []);
  return high >= low ? format(low + (high - low) * place, unit) : null;
}
/**
 * A gradient's color `k` moved to `at` (0 to 100, whole), between its neighbors, never on one (each its own place:
 * one on another would hide it): their order kept. @param {Layer} layer @param {number} k @param {number} at
 */
function moveStop(layer, k, at) {
  const stops = layer.gradient;
  if (!stops?.[k]) return;
  const low = k ? stops[k - 1][0] + 1 : 0, high = k < stops.length - 1 ? stops[k + 1][0] - 1 : 100;
  if (low <= high) stops[k][0] = Math.round(clamp(at, low, high));
}
/**
 * A color added to a gradient at `at` (in the widest gap if none), as the gradient is there (nothing changes till
 * it's set): its index, or -1 (as many as it holds). @param {Layer} layer @param {number} [at] @returns {number}
 */
function addStop(layer, at) {
  const stops = layer.gradient;
  if (!stops || stops.length >= GRADIENT_MOST) return -1;
  const gaps = [[0, stops[0][0]], ...stops.slice(1).map((stop, k) => [stops[k][0], stop[0]]), [stops[stops.length - 1][0], 100]];
  const widest = gaps.reduce((best, gap) => (gap[1] - gap[0] > best[1] - best[0] ? gap : best));
  const place = Math.round(at ?? (widest[0] + widest[1]) / 2), table = gradientTable(stops);
  // (Never on one already there: each color its own place, see moveStop.)
  if (stops.some(([where]) => where === place)) return -1;
  const k = stops.findIndex(([where]) => where > place), index = k < 0 ? stops.length : k;
  stops.splice(index, 0, [place, table[Math.round((place / 100) * (GRADIENT_STEPS - 1))]]);
  return index;
}
/**
 * A gradient's color `k` taken out, as long as it keeps its fewest (GRADIENT_LEAST): the index of the one in hand
 * then (the one before it), else `k`. @param {Layer} layer @param {number} k @returns {number}
 */
function removeStop(layer, k) {
  if (!layer.gradient || layer.gradient.length <= GRADIENT_LEAST) return k;
  layer.gradient.splice(k, 1);
  return Math.max(0, k - 1);
}
/** A color of four that fades (its default: its growing one, fainter). */
const fadingSlot = (/** @type {string} */ key) => key === "colorFading" || key === "colorBelowFading";
/** A layer's color as set (a fading one not set: none). */
const ownColor = (/** @type {Layer} */ layer, /** @type {string} */ key) => (key === "colorFading" ? layer.colorFading : key === "colorBelowFading" ? layer.colorBelowFading : colorOf(layer, key));
/**
 * Where each layer's color is picked from, as last chosen (by its chart's id, its own and which), with the color it was then: while
 * that's still its color; else as its color is (one of Bitview's, any other; a fading one not set, its own).
 * @type {Map<string, { source: string, color: string | undefined }>}
 */
const colorSource = new Map();
function sourceNow(/** @type {Layer} */ layer, /** @type {string} */ key) {
  const last = colorSource.get(`${chart.id}:${layer.id}:${key}`), own = ownColor(layer, key);
  if (last && last.color === own && (last.source !== "auto" || fadingSlot(key))) return last.source;
  return fadingSlot(key) && own === undefined ? "auto" : SWATCHES.includes(colorOf(layer, key)) ? "presets" : "custom";
}
/** Which of each layer's colors its palette sets now: the one picked (by its chart's id and its own), else its first. @type {Map<string, string>} */
const colorSlot = new Map();
const slotNow = (/** @type {Layer} */ layer) => ((places) => ((chosenSlot) => (places.some(([key]) => key === chosenSlot) ? /** @type {string} */ (chosenSlot) : places[0]?.[0] ?? "color"))(colorSlot.get(`${chart.id}:${layer.id}`)))(colorSlots(layer));
/** What a layer can take its colors from: another layer, none taking its own (no loop of two). @param {Layer} layer */
const colorSources = (layer) => chart.layers.filter((other) => other.id !== layer.id && other.colorFrom !== layer.id && !placedOn(other) && !heatLike(other));
/** What each way of coloring does, said beside it. */
const MODE_ASIDES = { one: "The same all along", sides: "Above and below its base", momentum: "Growing and fading", levels: "Each level's color above it", layers: "Each one's own, where it's over", gradient: "By its value, its lowest to its highest", from: "Another layer's, by date" };
/**
 * A layer's ways of coloring as choices (its menu's, ⌘K's): those its type has, each said; by its levels once it has
 * some; from a layer, which (a page of them). @returns {Action[]}
 */
function colorModeChoices(/** @type {string} */ id) {
  const layer = layerById(id);
  if (!layer) return [];
  const mode = colorModeOf(layer);
  return modesOf(layer).map((each) => ({
    key: `mode:${each}`, icon: ((copy) => (setColorMode(copy, each), modeSwatch(copy, each)))(structuredClone(layer)), label: modeName(each, layer), aside: each === "sides" && layer.type === "trend" ? "Resistance and support" : each === "sides" && layer.type === "cloud" ? "The one over's, parting where they cross" : /** @type {Record<string, string>} */ (MODE_ASIDES)[each], current: mode === each,
    ...(each === "from" ? (colorSources(layer).length ? { page: () => choicePage("Colors from", () => fromChoices(id)) } : { disabled: "There's no other layer to take them from" })
      : each === "levels" && !layer.levels?.length ? { disabled: "Add a level first" }
      : { run: () => editById(id, (item) => setColorMode(item, each)), look: (/** @type {Chart} */ doc) => ((item) => item && setColorMode(item, each))(doc.layers.find((other) => other.id === id)) }),
  }));
}
/** The layers another can take its colors from, as choices. @returns {Action[]} */
function fromChoices(/** @type {string} */ id) {
  const layer = layerById(id);
  if (!layer) return [];
  const gives = { one: "Its color", sides: "Its two colors", momentum: "Its four colors", levels: "Its levels' colors", layers: "Its layers' colors", from: "The colors it takes" };
  return colorSources(layer).map((other) => ({ key: `from:${other.id}`, icon: letterChip(other), label: labelOf(other), aside: /** @type {Record<string, string>} */ (gives)[colorModeOf(other)], current: layer.colorFrom === other.id, run: () => editById(id, (item) => setColorMode(item, "from", other.id)), look: (/** @type {Chart} */ doc) => ((item) => item && setColorMode(item, "from", other.id))(doc.layers.find((one) => one.id === id)) }));
}
/** The color under a layer's levels, from its select: Bitview's colors, Custom. */
function underColorMenu(/** @type {HTMLElement} */ button) {
  const layer = layerById(selected);
  if (!layer) return;
  const id = layer.id;
  colorMenu(button, colorOf(layer, "under"), "layer:under", (color) => {
    refocus('#inspector [data-color-select="under"]');
    editById(id, (item) => setColor(item, "under", color));
  }, () => ((now) => now && underColorMenu(now))($("color-select-under")), "Under its levels");
}
/**
 * A color from a select, its dropdown: Bitview's colors (a grid, at a glance), then Custom (any other: the surface, a
 * page of its own), the one it is ticked. Back from Custom, `again` opens it as things are then (a pick there redraws
 * it).
 */
function colorMenu(/** @type {HTMLElement} */ anchor, /** @type {string} */ current, /** @type {string} */ target, /** @type {(color: string) => void} */ apply, again = () => colorMenu(anchor, current, target, apply), title = "") {
  const own = !SWATCHES.includes(current);
  vividAhead();
  // (A dropdown as wide as its select, under it: Bitview's colors at a glance, each named on hover.)
  // (A phone's sheet, its width: the swatches across it.)
  const colors = `<div class="color-drop"${phone.matches ? "" : ` style="width: ${Math.max(232, Math.round(anchor.getBoundingClientRect().width) - 8)}px"`}>${swatchGrid(current)}</div><div class="menu-rule"></div>`;
  const custom = `<button type="button" data-pick-custom="1" aria-checked="${own}" aria-haspopup="menu">${tick(own)}<i class="swatch" style="${own ? `--c:${escape(cssColor(current))}` : "background: conic-gradient(from 90deg, #f43f5e, #f59e0b, #84cc16, #10b981, #06b6d4, #3b82f6, #8b5cf6, #ec4899, #f43f5e)"}"></i>Custom${own ? `<span>${escape(ink(current))}</span>` : ""}</button>`;
  // (A phone's sheet says what it's the color of: one of several, which.)
  showMenu(anchor, `${title && phone.matches ? `<div class="menu-group">${escape(title)}</div>` : ""}${colors}${custom}`, (hit) => {
    if (hit.pickCustom) colorPane(anchor, current, target, again);
    else if (hit.pickColor && hit.pickColor !== current) apply(hit.pickColor);
  }, null, anchor.closest("#belt") ? beltSide() : "");
}
/** A drawing's color, from the belt: a layer's picker, beside the belt; back from Custom, as things are then (the belt redrawn). */
function penColors(/** @type {HTMLElement} */ button) {
  const look = lookFor("color");
  if (look) colorMenu(button, look.color, "pen", (color) => {
    setLook("color", color);
    refocus(`#${button.id}`);
  }, () => ((now) => now && penColors(now))($(button.id)));
}
/**
 * Each window of `size` along `line` (its first window at the line's start), its minimum (`lowest`) or its mean,
 * written into `into` from `at`, a `step` apart: a running sum, a queue of the window's rising minima, one step a
 * point whatever the window's width.
 * @param {Float64Array} line @param {number} size @param {boolean} lowest @param {Float64Array} into @param {number} at @param {number} step
 */
function windowed(line, size, lowest, into, at, step) {
  if (!lowest) {
    let sum = 0;
    for (let k = 0; k < size; k++) sum += line[k];
    for (let k = 0; k + size <= line.length; k++) {
      into[at + k * step] = sum / size;
      if (k + size < line.length) sum += line[k + size] - line[k];
    }
    return;
  }
  const queue = new Int32Array(line.length);
  let head = 0, tail = 0;
  for (let k = 0; k < line.length; k++) {
    while (tail > head && line[queue[tail - 1]] >= line[k]) tail--;
    queue[tail++] = k;
    if (queue[head] <= k - size) head++;
    if (k >= size - 1) into[at + (k - size + 1) * step] = line[queue[head]];
  }
}
/**
 * One pass over the surface: each window folded across hue (12° either side, around the wheel), then down lightness
 * (0.03 either side, held at its ends), a line a step.
 * @param {Float64Array} source @param {boolean} lowest
 */
function* fold(source, lowest) {
  const rows = VIVID_ROWS, across = new Float64Array(rows * 360), out = new Float64Array(rows * 360);
  const line = new Float64Array(360 + 24), column = new Float64Array(rows + 12);
  for (let r = 0; r < rows; r++) {
    for (let k = 0; k < line.length; k++) line[k] = source[r * 360 + ((k - 12 + 360) % 360)];
    windowed(line, 25, lowest, across, r * 360, 1);
    yield;
  }
  for (let c = 0; c < 360; c++) {
    for (let k = 0; k < column.length; k++) column[k] = across[clamp(k - 6, 0, rows - 1) * 360 + c];
    windowed(column, 13, lowest, out, c, 360);
    yield;
  }
  return out;
}
/**
 * The surface's grid made in idle moments, a few lines at a time, none of it a long task (folded whole in one go, it
 * had held a phone without its JIT (Lockdown Mode) half a second), till it's made: once a color can be picked soon
 * (a layer's settings shown, a color menu open), not at every start (only a picker shows it).
 */
let vividStarted = false;
const vividAhead = () => !vividGrid && !vividStarted && ((vividStarted = true), whenIdle(vividSoon));
function vividSoon(/** @type {{ timeRemaining: () => number }} */ deadline) {
  // (Made: the surfaces shown meanwhile, waiting for it, drawn.)
  inSlices(deadline, () => Boolean(vividGrid || (vividWork.next(), vividGrid)) && (drawPlanes(), true), vividSoon);
}
/**
 * The grid made sooner, a surface showing and waiting for it: a few lines each frame (8 ms of one), not in idle
 * moments (Safari has none to say: its are a slice every 40 ms, seconds of them for this). Made, the surfaces drawn.
 */
let vividRushing = false;
function vividNow() {
  if (vividGrid || vividRushing) return;
  vividRushing = vividStarted = true;
  const step = () => {
    const until = performance.now() + 8;
    while (!vividGrid && performance.now() < until) vividWork.next();
    if (!vividGrid) return void requestAnimationFrame(step);
    vividRushing = false;
    drawPlanes();
  };
  requestAnimationFrame(step);
}
function vivid(l, h) {
  const rows = VIVID_ROWS;
  // (What isn't made yet in idle moments, made now: see vividSoon.)
  while (!vividGrid) vividWork.next();
  const g = vividGrid, r = clamp((1 - l) / 0.005, 0, rows - 1), c = ((h % 360) + 360) % 360;
  const r0 = Math.floor(r), r1 = Math.min(rows - 1, r0 + 1), c0 = Math.floor(c) % 360, c1 = (c0 + 1) % 360, fr = r - r0, fc = c - Math.floor(c);
  const at = (row, col) => g[row * 360 + col];
  return (at(r0, c0) * (1 - fc) + at(r0, c1) * fc) * (1 - fr) + (at(r1, c0) * (1 - fc) + at(r1, c1) * fc) * fr;
}
/**
 * The surface drawn: every hue across, lightness down, each as vivid as the surface is there (and never more than
 * sRGB holds, between the grid's points), so every point is a color the chart can draw. Drawn at half its CSS size
 * and stretched (it's all gradient), kept by size: nothing redraws it but a resize.
 */
const planes = new Map();
const drawPlanes = () => all$(".plane").forEach(drawPlane);
function drawPlane(/** @type {HTMLElement} */ plane) {
  const canvas = /** @type {HTMLCanvasElement | null} */ (plane.querySelector("canvas"));
  if (!canvas || !plane.clientWidth) return;
  // (Its grid not made yet, made in idle moments, not now: what it's in shows at once, the surface as it's made,
  // faded in (made at once, a phone without its JIT had held half a second first). A pick reads the grid at once.)
  plane.toggleAttribute("data-waiting", !vividGrid);
  if (!vividGrid) return void vividNow();
  // (Its colors worked out at half its size, stretched: a smooth field, the same to the eye, a quarter of the work (a
  // phone without its JIT: ~75 ms at its size, on the tap that showed it). A pick reads the grid, not the canvas.)
  const width = Math.ceil(plane.clientWidth / 2), height = Math.ceil(plane.clientHeight / 2), key = `${width}x${height}`;
  canvas.width = width;
  canvas.height = height;
  const context = /** @type {CanvasRenderingContext2D} */ (canvas.getContext("2d"));
  if (!planes.has(key)) {
    const image = context.createImageData(width, height), data = image.data;
    for (let y = 0; y < height; y++) {
      const l = lightAt((y + 0.5) / height);
      for (let x = 0; x < width; x++) {
        const u = (x + 0.5) / width, h = hueAt(u);
        // (Its limit searched for only where it's out of sRGB: the surface's kept in it, nearly always is.)
        const chroma = vivid(l, h), wanted = oklchToLinear(l, chroma * keepAt(u), h), i = (y * width + x) * 4;
        const rgb = inside(wanted) ? wanted : oklchToLinear(l, Math.min(chroma, limit(l, h, 12)) * keepAt(u), h);
        data[i] = gamma(clamp(rgb[0], 0, 1)) * 255;
        data[i + 1] = gamma(clamp(rgb[1], 0, 1)) * 255;
        data[i + 2] = gamma(clamp(rgb[2], 0, 1)) * 255;
        data[i + 3] = 255;
      }
    }
    if (planes.size > 24) planes.clear();
    planes.set(key, image);
  }
  context.putImageData(planes.get(key), 0, 0);
}
/**
 * What a surface picks for, by its name: the layers in hand's colors (the selected one's, or those picked, together: see
 * inHand; "layer:" and which: "color", "colorBelow"), the belt's (the pen's, and the selected
 * drawings') ("pen"), a line across's ("line:" and its key). Its color now, and how a pick sets it: one undo step a
 * drag (or run of keys), the popover it's in left open.
 * @returns {{ color: string, set: (color: string) => void } | null}
 */
function planeTarget(/** @type {string} */ name) {
  const layer = layerById(selected), layers = inHand();
  if (name.startsWith("layer:") && layers.length) {
    // (Those in hand now, kept: a pick is set a frame later, whatever's selected then. Their color, theirs alike;
    // else none, so any pick differs: one matching the first's sets the others too.)
    const key = name.slice(6), ids = new Set(layers.map((item) => item.id));
    // (A gradient's color: its knob and bar following as it's dragged, the panel held still meanwhile.)
    return { color: sharedBy(layers, (item) => colorOf(item, key)) ?? "", set: (color) => (commit((doc) => doc.layers.forEach((item) => ids.has(item.id) && setColor(item, key, color)), { coalesce: `${key}:${[...ids].join()}`, keepMenu: true }), key.startsWith("stop") && showStop(Number(key.slice(4)))) };
  }
  if (name === "pen") {
    const look = lookFor("color");
    return look ? { color: look.color, set: (color) => setLook("color", color, "color", true) } : null;
  }
  const key = name.startsWith("line:") ? name.slice(5) : "";
  if (key && layer && lineAt(layer, key)) return { color: lineLook(layer, key).color, set: (color) => setLineLook(layer.id, key, "color", color, { coalesce: `line:${key}:${layer.id}`, keepMenu: true }) };
  return null;
}
/** A surface by what it picks for (found again each time: the inspector redraws under a drag). */
const planeOf = (/** @type {string} */ name) => /** @type {HTMLElement | null} */ (document.querySelector(`.plane[data-plane="${CSS.escape(name)}"]`));
/** A color picked, one frame at a time while dragging; the whole drag (or run of keys) is one undo step. */
let pickFrame = 0, picking = /** @type {{ name: string, color: string, doc: Chart, target: { color: string, set: (color: string) => void } | null } | null} */ (null);
function pick(/** @type {string} */ name, /** @type {string} */ color) {
  // (What it's for, taken now, on this chart: by its frame, another may be selected, or open.)
  picking = { name, color, doc: chart, target: picking?.name === name && pickFrame ? picking.target : planeTarget(name) };
  pickFrame ||= requestAnimationFrame(applyPick);
}
function applyPick() {
  pickFrame = 0;
  if (picking?.doc === chart && picking.target && picking.target.color !== picking.color) picking.target.set(picking.color);
}
/** The color waiting for its frame, set now (a drag ending: in its step). */
function flushPick() {
  if (!pickFrame) return;
  cancelAnimationFrame(pickFrame);
  applyPick();
}
/** The color at a point of the surface (a hue and a lightness): at the surface's chroma there. */
function colorAt(u, t) {
  const l = lightAt(clamp(t, 0, 1)), h = hueAt(clamp(u, 0, 1));
  // At the very ends, white and black: no chroma to speak of.
  const c = l <= 0.001 || l >= 0.999 ? 0 : Math.min(vivid(l, h), maxChroma(l, h)) * keepAt(clamp(u, 0, 1));
  return kept(c < 0.0005 ? `oklch(${+l.toFixed(3)} 0 0)` : `oklch(${+l.toFixed(3)} ${+c.toFixed(4)} ${+h.toFixed(1)})`);
}
/** Where each color was picked this session: a gray (white, black) is the same anywhere along its row. */
const pickedAt = new Map();
/** Picks the color at a point of a surface, remembering the point. */
function pickPoint(/** @type {string} */ name, u, t) {
  const color = colorAt(u, t), place = { u: clamp(u, 0, 1), t: clamp(t, 0, 1) }, plane = planeOf(name);
  if (pickedAt.size > 500) pickedAt.clear();
  pickedAt.set(color, place);
  // The ring follows at once, even along white or black, where the color stays the same as it moves (in a popover,
  // nothing redraws it: its color too).
  plane?.style.setProperty("--x", String(place.u));
  plane?.style.setProperty("--y", String(place.t));
  plane?.querySelector(".ring")?.setAttribute("style", `--c:${cssColor(color)}`);
  const said = planeSays(color, place.u);
  plane?.setAttribute("aria-valuenow", String(said.now));
  plane?.setAttribute("aria-valuetext", said.text);
  // (Its name, and Bitview's colors, none of them ringed now, follow too.)
  const field = plane?.closest(".field, .color-editor"), named = field?.querySelector(".color-now"), hex = /** @type {HTMLInputElement | null | undefined} */ (field?.querySelector(".hex"));
  /** @type {HTMLElement | null | undefined} */ (field?.querySelector('.slot-chip[aria-pressed="true"]'))?.style.setProperty("--c", cssColor(color));
  if (named) named.textContent = colorName(color);
  if (hex && document.activeElement !== hex) hex.value = ink(color);
  for (const dot of field ? all$(".swatch-grid [aria-checked='true']", field) : []) dot.setAttribute("aria-checked", "false");
  pick(name, color);
}
/**
 * Where a color is on the surface: where it was picked; else by its hue, or in the fade to the grays by the share of
 * chroma it keeps; a gray (or nearly) amid the grays. A color from elsewhere goes where its hue and lightness are;
 * one of Bitview's, where it shows (in the light, its tone).
 */
function placeOf(color) {
  const spot = pickedAt.get(color);
  if (spot) return spot;
  const [l, c, h] = shownOklch(color), t = heightOf(l), fade = (1 - HUES) * 0.85;
  if (h >= 359) {
    const keep = clamp(c / Math.max(1e-6, Math.min(vivid(l, h), maxChroma(l, h))), 0, 1);
    if (keep < 0.999) return { u: HUES + (fade * Math.acos(2 * keep - 1)) / Math.PI, t };
  }
  // Amid the grays: the middle of where chroma is all gone.
  if (c < 0.02) return { u: (HUES + fade + 1) / 2, t };
  return { u: (Math.min(h, 359.9) / 359.9) * HUES, t };
}
/** The color being picked: the one waiting for its frame, else its target's now (several apart, the first's: as its surface shows). */
const pickedOf = (/** @type {string} */ name) => (pickFrame && picking?.name === name ? picking.color : planeTarget(name)?.color || (name.startsWith("layer:") && inHand()[0] ? colorOf(/** @type {Layer} */ (inHand()[0]), name.slice(6)) : "gray"));
function pickAt(/** @type {string} */ name, clientX, clientY) {
  const plane = planeOf(name);
  if (!plane || !planeTarget(name)) return;
  const box = plane.getBoundingClientRect();
  // The outermost pixel and a half of each side is its edge: white, black and the purest grays need no aim.
  const snap = (v, size) => (v < 1.5 / size ? 0 : v > 1 - 1.5 / size ? 1 : v);
  pickPoint(name, snap((clientX - box.left) / box.width, box.width), snap((clientY - box.top) / box.height, box.height));
}
/** What went wrong with a series layer, and the way out: another interval, its day's series, or a retry. */
function errorBox(result, /** @type {string} */ name) {
  if (result.info && !result.info.indexes.includes(chart.index)) {
    const dated = datedIntervals(result.info);
    // (The nearest of them, by how far it is from this one.)
    const away = (/** @type {string} */ id) => Math.abs(INTERVAL_ORDER.indexOf(id) - INTERVAL_ORDER.indexOf(chart.index)), nearest = dated.length ? dated.reduce((best, id) => (away(id) < away(best) ? id : best)) : "";
    if (dated.length) return `<div class="error-box">No ${escape(INTERVAL_LABEL[chart.index])} data; it has ${escape(dated.map((id) => INTERVAL_LABEL[id]).join(", "))}.<button type="button" class="btn small" data-index="${nearest}">Switch to ${escape(INTERVAL_LABEL[nearest])}</button></div>`;
    const day = dailyOf(name);
    return `<div class="error-box gone"><p>${BY_BLOCK}.</p><div class="replacement">${day ? `<span class="closest">By day: <code>${escape(day)}</code></span><button type="button" class="btn small" data-replace="${escape(day)}">Replace</button>` : ""}<button type="button" class="btn ${day ? "ghost " : ""}small" data-action="replace">${day ? "Find another" : "Find one"}</button></div></div>`;
  }
  // Renamed or removed in a newer Bitview: the closest series is offered, or another one to pick.
  if (result.missing) return `<div class="error-box gone"><p>Bitview has no series by this name: renamed, removed, or mistyped.</p>
          <div id="replacement" class="replacement"><span class="shimmer">Looking for the closest one…</span></div></div>`;
  return `<div class="error-box">${escape(result.error)}.<button type="button" class="btn small" data-action="retry">Retry</button></div>`;
}
/** A series by block, not added: why, and its day's series to use instead, if there's one (on this chart, while it's open). */
function blockOnly(/** @type {string} */ name, /** @type {(day: string) => void} */ use) {
  const day = dailyOf(name), target = chart;
  notice(`${name} is only available by block, and Studio charts by date`, day ? { label: `Use ${day}`, run: () => chart === target && !home && use(day) } : undefined);
}
/** The series closest to a gone one, by its words; asked once per name. */
const nearestTo = new Map();
/** @returns {Promise<string | null>} */
function closestSeries(/** @type {string} */ name) {
  const words = name.split("_").filter((word) => word.length > 2);
  // Close means sharing at least half of its words ("close" counts for "closed").
  const near = (other) => {
    const theirs = other.split("_");
    return words.filter((word) => theirs.some((their) => their.startsWith(word) || (their.length > 2 && word.startsWith(their)))).length * 2 >= words.length;
  };
  if (!nearestTo.has(name)) nearestTo.set(name, findSeries(name.replace(/_/g, " "), 8).then((names) => (indexCatalog(), names.find((other) => isSeriesName(other) && other !== name && !gone.has(other) && !byBlock.has(other) && near(other)) ?? null), () => null));
  return nearestTo.get(name);
}
async function suggestReplacement(layer) {
  const name = layer.series;
  const best = await closestSeries(name);
  const box = $("replacement");
  if (!box || layerById(selected)?.series !== name) return;
  box.innerHTML = `${best ? `<span class="closest">Closest: <code>${escape(best)}</code></span><button type="button" class="btn small" data-replace="${escape(best)}">Replace</button>` : `<span class="closest">Nothing close by name.</span>`}<button type="button" class="btn ${best ? "ghost " : ""}small" data-action="replace">${best ? "Find another" : "Find one"}</button>`;
}
/**
 * Another series for a layer, from ⌘K: opened in the catalog where its series is, on it, its neighbors around (a
 * search there looks in that folder, then everywhere); by its name when its place isn't known.
 */
function changeSeries(id) {
  const layer = layerById(id);
  if (!layer?.series) return;
  // A place the catalog no longer has (a folder renamed since) isn't known either.
  const keys = layer.path?.split(".") ?? [];
  openPalette(keys.length && nodeAt(keys) != null ? { replace: id, path: keys.slice(0, -1), landOn: keys.at(-1) } : { replace: id, query: layer.series.replace(/_/g, " ") });
}
/** Each replacement asked for, by layer: the latest one's, applied; any earlier one's answer, dropped. */
const replacing = new Map();
/** Swaps a layer's series, keeping everything else (its letter, so formulas using it come back). */
async function replaceSeries(id, name) {
  const target = chart, run = (replacing.get(id) ?? 0) + 1;
  replacing.set(id, run);
  const current = layerById(id)?.series;
  if (!current || !isSeriesName(name) || name === current) return;
  // As Add does: a series counted by block has no dates to chart.
  retryInfo(name);
  indexCatalog();
  const meta = await info(name).catch(() => null);
  const layer = layerById(id);
  if (chart !== target || !layer?.series || replacing.get(id) !== run) return;
  if (meta && !datedIntervals(meta).length) return void blockOnly(name, (day) => replaceSeries(id, day));
  const was = labelOf(layer), old = layer.series;
  editById(id, (target) => {
    target.series = name;
    delete target.path;
    // (Its name the old series': the new one's shows; one given, kept.)
    if (target.label === humanize(old)) delete target.label;
    // (Candles of a series without them: a line, as one added would be.)
    if (target.type === "candles" && meta && !isOhlc(name)) reshape(target, (item) => (item.type = "line"));
  });
  remember(name);
  undoToast(`Replaced “${was}” with “${humanize(name)}”`);
}

/** What a formula reads, in words: this chart's layers by letter and name, and series by theirs. @param {Node} node */
const usesOf = (node) => [
  ...[...refsOf(node)].map((letter) => {
    const used = chart.layers.find((item) => item.letter === letter);
    return used ? `${letter} (${labelOf(used)})` : letter;
  }),
  ...refsOf(node, "written"),
  // (Each point's date and year: what the formula reads too.)
  ...[...VALUES].filter((name) => JSON.stringify(node).includes(`"k":"${name}"`)),
];
/** The formula editor watched for its width (one at a time: one replaced, let go of). @type {Element | null} */
let formulaFitted = null;
function updateFormula(layer) {
  const textarea = $("formula");
  if (!textarea) return;
  if (formulaFitted !== textarea) {
    if (formulaFitted) formulaFit.unobserve(formulaFitted);
    formulaFit.observe((formulaFitted = textarea));
  }
  const colorOf = (letter) => {
    const target = chart.layers.find((item) => item.letter === letter);
    return target && target !== layer ? ink(target.color) : null;
  };
  $("formula-mirror").innerHTML = highlightFormula(textarea.value, colorOf, (name) => gone.has(name));
  textarea.style.height = "auto";
  textarea.style.height = `${textarea.scrollHeight}px`;
  $("formula-mirror").scrollTop = textarea.scrollTop;
  const result = resolved?.results.get(layer.id), text = textarea.value;
  let message = "", node = null;
  try {
    node = parse(text);
    message = text.trim() === (layer.formula ?? "").trim() ? result?.detail ?? result?.error ?? "" : formulaError(text) || loopError(layer, text);
  } catch (error) {
    message = /** @type {Error} */ (error).message;
  }
  const status = $("formula-status");
  if (message || !node) status.textContent = message;
  else {
    // (What it is as applied: its unit once its values are in, "Loading" till then; another formula typed, what it
    // uses alone, its own to say once applied. A signal's, nothing more: how many times it marked is its Times'.)
    const uses = usesOf(node), applied = text.trim() === (layer.formula ?? "").trim();
    const what = !applied || layer.type === "signal" ? "" : !result ? "Loading" : UNIT_NAMES[unitFor(layer, result)] ?? "Number";
    status.textContent = `${uses.length ? `Uses ${uses.join(", ")}` : "A constant"}${what ? ` · ${what}` : ""}`;
  }
  status.classList.toggle("error", Boolean(message));
  $("formula-editor").classList.toggle("invalid", Boolean(message));
  // (Said to a screen reader too: the field is in error, its status says why.)
  if (message) $("formula")?.setAttribute("aria-invalid", "true");
  else $("formula")?.removeAttribute("aria-invalid");
  updateCallHelp();
  const label = $("label");
  if (label) label.placeholder = labelOf({ ...layer, label: "" });
}

// Suggestions at the caret: for a capital, the layers; for a lowercase word, the functions, then series by name.
const suggest = {
  items: /** @type {{ insert: string, title: string, text: string, series?: boolean }[]} */ ([]),
  active: 0,
  start: 0,
  controller: /** @type {AbortController | null} */ (null),
  /** Where esc put the popover away (see assistAway). */
  away: "",
  /** Just accepted: the word now there needs no list. */
  quiet: false,
};
/**
 * The list under the formula. `offered`: unasked, between words (what fits the argument at the caret), none chosen,
 * so ↵ stays the field's own until an arrow picks one; typed, the first is chosen. `more`: a closing line.
 * @param {typeof suggest.items} items
 */
function showSuggestions(items, { offered = false, more = "" } = {}) {
  const list = $("suggest");
  if (!list) return;
  // A row chosen with the arrows stays chosen as more arrive.
  const chosen = suggest.active > 0 ? items.findIndex((item) => item.insert === suggest.items[suggest.active]?.insert) : -1;
  suggest.items = items;
  suggest.active = offered ? -1 : Math.max(0, chosen);
  list.hidden = !items.length;
  list.innerHTML = items.map((item, i) => `<li role="option" id="suggest-${i}" ${item.series ? 'class="series"' : ""} data-suggest="${i}" aria-selected="${i === suggest.active}"><b>${escape(item.title)}</b>${item.text ? `<span>${escape(item.text)}</span>` : ""}</li>`).join("");
  const line = $("suggest-more");
  if (line) (line.textContent = items.length ? more : ""), (line.hidden = !(items.length && more));
  chooseSuggestion(suggest.active);
  updateCallHelp();
}
/**
 * The popover under the formula, in the page's top layer (the panel neither grows nor scrolls for it), placed at the
 * field: up while it holds something and the field has the focus, as tall as the window leaves it below the field.
 */
function placeAssist() {
  const assist = /** @type {HTMLElement | null} */ (document.querySelector(".assist")), editor = $("formula-editor");
  if (!assist || !editor) return;
  const rect = editor.getBoundingClientRect(), view = $("inspector").getBoundingClientRect();
  const bottom = viewport.offsetTop + viewport.height;
  const holds = [...assist.children].some((child) => !(/** @type {HTMLElement} */ (child).hidden));
  if (!holds || document.activeElement?.id !== "formula" || rect.bottom < view.top || rect.bottom > Math.min(view.bottom, bottom - 48)) return void (assist.matches(":popover-open") && assist.hidePopover());
  Object.assign(assist.style, { left: `${rect.left}px`, top: `${rect.bottom + 4}px`, width: `${rect.width}px`, maxHeight: `${bottom - rect.bottom - 12}px` });
  if (!assist.matches(":popover-open")) assist.showPopover();
}
/** The row chosen (-1: none), as the field tells assistive tech. */
function chooseSuggestion(/** @type {number} */ i) {
  suggest.active = i;
  for (const option of all$("[data-suggest]", $("suggest") ?? undefined)) option.setAttribute("aria-selected", String(Number(option.dataset.suggest) === i));
  const textarea = $("formula");
  if (textarea && i >= 0 && !$("suggest")?.hidden) textarea.setAttribute("aria-activedescendant", `suggest-${i}`);
  else textarea?.removeAttribute("aria-activedescendant");
  $("suggest")?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" });
}
/** Where the popover is (the layer, the text, the selection): esc puts it away until that changes. */
const assistKey = () => `${selected} ${$("formula")?.value ?? ""}\u0000${$("formula")?.selectionStart ?? 0} ${$("formula")?.selectionEnd ?? 0}`;
const assistAway = () => {
  if (suggest.away && suggest.away !== assistKey()) suggest.away = "";
  return suggest.away !== "";
};
/**
 * What completes the word before the caret: for a capital, this chart's layers (but `self`); for a lowercase word,
 * the functions, and series by name after them (`completeSeries`); for a span begun where a span goes, the usual
 * spans it begins. A word already whole needs nothing.
 * @param {Layer | null | undefined} self
 */
function completionsAt(text, caret, self) {
  // (A word from its start: not the tail of another, nor a span's unit, 1ha of 1halving.)
  const match = /(?<![\w.])[A-Za-z_][A-Za-z0-9_]*$/.exec(text.slice(0, caret));
  const token = match?.[0] ?? "";
  const start = match ? match.index : caret;
  /** @type {{ insert: string, title: string, text: string, layer?: string, series?: boolean }[]} */
  let items = [];
  if (/^[A-Z]/.test(token)) {
    items = formulaInputs(self).filter((item) => item.insert.startsWith(token));
    if (items.length === 1 && items[0].insert === token) items = [];
  } else if (token) items = Object.entries(FUNCTIONS).filter(([name]) => name.startsWith(token) && name !== token).map(([name, [, signature, what]]) => ({ insert: `${name}(`, title: signature, text: what }));
  // (A span begun (its number, its letter), the whole of its argument, where a span goes: the usual ones it begins, as
  // an empty one's offered, see fitsAt; none, as before: a number's (part of one: 1.5, 1e+2).)
  const begun = token || !/^\s*(?:[,)]|$)/.test(text.slice(caret)) ? null : /[(,]\s*(\d+[a-z]*)$/.exec(text.slice(0, caret)), call = begun && callAt(text, caret);
  const name = call?.f && argNames(call.f)[call.arg];
  const spans = begun && (name === "span" || name === "span?" || name === "by") ? spansFor(/** @type {string} */ (call?.f)).filter((item) => item.insert.startsWith(begun[1]) && item.insert !== begun[1]) : [];
  if (begun && spans.length) return { start: caret - begun[1].length, token: begun[1], items: spans };
  return { start, token, items };
}
/**
 * The series a word being typed in a formula completes to (two letters on, not a layer's letter), as the search finds
 * them best (a few letters of each of their words find a name too): `add`ed unless the word's one already. `holder`
 * keeps the search: one after it lets it go.
 * @param {{ controller: AbortController | null }} holder @param {string} token @param {(series: { insert: string, title: string, text: string, series: true }[]) => void} add
 */
function completeSeries(holder, token, add) {
  if (token.length < 2 || !/^[a-z_]/.test(token)) return;
  const controller = (holder.controller = new AbortController());
  findSeries(token, 8, controller.signal).then((names) => {
    const series = names.filter(isSeriesName).map((name) => /** @type {const} */ ({ insert: name, title: name, text: "", series: true }));
    if (!controller.signal.aborted && series.length && !series.some((item) => item.insert === token)) add(series);
  }, () => {});
}
function updateSuggestions() {
  const textarea = $("formula");
  if (!textarea || !$("suggest")) return;
  suggest.controller?.abort();
  suggest.active = 0;
  const { start, token, items } = completionsAt(textarea.value, textarea.selectionStart, layerById(selected));
  suggest.start = start;
  const quiet = suggest.quiet;
  suggest.quiet = false;
  if (assistAway()) return showSuggestions([]);
  // Between words, what fits the argument at the caret; a word just completed needs nothing.
  if (!token) {
    const fits = fitsAt(textarea.value, textarea.selectionStart, layerById(selected), textarea.selectionEnd);
    return showSuggestions(fits?.items ?? [], { offered: true, more: fits?.more });
  }
  if (quiet) return showSuggestions([]);
  showSuggestions(items);
  completeSeries(suggest, token, (series) => document.activeElement === textarea && showSuggestions([...items, ...series]));
}
/** What each argument of a function takes, by its name in the signatures. */
const ARGUMENTS = {
  x: "the values, as a layer's letter, a series by its name, or a formula",
  a: "a value, as a layer, a series, a formula or a number",
  b: "a value, as a layer, a series, a formula or a number",
  p: "a percent, from 0 to 100: 50 is the median, 95 near the top",
  span: "how far back, as 12h, 30d, 4w, 6mo or 2y (a bare number is days)",
  "span?": "how far back, as 12h, 30d, 4w, 6mo or 2y; left out (at 1D and up), all of history so far",
  condition: "where it holds: a comparison, or two joined (date >= 2020-05-11 & A > 1); not 0",
  "by?": "a value whose change starts another stretch, as halving_epoch or year; left out, only a gap does",
  by: "how far, later (earlier where it's negative): 12h, 30d or 2w by points; 6mo or 1y to the same date; 1halving as far into the next halving cycle as into its own",
  n: "a halving cycle: 0 the first (from the genesis block), then 1 from 2012, 2 from 2016, 3 from 2020, 4 from 2024",
  y: "a year, as 2021 (2009 the first, the genesis block's)",
  "at?": "the cycle or year whose start it's moved to, day for day (halving(A, 0, 2): the first cycle over the third's dates; year(A, 2021, 2024), 2021 over 2024's); left out, where it is",
};
/** The innermost call the caret is in, and which of its arguments it's at; null outside of one. */
function callAt(/** @type {string} */ text, /** @type {number} */ caret) {
  const open = /** @type {{ f: string | null, arg: number }[]} */ ([]);
  for (let i = 0; i < caret; i++) {
    if (text[i] === "(") {
      let end = i;
      while (end > 0 && /\s/.test(text[end - 1])) end--;
      let start = end;
      while (start > 0 && /[a-z0-9_]/.test(text[start - 1])) start--;
      // A whole word only: the tail of another (Asma, 12sma) isn't a call.
      const name = start > 0 && /[A-Za-z0-9_.]/.test(text[start - 1]) ? "" : text.slice(start, end);
      open.push({ f: /^[a-z_]/.test(name) && hasFunction(name) ? name : null, arg: 0 });
    } else if (text[i] === ")") open.pop();
    else if (text[i] === "," && open.length) open[open.length - 1].arg++;
  }
  return open.findLast((call) => call.f) ?? null;
}
/** The call or function name at the caret: arguments, the current one marked and explained, then what it does. */
function callHelp(/** @type {string} */ text, /** @type {number} */ caret) {
  const word = /(?<![A-Za-z0-9_.])[a-z_][a-z0-9_]*$/.exec(text.slice(0, caret))?.[0] ?? "";
  // (A value that's a function's name too, year, is the function only with its parenthesis.)
  const call = hasFunction(word) && !(VALUES.has(word) && text[caret] !== "(") ? { f: word, arg: -1 } : callAt(text, caret);
  if (!call?.f) return "";
  const [, , what, , , usual] = FUNCTIONS[call.f], names = argNames(call.f), at = names[call.arg];
  return `<div class="call-sig"><b>${escape(call.f)}</b>(${names.map((name, i) => `<span${i === call.arg ? ' class="on"' : ""}>${escape(name)}</span>`).join(", ")})</div>
          ${call.arg < 0 ? "" : `<p>${at ? `<b>${escape(at.replace("?", ""))}</b>: ${escape(at === "span?" && usual ? `${ARGUMENTS.span}; left out, ${usual} points (as it's usually counted)` : ARGUMENTS[/** @type {keyof typeof ARGUMENTS} */ (at)] ?? "")}` : `No more arguments: ${escape(call.f)} takes ${names.length} at most`}</p>`}
          <p class="call-what">${escape(capitalized(what.replace(/\s*\([^)]*\)$/, "")))}.</p>`;
}
/**
 * The usual spans a function's span takes at this interval, each in words: a window's, two points at least (one is the
 * series itself); a shift's, a step at least; what's loaded holds them. In points where they aren't days.
 * @param {string} f
 */
function spansFor(f) {
  const shift = ["change", "diff", "shift"].includes(f), loaded = FINE.has(chart.index) ? resolved?.times.length ?? 0 : Infinity;
  // (A shift by halvings, each cycle from its first point: not at a fine interval, which loads part of history.)
  const fit = SPANS.map(([span, words]) => /** @type {[string, string, number]} */ ([span, words, windowPoints(spanDays(span), chart.index)])).filter(([span, , points]) => (f === "flow" ? flowFits(spanDays(span), chart.index) : shift ? spanDays(span) >= 0.9 * INTERVAL_DAYS[chart.index] && !(f === "shift" && FINE.has(chart.index) && span.endsWith("halving")) : points >= 2) && points <= loaded);
  return fit.map(([span, words, points]) => ({ insert: span, title: span, text: chart.index === "day1" ? words : `${words} · ${grouped.format(points)} points at ${INTERVAL_LABEL[chart.index]}`, group: "Spans" }));
}
/** Percents a percentile often takes, and what each is. */
const PERCENTS = /** @type {[number, string][]} */ ([[5, "near the bottom"], [25, "the lower quartile"], [50, "the median"], [75, "the upper quartile"], [95, "near the top"]]);
/**
 * What fits the argument at the caret, where it's empty: for values, this chart's layers (but `self` and those that
 * read it: a loop) and a few series (its formulas', the recent ones, the templates'), then how many more a search
 * finds; for a span, the usual ones a window holds two points of at this interval (a shift, one step: change(A, 1d))
 * and that what's loaded holds, with their points; for p, the usual percents. Null anywhere else.
 * @param {Layer | null | undefined} self
 * @returns {{ items: { insert: string, title: string, text: string, layer?: string, series?: boolean, group?: string }[], more: string } | null}
 */
function fitsAt(/** @type {string} */ text, /** @type {number} */ caret, self, end = caret) {
  // Where a value starts and nothing's there yet: after "(", "," or an operator, up to a ",", a ")" or the end.
  if (!/[-+*/^(,×÷−]\s*$/.test(text.slice(0, caret)) || !/^\s*(?:[,)]|$)/.test(text.slice(end))) return null;
  const call = callAt(text, caret);
  if (!call?.f) return null;
  const name = argNames(call.f)[call.arg];
  if (name === "x" || name === "a" || name === "b") {
    const layers = formulaInputs(self);
    // A series drawn as a layer is its letter; an open-high-low-close one reads as its close (offered as itself).
    const drawn = new Set(chart.layers.map((layer) => layer.series)), recommended = recommendedCards.flatMap(({ doc }) => doc.layers.flatMap((layer) => (layer.series ? [layer.series] : [])));
    const names = [...new Set([...seriesIn(chart), ...recent(), ...recommended])].filter((name) => isSeriesName(name) && !gone.has(name) && !drawn.has(name) && !isOhlc(name)).slice(0, 5);
    return { items: [...layers, ...names.map((name) => ({ insert: name, name, title: name, text: humanize(name), series: true }))], more: seriesCount > names.length ? `${grouped.format(seriesCount - names.length)} more series: type a few letters` : "" };
  }
  if (name === "span" || name === "span?" || name === "by") return { items: spansFor(call.f), more: "" };
  if (name === "p") return { items: PERCENTS.map(([p, what]) => ({ insert: String(p), title: String(p), text: what, group: "Percents" })), more: "" };
  return null;
}
/**
 * The letters a layer of a chart (`doc`) reads itself: its formula's; a layer made from layers' (a cloud's, a trend's,
 * its reference with them). @param {Layer} layer @param {Chart} doc
 */
function lettersRead(layer, doc) {
  const letters = new Set();
  if (layer.formula !== undefined) try { for (const letter of refsOf(parse(layer.formula))) letters.add(letter); } catch {}
  // (A signal's ref is the layer it's marked on, not one it reads; a trend's, the one that flips it: read.)
  for (const id of [...(layer.from ?? []), ...(layer.ref && !placedOn(layer) ? [layer.ref] : [])]) {
    const made = doc.layers.find((item) => item.id === id);
    if (made) letters.add(made.letter);
  }
  return letters;
}
/** The letters a layer reads, through those it reads too, in its chart (the open one unless said). @param {Layer} layer */
function readsOf(layer, doc = chart) {
  const seen = new Set(), queue = [layer];
  while (queue.length) {
    for (const letter of lettersRead(/** @type {Layer} */ (queue.pop()), doc)) {
      if (seen.has(letter)) continue;
      seen.add(letter);
      const next = doc.layers.find((item) => item.letter === letter);
      if (next) queue.push(next);
    }
  }
  return seen;
}
/**
 * The layers a formula can read, as it offers them: not its own, nor a cloud (a band, not a line), nor one reading it
 * (a loop). @param {Layer | null | undefined} self
 */
/**
 * What a formula's page offers while it's empty, a layer to add: the starters of one line (an average, a change, a
 * drawdown…) on the layer selected, else the first; with another of its unit (two prices), the one over the other;
 * those that can be computed at the chart's interval.
 * Each goes in the pane the page is for, if it's for one; else where derive puts a starter (over its layer, in its pane
 * on its side, what's in its unit; else in a new pane): ↵ adds it, ⇥ writes it in, to change first.
 */
function recipesFor() {
  const inputs = chart.layers.filter((layer) => layer.type !== "cloud" && layer.type !== "signal");
  const on = inputs.find((layer) => layer.id === selected) ?? inputs[0];
  if (!on) return [];
  // (Paired by units known: one just added isn't computed yet.)
  const unitOf = (/** @type {Layer} */ layer) => ((result) => (result ? unitFor(layer, result) : null))(resolved?.results.get(layer.id)), pair = unitOf(on) && inputs.find((other) => other !== on && unitOf(other) === unitOf(on));
  const recipe = (/** @type {string} */ formula, /** @type {string} */ text, over = false) => {
    const [into, side] = palette.into ? [palette.into === "new" ? null : palette.into, formulaSide(palette.into)] : over && !onStack(on) ? [on.pane, on.side ?? null] : [null, null];
    return { insert: formula, title: formula, text, recipe: true, into, side };
  };
  return [
    ...(pair ? [recipe(`${on.letter} / ${pair.letter}`, `${labelOf(on)} over ${labelOf(pair)}`)] : []),
    ...STARTERS.filter((starter) => starter.formulas.length === 1).map((starter) => recipe(starter.formulas[0].replaceAll("X", on.letter), `${starter.label} of ${labelOf(on)}`, starter.over)),
  ].filter((item) => !unfitHere([item.insert]));
}
const formulaInputs = (self) => chart.layers.filter((layer) => layer !== self && layer.type !== "cloud" && (!self || !readsOf(layer).has(self.letter))).map((layer) => ({ insert: layer.letter, title: layer.letter, text: labelOf(layer), layer: layer.id }));
/** Whether a formula's only fault is that it isn't finished. */
const isUnfinished = (/** @type {string} */ text) => {
  try {
    parse(text);
    return false;
  } catch (error) {
    return Boolean(/** @type {Error & { unfinished?: true }} */ (error).unfinished);
  }
};
/** ⌘K's formula line, outside of a call: how a formula reads. */
const FORMULA_GUIDE = `<p class="call-what">Layers by their letter, series by their name, numbers, + − × ÷ ^, comparisons (&gt; &lt; == !=, &amp;&nbsp;and&nbsp;|), date and dates (2020-05-11), and functions.</p><p class="call-what">Inside a function's parentheses, its help shows here.</p>`;
/**
 * The helper at the field being typed in, as the caret moves: under the inspector's formula, floating, over what
 * fits or completes the argument (one popover: nothing below it moves); in ⌘K, a line under the field for as long
 * as a formula is written there (in a call, its help; else how formulas read), so nothing jumps.
 */
let helping = 0;
function updateCallHelp() {
  helping ||= requestAnimationFrame(() => {
    helping = 0;
    drawCallHelp();
  });
}
function drawCallHelp() {
  const field = /** @type {HTMLInputElement | HTMLTextAreaElement | null} */ (document.activeElement);
  for (const [id, host] of [["formula", "formula-call"], ["palette-input", "palette-call"]]) {
    const element = $(host);
    if (!element) continue;
    const typing = field?.id === id && (id === "formula" ? !assistAway() : palette.formula);
    let help = typing && field ? callHelp(field.value, field.selectionStart ?? 0) : "";
    // Over the inspector's status line, it says what that line would: the formula's error (one that only says it
    // isn't finished, inside a call still open, it doesn't).
    const status = $("formula-status");
    if (help && id === "formula" && status?.classList.contains("error") && !isUnfinished(field?.value ?? "")) help += `<p class="call-error">${escape(status.textContent ?? "")}</p>`;
    const html = help || (id === "palette-input" && isOpen($("palette")) && palette.formula ? FORMULA_GUIDE : "");
    if (element.dataset.shown === html) continue;
    element.dataset.shown = html;
    element.innerHTML = html;
    element.hidden = !html;
  }
  placeAssist();
}
/** A completion as it goes in: a function's "(" left out where one follows already. */
const completion = (/** @type {string} */ insert, /** @type {string} */ after) => (insert.endsWith("(") && after.startsWith("(") ? insert.slice(0, -1) : insert);
function acceptSuggestion(i = suggest.active) {
  const item = suggest.items[i];
  const textarea = $("formula");
  if (!item || !textarea || suggest.start > textarea.selectionStart) return;
  // Over what's selected, and the rest of the word the caret is in.
  const end = textarea.selectionEnd + (/^\w*/.exec(textarea.value.slice(textarea.selectionEnd))?.[0].length ?? 0);
  textarea.setRangeText(completion(item.insert, textarea.value.slice(end)), suggest.start, end, "end");
  suggest.quiet = true;
  textarea.dispatchEvent(new Event("input", { bubbles: true }));
  textarea.focus();
}
function insertAtCaret(text) {
  const textarea = $("formula");
  if (!textarea) return;
  textarea.focus();
  textarea.setRangeText(text, textarea.selectionStart, textarea.selectionEnd, "end");
  textarea.dispatchEvent(new Event("input", { bubbles: true }));
}

/** The formula box is "typing" until it's left (↵, esc, elsewhere): a pause to think isn't an error yet. */
function markTyping() {
  $("formula-editor")?.classList.add("typing");
}
/** Now, or once every press under way is let go (its click done: see handsHeld). @param {() => void} run */
function afterPress(run) {
  if (!handsHeld.size) return run();
  afterHands.push(run);
}
/**
 * An edit to one layer (or drawing), by its id; `editLayer`, to the selected one. A drawing's leaves the series as
 * they are: only the drawings are drawn again.
 * @param {string} id @param {(item: any, doc: Chart) => void} edit @param {Parameters<typeof commit>[1]} [options]
 */
function editById(id, edit, options = undefined) {
  const drawing = Boolean(drawingById(id));
  commit((doc) => {
    const item = drawing ? doc.drawings?.find((other) => other.id === id) : doc.layers.find((other) => other.id === id);
    if (item) edit(item, doc);
  }, drawing ? { redraw: "none", ...options } : options);
}
/** An edit to the layers in hand (see inHand): the selected one's, or each picked one's, edited together, in one step. */
const editLayer = (/** @type {(item: any, doc: Chart) => void} */ edit, /** @type {Parameters<typeof commit>[1]} */ options = undefined) =>
  editingMany() ? commit((doc) => doc.layers.forEach((item) => layerPicks.has(item.id) && edit(item, doc)), options) : editById(selected, edit, options);
/** A series' name to the clipboard, and a word either way. @param {string} name */
const copyName = (name) => (navigator.clipboard?.writeText(name) ?? Promise.reject()).then(() => toast("Copied the series name"), () => notice("Couldn't copy the series name"));

/**
 * A number as typed in a field, or pasted from the app ("−2", "1,250.5"): its minus any dash, its spaces none;
 * commas group thousands ("1,000", "1,250,000"), but one with no dot and not followed by three digits is the
 * decimal one ("2,4", "1,25"); nothing typed, or not a number, NaN.
 */
const readNumber = (/** @type {string} */ text) => {
  const typed = text.replace(/[\s\u00a0\u2009\u202f]/g, "").replace(/^[−–—]/, "-");
  if (!typed) return NaN;
  // (One comma, its three digits a thousands' (1,500), but after a 0 a decimal's (0,500).)
  const decimalComma = !typed.includes(".") && (typed.match(/,/g)?.length ?? 0) === 1 && (!/,\d{3}$/.test(typed) || /^-?0,/.test(typed));
  return Number(decimalComma ? typed.replace(",", ".") : typed.replace(/,/g, ""));
};
/** How the selected layer's values read, from the inspector. */
const formatMenu = (/** @type {HTMLElement} */ anchor) => menuOf(anchor, formatChoices(selected));
/** How a layer's values read, as choices: automatic (its unit's), or one chosen. @returns {Action[]} */
function formatChoices(/** @type {string} */ id) {
  const layer = layerById(id);
  if (!layer) return [];
  const result = resolved?.results.get(layer.id), now = layer.format ?? "auto";
  return FORMATS.map(([kind]) => ({ key: kind, label: formatName({ ...layer, format: kind === "auto" ? undefined : kind }, result), current: now === kind, apart: FORMAT_GROUPS.has(kind), run: () => editById(id, (item) => (item.format = kind === "auto" ? undefined : kind)) }));
}

/** A layer shown or hidden; a cloud with its folded layers (see foldedInto), all as it goes. @param {string} id */
const toggleHidden = (id) => {
  const layer = layerById(id), members = layer ? [...foldedInto(chart)].filter(([, cloud]) => cloud === id).map(([member]) => member) : [];
  if (!members.length) return editById(id, (layer) => (layer.hidden = !layer.hidden || undefined), { look: true });
  const hide = !layer?.hidden || undefined;
  commit((doc) => doc.layers.forEach((other) => (other.id === id || members.includes(other.id)) && (other.hidden = hide)), { look: true });
};
const showEvery = () => commit((doc) => doc.layers.forEach((layer) => delete layer.hidden), { look: true });
/**
 * A line across (a level, by index, or a baseline's "base"), and its color and style unless set: a level its layer's
 * color in long dashes, a base gray in sparse dots.
 */
const lineAt = (layer, key) => (key === "base" ? layer.baseLine ?? {} : layer.levels?.[Number(key)]);
const lineDefaults = (layer, key) => (key === "base" ? { color: "gray", dash: LINE_STYLE.SparseDotted } : { color: layer.color, dash: LINE_STYLE.LargeDashed });
const lineLook = (layer, key) => ({ ...lineDefaults(layer, key), ...lineAt(layer, key) });
/** Why a line's value on the axis can't be turned on, said one way everywhere. */
const TAG_NEEDS_LINE = "Turn on the line to show its value on the axis";
/**
 * A value as its layer's levels name it: the highest level at or under it (as they color it, by levels; of two at one
 * value, the first), if it's named: its name, in its color; under them all, the layer's `underName`, in its own. Not
 * named there, none (the number, as ever).
 * @param {Layer} layer @param {number} value @returns {{ name: string, color: string } | null}
 */
function bandOf(layer, value) {
  const levels = layer.levels;
  if (!levels?.length || !Number.isFinite(value) || !(layer.underName || levels.some((level) => level.name))) return null;
  let at = -1;
  levels.forEach((level, i) => level.value <= value && (at < 0 || level.value > levels[at].value) && (at = i));
  if (at < 0) return layer.underName ? { name: layer.underName, color: layer.color } : null;
  return levels[at].name ? { name: /** @type {string} */ (levels[at].name), color: lineLook(layer, at).color } : null;
}
/** Whether a layer's values can read as names (a level named, or the band under them). @param {Layer} layer */
const named = (layer) => Boolean(layer.levels?.length && (layer.underName || layer.levels.some((level) => level.name)));
/** A line's style as said and drawn (none: its band alone). */
const lineStyleName = (/** @type {{ dash: number, line?: false }} */ look) => (look.line === false ? "None" : dashOf(look.dash)[1]);
const lineStyleGlyph = (/** @type {{ dash: number, line?: false }} */ look) => (look.line === false ? NO_LINE : ((d) => dashGlyph(d[2], d[3]))(dashOf(look.dash)));
/**
 * A line across's styles, as choices: none first (a level that only names or colors its band, a base not drawn),
 * then the line's dashes, the one in effect ticked; each kept with ↵, shown on the chart while highlighted (⌘K).
 * @param {string} id @param {string | number} key @returns {any[]} (Actions: typed as such, they'd refer to themselves)
 */
function lineChoices(id, key) {
  const layer = layerById(id);
  if (!layer || !lineAt(layer, key)) return [];
  const look = lineLook(layer, key), set = (/** @type {Chart} */ doc, /** @type {number | null} */ dash) => {
    lineLookOn(doc, id, key, "line", dash === null ? false : undefined);
    if (dash !== null) lineLookOn(doc, id, key, "dash", dash);
  };
  return /** @type {any[]} */ ([
    /** @type {any} */ ({ key: "none", icon: NO_LINE, label: "None", aside: "No line: its band alone", current: look.line === false, run: () => commit((doc) => set(doc, null)), look: (/** @type {Chart} */ doc) => set(doc, null) }),
    ...dashChoices(look.line === false ? undefined : look.dash, (style) => ({ run: () => commit((doc) => set(doc, style)), look: (/** @type {Chart} */ doc) => set(doc, style) })),
  ]);
}
/**
 * A layer's levels, as the bands they make, the way the chart has them: the highest first, each its color, its name
 * (what its values read as), the value it starts at (≥: up to the next), and its ⋯ (its line, its value on the axis,
 * removing it); under them, the band under them all (<: its color the layer's own; its name). The band under shows
 * where they're bands (coloring the layer, or named); lines across alone, their rows.
 * @param {Layer} layer @param {string} unit @param {any} result
 */
function levelsField(layer, unit, result) {
  const levels = layer.levels ?? [], banded = Boolean(layer.byLevels || named(layer));
  const order = levels.map((_, i) => i).sort((a, b) => levels[b].value - levels[a].value || a - b);
  const swatch = (/** @type {string} */ color) => `<i class="swatch" style="--c:${escape(cssColor(color))}"></i>`;
  const row = (/** @type {number} */ i) => {
    const level = levels[i], { color } = lineLook(layer, i), said = `At ${format(level.value, unitFor(layer, result))} and up`;
    return `<div class="level" role="listitem" data-level="${i}">
            <button type="button" class="cap small" data-line-color="${i}" aria-haspopup="menu" aria-label="${escape(said)}, its color: ${escape(colorName(color))}" data-tip="Color">${swatch(color)}</button>
            <input id="level-name-${i}" class="level-name named" style="--c:${escape(cssColor(color))}" enterkeyhint="done" aria-label="${escape(said)}, its name" placeholder="Name" type="text" autocomplete="off" spellcheck="false" maxlength="40" value="${escape(level.name ?? "")}" />
            <label class="number-field"><span class="sign" aria-hidden="true">≥</span><input id="level-${i}" class="number-input" enterkeyhint="done" aria-label="${escape(said)}, its value" type="text" autocomplete="off" spellcheck="false" value="${level.value}" /></label>
            <button type="button" class="cap small" data-level-menu="${i}" aria-haspopup="menu" aria-label="More for the level ${escape(said.toLowerCase())}" data-tip="More">${ICONS.more}</button>
          </div>`;
  };
  const lowest = levels.length ? Math.min(...levels.map((level) => level.value)) : 0;
  const under = banded && levels.length ? `<div class="level under" role="listitem">
            ${layer.byLevels ? `<button type="button" id="color-select-under" class="cap small" data-color-select="under" aria-haspopup="menu" aria-label="Under its levels, its color: ${escape(colorName(layer.color))}" data-tip="Color below all its levels">${swatch(layer.color)}</button>` : `<span class="cap small" role="img" aria-label="The line's own color" data-tip="The line's color (its levels color it: see Colors)">${swatch(layer.color)}</span>`}
            <input id="level-name-under" class="level-name named" style="--c:${escape(cssColor(layer.color))}" enterkeyhint="done" aria-label="Under its levels, its name" placeholder="Name" type="text" autocomplete="off" spellcheck="false" maxlength="40" value="${escape(layer.underName ?? "")}" />
            <span class="under-label"><span class="sign" aria-hidden="true">&lt;</span><span class="vh">Under </span>${lowest}</span>
            <i class="cap small spacer" aria-hidden="true"></i>
          </div>` : "";
  // (Their unit said once, by their name: every value's; none said before there are any.)
  return `<div class="field levels-field"><span><span>Levels${levels.length && fieldUnit(unit) ? `<span class="levels-unit"> · ${escape(fieldUnit(unit))}</span>` : ""}</span>${layer.byLevels ? '<span class="color-now">Colors the line</span>' : ""}<button type="button" class="reset" data-action="add-level" aria-label="Add a level" ${levels.length >= MAX_LEVELS ? `aria-disabled="true" data-tip="A layer holds up to ${MAX_LEVELS} levels"` : 'data-tip="Add a level at a round value above the latest"'}>${ICONS.plus}</button></span>${levels.length ? `<div class="levels" role="list" aria-label="Levels, the highest first">${order.map(row).join("")}${under}</div>` : ""}${((why) => (why ? `<p class="from-status" role="status">${escape(why)}.</p>` : ""))(levels.length ? acrossUnfit(chart, layer) || (modeOf(chart, layer) === 1 && levels.some((level) => level.value <= 0) ? "Those at 0 or under aren't drawn on a log scale" : "") : "")}</div>`;
}
/** A level's ⋯: its line (none, its band alone; a dash), its value on the axis, removing it. @param {string} id @param {number} i */
function levelRows(id, i) {
  const layer = layerById(id);
  if (!layer?.levels?.[i]) return [];
  const look = lineLook(layer, i);
  return /** @type {any[]} */ ([
    { key: "line", icon: lineStyleGlyph(look), label: "Line", aside: lineStyleName(look), page: () => ({ title: "Line", placeholder: "Search the choices", rows: () => lineChoices(id, i) }) },
    { key: "tag", icon: ICONS.label, label: "Value on the axis", current: Boolean(look.tag) && look.line !== false, ...(look.line === false ? { disabled: TAG_NEEDS_LINE } : {}), run: () => setLineLook(id, i, "tag", look.tag ? undefined : true) },
    { key: "remove", icon: ICONS.trash, label: "Remove the level", danger: true, run: () => {
      editById(id, (item) => withoutLevel(item, i));
      refocus('[data-action="add-level"]');
    } },
  ]);
}
function lineButtons(layer, key, label) {
  const look = lineLook(layer, key), { color, tag } = look, dashName = lineStyleName(look);
  return `<button type="button" class="cap small" data-line-tag="${key}" aria-pressed="${Boolean(tag) && look.line !== false}" ${look.line === false ? 'aria-disabled="true"' : ""} aria-label="${label}: its value on the axis" data-tip="${look.line === false ? "Turn on the line to show its value on the axis" : tag ? "Its value on the axis" : "Show its value on the axis"}">${ICONS.label}</button><button type="button" class="cap small" data-line-color="${key}" aria-haspopup="menu" aria-label="${label} color: ${escape(colorName(color))}" data-tip="Color"><i class="swatch" style="--c:${escape(cssColor(color))}"></i></button><button type="button" class="cap small" data-line-dash="${key}" aria-haspopup="menu" aria-label="${label} line style: ${dashName}" data-tip="Line style">${lineStyleGlyph(look)}</button>`;
}
/** A line's color (Bitview's) or style, from a menu under its button; picking its default forgets the choice. */
function lineMenu(anchor, key, part) {
  const layer = layerById(selected);
  if (!layer || !lineAt(layer, key)) return;
  const current = lineLook(layer, key)[part];
  const apply = (value) => {
    refocus(`[data-line-${part}="${key}"]`);
    setLineLook(selected, key, part, value);
  };
  if (part === "color") return colorMenu(anchor, current, `line:${key}`, apply, () => {
    const now = /** @type {HTMLElement | null} */ (document.querySelector(`#inspector [data-line-color="${key}"]`));
    if (now) lineMenu(now, key, "color");
  });
  menuOf(anchor, lineChoices(selected, key));
}
/**
 * The line styles, as choices, the one in effect ticked; `does`: what a style does (and shows, in ⌘K).
 * @returns {Action[]}
 */
const dashChoices = (/** @type {number | undefined} */ now, /** @type {(dash: number) => { run: () => void, look?: (doc: Chart) => void }} */ does) => DASHES.map(([dash, text, length, gap]) => ({ key: String(dash), icon: dashGlyph(length, gap), label: text, current: now === dash, ...does(dash) }));
/**
 * A line's Width and Line style rows (a layer's, a drawing's): `width` and `style` (see dashOf) as it is, `now` it as
 * it is when a page opens, `both` a choice's change (kept, and shown while highlighted: see bothOf).
 * @param {() => { width?: number, dash?: number }} now @param {(edit: (item: any) => void) => { run: () => void, look?: (doc: Chart) => void }} both
 */
const widthRow = (/** @type {number | undefined} */ width, now, both) => ({ key: "width", icon: ICONS.sliders, label: "Width", aside: `${width}px`, group: "Style", page: () => choicePage("Width", () => WIDTHS.map((each) => ({ key: String(each), icon: ICONS.sliders, label: `${each}px`, current: now().width === each, ...both((item) => (item.width = each)) }))) });
const dashRow = (/** @type {ReturnType<typeof dashOf>} */ style, /** @type {Parameters<typeof widthRow>[1]} */ now, /** @type {Parameters<typeof widthRow>[2]} */ both) => ({ key: "dash", icon: dashGlyph(style[2], style[3]), label: "Line style", aside: style[1], group: "Style", page: () => choicePage("Line style", () => dashChoices(now().dash, (dash) => both((item) => (item.dash = dash)))) });
/** A layer one place up or down among its pane's (listed, and drawn, before or after its neighbor). */
function moveLayer(id, by) {
  slideLayers(id, null, () => editById(id, (layer, doc) => {
    const mates = doc.layers.filter((item) => item.pane === layer.pane), other = mates[mates.indexOf(layer) + by];
    if (!other) return;
    const a = doc.layers.indexOf(layer), b = doc.layers.indexOf(other);
    [doc.layers[a], doc.layers[b]] = [doc.layers[b], doc.layers[a]];
  }));
}
/** A line across's color or style (a level's, by index, or the base's), on a layer; at its default, forgotten. */
const setLineLook = (id, key, part, value, /** @type {Parameters<typeof commit>[1]} */ options = undefined) => commit((doc) => lineLookOn(doc, id, key, part, value), options);
/** The same, on a doc as it is (a copy, to look at). @param {Chart} doc */
function lineLookOn(doc, id, key, part, value) {
  const target = doc.layers.find((item) => item.id === id);
  if (!target) return;
  if (key === "base") target.baseLine ??= {};
  const line = key === "base" ? target.baseLine : target.levels?.[Number(key)];
  if (!line) return;
  if (value === lineDefaults(target, key)[part] && !(target.byLevels && key !== "base" && part === "color")) delete line[part];
  else line[part] = value;
  if (key === "base" && !Object.keys(line).length) delete target.baseLine;
}
/**
 * A new level's value: round, off the line (on it, it'd hide under it), never one it has: the next multiple, at the
 * size of its latest value, above that value (82,416: 90,000; 1.54: 2), then the next free one above.
 */
function levelValue(/** @type {Layer} */ layer) {
  const value = valueAt(layer, { at: lastShown(), rest: true }), taken = new Set((layer.levels ?? []).map((level) => level.value));
  const step = Number.isFinite(value) && value !== 0 ? 10 ** Math.floor(Math.log10(Math.abs(value))) : 1;
  // (0.3 over 0.1 is 2.9999…: read at twelve figures, it's 3.)
  let at = Number.isFinite(value) ? (Math.floor(Number((value / step).toPrecision(12))) + 1) * step : 0;
  while (taken.has(Number(at.toPrecision(12)))) at += step;
  return Number(at.toPrecision(12));
}
/** A level for the selected layer (see levelValue), its field focused to adjust it (by a finger, shown: the keyboard on a tap). */
function addLevel() {
  const layer = layerById(selected);
  if (!layer) return;
  if ((layer.levels?.length ?? 0) >= MAX_LEVELS) return void notice(`A layer holds up to ${MAX_LEVELS} levels`);
  const level = levelValue(layer);
  editLayer((target) => withLevel(target, level));
  requestAnimationFrame(() => {
    const input = $(`level-${(layerById(selected)?.levels?.length ?? 1) - 1}`);
    if (byTouch) return void input?.scrollIntoView({ block: "nearest", behavior: reduced.matches ? "auto" : "smooth" });
    input?.focus();
    input?.select();
  });
}
const removeLayer = (/** @type {string} */ id) => removeLayers([id]);
/** Removes layers, in one step (one Undo), their drawings with them. @param {string[]} ids */
function removeLayers(ids) {
  const gone = chart.layers.filter((layer) => ids.includes(layer.id)), one = gone.length === 1;
  if (!gone.length) return;
  const goneName = labelOf(gone[0]), said = goneSaid(ids);
  const order = chart.layers.filter((item) => !ids.includes(item.id)), at = chart.layers.findIndex((item) => ids.includes(item.id)), after = chart.layers.slice(at).find((item) => !ids.includes(item.id)) ?? order.at(-1);
  // (Asked before: where the focus is, before what has it goes.)
  keepFocus(null, () => (after ? $("layer-list").querySelector(`[data-select="${CSS.escape(after.id)}"]`) : null), () => document.querySelector("#empty [data-action='add-series']"), () => $("plot-keys"));
  commit((doc) => (doc.layers = doc.layers.filter((item) => !ids.includes(item.id))));
  undoToast(`Removed ${one ? goneName : plural(gone.length, "layer")}${said}`);
}
/**
 * Layers about to go, as a toast says what goes with them: their drawings, and what needs them, named now (a cloud's
 * name is its layers'): formulas reading them, clouds and trends of them, signals marked on them, colors from them.
 * Their names kept, a formula reading one says it was removed. @param {string[]} ids
 */
function goneSaid(ids) {
  const gone = chart.layers.filter((layer) => ids.includes(layer.id)), letters = gone.map((layer) => layer.letter), one = gone.length === 1;
  for (const layer of gone) removedNames.set(`${chart.id}:${layer.letter}`, labelOf(layer));
  const reads = (/** @type {Layer} */ item) => {
    try {
      return item.formula !== undefined && ((refs) => letters.some((letter) => refs.has(letter)))(refsOf(parse(item.formula)));
    } catch {
      return false;
    }
  };
  const needs = (/** @type {Layer} */ item) => reads(item) || Boolean(item.from?.some((member) => ids.includes(member))) || ids.includes(item.ref ?? "") || ids.includes(item.colorFrom ?? "");
  const usedBy = chart.layers.filter((item) => !ids.includes(item.id) && needs(item)).map(labelOf), sketched = chart.drawings?.filter((drawing) => ids.includes(drawing.layer)).length ?? 0;
  return `${sketched ? ` and ${one ? "its" : "their"} ${plural(sketched, "drawing")}` : ""}${usedBy.length ? `; ${usedBy.join(", ")} ${usedBy.length === 1 ? "needs" : "need"} ${one ? "it" : "them"}` : ""}`;
}
/** A layer's right-click (its row's ⋯ by touch). */
const layerMenu = (/** @type {HTMLElement} */ anchor, /** @type {string} */ id, /** @type {{ x: number, y: number } | null} */ point = null) => menuOf(anchor, layerActions(id, "menu"), point);
/**
 * A layer's actions, where they show (`where`): "menu", its right-click; "panel", the list under its settings
 * (its fields have its series and its pane); "palette", ⌘K's page of it (its rows have them, with their values).
 * @param {string} id @param {"menu" | "panel" | "palette"} where @returns {Action[]}
 */
function layerActions(id, where) {
  const layer = layerById(id);
  if (!layer) return [];
  const menu = where === "menu", inPalette = where === "palette", named = turnsInto(layer), alone = aloneInPane(layer);
  const rows = [
    ...(where === "panel" ? [] : [{ key: "panel", icon: ICONS.sliders, label: "Settings", aside: inPalette ? "In the panel" : undefined, verb: "Open", run: () => select(id) }]),
    ...(menu && layer.series ? [{ key: "replace", icon: ICONS.swap, label: "Replace series", run: () => changeSeries(id) }] : []),
    ...(layer.type === "cloud" || layer.type === "signal" || heatLike(layer) ? [] : [{ key: "derive", icon: ICONS.formula, label: `New formula from ${layer.letter}`, aside: inPalette ? "An average, bands, a percentile…" : undefined, under: inPalette, page: () => listPage(`New formula from ${layer.letter}`, () => deriveChoices(id), "Add") }]),
    ...(layer.from || layer.type === "signal" || heatLike(layer) ? []
      : layer.series ? [{ key: "to-formula", icon: ICONS.formula, label: "Turn into a formula", aside: inPalette ? "Of its series, to write around it" : undefined, verb: "Turn", run: () => toFormula(id) }]
      : named.length === 1 ? [{ key: "to-series", icon: ICONS.series, label: "Turn into a series", aside: inPalette ? named[0] : undefined, mono: true, verb: "Turn", run: () => toSeries(id, named[0]) }]
      : named.length ? [{ key: "to-series", icon: ICONS.series, label: "Turn into a series", aside: inPalette ? `${named.length} series` : undefined, page: () => listPage("Turn into a series", () => seriesChoices(id), "Turn") }]
      : []),
    // (Its values as bubbles on another layer, picked: see Bubbles.)
    ...(layer.from || placedOn(layer) || heatLike(layer) || !markable(chart).some((other) => other.id !== id) ? [] : [{ key: "bubbles", icon: glyph("bubbles"), label: "As bubbles on…", aside: inPalette ? "Sized by its values" : undefined, page: () => choicePage("As bubbles on", () => markable(chart).filter((other) => other.id !== id).map((other) => ({ key: other.id, icon: letterChip(other), label: labelOf(other), run: () => editById(id, (item) => reshape(item, (one) => Object.assign(one, { type: "bubbles", ref: other.id }))) }))) }]),
    // (Its settings', by its name: its eye there.)
    ...(where === "panel" ? [] : [{ key: "hide", icon: layer.hidden ? ICONS.eye : ICONS.eyeOff, label: layer.hidden ? "Show" : "Hide", kbd: "H", verb: layer.hidden ? "Show" : "Hide", stay: true, run: () => toggleHidden(id) }]),
    ...(layer.type !== "signal" && othersInPane(layer) ? [{ key: "solo", icon: ICONS.eye, label: alone ? "Show every layer in its pane" : "Show it alone in its pane", aside: inPalette && !alone ? "The others there hidden" : undefined, verb: "Show", stay: true, run: () => soloLayer(id) }] : []),
    // (Its right-click beside others picked: its copy alone, the picks as they are.)
    { key: "duplicate", icon: ICONS.duplicate, label: "Duplicate", kbd: "⌘D", verb: "Duplicate", run: () => (menu && layerPicks.size ? duplicateLayers([id]) : duplicateLayer(id)) },
    // (A cloud or a trend goes where its layers go, a signal where its layer goes.)
    ...(menu && !follows(layer) ? [{ key: "move", icon: ICONS.pane, label: "Move to pane", page: () => choicePage("Move to pane", () => paneChoices([id])) }] : []),
    // (Its data, as the chart has it, at its interval: any layer's, a formula's too; a series', where it comes from.)
    ...(heatLike(layer) ? [] : [{ key: "data", icon: ICONS.table, label: "Download data", aside: "CSV", verb: "Download", ...(noValues(id) ? { disabled: noValues(id) } : {}), run: () => exportData([layer]) }]),
    // (A heatmap's, its latest day's URPD, as it asks for it.)
    ...(layer.type === "heatmap" ? [{ key: "api", icon: ICONS.link, label: "Open in the API", aside: "Its latest day, as JSON", verb: "Open", run: () => window.open(`${client.baseUrl}${heatPath(chart, layer)}`, "_blank", "noopener") }] : []),
    ...(layer.series ? [{ key: "api", icon: ICONS.link, label: "Open in the API", aside: "As JSON", verb: "Open", run: () => window.open(`${client.baseUrl}/api/series/${encodeURIComponent(/** @type {string} */ (layer.series))}/${chart.index}`, "_blank", "noopener") }] : []),
    { key: "remove", icon: ICONS.trash, label: "Remove", kbd: "⌫", verb: "Remove", danger: true, run: () => removeLayer(id) },
  ];
  // (In a menu, a rule under what opens it, Settings and Replace series, whatever follows.)
  if (menu) /** @type {Action} */ (rows.find((row) => row.key !== "panel" && row.key !== "replace")).apart = true;
  return rows;
}
function duplicateLayer(/** @type {string} */ id) {
  const [copy] = duplicateLayers([id]);
  if (copy) select(copy);
}
/** Copies of layers, each under its original, in one step: their ids (none, without room for them all). @param {string[]} ids */
function duplicateLayers(ids) {
  const originals = chart.layers.filter((layer) => ids.includes(layer.id));
  if (!originals.length) return [];
  const copies = /** @type {string[]} */ ([]);
  commit((doc) => {
    for (const layer of originals) {
      // (One of two colors, or colored by another layer or its levels, kept as it looks: a color of its own, the next.)
      const paired = layer.type === "candles" || colorModeOf(layer) !== "one" || (layer.type === "signal" && (layer.colorBelow !== undefined || signedSignal(layer)));
      const copy = { ...structuredClone(layer), id: uid(), letter: nextLetter(doc), color: paired ? layer.color : nextColor(doc), label: layer.label ? `${layer.label} copy` : undefined };
      doc.layers.splice(doc.layers.findIndex((item) => item.id === layer.id) + 1, 0, copy);
      copies.push(copy.id);
    }
  });
  // (Said, as adding one is: what's new, by its letter.)
  toast(originals.length === 1 ? `Duplicated ${labelOf(originals[0])} as ${layerById(copies[0])?.letter}` : `Duplicated ${plural(originals.length, "layer")}`);
  return copies;
}

/** A series into this chart: with its unit's pane, else a new one; or a new pane, or `pane`, by choice. */
async function addSeries(name, { newPane = false, pane: into = /** @type {string | null} */ (null), bubbles = false } = {}) {
  const target = chart;
  retryInfo(name);
  // (Its path too, kept with its name in the same edit: see normalize.)
  indexCatalog();
  const meta = await info(name).catch(() => null);
  await presetsSettled();
  // (Another chart opened meanwhile, it's not added; the same one made anew (a preset refreshed), it is.)
  if (chart?.id !== target?.id) return;
  if (meta && !datedIntervals(meta).length) return void blockOnly(name, (day) => addSeries(day, { newPane, pane: into, bubbles }));
  const unit = meta ? unitOf(meta.type) : units.get(name) ?? "number";
  units.set(name, unit);
  // (A pane of its own wanted, every pane taken and the last a stack (where it would go): said, not summed into it.)
  const own = into ? null : newPane || !chart.layers.length ? undefined : paneShowing(chart, unit);
  if (!bubbles && !into && !own && chart.layers.length && chart.panes.length >= MAX_PANES && chart.panes.at(-1)?.stack) return void notice(`A chart holds up to ${MAX_PANES} panes, the last a stack: add it to a pane from ⌘K's Actions`);
  /** @type {Layer | undefined} */
  let added;
  commit((doc) => {
    const look = firstLook(name, new Set(doc.layers.map((layer) => layer.color)));
    // (As bubbles: on the chart's first layer that can carry them, where it is; none, a layer as any.)
    const on = bubbles ? markable(doc)[0] : undefined;
    if (on) return void doc.layers.push((added = makeLayer({ series: name, ...look, type: "bubbles", ref: on.id, pane: on.pane, side: on.side }, doc)));
    const found = into ? doc.panes.find((candidate) => candidate.id === into) : newPane || !doc.layers.length ? undefined : paneShowing(doc, unit);
    const sibling = found && doc.layers.find((layer) => layer.pane === found.id && unitFor(layer, resolved?.results.get(layer.id)) === unit);
    // (One of its unit stacked: not summed into it unasked, a pane of its own. A chart emptied: its pane as new.)
    const pane = found ?? (doc.layers.length ? newPaneIn(doc) : doc.panes[0]);
    if (!found && !doc.layers.some((layer) => layer.pane === pane.id)) (pane.mode = unit === "usd" ? 1 : 0), delete pane.stack;
    added = makeLayer({ series: name, pane: pane.id, side: sibling?.side ?? "right", ...look }, doc);
    doc.layers.push(added);
  });
  remember(name);
  if (added) select(added.id, false);
  toast(`Added ${humanize(name)} as ${added?.letter}`, editAdded(added));
  return added;
}

/**
 * Adding a formula is a page of ⌘K, as adding a series is: written there, previewed as it's written, added with ↵,
 * dropped with esc. It starts empty, its recipes leading (see recipesFor); on a chart with nothing to make one of, on
 * an example, selected, so ↵ takes it and typing replaces it.
 */
function addFormula(/** @type {string | null | undefined} */ into = null) {
  const letters = chart.layers.filter((layer) => layer.type !== "cloud").map((layer) => layer.letter);
  openPalette({ formula: true, into, query: letters.length ? "" : EXAMPLE });
}
/** Adding a signal: its formula written in ⌘K's formula page, as a formula's is, started on one to change (its layer over its 200-day average). */
function addSignal() {
  const on = markable(chart)[0];
  if (!on) return notice("Add a layer first: a signal marks one");
  openPalette({ formula: true, signal: true, query: `${on.letter} > sma(${on.letter}, 200d)` });
}
/** A signal as a layer: marked on `on`, else the layer it reads (see markFor; another picked in its panel), selected. */
function addSignalLayer(/** @type {string} */ formula, /** @type {string | undefined} */ on = undefined) {
  /** @type {Layer | undefined} */
  let added;
  commit((doc) => {
    added = makeLayer({ formula, type: "signal", ref: (doc.layers.find((layer) => layer.id === on) ?? markFor(doc, formula))?.id }, doc);
    doc.layers.push(added);
  });
  if (!added) return;
  select(added.id, false);
  toast(`Added ${formula} as ${added.letter}, marked on ${((on) => (on ? labelOf(on) : "no layer"))(layerById(added.ref ?? ""))}`, editAdded(added));
}
/** Where a new pane would go, at the most panes, is the last, and it's a stack (a layer there would be summed into it). @param {string | null | undefined} pane */
const stackedOnly = (pane) => !(pane && chart.panes.some((item) => item.id === pane)) && chart.panes.length >= MAX_PANES && Boolean(chart.panes.at(-1)?.stack);
/** A formula as a layer, selected: in `pane` (on `side`, its layer's it's made from), else in a new one (at the most panes, the last). */
function addFormulaLayer(formula, pane = /** @type {string | null} */ (null), side = /** @type {string | null} */ (null)) {
  if (stackedOnly(pane)) return void notice(STACKED_ONLY);
  /** @type {Layer | undefined} */
  let added;
  commit((doc) => {
    const kept = pane && doc.panes.some((item) => item.id === pane);
    added = makeLayer({ formula, pane: kept ? pane : newPaneIn(doc).id, ...(kept && side ? { side } : {}) }, doc);
    doc.layers.push(added);
  });
  if (!added) return;
  select(added.id, false);
  toast(`Added ${formula} as ${added.letter}`, editAdded(added));
}

/**
 * A layer's halving cycles (halving(A, n), each mined, named by its years) or its years (year(A, y)), from the first
 * it has a value at the start of, as drawn (else the genesis block's), to this one: each from its start at 1×. `over`: instead,
 * this one over the ones before it, all of history (rebase(shift(A, 1halving), halving_epoch): at each start, the
 * one before from its start, each line broken where it starts anew), as far back as it has values whole, three at
 * most (more cross into a tangle), the oldest first.
 * @param {Layer} layer @param {"cycles" | "years"} which @returns {{ formula: string, label: string }[]}
 */
function periodsOf(layer, which, over = false) {
  const year = (/** @type {number} */ time) => new Date(time * 1000).getUTCFullYear(), cycles = which === "cycles";
  const values = resolved?.results.get(layer.id)?.values, at = values ? values.findIndex(Number.isFinite) : -1, first = resolved && at >= 0 ? resolved.times[at] : GENESIS;
  // (Laid over each other, whole ones alone: those it has a value at the first point of, as rebase reads a moved
  // one (one begun later has no 1× at its start), a week's or a month's first point in it too. Each a period's
  // start and its next's; years not before the first year() takes: a week's or a month's first point can be dated
  // before the genesis block.)
  const times = resolved?.times ?? [], whole = (/** @type {number} */ start, /** @type {number} */ end) => {
    const i = lowerBound(times, start);
    // (Its first point the period's own: the timeline reaching back to its start (it begins where the data does).)
    return values !== undefined && i < times.length && times[i] < end && (i > 0 || times[0] <= start) && Number.isFinite(values[i]) && values[i] !== 0;
  };
  const now = year(Date.now() / 1000), from = Math.max(PERIODS_TAKEN.year[0], year(first));
  const starts = cycles ? [GENESIS, ...minedHalvings()] : Array.from({ length: now - from + 2 }, (_, k) => Date.UTC(from + k, 0, 1) / 1000);
  const periods = starts.slice(0, cycles ? starts.length : -1).map((start, n) => ({ n, start, whole: whole(start, starts[n + 1] ?? Infinity) }));
  if (over) {
    // (The ones whole before this one, back to the first that isn't: three at most (more cross into a tangle); none,
    // nothing to lay it over.)
    let before = 0;
    // (This one whole itself: one begun with no value has nothing over the others.)
    if (periods.at(-1)?.whole) while (before < 3 && before + 1 < periods.length && periods[periods.length - 2 - before].whole) before++;
    const [unit, by, what] = cycles ? ["halving", "halving_epoch", "cycle"] : ["y", "year", "year"];
    if (before < 1) return [];
    return Array.from({ length: before + 1 }, (_, i) => ((k) => ({ formula: k ? `rebase(shift(${layer.letter}, ${k}${unit}), ${by})` : `rebase(${layer.letter}, ${by})`, label: k ? `${k} ${what}${k > 1 ? "s" : ""} before` : `This ${what}` }))(before - i));
  }
  // (Side by side, each where it is: those it has values in, one begun late drawn from its first.)
  const any = (/** @type {number} */ start, /** @type {number} */ end) => {
    for (let i = lowerBound(times, start); i < times.length && times[i] < end; i++) if (values && Number.isFinite(values[i])) return true;
    return false;
  };
  return periods.filter(({ n, start }) => any(start, starts[n + 1] ?? Infinity)).map(({ n, start }) => (cycles ? { formula: `halving(${layer.letter}, ${n})`, label: `${year(start)}–${starts[n + 1] ? year(starts[n + 1]) : "now"}` } : { formula: `year(${layer.letter}, ${from + n})`, label: String(from + n) }));
}
/** Why there's nothing to lay over each other: no whole one before this one, by the layer's values. @param {"cycles" | "years"} which */
const overShort = (which) => (which === "cycles" ? "No whole halving cycle before this one" : "No whole year before this one");
/** Whether a trend is offered here: a layer that isn't made from layers (short of more, it says why: trendShort). */
const cloudable = () => chart.layers.some((layer) => !layer.from && !placedOn(layer));
/**
 * A layer made from layers (none made from layers: a cloud's two, its pane's edges among them maybe, a trend's two or
 * more), in the first one's pane, on its side; a cloud colored by them (each one's own where it's over); opened in
 * the panel.
 * A cloud goes behind them (after the last of them there), a trend in front (before the first: a line over what it
 * follows); in a pane whose last is in front, the other way round.
 */
function addFrom(/** @type {"cloud" | "trend"} */ type, /** @type {string[]} */ ids) {
  const members = fromOf(chart, ids, type);
  const short = type === "cloud" && membersOf(chart, ids, true).length < 2 ? "Pick what it's between" : fromShort({ type, from: members });
  if (short) return void notice(short === ONE_SCALE ? `A ${type} reads its layers as one: put them in one pane, on one side` : short === TREND_FLIP ? "A trend flips when another layer touches it: add one (a price)" : type === "cloud" ? "A cloud is between two layers or more" : "A trend is on one of two layers or more");
  /** @type {Layer | undefined} */
  let added;
  commit((doc) => {
    // (A cloud of its pane's two edges alone: in the pane ⌘K adds to, else the selected layer's, else the first,
    // behind all of its layers.)
    const first = doc.layers.find((layer) => layer.id === members.find((id) => !EDGES.includes(id)));
    const pane = first?.pane ?? doc.panes.find((item) => item.id === palette.into)?.id ?? doc.layers.find((layer) => layer.id === selected)?.pane ?? doc.panes[0].id;
    added = makeLayer({ type, from: members, pane, side: first?.side ?? "right", ...(type === "cloud" && first ? { color: first.color, byLayers: true } : {}) }, doc);
    if (type === "trend") added.ref = refFor(doc, added);
    const mine = doc.layers.flatMap((layer, i) => (layer.pane === pane && (!first || ids.includes(layer.id)) ? [i] : []));
    const reversed = Boolean(doc.panes.find((item) => item.id === pane)?.reverse), behind = type === "cloud" ? !reversed : reversed;
    doc.layers.splice(!mine.length ? doc.layers.length : behind ? mine[mine.length - 1] + 1 : mine[0], 0, added);
  });
  if (!added) return;
  layerPicks.clear();
  select(added.id, false);
  const unfit = fromUnfit(chart, added);
  toast(`Added the ${labelOf(added).replace(/^(Cloud|Trend)/, (noun) => noun.toLowerCase())}${unfit ? `, but ${unfit === "Not in a stack" ? "stacked panes don't draw it" : "this pane's scale can't draw it"}` : ""}`, editAdded(added));
}
/**
 * A heatmap of a cohort added: behind the price's layers, in dollars as they are (the pane ⌘K adds to; else the
 * selected layer's, where it reads in dollars or holds nothing; else one that reads in dollars, as a series goes
 * to a pane of its unit (see paneShowing); else the first, empty, or a new one, on a log scale as the price is); a pane
 * with nothing in it, the price as its candles with it (what it's read against).
 * A profile the same way (`profile`: its holder groups, the first at the edge).
 * @param {string} cohort @param {string[]} [profile]
 */
function addHeatmap(cohort, profile = undefined) {
  /** @type {Layer | undefined} */
  let added;
  commit((doc) => {
    // (A heatmap's axis is in dollars, whatever it measures: another's pane is one.)
    const inDollars = (/** @type {Layer} */ layer) => heatLike(layer) || unitFor(layer, resolved?.results.get(layer.id)) === "usd";
    // (Only a pane whose scale can draw it, linear or log: see heatGrid.)
    const drawable = (/** @type {string | undefined} */ id) => ((item) => Boolean(item) && paneMode(item) < 2)(doc.panes.find((item) => item.id === id));
    const empty = (/** @type {string | undefined} */ id) => drawable(id) && !doc.layers.some((layer) => layer.pane === id);
    const dollars = (/** @type {string | undefined} */ id) => drawable(id) && doc.layers.some((layer) => layer.pane === id && inDollars(layer)) && !doc.panes.find((item) => item.id === id)?.stack;
    const chosen = doc.layers.find((layer) => layer.id === selected)?.pane, first = doc.panes[0].id;
    const pane = doc.panes.find((item) => item.id === palette.into)?.id ?? (chosen && (empty(chosen) || dollars(chosen)) ? chosen : doc.panes.find((item) => dollars(item.id))?.id ?? (empty(first) ? first : doc.panes.length < MAX_PANES ? ((made) => ((made.mode = 1), made.id))(newPaneIn(doc)) : doc.panes.find((item) => drawable(item.id))?.id ?? first));
    if (!doc.layers.some((layer) => layer.pane === pane)) doc.layers.push(makeLayer({ series: tree.price.ohlc.name, label: "Price", type: "candles", color: "ink", colors: 1, pane }, doc));
    // (On the axis its dollars are read on, where the pane has one.)
    const beside = doc.layers.filter((layer) => layer.pane === pane), side = (beside.find(inDollars) ?? beside[0])?.side ?? "right";
    added = makeLayer(profile ? { type: "profile", profile, pane, side } : { type: "heatmap", heatmap: cohort, pane, side }, doc);
    kindLook(added, added);
    const mine = doc.layers.flatMap((layer, i) => (layer.pane === pane ? [i] : []));
    doc.layers.splice(doc.panes.find((item) => item.id === pane)?.reverse ? mine[0] : mine[mine.length - 1] + 1, 0, added);
  });
  if (!added) return;
  layerPicks.clear();
  select(added.id, false);
  const unfit = heatGrid(chart, added) ? "" : ", but this pane's scale can't draw it";
  toast(`Added a ${profile ? "profile" : "heatmap"}: ${labelOf(added)}${unfit}`, editAdded(added));
}
/**
 * The layers a cloud or a trend (`made`, one there; none, to be made, its first ticked `first`) can be between: none
 * made from layers, on the first's scale, not what flips a trend (those it's between kept, whatever they are now).
 * @param {Layer | null} made @param {Layer | null} first @param {(id: string) => boolean} has
 */
const memberCandidates = (made, first, has) => chart.layers.filter((layer) => (memberable(layer, made, first) || has(layer.id)) && layer.id !== made?.ref);
/**
 * ⌘K's page for a cloud or a trend: the chart's layers (memberCandidates) and a cloud's pane's two edges, each ticked
 * on or off, then the one made between them; for one made (`id`), what it's between, ticked in place. One page, one
 * way, for both (the inspector's: betweenField).
 * @param {"cloud" | "trend"} type @returns {Page}
 */
function fromPage(type, id = "") {
  const ticked = new Set(), icon = type === "cloud" ? ICONS.cloud : ICONS.riding;
  return { title: id ? "Between" : `Add a ${type}`, placeholder: type === "cloud" ? "Search the layers and edges" : "Search the layers", rows: () => {
    const made = id ? layerById(id) ?? null : null, has = (/** @type {string} */ member) => (made ? Boolean(made.from?.includes(member)) : ticked.has(member)), first = made ? placedBy(chart, made) : chart.layers.find((layer) => ticked.has(layer.id)) ?? null, layers = memberCandidates(made, first, has);
    const members = [...layers.map((layer) => layer.id), ...(type === "cloud" ? EDGES : [])], short = fromShort({ type, from: fromOf(chart, ticked, type) });
    /** @param {string[]} ids */
    const put = (ids) => {
      if (!made) return void (ticked.clear(), ids.forEach((other) => ticked.add(other)));
      editById(made.id, (item, doc) => (item.from = fromOf(doc, ids, type)));
    };
    const add = { icon, label: short || `Add the ${fromName({ type, from: fromOf(chart, ticked, type) }, chart).replace(/^(Cloud|Trend)/, (noun) => noun.toLowerCase())}`, kbd: "⌘↵", stay: Boolean(short), run: () => addFrom(type, [...ticked]) };
    /** A layer's row, or an edge's: ↵ ticks it on or off. @param {string} member @param {Record<string, any>} row */
    const memberRow = (member, row) => {
      const on = has(member), toggle = () => put(toggleMember(chart, members.filter(has), member));
      return { kind: "row", key: member, ...row, aside: on ? "In it" : "", current: on, verb: on ? "Deselect" : "Select", stay: true, run: toggle,
        actions: () => [
          { label: on ? "Deselect" : "Select", icon: ICONS.checkbox, kbd: "↵", stay: true, run: toggle },
          ...(EDGES.includes(member) ? [] : [{ label: "Select all", icon: ICONS.checkbox, stay: true, run: () => put([...layers.filter((other) => oneScale(other, /** @type {Layer} */ (layerById(member)))).map((other) => other.id), ...members.filter((other) => EDGES.includes(other) && has(other))]) }]),
          { label: "Deselect all", icon: ICONS.close, stay: true, run: () => put([]) },
          ...(made ? [] : [add]),
        ] };
    };
    return [
      ...layers.map((layer) => memberRow(layer.id, { icon: letterChip(layer), label: labelOf(layer), group: "Layers", preview: (/** @type {HTMLElement} */ panel) => previewLayer(panel, layer.id) })),
      ...(type === "cloud" ? EDGES.map((edge) => memberRow(edge, { icon: edgeIcon(edge), label: EDGE_NAMES[edge], group: "Edges" })) : []),
      ...(made ? [] : [{ kind: "row", key: "add", icon, label: add.label, kbd: "⌘↵", dim: Boolean(short), verb: "Add", stay: Boolean(short), group: type === "cloud" ? "Cloud" : "Trend", run: add.run }]),
    ];
  } };
}
/**
 * What a cloud or a trend is between, in its settings: a chip for each layer it can be (memberCandidates), a cloud's
 * pane's two edges after them, each pressed on or off. One field, one way, for both (⌘K's: fromPage).
 * @param {Layer} layer
 */
const betweenField = (layer) => {
  const has = (/** @type {string} */ member) => Boolean(layer.from?.includes(member)), edges = edgesOf(layer);
  const chips = [...memberCandidates(layer, placedBy(chart, layer), has).map((item) => ({ id: item.id, icon: letterChip(item), name: labelOf(item), tip: "" })), ...(layer.type === "cloud" ? EDGES.map((edge) => ({ id: edge, icon: edgeIcon(edge), name: capitalized(edge), tip: `${EDGE_NAMES[edge]}: from its layers ${edge === "top" ? "up" : "down"} to it` })) : [])];
  return `<div class="field"><span>Between<span class="color-now">${plural(fromLetters(layer, chart).length, "layer")}${edges.length ? `, the ${edges.join(" and ")}` : ""}</span></span><div class="from-of" role="group" aria-label="What it's between">${chips.map(({ id, icon, name, tip }) => `<button type="button" data-member-of="${id}" aria-pressed="${has(id)}"${tip ? ` data-tip="${escape(tip)}"` : ""}>${icon}<span class="name">${escape(name)}</span></button>`).join("")}</div></div>`;
};
/** What flips a trend, from its button. */
const refMenu = (/** @type {HTMLElement} */ button, /** @type {string} */ id) => menuOf(button, refChoices(id));
/**
 * What flips a trend, as choices: the layers that can, its own ticked (not shown on the chart while looked at: its
 * flips are a resolve away). @returns {Action[]}
 */
function refChoices(/** @type {string} */ id) {
  const trend = layerById(id);
  if (!trend) return [];
  return (placedOn(trend) ? markable(chart) : refsFor(chart, trend).filter((layer) => oneScale(layer, trend) || layer.id === trend.ref)).map((layer) => ({ key: layer.id, icon: letterChip(layer), label: labelOf(layer), current: trend.ref === layer.id, run: () => trend.ref === layer.id || editById(id, (item) => (item.ref = layer.id)) }));
}
/**
 * Formulas from a layer: the starters (each its first formula, on its letter), or one's own.
 * @returns {Action[]}
 */
function deriveChoices(/** @type {string} */ id) {
  const layer = layerById(id);
  if (!layer) return [];
  return [
    ...STARTERS.map((starter) => ({ key: starter.id, icon: ICONS.formula, label: starter.label, aside: starter.formulas.length > 1 ? `${starter.formulas.length} layers` : starter.formulas[0].replaceAll("X", layer.letter), mono: starter.formulas.length === 1, ...((why) => (why ? { disabled: why } : {}))(unfitHere(starter.formulas.map((formula) => formula.replaceAll("X", layer.letter)))), run: () => derive(id, starter.id) })),
    { key: "own", icon: ICONS.pencil, label: "Write your own", aside: `From ${layer.letter}`, verb: "Write", apart: true, run: () => derive(id, "own") },
  ];
}
/** Whether a layer's in a stack (what's added over it would be summed in). */
const onStack = (/** @type {Layer} */ layer) => Boolean(chart.panes.find((pane) => pane.id === layer.pane)?.stack);
/**
 * A starter's formulas from a layer, added: in its pane when they're in its unit, else in a new one (a band's two
 * lines in one color), the first selected, its window to set in its formula. "own": ⌘K's formula page on its letter.
 */
function derive(/** @type {string} */ id, /** @type {string} */ starter) {
  const source = layerById(id);
  if (!source) return;
  // (Its own, in its pane on its side; over a stack, which would sum it in, a pane of its own.)
  if (starter === "own") return ((stacked) => openPalette({ formula: true, into: stacked ? null : source.pane, query: source.letter, side: stacked ? null : source.side }))(onStack(source));
  const formulas = (STARTERS.find((item) => item.id === starter)?.formulas ?? []).map((text) => text.replaceAll("X", source.letter));
  if (!formulas.length) return;
  // (Over a stack, it would add to it: a pane of its own then too.)
  const over = Boolean(STARTERS.find((item) => item.id === starter)?.over) && !onStack(source);
  // (One that needs a pane of its own, at the most panes: not put in the last one, a stack's maybe.)
  if (!(over && chart.panes.some((item) => item.id === source.pane)) && chart.panes.length >= MAX_PANES) return void notice(`A chart holds up to ${MAX_PANES} panes: this needs a new one`);
  /** @type {Layer | undefined} */
  let first;
  commit((doc) => {
    const here = over && doc.panes.some((item) => item.id === source.pane), pane = here ? source.pane : newPaneIn(doc).id;
    for (const formula of formulas) {
      const added = makeLayer({ formula, pane, ...(here ? { side: source.side } : {}), ...(first ? { color: first.color } : {}) }, doc);
      doc.layers.push(added);
      first ??= added;
    }
  });
  if (!first) return;
  select(first.id);
  toast(`Added ${formulas.join(" and ")}`);
}
/** A series layer as a formula of its series (its name, color and place kept; candles a line), to write around it. */
function toFormula(/** @type {string} */ id) {
  editById(id, (layer) => {
    if (!layer.series) return;
    layer.label ??= labelOf(layer);
    if (layer.path) layer.paths = { [layer.series]: layer.path };
    layer.formula = layer.series;
    delete layer.series;
    delete layer.path;
    if (layer.type === "candles") layer.type = "line";
  });
  select(id);
  requestAnimationFrame(() => $("formula")?.focus());
}

/** The series a formula names, as choices to turn it into. @returns {Action[]} */
const seriesChoices = (/** @type {string} */ id) => ((layer) => (layer ? turnsInto(layer) : []).map((name) => ({ key: name, icon: ICONS.series, label: name, mono: true, run: () => toSeries(id, name) })))(layerById(id));
/**
 * A formula layer as one of the series it names (its color, place and given name kept; where the catalog has the
 * series, still found there): the formula goes.
 */
function toSeries(/** @type {string} */ id, /** @type {string} */ name) {
  editById(id, (layer) => {
    if (!turnsInto(layer).includes(name)) return;
    if (layer.label === humanize(name)) delete layer.label;
    if (layer.paths && Object.hasOwn(layer.paths, name)) layer.path = layer.paths[name];
    layer.series = name;
    delete layer.formula;
    delete layer.paths;
  });
  select(id);
  refocus('[data-action="change-series"]');
}
export {
  acceptSuggestion, actionsOf, addFormula, addFormulaLayer, addFrom, addHeatmap, addLevel, addSeries, addSignal,
  addSignalLayer, addStop, afterPress, assistKey, axisField, bandOf, barAt, betweenField, bubbleSection, candlesField,
  changeSeries, chooseSuggestion, CHROMA, closestSeries, cloudable, colorField, colorLevels, colorModeChoices,
  colorModeOf, colorOf, colorPicker, colorSlot, colorSlots, colorSource, completeSeries, completion, completionsAt,
  copyName, dashRow, drawPlanes, duplicateLayer, duplicateLayers, editAdded, editById, editingMany, editLayer,
  errorBox, eyeOf, fadingOf, faintOf, fieldsShape, fillField, fillOf, fitChoices, fitField, fitOf, FITS, fitsAt,
  flushPick, focusKey, fold, formatChoices, formatMenu, formulaInputs, fromChoices, fromPage, glidesUntil, goneSaid,
  GRADIENT_STEPS, gradientCss, gradientEdit, gradientPlaces, gradientSwatch, gradientTable, hexField, hideLayers, HUES,
  inHand, insertAtCaret, isUnfinished, keepFocus, keyOf, labelOf, layerActions, layerById, layerMenu, layersCommands,
  layersMenu, levelRows, levelsField, levelSteps, levelValue, lineAt, lineButtons, lineChoices, lineField, lineLook,
  lineLookOn, lineMenu, lineStyleGlyph, lineStyleName, markTyping, modeName, modeOf, modesFor, modeSwatch, moveLayer,
  movers, moveStop, named, opacityField, outlineField, overShort, ownColor, paneChoices, paneField, paneLayers,
  paneMenu, paneMode, penColors, periodsOf, pick, pickAt, pickedAt, pickedLayers, pickedOf, pickPoint, pickPress,
  placeAssist, placeOf, planeHtml, planeOf, planeTarget, priceLineField, rangeOf, readNumber, readsOf, readyName,
  readyPage, recipesFor, refChoices, refMenu, refPicked, removeLayer, removeLayers, removeStop, renderLayersBar,
  replaceSeries, resetFor, restoreFocus, reverseGradient, rewrite, setColor, setFit, setLineLook, setSlider, sharedBy,
  showEvery, showPicks, showScale, showSelection, showStop, showSuggestions, slider, sliderAt, sliders, sliderTrack,
  slotName, soloLayer, spaceAt, STACK_FILL, STACK_WIDTH, stackedIn, stackedOnly, stackOf, suggest, suggestReplacement,
  TAG_NEEDS_LINE, toggleHidden, typeField, underColorMenu, updateCallHelp, updateFormula, updateSuggestions, usesOf,
  vividAhead, widthField, widthRow, withLevel, withoutLevel,
};
