// @ts-check
// A layer's inspector: picking layers, their settings, gradients, adding layers.
import { placeIndex, press } from "/modules/bitview-charts/index.js";
import { capitalized } from "/studio/words.js";
import {
  acrossUnfit, afterHands, byBlock, byTouch, chart, chartById, chooseDrawings, chosen, clearLayerPicks, client,
  cohortName, colorName, colorPane, commit, DEFAULTS, dismissPalette, EXAMPLE, firstLook, formatName, formulaFit,
  fromLetters, fromShort, fromUnfit, handsHeld, holding, home, indexCatalog, isOhlc, layerPicks, layersBatch,
  letterChip, markFor, markPicks, marksOf, memberable, minedHalvings, MOTION, notice, openPalette, ownerOf, panelShown,
  paneShowing, phone, profileColor, recent, reduced, refFor, refsFor, remember, removedNames, renderBelt, renderLayers,
  reshape, resolved, say, secondColor, select, selected, seriesCount, showList, shownOklch, signedSignal, SLIDERS,
  STACKED_ONLY, toast, track, tree, undoToast, widest, keepFocus, nodeAt,
} from "/studio/app.js";
import { isSeriesName } from "/studio/catalog-walk.js";
import { $, all$, closest, drawKeys, escapeHtml, keyText, proseHtml, SLOP, viewport, focusKey, restoreFocus, isOpen, cutFade, fadeOut, scrolled } from "/studio/dom.js";
import { inSlices, whenIdle } from "/studio/timing.js";
import { clamp, lowerBound, uid } from "/studio/math.js";
import {
  axisLabel, fieldUnit, format, FORMATS, grouped, humanize, plural, UNIT_NAMES, unitOf,
} from "/studio/format.js";
import { CHEVRON, dashGlyph, glyph, glyphSvg, ICONS, markBody, shapeGlyph } from "/studio/icons.js";
import {
  belowOf, DASHES, dashOf, EDGES, edgesOf, fittable, FLIP_APART, follows, FORMULA_MAX, fromOf, goesWith, GRADIENT_LEAST,
  GRADIENT_MOST, heatLike, hollowOf, kindLook, layersColors, LINED, makeLayer, MARK_PLACES, markable, MAX_LEVELS,
  MAX_PANES, membersOf, MODES, modesOf, newPaneIn, ownGradient, nextColor, nextLetter, ONE_SCALE, oneScale, drawnInto,
  gradable, HEAT_GRADIENTS, nameLayer, OUTLINED, OUTLINES, outlinable, PAIRS, paneGradient, placedBy, placedOn,
  placeName, rowsUnfit, seriesLooked, SHAPES, STACKS, summed, toggleMember, TREND_FLIP, TREND_SAYS, TWO_KEPT,
  TWO_OR_ONE, typeName, TYPES, WIDTHS,
} from "/studio/vocabulary.js";
import {
  FINE, flowFits, INTERVAL_DAYS, INTERVAL_LABEL, INTERVAL_ORDER, spanDays, SPANS, windowPoints,
} from "/studio/intervals.js";
import { gamma, inside, limit, maxChroma, oklchToHex, oklchToLinear, SWATCHES } from "/studio/color.js";
import {
  baseOf, BY_BLOCK, cssColor, dailyOf, datedIntervals, foldedInto, gone, info, ink, lastShown, noValues, respot,
  retryInfo, seriesIn, unitFor, units, updateLegends, valueAt,
} from "/studio/chart.js";
import {
  aboutOf, bothOf, choicePage, finder, findSeries, formulaError, formulaSide, listPage, loopError, palette,
  paneNames, previewLayer, pushPage,
} from "/studio/command.js";
import { beltSide, drawingById, lookFor, setLook } from "/studio/drawing.js";
import {
  argNames, FUNCTIONS, GENESIS, hasFunction, highlightFormula, parse, PERIODS_TAKEN, refsOf, turnsInto, unfitHere,
  VALUES,
} from "/studio/formulas.js";
import { recommendedCards } from "/studio/grid.js";
import {
  heatGrid, heatPath, heatSection,
} from "/studio/heatmaps.js";
import { exportData, letGo, menuOf, showMenu } from "/studio/menus.js";
import { ordersOf, refocus, slideLayers, tick } from "/studio/presets-edit.js";
import { lineAt, lineLook, lineDefaults, named, modeOf, paneMode, labelOf, fromName, layerById, gradientCss, mixedOf, rangeOf, levelSteps, fadingOf, faintOf } from "/studio/looks.js";
import { presetsSettled } from "/studio/preset-charts.js";
import { readsOf } from "/studio/charts-read.js";
/** @import { Chart, Layer, Pane } from "/studio/vocabulary.js" */
/** @import { Action, Command, Page } from "/studio/command.js" */
/** @import { Node } from "/studio/formulas.js" */

/** @param {Layer} layer */
const stackedIn = (layer, doc = chart) => summed(doc.panes.find((pane) => pane.id === layer.pane));

/** @param {Pane} pane */
const modesFor = (pane) => (summed(pane) ? [] : pane.stack ? [0, 1] : [0, 1, 2, 3]);

/**
 * Safari on iOS sends some views whose content is replaced back to their top (a layer's settings, at each edit, on iOS 26.5).
 * @param {HTMLElement} container @param {string} html
 */
function rewrite(container, html) {
  // Never scrolled (see scrolled), at its top: nothing read, nothing put back. Either lays the page out at once.
  const top = scrolled.has(container) ? container.scrollTop : 0;
  container.innerHTML = html;
  if (top) container.scrollTop = top;
}

function showPicks() {
  if (layerPicks.size) {
    showList();
    if (chosen.size) chooseDrawings([]);
  }
  markPicks();
}
const pickedLayers = () => chart.layers.filter((layer) => layerPicks.has(layer.id));
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
  const cloud = /** @type {HTMLElement} */ (bar.querySelector('[data-layers-batch="cloud"]')), bases = layers.filter((layer) => !layer.from && layer.type !== "signal"), apart = bases.length >= 2 && fromShort({ type: "cloud", from: bases.map((layer) => layer.id) });
  cloud.setAttribute("aria-disabled", String(bases.length < 2 || Boolean(apart)));
  cloud.dataset.tip = bases.length < 2 ? "A cloud is between two layers or more" : apart || "A cloud between them";
}
/** @param {string[]} ids @param {boolean} [hide] */
function hideLayers(ids, hide = chart.layers.some((layer) => ids.includes(layer.id) && !layer.hidden)) {
  commit((doc) => doc.layers.forEach((layer) => ids.includes(layer.id) && (layer.hidden = hide || undefined)), { look: true });
}
function soloLayer(/** @type {string} */ id) {
  const layer = layerById(id);
  if (!layer) return;
  if (layer.type === "signal" || !othersInPane(layer)) return toggleHidden(id);
  const alone = aloneInPane(layer);
  commit((doc) => doc.layers.forEach((other) => other.pane === layer.pane && (other.hidden = alone || itsOwn(other, layer) ? undefined : true)), { look: true });
}
const aloneInPane = (/** @type {Layer} */ layer) => !layer.hidden && othersInPane(layer) && chart.layers.every((other) => other.pane !== layer.pane || itsOwn(other, layer) || other.hidden);
/** @param {Layer} layer */
const othersInPane = (layer) => chart.layers.some((other) => other.pane === layer.pane && !itsOwn(other, layer));
/** @param {Layer} other @param {Layer} layer */
const itsOwn = (other, layer) => other.id === layer.id || (other.type === "signal" && other.ref === layer.id) || foldedInto(chart).get(other.id) === layer.id;
function movers(/** @type {string[]} */ ids, /** @type {string | null} */ into) {
  const going = ids.filter((id) => ((layer) => !follows(layer) || layer?.pane === into)(layerById(id)));
  const left = ids.map(layerById).find((layer) => layer && !going.includes(layer.id) && goesWith(layer).some((id) => !ids.includes(id)));
  const its = left?.type === "signal" ? ["its layer", "goes", "it"] : ["its layers", "go", "them"];
  if (left) notice(going.length ? `${labelOf(left)} stays with ${its[0]}` : `${labelOf(left)} goes where ${its[0]} ${its[1]}: move ${its[2]}`);
  return going;
}
/** @param {string[]} asked @param {string | null} into */
function moveLayers(asked, into) {
  const ids = movers(asked, into);
  if (!ids.length) return;
  commit((doc) => {
    const pane = into ? doc.panes.find((item) => item.id === into) : newPaneIn(doc);
    if (!pane) return;
    // Those that go where these go (bubbles on one, a cloud between them) go along, where they were among them: what's drawn over what, as it was.
    const along = new Set(ids);
    for (let more = true; more; ) {
      more = false;
      for (const layer of doc.layers) if (!along.has(layer.id) && follows(layer) && goesWith(layer).every((id) => along.has(id))) (along.add(layer.id), (more = true));
    }
    const moving = doc.layers.filter((layer) => along.has(layer.id));
    for (const layer of moving) layer.pane = pane.id;
    doc.layers = [...doc.layers.filter((layer) => !along.has(layer.id)), ...moving];
  });
}
/** @param {HTMLElement} anchor @param {string[]} ids */
function paneMenu(anchor, ids, after = () => {}) {
  menuOf(anchor, paneChoices(ids, after));
}
/** @param {string[]} ids @returns {Action[]} */
function paneChoices(ids, after = () => {}) {
  const inPanes = new Set(chart.layers.filter((layer) => ids.includes(layer.id)).map((layer) => layer.pane)), theirs = (/** @type {string} */ id) => inPanes.size === 1 && inPanes.has(id);
  return [
    ...chart.panes.map((pane, i) => ({ key: pane.id, icon: ICONS.pane, label: `Pane ${i + 1}`, aside: paneNames(pane), current: theirs(pane.id), run: () => (theirs(pane.id) || moveLayers(ids, pane.id), after()) })),
    { key: "new", icon: ICONS.plus, label: "New pane", apart: true, ...(chart.panes.length < MAX_PANES ? {} : { disabled: `A chart holds up to ${MAX_PANES} panes` }), run: () => (moveLayers(ids, null), after()) },
  ];
}
/** @param {HTMLElement} anchor */
function layersMenu(anchor, point) {
  menuOf(anchor, layersActions(), point);
}
/** @returns {Action[]} */
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
const iconNames = new Map(Object.entries(ICONS).map(([name, markup]) => [markup, name]));
/** @returns {Command[]} */
function layersCommands() {
  if (!layerPicks.size) return [];
  const commands = layersActions().map((row) => ({ title: row.label, text: row.disabled ? `${row.disabled}.` : row.text ?? "", icon: /** @type {string} */ (iconNames.get(row.icon ?? "")), kbd: row.kbd, danger: row.danger, suggested: true, ...(row.disabled ? {} : row.page ? { keepOpen: true, run: () => pushPage(/** @type {() => Page} */ (row.page)()) } : { run: row.run }) }));
  return [...commands, { title: "Clear the selection", text: `${plural(pickedLayers().length, "layer")} selected.`, icon: "close", kbd: "esc", run: () => clearLayerPicks(), suggested: true }];
}

/** @param {Layer | undefined} layer */
const editAdded = (layer) => (phone.matches && layer ? { label: "Edit", run: () => {
  if (isOpen($("palette"))) dismissPalette();
  if (layerById(layer.id)) select(layer.id);
} } : undefined);

function showSelection() {
  renderLayers();
  renderInspector();
  respot();
  updateLegends();
  renderBelt();
}
let inspected = "";
let multiEdit = false;
let inspectedShape = "";
let expanded = false;
let actionsShown = "";
let inspectorDue = false;
function renderInspector() {
  // Not under a hex being typed: its caret, its undo, what's half typed; drawn again once it's left.
  if (document.activeElement?.matches?.("#inspector .color-editor .hex")) return void (inspectorDue = true);
  inspectorDue = false;
  const layer = layerById(selected);
  if (layer) vividAhead();
  if (multiEdit && !editingMany()) multiEdit = false;
  const many = editingMany() ? pickedLayers() : null, open = Boolean(layer || many);
  if (document.body.hasAttribute("data-inspect") !== open && panelShown()) {
    cutFade(open ? $("inspector") : $("layers-view"));
    fadeOut(open ? $("layers-view") : $("inspector"), [{ opacity: 1, transform: "none" }, { opacity: 0, transform: `translateX(${open ? -8 : 8}px)` }], MOTION.micro, true);
  }
  document.body.toggleAttribute("data-inspect", open);
  const panel = $("inspector");
  if (many) return renderMany(panel, many);
  if (!layer) {
    panel.innerHTML = "";
    inspected = "";
    return;
  }
  // While typing into this layer's formula or name (or a number of it: its base, a level, a gradient's place), keep the field and its caret as it is.
  const active = document.activeElement;
  const shape = fieldsShape(layer);
  const typed = !panel.dataset.pressed;
  delete panel.dataset.pressed;
  if (typed && active && inspected === layer.id && shape === inspectedShape && Boolean($("formula")) === (layer.formula !== undefined) && panel.contains(active) && (["formula", "label", "base"].includes(active.id) || active.id.startsWith("level-") || /** @type {HTMLElement} */ (active).dataset.stopAt !== undefined)) {
    updateFormula(layer);
    suggestHere();
    const fresh = actionsOf(layer);
    if (fresh !== actionsShown) /** @type {HTMLElement} */ (panel.querySelector(".layer-actions")).outerHTML = actionsShown = fresh;
    const eye = $("inspector-eye");
    if (eye && eye.dataset.hidden !== String(Boolean(layer.hidden))) eye.outerHTML = eyeOf(layer);
    return;
  }
  const another = inspected !== layer.id;
  if (another) expanded = false;
  const focus = inspected === layer.id && !(shape !== inspectedShape && active?.closest?.(".levels, .gradient-hand")) ? focusKey(panel) : null;
  inspected = layer.id;
  inspectedShape = shape;
  const result = resolved?.results.get(layer.id);
  const meta = result?.info;
  const isFormula = layer.formula !== undefined, signal = layer.type === "signal";
  const unit = unitFor(layer, result);
  const layerPane = /** @type {Pane} */ (chart.panes.find((pane) => pane.id === layer.pane)), stackOfPane = layerPane?.stack ?? "", ordered = chart.layers.filter((item) => item.pane === layer.pane).length > 1;
  const fromError = layer.from ? fromShort(layer) || result?.error : "";
  const fromSays = layer.from && !fromError ? ((unfit) => [layer.type === "trend" ? TREND_SAYS : "", unfit === "Not in a stack" ? "not drawn in a stacked pane" : unfit === ONE_SCALE ? `not drawn: ${FLIP_APART.toLowerCase()}` : unfit ? "not drawn on a percent or indexed scale" : ""].filter(Boolean).join("; "))(fromUnfit(chart, layer)) : "";
  const sourceSection = heatLike(layer)
    ? heatSection(layer)
    : layer.from
    ? `<section>
              <h3>${typeName(layer.type)}</h3>
              ${betweenField(layer)}
              ${layer.type === "cloud" ? "" : `<div class="field"><span>Flips when it touches</span><button type="button" class="preset" id="trend-ref" aria-haspopup="menu" data-tip="The layer that flips the trend when it touches it (an OHLC series by its highs and lows, drawn as candles or not)">${refPicked(layer)}${CHEVRON}</button></div>`}
              ${fromError || fromSays ? `<p class="from-status ${fromError ? "error" : ""}" role="status">${escapeHtml(fromError || `${capitalized(fromSays)}.`)}</p>` : ""}
            </section>`
    : isFormula
    ? `<section>
              <h3>${signal ? "Signal" : "Formula"}</h3>
              <div class="formula-editor" id="formula-editor">
                <pre id="formula-mirror" aria-hidden="true"></pre>
                <textarea id="formula" rows="1" aria-controls="suggest" aria-autocomplete="list" maxlength="${FORMULA_MAX}" spellcheck="false" autocapitalize="off" autocorrect="off" autocomplete="off" enterkeyhint="done" aria-label="Formula" aria-describedby="formula-status formula-call">${escapeHtml(((draft) => (draft && draft.over === layer.formula ? draft.text : layer.formula))(formulaDrafts.get(chart.id)?.get(layer.id)))}</textarea>
                <div class="assist" popover="manual">
                  <div id="formula-call" class="call-help" aria-live="polite" hidden></div>
                  <ul id="suggest" class="suggest" role="listbox" hidden></ul>
                  <p id="suggest-more" class="more" hidden></p>
                </div>
              </div>
              <p id="formula-status" class="formula-status"></p>
              <details class="help"><summary>Functions and how letters work</summary>
                <div class="functions">${Object.entries(FUNCTIONS).map(([name, [, signature, what]]) => `<button type="button" data-insert="${name}("><b>${escapeHtml(signature)}</b><span>${escapeHtml(what)}</span></button>`).join("")}</div>
                <p>Capital letters are layers as drawn; a series by its name (${escapeHtml(tree.price.close.name)}) is the series itself. Comparisons (&gt; &lt; &gt;= &lt;= == !=) are 1 where true, 0 where not, joined with &amp; (and) and | (or): a signal marks where one turns true, or, signed, where it turns up (above 0) or down (below): (A &gt; B) - (A &lt; B). date is each point's date, compared with one written 2020-05-11: when(A, date &gt;= 2020-05-11) is A from then on. Use + − × ÷ ^ and parentheses. Windows are spans of time: 12h, 30d, 4w, 6mo, 2y, or 4 * 365d (a bare number is days); shorter than the interval, one point. A window is always last; at 1D and up, it holds what there is until it's full, and one left out (where it shows span?) is the function's usual count of points where it has one (rsi's 14), else all of history so far.</p>
              </details>
              ${signal ? `<div class="field"><span>Marked on</span><button type="button" class="preset" id="signal-ref" aria-haspopup="menu" data-tip="The layer its marks are on">${refPicked(layer)}${CHEVRON}</button></div>
              <div class="field"><span>Shown as<span class="color-now">${escapeHtml(placeName(layer.place ?? "above"))}</span></span><div class="tabs glyphs" role="group" aria-label="How it shows">${MARK_PLACES.map(([place]) => `<button type="button" data-place="${place}" aria-pressed="${(layer.place ?? "above") === place}" aria-label="${placeName(place)}" data-tip="${placeName(place)}">${glyphSvg(markBody(place, "currentColor"))}</button>`).join("")}</div></div>
              <div class="field switch"><span>Results</span><div class="tabs" role="group" aria-label="Results">${[[false, "Off", "Marks alone"], [true, "On", "On each mark that ends a stretch, what it made: its layer's change over it, up; the opposite, down"]].map(([on, text, tip]) => `<button type="button" data-results="${on ? 1 : 0}" aria-pressed="${Boolean(layer.results) === on}" data-tip="${tip}">${text}</button>`).join("")}</div></div>
              <div class="field"><span>Times marked</span><button type="button" class="preset" id="signal-times" aria-haspopup="dialog" data-tip="Every time it signaled; pick one to jump to it">${((n) => (n === null ? `<span class="name">Loading</span>` : `<span class="name">${plural(n, "time")}</span>`))(result?.values ? marksOf(layer, result.values).length : null)}${CHEVRON}</button></div>` : ""}
            </section>`
    : `<section>
              <h3>Series</h3>
              <div class="source-name"><button type="button" class="source-pick" data-action="change-series" aria-label="Replace series, ${escapeHtml(layer.series)}" data-tip="Replace series"><code>${escapeHtml(layer.series)}</code>${ICONS.swap}</button>
                <button type="button" class="cap small" data-action="copy-name" aria-label="Copy the series name" data-tip="Copy the series name">${ICONS.copy}</button>
              </div>
              ${result?.error ? errorBox(result, layer.series ?? "") : ""}
              ${meta?.description ? `<div class="description ${expanded ? "" : "clamped"}">${proseHtml(meta.description)}</div>${meta.description.length > 220 ? `<button type="button" class="more" data-action="more" aria-expanded="${expanded}">${expanded ? "Show less" : "Show more"}</button>` : ""}` : ""}
              ${meta ? `<div class="facts"><span class="chip">${escapeHtml(UNIT_NAMES[unitOf(meta.type)] ?? "Number")}</span><span class="chip">${meta.indexes.length} intervals</span></div>` : ""}
            </section>`;
  const html = `
          <div class="inspector-head">
            <button type="button" class="cap small" data-action="deselect" aria-label="Back to the layers" data-tip="Back to the layers" data-kbd="esc">${ICONS.back}</button>
            ${letterChip(layer)}
            <input id="label" class="label-input" maxlength="120" enterkeyhint="done" value="${escapeHtml(layer.label ?? "")}" placeholder="${escapeHtml(labelOf({ ...layer, label: "" }))}" aria-label="Layer name" spellcheck="false" />
            ${eyeOf(layer)}
            <button type="button" class="cap small sheet-close" data-action="close-sheet" aria-label="Close" data-tip="Close">${ICONS.close}</button>
          </div>
          ${sourceSection}
          ${layer.type === "bubbles" ? bubbleSection(layer) : ""}
          <section>
            <h3>Style</h3>
            ${typeField([layer])}
            ${["baseline", "histogram"].includes(layer.type) ? `<div class="field"><span>Base${resetFor("base", layer)}</span><span class="level">${layer.type === "baseline" ? lineButtons(layer, "base", "Base line") : ""}<label class="number-field"><input id="base" class="number-input" aria-label="Base" enterkeyhint="done" type="text" autocomplete="off" spellcheck="false" value="${baseOf(layer)}" data-tip="${layer.type === "histogram" ? "Bars grow from here: above in the first color, below in the second" : "The line's colors part here, marked by a line across"}" /><span class="unit">${escapeHtml(fieldUnit(unit))}</span></label></span></div>${layer.type === "baseline" && acrossUnfit(chart, layer) ? `<p class="from-status" role="status">Its line ${escapeHtml(acrossUnfit(chart, layer).toLowerCase())}.</p>` : ""}` : ""}
            ${colorPicker(layer)}
            ${opacityField([layer])}
            ${widthField([layer])}
            ${lineField([layer])}
            ${fillField([layer])}
            ${candlesField([layer])}
            ${outlineField([layer])}
            ${priceLineField([layer])}
            ${layer.from || placedOn(layer) || heatLike(layer) ? "" : levelsField(layer, unit, result)}
          </section>
          ${signal ? "" : `<section>
            <h3>Layout</h3>
            ${paneField([layer])}
            ${axisField([layer])}
            ${fitField([layer])}
            ${heatLike(layer) ? "" : `<div class="field"><span>Format${resetFor("format", layer)}</span><button type="button" id="format" class="preset" data-action="format-menu" aria-haspopup="menu" aria-label="Format: ${escapeHtml(formatName(layer, result))}"><span class="name">${escapeHtml(formatName(layer, result))}</span>${CHEVRON}</button></div>`}
            <p class="shared-head">Pane ${chart.panes.findIndex((pane) => pane.id === layer.pane) + 1}'s, for every layer in it</p>
            ${summed(layerPane) ? "" : `<div class="field"><span>Scale</span><div class="tabs" role="group" aria-label="Pane scale">${MODES.map(({ chip, tip, name }, mode) => `<button type="button" data-pane-mode="${layer.pane}" data-mode="${mode}" aria-pressed="${modeOf(chart, layer) === mode}" ${modesFor(layerPane).includes(mode) ? `data-tip="${mode < 2 ? `${tip} scale` : tip}"` : `aria-disabled="true" data-tip="${name} isn't available in a heatmap (use linear or log)"`}>${chip}</button>`).join("")}</div></div>`}
            <div class="field"><span>Stack</span><div class="tabs" role="group" aria-label="Pane stack">${STACKS.map(([stack, name, tip]) => `<button type="button" data-pane-stack="${layer.pane}" data-stack="${stack}" aria-pressed="${stackOfPane === stack}" ${stack === "rows" && rowsUnfit(chart, layerPane) ? `aria-disabled="true" data-tip="${rowsUnfit(chart, layerPane)}"` : `data-tip="${tip}"`}>${stack === "percent" ? "100%" : name}</button>`).join("")}</div></div>
            ${ordered ? `<div class="field"><span>Order</span><div class="tabs" role="group" aria-label="Order">${ordersOf(layerPane).map(([reverse, name]) => `<button type="button" data-pane-reverse="${layer.pane}" data-reverse="${reverse ? 1 : ""}" aria-pressed="${Boolean(layerPane.reverse) === reverse}">${name}</button>`).join("")}</div></div>` : ""}
            ${gradable(chart, layerPane) ? `<div class="field"><span>Gradient</span><div class="tabs" role="group" aria-label="Gradient">${HEAT_GRADIENTS.map(([id, name]) => `<button type="button" data-pane-gradient="${layer.pane}" data-gradient="${id}" aria-pressed="${paneGradient(layerPane)[0] === id}">${name}</button>`).join("")}</div></div>` : ""}
            ${outlinable(chart, layerPane) ? `<div class="field"><span>Outline</span><div class="tabs" role="group" aria-label="Outline">${OUTLINES.map(([on, name]) => `<button type="button" data-pane-outline="${layer.pane}" data-outline="${on ? 1 : ""}" aria-pressed="${Boolean(layerPane.outline) === on}">${name}</button>`).join("")}</div></div>` : ""}
          </section>`}
          ${(actionsShown = actionsOf(layer))}`;
  // The sliders' handles and fills as they are on screen (maybe mid-glide), for the redrawn ones to go on from: only those of this layer whose stop changes alone (reading one lays the page's style out first).
  const moves = (/** @type {HTMLElement} */ element) => element.id in SLIDERS && (Number(element.style.getPropertyValue("--at")) !== sliderAt(/** @type {keyof typeof SLIDERS} */ (element.id), layer) || (glidesUntil.get(element.id) ?? 0) > performance.now());
  const was = new Map(another || reduced() ? [] : all$("#inspector .stepper").filter(moves).map((element) => [element.id, { handle: getComputedStyle(sliderTrack(element), "::after").left, fill: getComputedStyle(sliderTrack(element), "::before").width }]));
  rewrite(panel, html);
  gradientRoom.disconnect();
  ((bar) => bar && gradientRoom.observe(bar))(panel.querySelector("[data-gradient-bar]"));
  drawKeys(panel);
  if (isFormula) updateFormula(layer);
  restoreFocus(panel, focus);
  drawPlanes();
  // Redrawn, each slider glides on from where it was to its stop, however many times it's redrawn on the way.
  for (const element of all$("#inspector .stepper")) {
    const from = was.get(element.id);
    if (!from) continue;
    const track = sliderTrack(element), handle = getComputedStyle(track, "::after").left, fill = getComputedStyle(track, "::before").width;
    const timing = { duration: MOTION.micro, easing: MOTION.ease };
    glidesUntil.set(element.id, performance.now() + MOTION.micro);
    if (handle !== from.handle) track.animate({ left: [from.handle, handle] }, { ...timing, pseudoElement: "::after" });
    if (fill !== from.fill) track.animate({ width: [from.fill, fill] }, { ...timing, pseudoElement: "::before" });
  }
  if (another) panel.scrollTop = 0;
  if (result?.missing) suggestReplacement(layer);
}
/** @param {HTMLElement} panel @param {Layer[]} layers */
function renderMany(panel, layers) {
  const key = `many:${layers.map((layer) => layer.id).join()}`, focus = inspected === key ? focusKey(panel) : null, another = inspected !== key;
  inspected = key;
  actionsShown = "";
  const style = [typeField(layers), colorField(layers), opacityField(layers), widthField(layers), lineField(layers), fillField(layers), candlesField(layers), outlineField(layers), priceLineField(layers)].join(""), layout = [paneField(layers), axisField(layers), fitField(layers)].join("");
  rewrite(panel, `
          <div class="inspector-head">
            <button type="button" class="cap small" data-action="deselect" aria-label="Back to the layers" data-tip="Back to the layers" data-kbd="esc">${ICONS.back}</button>
            <span class="many-letters">${layers.slice(0, 4).map((layer) => letterChip(layer, true)).join("")}${layers.length > 4 ? `<span class="letter rest" data-tip="${escapeHtml(layers.slice(4).map(labelOf).join(", "))}">+${layers.length - 4}</span>` : ""}</span>
            <span class="many-name">${plural(layers.length, "layer")}<span class="many-of">${escapeHtml(layers.map(labelOf).join(", "))}</span></span>
            <button type="button" class="cap small sheet-close" data-action="close-sheet" aria-label="Close" data-tip="Close">${ICONS.close}</button>
          </div>
          <section><h3>Style</h3>${style}</section>
          ${layout ? `<section><h3>Layout</h3>${layout}</section>` : ""}`);
  drawKeys(panel);
  restoreFocus(panel, focus);
  drawPlanes();
  if (another) panel.scrollTop = 0;
}
function editPicked(/** @type {boolean} */ on) {
  multiEdit = on;
  renderInspector();
}
function toggleDescription(/** @type {HTMLElement} */ button) {
  expanded = !expanded;
  const description = $("inspector").querySelector(".description");
  description?.classList.toggle("clamped", !expanded);
  button.textContent = expanded ? "Show less" : "Show more";
  button.setAttribute("aria-expanded", String(expanded));
}
const editingMany = () => multiEdit && !selected && pickedLayers().length > 1;
const inHand = () => (editingMany() ? pickedLayers() : [layerById(selected)].filter((layer) => layer !== undefined));
/** @template T @param {Layer[]} layers @param {(layer: Layer) => T} of */
const sharedBy = (layers, of) => (layers.length && layers.every((layer) => of(layer) === of(layers[0])) ? of(layers[0]) : undefined);
/** @param {Layer} layer */
const fieldsShape = (layer) => `${layer.levels?.length ?? 0}:${layer.byGradient ? (layer.gradient?.length ?? 0) : 0}`;
/** @param {Layer} layer */
const eyeOf = (/** @type {Layer} */ layer) => `<button type="button" id="inspector-eye" data-hidden="${Boolean(layer.hidden)}" class="cap small" data-toggle="${layer.id}" aria-label="${layer.hidden ? "Show" : "Hide"} ${escapeHtml(labelOf(layer))}" data-tip="${layer.hidden ? "Show" : "Hide"}${layer.type === "signal" ? "" : `\n${keyText("⌥-click: alone in its pane")}`}" data-kbd="H">${layer.hidden ? ICONS.eyeOff : ICONS.eye}</button>`;
const actionsOf = (/** @type {Layer} */ layer) => `<section class="layer-actions">${layerActions(layer.id, "panel").map((row, i) => `<button type="button" id="act-${row.key}" class="nav-item${row.danger ? " danger" : ""}" data-act="${i}"${row.page ? ' aria-haspopup="menu"' : ""}${row.disabled ? ` aria-disabled="true" data-tip="${escapeHtml(row.disabled)}"` : ""}${row.kbd?.startsWith("⌘") ? ` data-kbd="${[...row.kbd].join(" ")}"` : ""}>${row.icon ?? ""}<span class="label">${escapeHtml(row.label)}</span>${row.kbd ? `<kbd${row.kbd.startsWith("⌘") ? ' class="mod"' : ""}>${escapeHtml(keyText(row.kbd))}</kbd>` : ""}</button>`).join("")}</section>`;
/** @param {Layer} layer */
const refPicked = (layer) => ((ref) => (ref ? `${letterChip(ref)}<span class="name">${escapeHtml(labelOf(ref))}</span>` : `<span class="name">Pick a layer</span>`))(layerById(layer.ref ?? ""));

const typeField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every(seriesLooked)) return "";
  const now = sharedBy(layers, (layer) => layer.type), ohlc = layers.every((layer) => resolved?.results.get(layer.id)?.opens || (layer.series && isOhlc(layer.series)));
  return `<div class="field"><span>Type<span class="color-now">${escapeHtml(now === undefined ? "Mixed" : TYPES.find(([type]) => type === now)?.[1] ?? "")}</span></span><div class="tabs glyphs" role="group" aria-label="Chart type">${TYPES.map(([type, text]) => `<button type="button" data-type="${type}" aria-pressed="${now === type}" aria-label="${text}" ${type === "candles" && !ohlc ? "aria-disabled=\"true\" data-tip=\"Candles: only for open-high-low-close series\"" : `data-tip="${text}"`}>${glyph(type)}<span>${text}</span></button>`).join("")}</div></div>`;
};
const colorField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every((layer) => colorModeOf(layer) === "one" && layer.type !== "profile")) return "";
  const now = sharedBy(layers, (layer) => layer.color), last = colorSource.get(`${chart.id}:${layers.map((layer) => layer.id).join()}:color`);
  const from = last && last.color === now ? last.source : now === undefined || SWATCHES.includes(now) ? "presets" : "custom";
  const tabs = colorSourceTabs("Color", [["presets", "Palette"], ["custom", "Custom"]], from);
  const body = from === "custom" ? `<div class="color-box">${planeHtml(now ?? layers[0].color, "layer:color", "Color")}${hexField("layer:color", now ?? "")}</div>` : swatchGrid(now && SWATCHES.includes(now) ? now : "");
  return `<div class="field"><span>Color<span class="color-now">${escapeHtml(now === undefined ? "Mixed" : colorName(now))}</span></span></div><div class="color-editor" data-palette="color">${tabs}<div class="color-pick">${body}</div></div>`;
};
/** @param {string} label @param {[string, string][]} sources @param {string} from */
const colorSourceTabs = (label, sources, from) => `<div class="tabs color-source" role="group" aria-label="${escapeHtml(label)}, its color from">${sources.map(([source, name]) => `<button type="button" data-color-source="${source}" aria-pressed="${source === from}">${name}</button>`).join("")}</div>`;
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
  const fades = layers.every((layer) => layer.fill !== "above" && !stackedIn(layer));
  return `<div class="field"><span>Fill</span><div class="tabs" role="group" aria-label="Fill">${[["below", "Below", "Filled from the line down"], ["above", "Above", "Filled from the line up"]].map(([fill, text, tip]) => `<button type="button" data-fill="${fill}" aria-pressed="${now === fill}" data-tip="${tip}">${text}</button>`).join("")}</div></div>
          <div class="field"><span>Fill opacity${resetFor("fill-opacity", layers)}</span>${slider("fill-opacity", layers)}</div>
          ${fades ? `<div class="field"><span>Fill style</span><div class="tabs" role="group" aria-label="Fill style">${[[false, "Gradient", "Fading to nothing at its foot"], [true, "Solid", "Even all the way down"]].map(([on, text, tip]) => `<button type="button" data-solid="${on ? 1 : 0}" aria-pressed="${solid === on}" data-tip="${tip}">${text}</button>`).join("")}</div></div>` : ""}`;
};
const candlesField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every((layer) => layer.type === "candles")) return "";
  const now = sharedBy(layers, hollowOf);
  return `<div class="field"><span>Candles</span><div class="tabs" role="group" aria-label="Candles">${[[true, "Hollow", "Hollow when rising, filled when falling"], [false, "Filled", "Filled, rising or falling"]].map(([on, text, tip]) => `<button type="button" data-hollow="${on ? 1 : 0}" aria-pressed="${now === on}" data-tip="${tip}">${text}</button>`).join("")}</div></div>`;
};
const priceLineField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every((layer) => layer.type !== "cloud" && !placedOn(layer) && !heatLike(layer))) return "";
  const now = sharedBy(layers, (layer) => Boolean(layer.priceLine)), into = layers.map((layer) => (layer.priceLine ? drawnInto(chart, chart.panes.find((pane) => pane.id === layer.pane)) : "")).find(Boolean);
  return `<div class="field switch"><span>Price line</span><div class="tabs" role="group" aria-label="Price line">${[[false, "Off", "No line"], [true, "On", "A line across the chart at its latest value"]].map(([on, text, tip]) => `<button type="button" data-price-line="${on ? 1 : 0}" aria-pressed="${now === on}" data-tip="${tip}">${text}</button>`).join("")}</div></div>${into ? `<p class="from-status" role="status">Not drawn in ${into}.</p>` : ""}`;
};
const outlineField = (/** @type {Layer[]} */ layers) => {
  if (!layers.every((layer) => OUTLINED.has(layer.type))) return "";
  const now = sharedBy(layers, (layer) => Boolean(layer.outline));
  return `<div class="field switch"><span>Outline</span><div class="tabs" role="group" aria-label="Outline">${[[false, "Off", "No edge"], [true, "On", "Edged in the chart's background, to stand out over what's under it"]].map(([on, text, tip]) => `<button type="button" data-outline="${on ? 1 : 0}" aria-pressed="${now === on}" data-tip="${tip}">${text}</button>`).join("")}</div></div>`;
};
/** @param {Layer} layer */
const bubbleSection = (layer) => `<section>
              <h3>Bubbles</h3>
              <div class="field"><span>On</span><button type="button" class="preset" id="bubble-ref" aria-haspopup="menu" data-tip="The layer they're on">${((ref) => (ref ? `${letterChip(ref)}<span class="name">${escapeHtml(labelOf(ref))}</span>` : `<span class="name">Its pane's middle</span>`))(layerById(layer.ref ?? ""))}${CHEVRON}</button></div>
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
  const text = sharedBy(layers, (layer) => (typeof spec.text === "function" ? spec.text(layer) : spec.text)) ?? "each one's default";
  return `<button type="button" class="reset" data-reset="${key}" aria-label="Reset to ${text}" data-tip="Reset to ${text}">${ICONS.undo}</button>`;
};
const glidesUntil = new Map();
const sliderAt = (/** @type {keyof typeof SLIDERS} */ id, /** @type {Layer} */ layer) => Math.max(0, SLIDERS[id].stops.indexOf(SLIDERS[id].of(layer))) / (SLIDERS[id].stops.length - 1);
const sliderTrack = (/** @type {HTMLElement} */ element) => /** @type {HTMLElement} */ (element.firstElementChild);
function slider(id, /** @type {any} */ target, tip = "") {
  const spec = SLIDERS[id], list = [target].flat(), value = sharedBy(list, spec.of), mixed = value === undefined, index = spec.stops.indexOf(value), n = spec.stops.length - 1;
  const at = mixed ? 0 : sliderAt(id, list[0]);
  return `<div id="${id}" class="stepper${mixed ? " mixed" : ""}" role="slider" tabindex="0" aria-label="${spec.label}" aria-valuemin="0" aria-valuemax="${n}" aria-valuenow="${Math.max(0, index)}" aria-valuetext="${mixed ? "Mixed" : escapeHtml(spec.say(value))}" data-value-side="${at > 0.5 ? "start" : "end"}" style="--at: ${at}" ${tip ? `data-tip="${escapeHtml(tip)}"` : ""}>
          <span class="stepper-track">${spec.stops.map((_, i) => `<i ${(at > 0.5 ? i / n <= 0.25 : i / n >= 0.75) ? 'class="under"' : ""} style="--k: ${i / n}"></i>`).join("")}<span class="value">${mixed ? "Mixed" : escapeHtml(spec.text(value))}</span></span></div>`;
}
function setSlider(id, value) {
  const layers = inHand();
  if (!layers.length || layers.every((layer) => SLIDERS[id].of(layer) === value)) return;
  editLayer((target) => SLIDERS[id].set(target, value), { coalesce: `${id}:${layers.map((layer) => layer.id).join()}` });
  // Dragged, the page isn't drawn again till it ends: the slider shows its new value itself, as the belt's do.
  const element = $(id), now = inHand();
  if (holding && element && now.length) {
    element.outerHTML = slider(id, now, element.dataset.tip ?? "");
    $(id)?.focus({ preventScroll: true });
  }
}
// Found again each time: the inspector redraws under the drag.
function stopAt(id, clientX) {
  const element = $(id);
  if (!element) return undefined;
  const box = element.getBoundingClientRect(), stops = SLIDERS[id].stops;
  if (!box.width) return undefined;
  return stops[clamp(Math.round(((clientX - box.left - 3) / (box.width - 6)) * (stops.length - 1)), 0, stops.length - 1)];
}
// By touch (in what scrolls, a sheet), as a finger on a page means: a tap picks where it's lifted, a drag clearly across picks along it, a drag up or down scrolls (the browser's) and picks nothing (a slider scrolled past keeps its value).
function pickPress(event, element, cursor, move) {
  if (holding) return;
  if (event.pointerType !== "touch") return track(event, element, cursor, move);
  const stop = press(event, {
    move: (next) => {
      const dx = next.clientX - event.clientX, dy = next.clientY - event.clientY;
      if (Math.hypot(dx, dy) <= SLOP) return;
      stop();
      // Clearly across: an arc as a scroll begins, or a slant, is a scroll.
      if (Math.abs(dx) > 2 * Math.abs(dy)) track(next, element, cursor, move);
    },
    // A tap: picked where it's lifted, unless a drag's held meanwhile (one at a time, see track).
    end: (release) => release && !holding && move(release),
  });
}
/** @param {HTMLElement} host @param {{ valueOf: (id: string) => number | undefined, set: (id: string, value: number) => void }} how */
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
/** @param {(item: Layer) => number} edit */
function gradientEdit(edit, again = false) {
  let k = -1;
  editLayer((item) => (k = item.gradient ? edit(item) : k));
  if (k < 0) return;
  colorSlot.set(`${chart.id}:${selected}`, `stop${k}`);
  if (again) renderInspector(), refocus(`#inspector .gradient-stop[data-stop="${k}"]`);
}
function showStop(/** @type {number} */ k, field = true) {
  const layer = layerById(selected), stops = layer?.gradient, at = stops?.[k]?.[0];
  if (!stops || at === undefined) return;
  const cursor = /** @type {HTMLElement | null} */ ($("inspector").querySelector(`.gradient-stop[data-stop="${k}"]`)), bar = /** @type {HTMLElement | null} */ ($("inspector").querySelector("[data-gradient-bar]"));
  if (cursor) {
    const said = stopSaid(stops[k]), tip = cursor.dataset.tip ?? "";
    cursor.style.setProperty("left", `${at}%`);
    cursor.style.setProperty("--c", cssColor(stops[k][1]));
    cursor.setAttribute("aria-label", said);
    cursor.dataset.tip = said + tip.slice(tip.indexOf("\n") < 0 ? tip.length : tip.indexOf("\n"));
  }
  bar?.style.setProperty("--g", gradientCss(stops));
  const hand = $("inspector").querySelector(".gradient-hand");
  if (hand && !hand.querySelector(`[data-stop-at="${k}"]`)) {
    hand.outerHTML = gradientHand(layer, k);
    for (const other of all$(".gradient-stop", $("inspector"))) other.setAttribute("aria-pressed", String(other.dataset.stop === String(k)));
  }
  const input = /** @type {HTMLInputElement | null} */ ($("inspector").querySelector(`[data-stop-at="${k}"]`));
  if (input && field) input.value = String(at);
  const ready = /** @type {HTMLElement | null} */ ($("inspector").querySelector('[data-action="gradient-ready"]'));
  showScale();
  const named = hand?.isConnected ? hand.querySelector("[data-stop-name]") : $("inspector").querySelector("[data-stop-name]");
  if (named) named.innerHTML = `<i class="swatch" style="background: ${escapeHtml(cssColor(stops[k][1]))}"></i>${escapeHtml(colorName(stops[k][1]))}`;
  if (ready) ready.innerHTML = `${gradientSwatch(stops)}<span class="name">${escapeHtml(readyName(stops))}</span>${CHEVRON}`;
}
/** @param {[number, string]} stop */
const stopSaid = ([at, color]) => `${colorName(color)}, at ${at}%`;
const barAt = (/** @type {HTMLElement} */ bar, /** @type {number} */ clientX) => ((box) => (box.width ? ((clientX - box.left) / box.width) * 100 : 0))(bar.getBoundingClientRect());

/** @param {Layer} layer */
function colorModeOf(layer) {
  const modes = modesOf(layer, chart);
  if (ownGradient(chart, layer)) return "gradient";
  if (layer.colorFrom && modes.includes("from")) return "from";
  if (layer.byLayers && modes.includes("layers")) return "layers";
  if (layer.byLevels && modes.includes("levels")) return "levels";
  if (layer.type === "histogram") return layer.colors === 1 ? "one" : layer.colors === 4 ? "momentum" : "sides";
  if (TWO_OR_ONE.has(layer.type)) return layer.colors === 1 ? "one" : "sides";
  if (layer.type === "baseline" || (TWO_KEPT.has(layer.type) && layer.colors !== 1 && layer.colorBelow !== undefined && modes.includes("sides"))) return "sides";
  return "one";
}
const modeName = (/** @type {string} */ mode, /** @type {Layer} */ layer) => (mode === "one" ? (signedSignal(layer) ? "Two" : "One") : mode === "sides" ? "Two" : mode === "momentum" ? "Four" : mode === "levels" ? "By its levels" : mode === "layers" ? "By its layers" : mode === "gradient" ? "Gradient" : "From a layer");
/** @param {Layer} layer @returns {[string, string][]} */
function colorSlots(layer) {
  const stops = ownGradient(chart, layer);
  if (stops) return stops.map(([at], k) => [`stop${k}`, `At ${at}%`]);
  if (layer.type === "profile") return (layer.profile ?? []).map((cohort) => [`cohort:${cohort}`, cohortName(cohort)]);
  if (layer.type === "heatmap") return [];
  const mode = colorModeOf(layer), pair = layer.type === "cloud" ? ((letters) => (letters.length === 2 ? [`${letters[0]} over ${letters[1]}`, `${letters[1]} over ${letters[0]}`] : ["First over", "Second over"]))(fromLetters(layer)) : PAIRS[layer.type];
  return mode === "from" ? [["from", "Layer"]] : mode === "levels" ? [["color", "Under its levels"]] : mode === "layers" ? []
    : mode === "sides" || signedSignal(layer) ? [["color", pair[0]], ["colorBelow", pair[1]]]
    : mode === "momentum" ? [["color", "Above, growing"], ["colorFading", "Above, fading"], ["colorBelow", "Below, growing"], ["colorBelowFading", "Below, fading"]]
    : [["color", "Color"]];
}

const colorOf = (/** @type {Layer} */ layer, /** @type {string} */ key) => (key.startsWith("cohort:") ? profileColor(layer, key.slice(7)) : key.startsWith("stop") ? layer.gradient?.[Number(key.slice(4))]?.[1] ?? layer.color : key === "colorBelow" ? belowOf(layer) : key === "colorFading" ? fadingOf(layer, false) : key === "colorBelowFading" ? fadingOf(layer, true) : layer.color);
function setColor(/** @type {Layer} */ layer, /** @type {string} */ key, /** @type {string} */ color) {
  if (key.startsWith("stop")) ((stop) => stop && color && (stop[1] = color))(layer.gradient?.[Number(key.slice(4))]);
  else if (key.startsWith("cohort:")) color && (layer.cohortColors = { ...layer.cohortColors, [key.slice(7)]: color });
  else if (key === "colorFading") layer.colorFading = color || undefined;
  else if (key === "colorBelowFading") layer.colorBelowFading = color || undefined;
  // Red, the default, is left unset, but candles' two colors are said, as unset reads as one (see normalize).
  else if (key === "colorBelow") layer.colorBelow = !TWO_KEPT.has(layer.type) && layer.type !== "candles" && color === "red" ? undefined : color;
  else layer.color = color;
}
function setColorMode(/** @type {Layer} */ layer, /** @type {string} */ mode, from = "") {
  delete layer.colorFrom;
  delete layer.byLevels;
  delete layer.byLayers;
  delete layer.colors;
  delete layer.byGradient;
  if (mode === "gradient") (layer.byGradient = true), (layer.gradient ??= gradientFor());
  else if (mode === "levels") (layer.byLevels = true), colorLevels(layer);
  else if (mode === "layers") (layer.byLayers = true), layersColors(layer, ownerOf(layer));
  else if (mode === "from") layer.colorFrom = from;
  else if ((layer.type === "histogram" && (mode === "one" || mode === "momentum")) || (TWO_OR_ONE.has(layer.type) && mode === "one")) layer.colors = mode === "one" ? 1 : 4;
  // Two the same, as its layers' may be: a second apart.
  else if (TWO_KEPT.has(layer.type) && mode === "sides") layer.colorBelow = layer.colorBelow === undefined || layer.colorBelow === layer.color ? secondColor(layer) : layer.colorBelow;
  else if (TWO_KEPT.has(layer.type) && mode === "one" && layer.colorBelow !== undefined) layer.colors = 1;
  else if (TWO_OR_ONE.has(layer.type) && mode === "sides") layer.colorBelow ??= secondColor(layer);
}

/** @returns {[number, string][]} */
const gradientFor = () => [[0, "blue"], [100, "red"]];

// The surface, one plane: down, lightness, each given its room (from the dark tones still seen to the palest tints, little at the very ends, where everything reads as black or white), one smooth curve, no seams. Across, the hues over HUES of the width; past them chroma eases away into the grays at the right.
const HUES = 0.86;
const smoothstep = (from, to, x) => {
  const s = clamp((x - from) / (to - from), 0, 1);
  return s * s * (3 - 2 * s);
};
const spaceAt = (l) => 0.05 + 0.95 * smoothstep(0.05, 0.22, l) * (1 - smoothstep(0.95, 1, l));
const BELOW = (() => {
  const below = new Float64Array(1001);
  for (let i = 0; i < 1000; i++) below[i + 1] = below[i] + spaceAt((i + 0.5) / 1000);
  return below.map((value) => value / below[1000]);
})();
function heightOf(l) {
  const i = clamp(l, 0, 1) * 1000, k = Math.min(999, Math.floor(i));
  return 1 - (BELOW[k] + (BELOW[k + 1] - BELOW[k]) * (i - k));
}
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
const hueAt = (u) => Math.min(u / HUES, 1) * 359.9;
const keepAt = (u) => (u <= HUES ? 1 : 0.5 + 0.5 * Math.cos(Math.PI * Math.min(1, (u - HUES) / ((1 - HUES) * 0.85))));
const CHROMA = 0.18;

/** @param {string} target @param {string} color */
const hexField = (target, color) => `<label class="hex-field"><span class="hex-box"><input class="hex" data-hex="${escapeHtml(target)}" value="${color ? escapeHtml(ink(color)) : ""}" maxlength="40" spellcheck="false" autocomplete="off" aria-label="Hex color" /><span class="color-now">${color ? escapeHtml(colorName(color)) : ""}</span></span></label>`;
const swatchGrid = (/** @type {string} */ current) => `<div class="swatch-grid" role="radiogroup" aria-label="Bitview's colors">${SWATCHES.map((color) => `<button type="button" role="radio" data-pick-color="${escapeHtml(color)}" aria-checked="${current === color}" aria-label="${escapeHtml(colorName(color))}" data-tip="${escapeHtml(colorName(color))}"><i class="swatch" style="--c:${escapeHtml(cssColor(color))}"></i></button>`).join("")}</div>`;
function planeHtml(/** @type {string} */ color, /** @type {string} */ target, name = "Color") {
  const { u, t } = placeOf(color), said = planeSays(color, u);
  return `<div class="plane" data-plane="${escapeHtml(target)}" role="slider" tabindex="0" aria-label="${escapeHtml(name)}: hue across, then grays; lightness down" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${said.now}" aria-valuetext="${said.text}" style="--x: ${u}; --y: ${t}"><canvas></canvas><i class="ring" style="--c:${escapeHtml(cssColor(color))}"></i></div>`;
}
function planeSays(/** @type {string} */ color, /** @type {number} */ u) {
  const [l, c, h] = shownOklch(color), grayish = u > HUES + (1 - HUES) * 0.85 - 1e-9;
  return { now: Math.round(l * 100), text: `${grayish || c < 0.02 ? "Gray" : `Hue ${Math.round(h)} degrees`}, lightness ${Math.round(l * 100)} percent` };
}

const LEVEL_COLORS = ["yellow", "orange", "red", "pink", "purple", "violet", "blue", "sky", "teal", "green", "lime", "amber"];
/** @param {Layer} layer @param {number} value */
function withLevel(layer, value) {
  layer.levels = [...(layer.levels ?? []), { value }];
  if (layer.byLevels) colorLevels(layer);
}
/** @param {Layer} layer @param {number} i */
function withoutLevel(layer, i) {
  const others = layer.levels?.filter((_, k) => k !== i);
  layer.levels = others?.length ? others : undefined;
  if (!layer.levels) delete layer.underName;
}
/**
 * Each level without a color of its own gets one (by value, lowest first, from LEVEL_COLORS), so its bands show at once and don't follow its own color under them.
 * @param {Layer} layer
 */
function colorLevels(layer) {
  const taken = new Set([layer.color, ...(layer.levels ?? []).map((level) => level.color).filter(Boolean)]);
  const free = LEVEL_COLORS.filter((color) => !taken.has(color));
  for (const level of [...(layer.levels ?? [])].sort((a, b) => a.value - b.value)) if (!level.color) level.color = free.shift() ?? layer.color;
}
/** @param {Layer} layer */
function bandsSwatch(layer) {
  const colors = [layer.color, ...levelSteps(layer).map((step) => step.color)];
  const share = 100 / colors.length;
  return `<i class="swatch" style="background: linear-gradient(to top, ${colors.map((color, k) => `${escapeHtml(cssColor(color))} ${(k * share).toFixed(1)}% ${((k + 1) * share).toFixed(1)}%`).join(", ")})"></i>`;
}
function modeSwatch(/** @type {Layer} */ layer, /** @type {string} */ mode) {
  if (mode === "levels") return bandsSwatch(layer);
  if (mode === "gradient") return gradientSwatch(layer.gradient ?? gradientFor());
  if (mode === "from") return ((source) => (source ? letterChip(source) : ICONS.series))(layerById(layer.colorFrom ?? ""));
  const colors = mode === "momentum" ? [layer.color, fadingOf(layer, false), fadingOf(layer, true), belowOf(layer)] : mode === "sides" || (mode === "one" && signedSignal(layer)) || (mode === "layers" && !edgesOf(layer).length) ? [layer.color, belowOf(layer)] : [layer.color];
  const share = 100 / colors.length;
  return `<i class="swatch" style="background: linear-gradient(to bottom, ${colors.map((color, k) => `${escapeHtml(cssColor(color))} ${(k * share).toFixed(1)}% ${((k + 1) * share).toFixed(1)}%`).join(", ")})"></i>`;
}
const slotName = (/** @type {Layer} */ layer, /** @type {string} */ key) => ((own) => (own === undefined && (key === "colorFading" || key === "colorBelowFading") ? `Fainter ${colorName(key === "colorFading" ? layer.color : belowOf(layer)).toLowerCase()}` : colorName(colorOf(layer, key))))(key === "colorFading" ? layer.colorFading : key === "colorBelowFading" ? layer.colorBelowFading : "");
function colorPicker(/** @type {Layer} */ layer) {
  if ((layer.type === "profile" && !layer.profile?.length) || layer.type === "heatmap") return "";
  const mode = colorModeOf(layer), name = modeName(mode, layer), source = layerById(layer.colorFrom ?? ""), ways = modesOf(layer, chart).length > 1;
  const head = `<div class="field"><span>${mode === "one" && !signedSignal(layer) && !(layer.type === "profile" && (layer.profile?.length ?? 0) > 1) ? "Color" : "Colors"}</span>${ways ? `<button type="button" id="color-mode" class="preset" data-action="color-mode" aria-haspopup="menu" aria-label="Colors: ${escapeHtml(name)}"><span class="name">${escapeHtml(name)}</span>${CHEVRON}</button>` : ""}</div>`;
  if (mode === "from") return `${head}<div class="field color-slot"><span>Layer</span><button type="button" id="color-from" class="preset" data-action="color-from" aria-haspopup="menu" aria-label="Colors from: ${escapeHtml(source ? labelOf(source) : "")}">${source ? letterChip(source) : ""}<span class="name">${escapeHtml(source ? labelOf(source) : "")}</span>${CHEVRON}</button></div>`;
  if (mode === "levels" || mode === "layers") return head;
  const places = colorSlots(layer), key = slotNow(layer), label = places.find(([slot]) => slot === key)?.[1] ?? "Color";
  const own = key === "colorFading" ? layer.colorFading : key === "colorBelowFading" ? layer.colorBelowFading : colorOf(layer, key), from = sourceNow(layer, key);
  const chip = (/** @type {string} */ slot, /** @type {string} */ name) => `<button type="button" class="slot-chip" data-color-slot="${slot}" aria-pressed="${slot === key}" aria-label="${escapeHtml(name)}: ${escapeHtml(slotName(layer, slot))}" data-tip="${escapeHtml(slotName(layer, slot))}" style="--c:${escapeHtml(cssColor(colorOf(layer, slot)))}"></button>`;
  const which = mode === "gradient" ? gradientEditor(layer, key) : layer.type === "profile" ? (places.length > 1 ? `<div class="color-slots cohorts" role="group" aria-label="Which color">${places.map(([slot, name]) => `<span class="cohort-slot"><span class="head">${escapeHtml(name)}</span>${chip(slot, name)}</span>`).join("")}</div>` : "") : places.length > 2
    ? `<div class="color-slots four" role="group" aria-label="Which color"><span></span><span class="head">Growing</span><span class="head">Fading</span><span>Above</span>${chip("color", "Above, growing")}${chip("colorFading", "Above, fading")}<span>Below</span>${chip("colorBelow", "Below, growing")}${chip("colorBelowFading", "Below, fading")}</div>`
    : places.length > 1 ? `<div class="color-slots" role="group" aria-label="Which color">${places.map(([, name]) => `<span class="head">${escapeHtml(name)}</span>`).join("")}${places.map(([slot, name]) => chip(slot, name)).join("")}</div>` : "";
  const sources = /** @type {[string, string][]} */ ([["presets", "Palette"], ["custom", "Custom"], ...(fadingSlot(key) ? [["auto", "Auto"]] : [])]);
  const tabs = colorSourceTabs(label, sources, from);
  const target = `layer:${key}`, side = key === "colorFading" ? layer.color : belowOf(layer);
  const body = from === "custom" ? `<div class="color-box">${planeHtml(colorOf(layer, key), target, label)}${hexField(target, colorOf(layer, key))}</div>`
    : from === "auto" ? `<p class="color-auto"><i class="swatch" style="--c:${escapeHtml(cssColor(faintOf(side)))}"></i>Fainter ${escapeHtml(colorName(side).toLowerCase())}</p>`
    : swatchGrid(own && SWATCHES.includes(own) ? own : "");
  const relative = mode === "gradient" ? `<div class="field"><span>Relative to</span><div class="tabs" role="group" aria-label="Relative to">${/** @type {[boolean, string, string][]} */ ([[false, "All of history", "Its lowest and highest of all its history"], [true, "Up to each date", "Its lowest and highest up to each date: what that day showed, no hindsight"]]).map(([on, text, tip]) => `<button type="button" data-gradient-to="${on ? 1 : ""}" aria-pressed="${Boolean(layer.upToDate) === on}" data-tip="${escapeHtml(tip)}">${text}</button>`).join("")}</div></div>` : "";
  return `${head}<div class="color-editor" data-palette="${key}">${which}${tabs}<div class="color-pick">${body}</div></div>${relative}`;
}
/** @type {[string, string, [number, string][]][]} */
const GRADIENT_PRESETS = [...HEAT_GRADIENTS, ["sides", "Loss to profit", [[0, "red"], [50, "gray"], [100, "green"]]]];
/** @param {[number, string][] | undefined} stops */
const readyGradient = (stops) => GRADIENT_PRESETS.find(([, , preset]) => JSON.stringify(preset) === JSON.stringify(stops))?.[0] ?? null;
/** @param {[number, string][] | undefined} stops */
const readyName = (stops) => GRADIENT_PRESETS.find(([id]) => id === readyGradient(stops))?.[1] ?? "Your own";
/** @param {[number, string][] | undefined} stops */
const gradientSwatch = (stops) => `<i class="swatch gradient-swatch" style="background: ${escapeHtml(gradientCss(stops ?? []))}"></i>`;
/** @type {Map<string, [number, string][]>} */
const ownGradients = new Map();
/** @param {string} id */
const readyPage = (id, /** @type {any} */ preview = null) => ((both) => choicePage("Ready made", () => {
  const key = `${chart.id}:${id}`, stops = layerById(id)?.gradient, own = readyGradient(stops) ? ownGradients.get(key) : stops;
  return [...(own ? /** @type {const} */ ([["own", "Your own", own]]) : []), ...GRADIENT_PRESETS].map(([choice, name, ready]) => ({ key: choice, icon: gradientSwatch(ready), label: name, current: choice === "own" ? !readyGradient(stops) : readyGradient(stops) === choice, ...both((item) => {
    if (item.gradient && !readyGradient(item.gradient)) ownGradients.set(key, structuredClone(item.gradient));
    item.gradient = structuredClone(ready);
  }) }));
}, preview))(bothOf(id));
/** @param {string} id */
function reverseGradient(id) {
  // Drawn again whole, as after a press in it: a place's field in focus (⌘K's, closed, gives it back) is no place being typed.
  const pressed = () => ($("inspector").dataset.pressed = "1");
  pressed();
  editById(id, (item) => {
    if (!item.gradient) return;
    colorSlot.set(`${chart.id}:${id}`, `stop${item.gradient.length - 1 - (Number(slotNow(item).slice(4)) || 0)}`);
    item.gradient = item.gradient.map(([at, color]) => /** @type {[number, string]} */ ([100 - at, color])).reverse();
  });
  if (id === selected) pressed(), renderInspector();
}
/**
 * Round steps from its lowest to its highest, each its place (0 to 100) and its label's left on the bar as laid out (`width` across); the strongest kept where two would come closer than a gap, the color in hand's own worth (at `hand`: `own`) first.
 * @param {Layer} layer @param {number} width @param {number} [hand] @returns {[number, string, number, boolean][]}
 */
function gradientTicks(layer, width, hand) {
  if (!width) return [];
  const result = resolved?.results.get(layer.id), unit = unitFor(layer, result), ticks = /** @type {[number, number][][]} */ ([]), worth = hand === undefined ? null : gradientWorth(layer, hand);
  const [low, high] = rangeOf(result?.values ?? []), rough = (high - low) / 4, step = ((ten) => [1, 2, 5, 10].map((k) => k * ten).find((size) => size >= rough) ?? 10 * ten)(10 ** Math.floor(Math.log10(rough)));
  // Eleven at most, counted apart from the values: a step too small to move one (a range of a few ulps) still ends, its ticks as one, the overlaps dropped below.
  for (let i = 0, first = Math.ceil(low / step); high > low && i <= 10 && first + i <= Math.floor(high / step); i++) ticks.push([[(((first + i) * step - low) / (high - low)) * 100, (first + i) * step]]);
  ticks.sort((a, b) => Math.abs(b[0][1]) - Math.abs(a[0][1]));
  // Kept, the first first, while its label is a gap clear of those kept; those under the color in hand's worth give way alone.
  const GAP = 12, label = (/** @type {number} */ at, /** @type {string} */ text, own = false) => ((size) => /** @type {[number, string, number, number, boolean]} */ ([at, text, clamp((at / 100) * width - size / 2, 0, width - size), size, own]))(widest([text], own ? 510 : 400, 11));
  const clear = (/** @type {[number, string, number, number, boolean]} */ [, , left, size], /** @type {[number, string, number, number, boolean][]} */ others) => others.every(([, , other, room]) => left >= other + room + GAP || left + size <= other - GAP);
  const own = worth && hand !== undefined ? [label(hand, worth, true)] : [], kept = [...own];
  for (const group of ticks) {
    const shown = group.map(([at, value]) => label(at, axisLabel(value, "", unit))).filter((tick) => clear(tick, own));
    if (shown.every((tick) => clear(tick, kept))) kept.push(...shown);
  }
  return kept.sort((a, b) => a[0] - b[0]).map(([at, text, left, , own]) => /** @type {[number, string, number, boolean]} */ ([at, text, left, own]));
}
/** @param {Layer} layer @param {string} key */
function gradientEditor(layer, key) {
  const stops = layer.gradient ?? [], few = stops.length <= GRADIENT_LEAST, k = clamp(Number(key.slice(4)) || 0, 0, stops.length - 1);
  const stop = (/** @type {[number, string]} */ [place, tint], /** @type {number} */ i) => `<button type="button" class="gradient-stop" data-color-slot="stop${i}" data-stop="${i}" aria-pressed="${i === k}" style="left: ${place}%; --c: ${escapeHtml(cssColor(tint))}" aria-label="${escapeHtml(stopSaid([place, tint]))}" data-tip="${escapeHtml(stopSaid([place, tint]))}${escapeHtml(`\n${keyText(`Drag, or ← → to move it${few ? "" : "; ⌫ to remove it"}`)}`)}"></button>`;
  const head = `<div class="gradient-head"><button type="button" class="preset" data-action="gradient-ready" aria-haspopup="menu" aria-label="Ready made: ${escapeHtml(readyName(stops))}" data-tip="Gradients ready made">${gradientSwatch(stops)}<span class="name">${escapeHtml(readyName(stops))}</span>${CHEVRON}</button><button type="button" class="cap" data-action="gradient-reverse" aria-label="Reverse the gradient" data-tip="Reverse the gradient">${ICONS.swap}</button><button type="button" class="cap" data-action="gradient-add" aria-label="Add a color" ${stops.length >= GRADIENT_MOST ? `aria-disabled="true" data-tip="A gradient holds up to ${GRADIENT_MOST} colors"` : 'data-tip="Add a color, in the widest gap"'}>${ICONS.plus}</button></div>`;
  return `<div class="gradient-edit" role="group" aria-label="Its gradient">${head}<div class="gradient-scale" data-gradient-bar style="--g: ${escapeHtml(gradientCss(stops))}"${stops.length < GRADIENT_MOST ? ' data-tip="A press adds a color here"' : ""}><span class="gradient-fill"></span>${stops.map(stop).join("")}</div>${gradientScale(layer, stops[k]?.[0])}</div>${gradientHand(layer, k)}`;
}
// Kept as it changes (the panel's grip, a phone turned): ticks are fitted to it.
let gradientWidth = 0;
const gradientRoom = new ResizeObserver((entries) => {
  const width = entries[entries.length - 1].contentRect.width;
  if (width && width !== gradientWidth) (gradientWidth = width), showScale();
});
/** @param {Layer} layer @param {number} [hand] */
function gradientScale(layer, hand) {
  const ticks = gradientTicks(layer, gradientWidth, hand), ends = [gradientWorth(layer, 0), gradientWorth(layer, 100)], own = ticks.find((tick) => tick[3]);
  return ticks.length ? `<div class="gradient-ticks" role="img" aria-label="${escapeHtml(`${ends[0] && ends[1] ? `From ${ends[0]} to ${ends[1]}` : ticks.map(([, text]) => text).join(", ")}${own ? `; the color in hand at ${own[1]}` : ""}`)}">${ticks.map(([, text, left, mine]) => `<span${mine ? ' class="own"' : ""} style="left: ${left.toFixed(1)}px">${escapeHtml(text)}</span>`).join("")}</div>` : "";
}
function showScale() {
  const layer = layerById(selected), bar = $("inspector").querySelector("[data-gradient-bar]");
  if (!layer || !bar) return;
  const input = /** @type {HTMLInputElement | null} */ ($("inspector").querySelector("[data-stop-at]"));
  $("inspector").querySelector(".gradient-ticks")?.remove();
  bar.insertAdjacentHTML("afterend", gradientScale(layer, input ? layer.gradient?.[Number(input.dataset.stopAt)]?.[0] : undefined));
}
/** @param {Layer} layer @param {number} k */
function gradientHand(layer, k) {
  const stops = layer.gradient ?? [], few = stops.length <= GRADIENT_LEAST, at = stops[k]?.[0] ?? 0, color = stops[k]?.[1] ?? layer.color;
  return `<div class="field gradient-hand"><span class="gradient-named" data-stop-name><i class="swatch" style="background: ${escapeHtml(cssColor(color))}"></i>${escapeHtml(colorName(color))}</span><span class="gradient-at"><label class="number-field"><input class="number-input" type="text" inputmode="numeric" autocomplete="off" spellcheck="false" value="${at}" data-stop-at="${k}" aria-label="${escapeHtml(colorName(color))}, its place" enterkeyhint="done" /><span class="unit">%</span></label><button type="button" class="cap small" data-stop-remove="${k}" aria-label="Remove ${escapeHtml(colorName(color))}" data-tip="${few ? `A gradient has ${GRADIENT_LEAST} colors at least` : "Remove"}"${few ? ' aria-disabled="true"' : ""}>${ICONS.trash}</button></span></div>`;
}
/** @param {Layer} layer @param {number} at @returns {string | null} */
function gradientWorth(layer, at) {
  const result = resolved?.results.get(layer.id), unit = unitFor(layer, result), place = at / 100;
  const [low, high] = rangeOf(result?.values ?? []);
  return high >= low ? format(low + (high - low) * place, unit) : null;
}
/**
 * Never on another: one on another would hide it.
 * @param {Layer} layer @param {number} k @param {number} at
 */
function moveStop(layer, k, at) {
  const stops = layer.gradient;
  if (!stops?.[k]) return;
  const low = k ? stops[k - 1][0] + 1 : 0, high = k < stops.length - 1 ? stops[k + 1][0] - 1 : 100;
  if (low <= high) stops[k][0] = Math.round(clamp(at, low, high));
}
/** @param {Layer} layer @param {number} [at] @returns {number} */
function addStop(layer, at) {
  const stops = layer.gradient;
  if (!stops || stops.length >= GRADIENT_MOST) return -1;
  const gaps = [[0, stops[0][0]], ...stops.slice(1).map((stop, k) => [stops[k][0], stop[0]]), [stops[stops.length - 1][0], 100]];
  const widest = gaps.reduce((best, gap) => (gap[1] - gap[0] > best[1] - best[0] ? gap : best));
  const place = Math.round(at ?? (widest[0] + widest[1]) / 2);
  if (stops.some(([where]) => where === place)) return -1;
  const k = stops.findIndex(([where]) => where > place), index = k < 0 ? stops.length : k;
  const table = mixedOf(stops);
  stops.splice(index, 0, [place, table[placeIndex(place / 100, table.length)]]);
  return index;
}
/** @param {Layer} layer @param {number} k @returns {number} */
function removeStop(layer, k) {
  if (!layer.gradient || layer.gradient.length <= GRADIENT_LEAST) return k;
  layer.gradient.splice(k, 1);
  return Math.max(0, k - 1);
}
const fadingSlot = (/** @type {string} */ key) => key === "colorFading" || key === "colorBelowFading";
const ownColor = (/** @type {Layer} */ layer, /** @type {string} */ key) => (key === "colorFading" ? layer.colorFading : key === "colorBelowFading" ? layer.colorBelowFading : colorOf(layer, key));
/** @type {Map<string, { source: string, color: string | undefined }>} */
const colorSource = new Map();
function sourceNow(/** @type {Layer} */ layer, /** @type {string} */ key) {
  const last = colorSource.get(`${chart.id}:${layer.id}:${key}`), own = ownColor(layer, key);
  if (last && last.color === own && (last.source !== "auto" || fadingSlot(key))) return last.source;
  return fadingSlot(key) && own === undefined ? "auto" : SWATCHES.includes(colorOf(layer, key)) ? "presets" : "custom";
}
/** @type {Map<string, string>} */
const colorSlot = new Map();
const slotNow = (/** @type {Layer} */ layer) => ((places) => ((chosenSlot) => (places.some(([key]) => key === chosenSlot) ? /** @type {string} */ (chosenSlot) : places[0]?.[0] ?? "color"))(colorSlot.get(`${chart.id}:${layer.id}`)))(colorSlots(layer));
/** @param {Layer} layer */
const colorSources = (layer) => chart.layers.filter((other) => other.id !== layer.id && other.colorFrom !== layer.id && !placedOn(other) && !heatLike(other));
const MODE_ASIDES = { one: "The same all along", sides: "Above and below its base", momentum: "Growing and fading", levels: "Each level's color above it", layers: "Each one's own, where it's over", gradient: "By its value, its lowest to its highest", from: "Another layer's, by date" };
/** @returns {Action[]} */
function colorModeChoices(/** @type {string} */ id) {
  const layer = layerById(id);
  if (!layer) return [];
  const mode = colorModeOf(layer);
  return modesOf(layer, chart).map((each) => ({
    key: `mode:${each}`, icon: ((copy) => (setColorMode(copy, each), modeSwatch(copy, each)))(structuredClone(layer)), label: modeName(each, layer), aside: each === "sides" && layer.type === "trend" ? "Resistance and support" : each === "sides" && layer.type === "cloud" ? "The one over's, parting where they cross" : /** @type {Record<string, string>} */ (MODE_ASIDES)[each], current: mode === each,
    ...(each === "from" ? (colorSources(layer).length ? { page: () => choicePage("Colors from", () => fromChoices(id)) } : { disabled: "There's no other layer to take them from" })
      : each === "levels" && !layer.levels?.length ? { disabled: "Add a level first" }
      : { run: () => editById(id, (item) => setColorMode(item, each)), look: (/** @type {Chart} */ doc) => ((item) => item && setColorMode(item, each))(doc.layers.find((other) => other.id === id)) }),
  }));
}
/** @returns {Action[]} */
function fromChoices(/** @type {string} */ id) {
  const layer = layerById(id);
  if (!layer) return [];
  const gives = { one: "Its color", sides: "Its two colors", momentum: "Its four colors", levels: "Its levels' colors", layers: "Its layers' colors", from: "The colors it takes" };
  return colorSources(layer).map((other) => ({ key: `from:${other.id}`, icon: letterChip(other), label: labelOf(other), aside: /** @type {Record<string, string>} */ (gives)[colorModeOf(other)], current: layer.colorFrom === other.id, run: () => editById(id, (item) => setColorMode(item, "from", other.id)), look: (/** @type {Chart} */ doc) => ((item) => item && setColorMode(item, "from", other.id))(doc.layers.find((one) => one.id === id)) }));
}
function underColorMenu(/** @type {HTMLElement} */ button) {
  const layer = layerById(selected);
  if (!layer) return;
  const id = layer.id;
  colorMenu(button, colorOf(layer, "under"), "layer:under", (color) => {
    refocus('#inspector [data-color-select="under"]');
    editById(id, (item) => setColor(item, "under", color));
  }, () => ((now) => now && underColorMenu(now))($("color-select-under")), "Under its levels");
}
function colorMenu(/** @type {HTMLElement} */ anchor, /** @type {string} */ current, /** @type {string} */ target, /** @type {(color: string) => void} */ apply, again = () => colorMenu(anchor, current, target, apply), title = "") {
  const own = !SWATCHES.includes(current);
  vividAhead();
  const colors = `<div class="color-drop"${phone.matches ? "" : ` style="width: ${Math.max(232, Math.round(anchor.getBoundingClientRect().width) - 8)}px"`}>${swatchGrid(current)}</div><div class="menu-rule"></div>`;
  const custom = `<button type="button" data-pick-custom="1" aria-checked="${own}" aria-haspopup="menu">${tick(own)}<i class="swatch" style="${own ? `--c:${escapeHtml(cssColor(current))}` : "background: conic-gradient(from 90deg, #f43f5e, #f59e0b, #84cc16, #10b981, #06b6d4, #3b82f6, #8b5cf6, #ec4899, #f43f5e)"}"></i>Custom${own ? `<span>${escapeHtml(ink(current))}</span>` : ""}</button>`;
  showMenu(anchor, `${title && phone.matches ? `<div class="menu-group">${escapeHtml(title)}</div>` : ""}${colors}${custom}`, (hit) => {
    if (hit.pickCustom) colorPane(anchor, current, target, again);
    else if (hit.pickColor && hit.pickColor !== current) apply(hit.pickColor);
  }, null, anchor.closest("#belt") ? beltSide() : "");
}
function penColors(/** @type {HTMLElement} */ button) {
  const look = lookFor("color");
  if (look) colorMenu(button, look.color, "pen", (color) => {
    setLook("color", color);
    refocus(`#${button.id}`);
  }, () => ((now) => now && penColors(now))($(button.id)));
}
// sRGB's limit bends sharply at its primaries (blue's chroma nearly doubles within 5° of hue), which would crease the surface: taken at its lowest, then averaged over 12° of hue and 0.03 of lightness around each point, it has no creases and never leaves sRGB, and a soft minimum eases CHROMA into it without a seam. Built once on a grid, read between its points.
let vividGrid = /** @type {Float64Array | null} */ (null);
const VIVID_ROWS = Math.round(1 / 0.005) + 1;
const vividWork = (function* () {
  const rows = VIVID_ROWS, limits = new Float64Array(rows * 360);
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < 360; c++) limits[r * 360 + c] = limit(1 - r * 0.005, c, 12);
    yield;
  }
  const smooth = yield* fold(yield* fold(limits, true), false), eased = new Float64Array(rows * 360);
  // A soft minimum: below both, and smooth where they cross.
  for (let r = 0; r < rows; r++) {
    for (let i = r * 360; i < (r + 1) * 360; i++) eased[i] = -0.012 * Math.log(Math.exp(-CHROMA / 0.012) + Math.exp(-smooth[i] / 0.012));
    yield;
  }
  vividGrid = eased;
})();
/**
 * @param {Float64Array} line
 * @param {number} size
 * @param {boolean} lowest
 * @param {Float64Array} into
 * @param {number} at
 * @param {number} step
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
/** @param {Float64Array} source @param {boolean} lowest */
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
// Made in idle moments, a few lines at a time: folded whole in one go it held a phone without its JIT (Lockdown Mode) half a second. Only once a color can be picked soon (a layer's settings shown, a color menu open), not at every start.
let vividStarted = false;
const vividAhead = () => !vividGrid && !vividStarted && ((vividStarted = true), whenIdle(vividSoon));
function vividSoon(/** @type {{ timeRemaining: () => number }} */ deadline) {
  inSlices(deadline, () => Boolean(vividGrid || (vividWork.next(), vividGrid)) && (drawPlanes(), true), vividSoon);
}
// Made sooner when a surface is showing and waiting: a few lines each frame, not in idle moments (Safari has none to say: its are a slice every 40 ms, seconds of them for this).
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
  while (!vividGrid) vividWork.next();
  const g = vividGrid, r = clamp((1 - l) / 0.005, 0, rows - 1), c = ((h % 360) + 360) % 360;
  const r0 = Math.floor(r), r1 = Math.min(rows - 1, r0 + 1), c0 = Math.floor(c) % 360, c1 = (c0 + 1) % 360, fr = r - r0, fc = c - Math.floor(c);
  const at = (row, col) => g[row * 360 + col];
  return (at(r0, c0) * (1 - fc) + at(r0, c1) * fc) * (1 - fr) + (at(r1, c0) * (1 - fc) + at(r1, c1) * fc) * fr;
}
// Drawn at half its CSS size and stretched (it's all gradient), kept by size: nothing redraws it but a resize.
const planes = new Map();
const drawPlanes = () => all$(".plane").forEach(drawPlane);
function drawPlane(/** @type {HTMLElement} */ plane) {
  const canvas = /** @type {HTMLCanvasElement | null} */ (plane.querySelector("canvas"));
  if (!canvas || !plane.clientWidth) return;
  // Its grid not made yet: what it's in shows at once, the surface faded in as it's made (made at once, a phone without its JIT held half a second first). A pick reads the grid at once.
  plane.toggleAttribute("data-waiting", !vividGrid);
  if (!vividGrid) return void vividNow();
  // Worked out at half its size, stretched: a smooth field, the same to the eye, a quarter of the work. A pick reads the grid, not the canvas.
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
        // Its limit searched for only where it's out of sRGB: the surface's kept in it, nearly always is.
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
/** @returns {{ color: string, set: (color: string) => void } | null} */
function planeTarget(/** @type {string} */ name) {
  const layer = layerById(selected), layers = inHand();
  if (name.startsWith("layer:") && layers.length) {
    const key = name.slice(6), ids = new Set(layers.map((item) => item.id));
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
const planeOf = (/** @type {string} */ name) => /** @type {HTMLElement | null} */ (document.querySelector(`.plane[data-plane="${CSS.escape(name)}"]`));
let pickFrame = 0, picking = /** @type {{ name: string, color: string, doc: Chart, target: { color: string, set: (color: string) => void } | null } | null} */ (null);
function pick(/** @type {string} */ name, /** @type {string} */ color) {
  picking = { name, color, doc: chart, target: picking?.name === name && pickFrame ? picking.target : planeTarget(name) };
  pickFrame ||= requestAnimationFrame(applyPick);
}
function applyPick() {
  pickFrame = 0;
  if (picking?.doc === chart && picking.target && picking.target.color !== picking.color) picking.target.set(picking.color);
}
function flushPick() {
  if (!pickFrame) return;
  cancelAnimationFrame(pickFrame);
  applyPick();
}
function colorAt(u, t) {
  const l = lightAt(clamp(t, 0, 1)), h = hueAt(clamp(u, 0, 1));
  const c = l <= 0.001 || l >= 0.999 ? 0 : Math.min(vivid(l, h), maxChroma(l, h)) * keepAt(clamp(u, 0, 1));
  return oklchToHex(l, c < 0.0005 ? 0 : c, h);
}
const pickedAt = new Map();
function pickPoint(/** @type {string} */ name, u, t) {
  const color = colorAt(u, t), place = { u: clamp(u, 0, 1), t: clamp(t, 0, 1) }, plane = planeOf(name);
  if (pickedAt.size > 500) pickedAt.clear();
  pickedAt.set(color, place);
  // The ring follows at once, even along white or black, where the color stays the same as it moves (in a popover nothing redraws it: its color too).
  plane?.style.setProperty("--x", String(place.u));
  plane?.style.setProperty("--y", String(place.t));
  plane?.querySelector(".ring")?.setAttribute("style", `--c:${cssColor(color)}`);
  const said = planeSays(color, place.u);
  plane?.setAttribute("aria-valuenow", String(said.now));
  plane?.setAttribute("aria-valuetext", said.text);
  const field = plane?.closest(".field, .color-editor"), named = field?.querySelector(".color-now"), hex = /** @type {HTMLInputElement | null | undefined} */ (field?.querySelector(".hex"));
  /** @type {HTMLElement | null | undefined} */ (field?.querySelector('.slot-chip[aria-pressed="true"]'))?.style.setProperty("--c", cssColor(color));
  if (named) named.textContent = colorName(color);
  if (hex && document.activeElement !== hex) hex.value = ink(color);
  for (const dot of field ? all$(".swatch-grid [aria-checked='true']", field) : []) dot.setAttribute("aria-checked", "false");
  pick(name, color);
}
function placeOf(color) {
  const spot = pickedAt.get(color);
  if (spot) return spot;
  const [l, c, h] = shownOklch(color), t = heightOf(l), fade = (1 - HUES) * 0.85;
  if (h >= 359) {
    const keep = clamp(c / Math.max(1e-6, Math.min(vivid(l, h), maxChroma(l, h))), 0, 1);
    if (keep < 0.999) return { u: HUES + (fade * Math.acos(2 * keep - 1)) / Math.PI, t };
  }
  if (c < 0.02) return { u: (HUES + fade + 1) / 2, t };
  return { u: (Math.min(h, 359.9) / 359.9) * HUES, t };
}
const pickedOf = (/** @type {string} */ name) => (pickFrame && picking?.name === name ? picking.color : planeTarget(name)?.color || (name.startsWith("layer:") && inHand()[0] ? colorOf(/** @type {Layer} */ (inHand()[0]), name.slice(6)) : "gray"));
function pickAt(/** @type {string} */ name, clientX, clientY) {
  const plane = planeOf(name);
  if (!plane || !planeTarget(name)) return;
  const box = plane.getBoundingClientRect();
  // The outermost pixel and a half of each side is its edge: white, black and the purest grays need no aim.
  const snap = (v, size) => (v < 1.5 / size ? 0 : v > 1 - 1.5 / size ? 1 : v);
  pickPoint(name, snap((clientX - box.left) / box.width, box.width), snap((clientY - box.top) / box.height, box.height));
}
function errorBox(result, /** @type {string} */ name) {
  if (result.info && !result.info.indexes.includes(chart.index)) {
    const dated = datedIntervals(result.info);
    const away = (/** @type {string} */ id) => Math.abs(INTERVAL_ORDER.indexOf(id) - INTERVAL_ORDER.indexOf(chart.index)), nearest = dated.length ? dated.reduce((best, id) => (away(id) < away(best) ? id : best)) : "";
    if (dated.length) return `<div class="error-box">No ${escapeHtml(INTERVAL_LABEL[chart.index])} data; it has ${escapeHtml(dated.map((id) => INTERVAL_LABEL[id]).join(", "))}.<button type="button" class="btn small" data-index="${nearest}">Switch to ${escapeHtml(INTERVAL_LABEL[nearest])}</button></div>`;
    const day = dailyOf(name);
    return `<div class="error-box gone"><p>${BY_BLOCK}.</p><div class="replacement">${day ? `<span class="closest">By day: <code>${escapeHtml(day)}</code></span><button type="button" class="btn small" data-replace="${escapeHtml(day)}">Replace</button>` : ""}<button type="button" class="btn ${day ? "ghost " : ""}small" data-action="replace">${day ? "Find another" : "Find one"}</button></div></div>`;
  }
  if (result.missing) return `<div class="error-box gone"><p>Bitview has no series by this name: renamed, removed, or mistyped.</p>
          <div id="replacement" class="replacement"><span class="shimmer">Looking for the closest one…</span></div></div>`;
  return `<div class="error-box">${escapeHtml(result.error)}.<button type="button" class="btn small" data-action="retry">Retry</button></div>`;
}
function blockOnly(/** @type {string} */ name, /** @type {(day: string) => void} */ use) {
  const day = dailyOf(name), target = chart;
  notice(`${name} is only available by block, and Studio charts by date`, day ? { label: `Use ${day}`, run: () => chart === target && !home && use(day) } : undefined);
}
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
  box.innerHTML = `${best ? `<span class="closest">Closest: <code>${escapeHtml(best)}</code></span><button type="button" class="btn small" data-replace="${escapeHtml(best)}">Replace</button>` : `<span class="closest">Nothing close by name.</span>`}<button type="button" class="btn ${best ? "ghost " : ""}small" data-action="replace">${best ? "Find another" : "Find one"}</button>`;
}
function changeSeries(id) {
  const layer = layerById(id);
  if (!layer?.series) return;
  // A place the catalog no longer has (a folder renamed since) isn't known either.
  const keys = layer.path?.split(".") ?? [];
  openPalette(keys.length && nodeAt(keys) != null ? { replace: id, path: keys.slice(0, -1), landOn: keys.at(-1) } : { replace: id, query: layer.series.replace(/_/g, " ") });
}
// The latest replacement asked for is applied; any earlier one's answer is dropped.
const replacing = new Map();
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
    if (target.label === humanize(old)) delete target.label;
    // Candles of a series without them: a line, as one added would be.
    if (target.type === "candles" && meta && !isOhlc(name)) reshape(target, (item) => (item.type = "line"));
  });
  remember(name);
  undoToast(`Replaced “${was}” with “${humanize(name)}”`);
}

/** @param {Node} node */
const usesOf = (node) => [
  ...[...refsOf(node)].map((letter) => {
    const used = chart.layers.find((item) => item.letter === letter);
    return used ? `${letter} (${labelOf(used)})` : letter;
  }),
  ...refsOf(node, "written"),
  ...[...VALUES].filter((name) => JSON.stringify(node).includes(`"k":"${name}"`)),
];
/** @type {Element | null} */
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
    const uses = usesOf(node), applied = text.trim() === (layer.formula ?? "").trim();
    const what = !applied || layer.type === "signal" ? "" : !result ? "Loading" : UNIT_NAMES[unitFor(layer, result)] ?? "Number";
    status.textContent = `${uses.length ? `Uses ${uses.join(", ")}` : "A constant"}${what ? ` · ${what}` : ""}`;
  }
  status.classList.toggle("error", Boolean(message));
  $("formula-editor").classList.toggle("invalid", Boolean(message));
  if (message) $("formula")?.setAttribute("aria-invalid", "true");
  else $("formula")?.removeAttribute("aria-invalid");
  updateCallHelp();
  const label = $("label");
  if (label) label.placeholder = labelOf({ ...layer, label: "" });
}
/**
 * Shown again with its error when its layer opens (the layer keeps the last that read), till it reads and is applied, esc lets it go, or the layer's formula is changed otherwise (`over`: the formula it was typed over; see pruneDrafts).
 * @type {Map<string, Map<string, { text: string, over: string }>>}
 */
const formulaDrafts = new Map();
function pruneDrafts() {
  let shown = false;
  for (const [id, drafts] of formulaDrafts) {
    const doc = chartById(id) ?? (id === chart?.id ? chart : null);
    if (doc) for (const [layer, draft] of drafts) if (doc.layers.find((item) => item.id === layer)?.formula !== draft.over) (drafts.delete(layer), (shown ||= id === chart?.id && layer === selected));
  }
  const field = document.getElementById("formula"), layer = layerById(selected);
  if (!shown || !(field instanceof HTMLTextAreaElement) || document.activeElement === field || layer?.formula === undefined) return;
  formulaTyped++;
  field.value = layer.formula;
  updateFormula(layer);
}
let formulaLeft = /** @type {{ id: string, text: string } | null} */ (null);
function applyFormulaLeft() {
  const left = formulaLeft;
  formulaLeft = null;
  if (left) applyFormula(left.id, left.text);
}
function formulaFieldLeft(/** @type {HTMLTextAreaElement} */ field) {
  $("formula-editor")?.classList.remove("typing");
  // After the press that left it, if one did: the panel drawn anew now would lose the control pressed. Before anything leaves the chart, though (see leaveField). Left with the window too: one that reads is applied, else kept as typed, a draft.
  formulaLeft = { id: selected, text: field.value };
  formulaTyped++;
  afterPress(applyFormulaLeft);
}
function applyFormula(/** @type {string} */ id, /** @type {string} */ text) {
  const layer = layerById(id);
  if (layer?.formula === undefined) return;
  if (layer.formula === text) return void formulaDrafts.get(chart.id)?.delete(id);
  if (!formulaError(text) && !loopError(layer, text)) return (formulaDrafts.get(chart.id)?.delete(id), editById(id, (item) => (item.formula = text), { coalesce: `formula:${id}` }));
  if (!formulaDrafts.has(chart.id)) formulaDrafts.set(chart.id, new Map());
  formulaDrafts.get(chart.id)?.set(id, { text, over: layer.formula });
  const field = /** @type {HTMLTextAreaElement | null} */ ($("formula"));
  if (!field || selected !== id || document.activeElement === field) return;
  // Typing done: no draw of what was typed left to come, nor the field's own input run. The panel drawn anew meanwhile (a click that left it, on a style) shows it again.
  formulaTyped++;
  field.value = text;
  updateFormula(layer);
  $("formula-editor")?.classList.remove("typing");
}
function leaveFormula(/** @type {HTMLTextAreaElement} */ field, /** @type {boolean} */ enter) {
  const layer = layerById(selected), problem = formulaError(field.value) || (layer ? loopError(layer, field.value) : "");
  if (problem && enter) return void ($("formula-editor")?.classList.remove("typing"), say(problem));
  if (problem && layer?.formula !== undefined) {
    formulaDrafts.get(chart.id)?.delete(layer.id);
    formulaTyped++;
    field.value = layer.formula;
    updateFormula(layer);
  }
  letGo(field);
}
let formulaTyped = 0;
// Drawn once it's one to draw: it reads, its layers are here, its series are Bitview's (this machine's catalog says; before it's up, as soon as it reads). Till then the chart keeps the last one that was.
async function drawWhenReady(/** @type {string} */ id, /** @type {string} */ text) {
  const turn = ++formulaTyped, layer = layerById(id);
  if (formulaError(text) || (layer && loopError(layer, text))) return;
  const series = [...refsOf(parse(text), "series")];
  if (finder.ready && finder.worker && series.length && (await Promise.all(series.map(aboutOf))).some((about) => !about)) return;
  if (turn === formulaTyped && layerById(id)?.formula?.trim() !== text.trim()) editById(id, (layer) => (layer.formula = text), { coalesce: `formula:${id}` });
}

const suggest = {
  items: /** @type {{ insert: string, title: string, text: string, series?: boolean }[]} */ ([]),
  active: 0,
  start: 0,
  controller: /** @type {AbortController | null} */ (null),
  away: "",
  quiet: false,
};
/** @param {typeof suggest.items} items */
function showSuggestions(items, { offered = false, more = "" } = {}) {
  const list = $("suggest");
  if (!list) return;
  const chosen = suggest.active > 0 ? items.findIndex((item) => item.insert === suggest.items[suggest.active]?.insert) : -1;
  suggest.items = items;
  suggest.active = offered ? -1 : Math.max(0, chosen);
  list.hidden = !items.length;
  list.innerHTML = items.map((item, i) => `<li role="option" id="suggest-${i}" ${item.series ? 'class="series"' : ""} data-suggest="${i}" aria-selected="${i === suggest.active}"><b>${escapeHtml(item.title)}</b>${item.text ? `<span>${escapeHtml(item.text)}</span>` : ""}</li>`).join("");
  const line = $("suggest-more");
  if (line) (line.textContent = items.length ? more : ""), (line.hidden = !(items.length && more));
  chooseSuggestion(suggest.active);
  updateCallHelp();
}
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
function chooseSuggestion(/** @type {number} */ i) {
  suggest.active = i;
  for (const option of all$("[data-suggest]", $("suggest") ?? undefined)) option.setAttribute("aria-selected", String(Number(option.dataset.suggest) === i));
  const textarea = $("formula");
  if (textarea && i >= 0 && !$("suggest")?.hidden) textarea.setAttribute("aria-activedescendant", `suggest-${i}`);
  else textarea?.removeAttribute("aria-activedescendant");
  $("suggest")?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" });
}
const assistKey = () => `${selected} ${$("formula")?.value ?? ""}\u0000${$("formula")?.selectionStart ?? 0} ${$("formula")?.selectionEnd ?? 0}`;
const assistAway = () => {
  if (suggest.away && suggest.away !== assistKey()) suggest.away = "";
  return suggest.away !== "";
};
/** @param {Layer | null | undefined} self */
function completionsAt(text, caret, self) {
  // A word from its start: not the tail of another, nor a span's unit, 1ha of 1halving.
  const match = /(?<![\w.])[A-Za-z_][A-Za-z0-9_]*$/.exec(text.slice(0, caret));
  const token = match?.[0] ?? "";
  const start = match ? match.index : caret;
  /** @type {{ insert: string, title: string, text: string, layer?: string, series?: boolean }[]} */
  let items = [];
  if (/^[A-Z]/.test(token)) {
    items = formulaInputs(self).filter((item) => item.insert.startsWith(token));
    if (items.length === 1 && items[0].insert === token) items = [];
  } else if (token) items = Object.entries(FUNCTIONS).filter(([name]) => name.startsWith(token) && name !== token).map(([name, [, signature, what]]) => ({ insert: `${name}(`, title: signature, text: what }));
  const begun = token || !/^\s*(?:[,)]|$)/.test(text.slice(caret)) ? null : /[(,]\s*(\d+[a-z]*)$/.exec(text.slice(0, caret)), call = begun && callAt(text, caret);
  const name = call?.f && argNames(call.f)[call.arg];
  const spans = begun && (name === "span" || name === "span?" || name === "by") ? spansFor(/** @type {string} */ (call?.f)).filter((item) => item.insert.startsWith(begun[1]) && item.insert !== begun[1]) : [];
  if (begun && spans.length) return { start: caret - begun[1].length, token: begun[1], items: spans };
  return { start, token, items };
}
/**
 * @param {{ controller: AbortController | null }} holder
 * @param {string} token
 * @param {(series: { insert: string, title: string, text: string, series: true }[]) => void} add
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
  if (!token) {
    const fits = fitsAt(textarea.value, textarea.selectionStart, layerById(selected), textarea.selectionEnd);
    return showSuggestions(fits?.items ?? [], { offered: true, more: fits?.more });
  }
  if (quiet) return showSuggestions([]);
  showSuggestions(items);
  completeSeries(suggest, token, (series) => document.activeElement === textarea && showSuggestions([...items, ...series]));
}
let suggestedAt = "";
function suggestHere() {
  const at = `${assistKey()} ${chart.index} ${resolved?.times.length ?? 0}`;
  if (at === suggestedAt) return;
  suggestedAt = at;
  updateSuggestions();
}
function dropSuggestions() {
  suggest.controller?.abort();
  // Out of the field, it all goes at once (a press on the popover keeps the field's focus); back in, anew.
  suggest.away = suggestedAt = "";
  showSuggestions([]);
}
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
function callHelp(/** @type {string} */ text, /** @type {number} */ caret) {
  const word = /(?<![A-Za-z0-9_.])[a-z_][a-z0-9_]*$/.exec(text.slice(0, caret))?.[0] ?? "";
  // A value that's a function's name too (year) is the function only with its parenthesis.
  const call = hasFunction(word) && !(VALUES.has(word) && text[caret] !== "(") ? { f: word, arg: -1 } : callAt(text, caret);
  if (!call?.f) return "";
  const [, , what, , , usual] = FUNCTIONS[call.f], names = argNames(call.f), at = names[call.arg];
  return `<div class="call-sig"><b>${escapeHtml(call.f)}</b>(${names.map((name, i) => `<span${i === call.arg ? ' class="on"' : ""}>${escapeHtml(name)}</span>`).join(", ")})</div>
          ${call.arg < 0 ? "" : `<p>${at ? `<b>${escapeHtml(at.replace("?", ""))}</b>: ${escapeHtml(at === "span?" && usual ? `${ARGUMENTS.span}; left out, ${usual} points (as it's usually counted)` : ARGUMENTS[/** @type {keyof typeof ARGUMENTS} */ (at)] ?? "")}` : `No more arguments: ${escapeHtml(call.f)} takes ${names.length} at most`}</p>`}
          <p class="call-what">${escapeHtml(capitalized(what.replace(/\s*\([^)]*\)$/, "")))}.</p>`;
}
/** @param {string} f */
function spansFor(f) {
  const shift = ["change", "diff", "shift"].includes(f), loaded = FINE.has(chart.index) ? resolved?.times.length ?? 0 : Infinity;
  // A shift by halvings, each cycle from its first point: not at a fine interval, which loads part of history.
  const fit = SPANS.map(([span, words]) => /** @type {[string, string, number]} */ ([span, words, windowPoints(spanDays(span), chart.index)])).filter(([span, , points]) => (f === "flow" ? flowFits(spanDays(span), chart.index) : shift ? spanDays(span) >= 0.9 * INTERVAL_DAYS[chart.index] && !(f === "shift" && FINE.has(chart.index) && span.endsWith("halving")) : points >= 2) && points <= loaded);
  return fit.map(([span, words, points]) => ({ insert: span, title: span, text: chart.index === "day1" ? words : `${words} · ${grouped.format(points)} points at ${INTERVAL_LABEL[chart.index]}`, group: "Spans" }));
}
const PERCENTS = /** @type {[number, string][]} */ ([[5, "near the bottom"], [25, "the lower quartile"], [50, "the median"], [75, "the upper quartile"], [95, "near the top"]]);
/**
 * @param {Layer | null | undefined} self
 * @returns {{ items: { insert: string, title: string, text: string, layer?: string, series?: boolean, group?: string }[], more: string } | null}
 */
function fitsAt(/** @type {string} */ text, /** @type {number} */ caret, self, end = caret) {
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
 * One layer each, but where it's a pair by its nature (a band's two edges, MACD's line and signal), named so and said as two (no surprise: a percentile is one line, the 95th; another is an edit of its formula away). Each is a formula like any, its window to change in it.
 * @type {{ id: string, label: string, text: string, formulas: string[], over: boolean }[]}
 */
const STARTERS = [
  { id: "sma", label: "Moving average", text: "Its mean over a span", formulas: ["sma(X, 200d)"], over: true },
  { id: "ema", label: "Exponential average", text: "Its mean, weighting recent values more", formulas: ["ema(X, 200d)"], over: true },
  { id: "bands", label: "Deviation bands", text: "Two lines, two standard deviations above and below its mean", formulas: ["sma(X, 1y) + 2 * stdev(X, 1y)", "sma(X, 1y) - 2 * stdev(X, 1y)"], over: true },
  { id: "percentile", label: "Percentile", text: "Its 95th percentile over a span", formulas: ["percentile(X, 95, 1y)"], over: true },
  { id: "median", label: "Median", text: "The middle of its values over a span", formulas: ["median(X, 1y)"], over: true },
  { id: "zscore", label: "Z-score", text: "How many standard deviations it is from its rolling mean", formulas: ["zscore(X, 1y)"], over: false },
  { id: "change", label: "Change over time", text: "Its percent change over a span", formulas: ["change(X, 30d)"], over: false },
  { id: "ath", label: "All-time high", text: "Its highest so far", formulas: ["highest(X)"], over: true },
  { id: "log", label: "Logarithm", text: "Its base-10 logarithm", formulas: ["log(X)"], over: false },
  { id: "rsi", label: "RSI", text: "Its relative strength, 0 to 100, over its last 14 points", formulas: ["rsi(X)"], over: false },
  { id: "macd", label: "MACD and signal", text: "Two lines, its MACD and its signal (12, 26, 9)", formulas: ["macd(X)", "macd_signal(X)"], over: false },
  { id: "volatility", label: "Volatility", text: "Its annualized volatility over 30 days, in %", formulas: ["volatility(X, 30d)"], over: false },
  { id: "drawdown", label: "Drawdown", text: "How far below its highest so far, in %", formulas: ["drawdown(X)"], over: false },
];
function recipesFor() {
  const inputs = chart.layers.filter((layer) => layer.type !== "cloud" && layer.type !== "signal");
  const on = inputs.find((layer) => layer.id === selected) ?? inputs[0];
  if (!on) return [];
  // Paired by units known: one just added isn't computed yet.
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
const isUnfinished = (/** @type {string} */ text) => {
  try {
    parse(text);
    return false;
  } catch (error) {
    return Boolean(/** @type {Error & { unfinished?: true }} */ (error).unfinished);
  }
};
const FORMULA_GUIDE = `<p class="call-what">Layers by their letter, series by their name, numbers, + − × ÷ ^, comparisons (&gt; &lt; == !=, &amp;&nbsp;and&nbsp;|), date and dates (2020-05-11), and functions.</p><p class="call-what">Inside a function's parentheses, its help shows here.</p>`;
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
    // Over the inspector's status line, it says what that line would: the formula's error (one that only says it isn't finished, inside a call still open, it doesn't).
    const status = $("formula-status");
    if (help && id === "formula" && status?.classList.contains("error") && !isUnfinished(field?.value ?? "")) help += `<p class="call-error">${escapeHtml(status.textContent ?? "")}</p>`;
    const html = help || (id === "palette-input" && isOpen($("palette")) && palette.formula ? FORMULA_GUIDE : "");
    if (element.dataset.shown === html) continue;
    element.dataset.shown = html;
    element.innerHTML = html;
    element.hidden = !html;
  }
  placeAssist();
}
const completion = (/** @type {string} */ insert, /** @type {string} */ after) => (insert.endsWith("(") && after.startsWith("(") ? insert.slice(0, -1) : insert);
function acceptSuggestion(i = suggest.active) {
  const item = suggest.items[i];
  const textarea = $("formula");
  if (!item || !textarea || suggest.start > textarea.selectionStart) return;
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

// The formula box is "typing" until it's left (↵, esc, elsewhere): a pause to think isn't an error yet.
function markTyping() {
  $("formula-editor")?.classList.add("typing");
}
/** @param {() => void} run */
function afterPress(run) {
  if (!handsHeld.size) return run();
  afterHands.push(run);
}
/** @param {string} id @param {(item: any, doc: Chart) => void} edit @param {Parameters<typeof commit>[1]} [options] */
function editById(id, edit, options = undefined) {
  const drawing = Boolean(drawingById(id));
  commit((doc) => {
    const item = drawing ? doc.drawings?.find((other) => other.id === id) : doc.layers.find((other) => other.id === id);
    if (item) edit(item, doc);
  }, options);
}
const editLayer = (/** @type {(item: any, doc: Chart) => void} */ edit, /** @type {Parameters<typeof commit>[1]} */ options = undefined) =>
  editingMany() ? commit((doc) => doc.layers.forEach((item) => layerPicks.has(item.id) && edit(item, doc)), options) : editById(selected, edit, options);
/** @param {string} name */
const copyName = (name) => (navigator.clipboard?.writeText(name) ?? Promise.reject()).then(() => toast("Copied the series name"), () => notice("Couldn't copy the series name"));

// A number as typed or pasted from the app ("−2", "1,250.5"): its minus any dash, its spaces none; commas group thousands, but one with no dot and not followed by three digits is the decimal one ("2,4", "1,25"); nothing typed, or not a number, NaN.
const readNumber = (/** @type {string} */ text) => {
  const typed = text.replace(/[\s\u00a0\u2009\u202f]/g, "").replace(/^[−–—]/, "-");
  if (!typed) return NaN;
  // One comma, its three digits a thousands' (1,500), but after a 0 a decimal's (0,500).
  const decimalComma = !typed.includes(".") && (typed.match(/,/g)?.length ?? 0) === 1 && (!/,\d{3}$/.test(typed) || /^-?0,/.test(typed));
  return Number(decimalComma ? typed.replace(",", ".") : typed.replace(/,/g, ""));
};
const formatMenu = (/** @type {HTMLElement} */ anchor) => menuOf(anchor, formatChoices(selected));
const FORMAT_GROUPS = new Set(["usd", "percent", "days", "bytes", "hashrate", "coindays"]);
/** @returns {Action[]} */
function formatChoices(/** @type {string} */ id) {
  const layer = layerById(id);
  if (!layer) return [];
  const result = resolved?.results.get(layer.id), now = layer.format ?? "auto";
  return FORMATS.map(([kind]) => ({ key: kind, label: formatName({ ...layer, format: kind === "auto" ? undefined : kind }, result), current: now === kind, apart: FORMAT_GROUPS.has(kind), run: () => editById(id, (item) => (item.format = kind === "auto" ? undefined : kind)) }));
}

/** @param {string} id */
const toggleHidden = (id) => {
  const layer = layerById(id), members = layer ? [...foldedInto(chart)].filter(([, cloud]) => cloud === id).map(([member]) => member) : [];
  if (!members.length) return editById(id, (layer) => (layer.hidden = !layer.hidden || undefined), { look: true });
  const hide = !layer?.hidden || undefined;
  commit((doc) => doc.layers.forEach((other) => (other.id === id || members.includes(other.id)) && (other.hidden = hide)), { look: true });
};
const showEvery = () => commit((doc) => doc.layers.forEach((layer) => delete layer.hidden), { look: true });

const TAG_NEEDS_LINE = "Turn on the line to show its value on the axis";

const lineStyleName = (/** @type {{ dash: number, line?: false }} */ look) => (look.line === false ? "None" : dashOf(look.dash)[1]);
const NO_LINE = glyphSvg('<circle cx="8" cy="6" r="4" fill="none" stroke="currentColor" stroke-width="1.3"/><path d="M5.2 8.8 10.8 3.2" stroke="currentColor" stroke-width="1.3"/>');
const lineStyleGlyph = (/** @type {{ dash: number, line?: false }} */ look) => (look.line === false ? NO_LINE : ((d) => dashGlyph(d[2], d[3]))(dashOf(look.dash)));
/** @param {string} id @param {string | number} key @returns {any[]} */
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
/** @param {Layer} layer @param {string} unit @param {any} result */
function levelsField(layer, unit, result) {
  const levels = layer.levels ?? [], banded = Boolean(layer.byLevels || named(layer));
  const order = levels.map((_, i) => i).sort((a, b) => levels[b].value - levels[a].value || a - b);
  const swatch = (/** @type {string} */ color) => `<i class="swatch" style="--c:${escapeHtml(cssColor(color))}"></i>`;
  const row = (/** @type {number} */ i) => {
    const level = levels[i], { color } = lineLook(layer, i), said = `At ${format(level.value, unitFor(layer, result))} and up`;
    return `<div class="level" role="listitem" data-level="${i}">
            <button type="button" class="cap small" data-line-color="${i}" aria-haspopup="menu" aria-label="${escapeHtml(said)}, its color: ${escapeHtml(colorName(color))}" data-tip="Color">${swatch(color)}</button>
            <input id="level-name-${i}" class="level-name named" style="--c:${escapeHtml(cssColor(color))}" enterkeyhint="done" aria-label="${escapeHtml(said)}, its name" placeholder="Name" type="text" autocomplete="off" spellcheck="false" maxlength="40" value="${escapeHtml(level.name ?? "")}" />
            <label class="number-field"><span class="sign" aria-hidden="true">≥</span><input id="level-${i}" class="number-input" enterkeyhint="done" aria-label="${escapeHtml(said)}, its value" type="text" autocomplete="off" spellcheck="false" value="${level.value}" /></label>
            <button type="button" class="cap small" data-level-menu="${i}" aria-haspopup="menu" aria-label="More for the level ${escapeHtml(said.toLowerCase())}" data-tip="More">${ICONS.more}</button>
          </div>`;
  };
  const lowest = levels.length ? Math.min(...levels.map((level) => level.value)) : 0;
  const under = banded && levels.length ? `<div class="level under" role="listitem">
            ${layer.byLevels ? `<button type="button" id="color-select-under" class="cap small" data-color-select="under" aria-haspopup="menu" aria-label="Under its levels, its color: ${escapeHtml(colorName(layer.color))}" data-tip="Color below all its levels">${swatch(layer.color)}</button>` : `<span class="cap small" role="img" aria-label="The line's own color" data-tip="The line's color (its levels color it: see Colors)">${swatch(layer.color)}</span>`}
            <input id="level-name-under" class="level-name named" style="--c:${escapeHtml(cssColor(layer.color))}" enterkeyhint="done" aria-label="Under its levels, its name" placeholder="Name" type="text" autocomplete="off" spellcheck="false" maxlength="40" value="${escapeHtml(layer.underName ?? "")}" />
            <span class="under-label"><span class="sign" aria-hidden="true">&lt;</span><span class="vh">Under </span>${lowest}</span>
            <i class="cap small spacer" aria-hidden="true"></i>
          </div>` : "";
  return `<div class="field levels-field"><span><span>Levels${levels.length && fieldUnit(unit) ? `<span class="levels-unit"> · ${escapeHtml(fieldUnit(unit))}</span>` : ""}</span>${layer.byLevels ? '<span class="color-now">Colors the line</span>' : ""}<button type="button" class="reset" data-action="add-level" aria-label="Add a level" ${levels.length >= MAX_LEVELS ? `aria-disabled="true" data-tip="A layer holds up to ${MAX_LEVELS} levels"` : 'data-tip="Add a level at a round value above the latest"'}>${ICONS.plus}</button></span>${levels.length ? `<div class="levels" role="list" aria-label="Levels, the highest first">${order.map(row).join("")}${under}</div>` : ""}${((why) => (why ? `<p class="from-status" role="status">${escapeHtml(why)}.</p>` : ""))(levels.length ? acrossUnfit(chart, layer) || (modeOf(chart, layer) === 1 && levels.some((level) => level.value <= 0) ? "Those at 0 or under aren't drawn on a log scale" : "") : "")}</div>`;
}
/** @param {string} id @param {number} i */
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
  return `<button type="button" class="cap small" data-line-tag="${key}" aria-pressed="${Boolean(tag) && look.line !== false}" ${look.line === false ? 'aria-disabled="true"' : ""} aria-label="${label}: its value on the axis" data-tip="${look.line === false ? "Turn on the line to show its value on the axis" : tag ? "Its value on the axis" : "Show its value on the axis"}">${ICONS.label}</button><button type="button" class="cap small" data-line-color="${key}" aria-haspopup="menu" aria-label="${label} color: ${escapeHtml(colorName(color))}" data-tip="Color"><i class="swatch" style="--c:${escapeHtml(cssColor(color))}"></i></button><button type="button" class="cap small" data-line-dash="${key}" aria-haspopup="menu" aria-label="${label} line style: ${dashName}" data-tip="Line style">${lineStyleGlyph(look)}</button>`;
}
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
/** @returns {Action[]} */
const dashChoices = (/** @type {number | undefined} */ now, /** @type {(dash: number) => { run: () => void, look?: (doc: Chart) => void }} */ does) => DASHES.map(([dash, text, length, gap]) => ({ key: String(dash), icon: dashGlyph(length, gap), label: text, current: now === dash, ...does(dash) }));
/**
 * @param {() => { width?: number, dash?: number }} now
 * @param {(edit: (item: any) => void) => { run: () => void, look?: (doc: Chart) => void }} both
 */
const widthRow = (/** @type {number | undefined} */ width, now, both) => ({ key: "width", icon: ICONS.sliders, label: "Width", aside: `${width}px`, group: "Style", page: () => choicePage("Width", () => WIDTHS.map((each) => ({ key: String(each), icon: ICONS.sliders, label: `${each}px`, current: now().width === each, ...both((item) => (item.width = each)) }))) });
const dashRow = (/** @type {ReturnType<typeof dashOf>} */ style, /** @type {Parameters<typeof widthRow>[1]} */ now, /** @type {Parameters<typeof widthRow>[2]} */ both) => ({ key: "dash", icon: dashGlyph(style[2], style[3]), label: "Line style", aside: style[1], group: "Style", page: () => choicePage("Line style", () => dashChoices(now().dash, (dash) => both((item) => (item.dash = dash)))) });
function moveLayer(id, by) {
  slideLayers(id, null, () => editById(id, (layer, doc) => {
    const mates = doc.layers.filter((item) => item.pane === layer.pane), other = mates[mates.indexOf(layer) + by];
    if (!other) return;
    const a = doc.layers.indexOf(layer), b = doc.layers.indexOf(other);
    [doc.layers[a], doc.layers[b]] = [doc.layers[b], doc.layers[a]];
  }));
}
const setLineLook = (id, key, part, value, /** @type {Parameters<typeof commit>[1]} */ options = undefined) => commit((doc) => lineLookOn(doc, id, key, part, value), options);
/** @param {Chart} doc */
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
// Round, off the line (on it, it'd hide under it), never one it has: the next multiple at the size of its latest value, above that value (82,416: 90,000; 1.54: 2), then the next free one above.
function levelValue(/** @type {Layer} */ layer) {
  const value = valueAt(layer, { at: lastShown(), rest: true }), taken = new Set((layer.levels ?? []).map((level) => level.value));
  const step = Number.isFinite(value) && value !== 0 ? 10 ** Math.floor(Math.log10(Math.abs(value))) : 1;
  // 0.3 over 0.1 is 2.9999…: read at twelve figures, it's 3.
  let at = Number.isFinite(value) ? (Math.floor(Number((value / step).toPrecision(12))) + 1) * step : 0;
  while (taken.has(Number(at.toPrecision(12)))) at += step;
  return Number(at.toPrecision(12));
}
function addLevel() {
  const layer = layerById(selected);
  if (!layer) return;
  if ((layer.levels?.length ?? 0) >= MAX_LEVELS) return void notice(`A layer holds up to ${MAX_LEVELS} levels`);
  const level = levelValue(layer);
  editLayer((target) => withLevel(target, level));
  requestAnimationFrame(() => {
    const input = $(`level-${(layerById(selected)?.levels?.length ?? 1) - 1}`);
    if (byTouch) return void input?.scrollIntoView({ block: "nearest", behavior: reduced() ? "auto" : "smooth" });
    input?.focus();
    input?.select();
  });
}
const removeLayer = (/** @type {string} */ id) => removeLayers([id]);
/** @param {string[]} ids */
function removeLayers(ids) {
  const gone = chart.layers.filter((layer) => ids.includes(layer.id)), one = gone.length === 1;
  if (!gone.length) return;
  const goneName = labelOf(gone[0]), said = goneSaid(ids);
  const order = chart.layers.filter((item) => !ids.includes(item.id)), at = chart.layers.findIndex((item) => ids.includes(item.id)), after = chart.layers.slice(at).find((item) => !ids.includes(item.id)) ?? order.at(-1);
  // Asked before: where the focus is, before what has it goes.
  keepFocus(null, () => (after ? $("layer-list").querySelector(`[data-select="${CSS.escape(after.id)}"]`) : null), () => document.querySelector("#empty [data-action='add-series']"), () => $("plot-keys"));
  commit((doc) => (doc.layers = doc.layers.filter((item) => !ids.includes(item.id))));
  undoToast(`Removed ${one ? goneName : plural(gone.length, "layer")}${said}`);
}
/** @param {string[]} ids */
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
const layerMenu = (/** @type {HTMLElement} */ anchor, /** @type {string} */ id, /** @type {{ x: number, y: number } | null} */ point = null) => menuOf(anchor, layerActions(id, "menu"), point);
/** @param {string} id @param {"menu" | "panel" | "palette"} where @returns {Action[]} */
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
    ...(layer.from || placedOn(layer) || heatLike(layer) || !markable(chart).some((other) => other.id !== id) ? [] : [{ key: "bubbles", icon: glyph("bubbles"), label: "As bubbles on…", aside: inPalette ? "Sized by its values" : undefined, page: () => choicePage("As bubbles on", () => markable(chart).filter((other) => other.id !== id).map((other) => ({ key: other.id, icon: letterChip(other), label: labelOf(other), run: () => editById(id, (item) => reshape(item, (one) => Object.assign(one, { type: "bubbles", ref: other.id }))) }))) }]),
    ...(where === "panel" ? [] : [{ key: "hide", icon: layer.hidden ? ICONS.eye : ICONS.eyeOff, label: layer.hidden ? "Show" : "Hide", kbd: "H", verb: layer.hidden ? "Show" : "Hide", stay: true, run: () => toggleHidden(id) }]),
    ...(layer.type !== "signal" && othersInPane(layer) ? [{ key: "solo", icon: ICONS.eye, label: alone ? "Show every layer in its pane" : "Show it alone in its pane", aside: inPalette && !alone ? "The others there hidden" : undefined, verb: "Show", stay: true, run: () => soloLayer(id) }] : []),
    { key: "duplicate", icon: ICONS.duplicate, label: "Duplicate", kbd: "⌘D", verb: "Duplicate", run: () => (menu && layerPicks.size ? duplicateLayers([id]) : duplicateLayer(id)) },
    ...(menu && !follows(layer) ? [{ key: "move", icon: ICONS.pane, label: "Move to pane", page: () => choicePage("Move to pane", () => paneChoices([id])) }] : []),
    ...(heatLike(layer) ? [] : [{ key: "data", icon: ICONS.table, label: "Download data", aside: "CSV", verb: "Download", ...(noValues(id) ? { disabled: noValues(id) } : {}), run: () => exportData([layer]) }]),
    ...(layer.type === "heatmap" ? [{ key: "api", icon: ICONS.link, label: "Open in the API", aside: "Its latest day, as JSON", verb: "Open", run: () => window.open(`${client.baseUrl}${heatPath(chart, layer)}`, "_blank", "noopener") }] : []),
    ...(layer.series ? [{ key: "api", icon: ICONS.link, label: "Open in the API", aside: "As JSON", verb: "Open", run: () => window.open(`${client.baseUrl}/api/series/${encodeURIComponent(/** @type {string} */ (layer.series))}/${chart.index}`, "_blank", "noopener") }] : []),
    { key: "remove", icon: ICONS.trash, label: "Remove", kbd: "⌫", verb: "Remove", danger: true, run: () => removeLayer(id) },
  ];
  if (menu) /** @type {Action} */ (rows.find((row) => row.key !== "panel" && row.key !== "replace")).apart = true;
  return rows;
}
function duplicateLayer(/** @type {string} */ id) {
  const [copy] = duplicateLayers([id]);
  if (copy) select(copy);
}
/** @param {string[]} ids */
function duplicateLayers(ids) {
  const originals = chart.layers.filter((layer) => ids.includes(layer.id));
  if (!originals.length) return [];
  const copies = /** @type {string[]} */ ([]);
  commit((doc) => {
    for (const layer of originals) {
      // One of two colors, or colored by another layer or its levels, kept as it looks: a color of its own, the next.
      const paired = layer.type === "candles" || colorModeOf(layer) !== "one" || (layer.type === "signal" && (layer.colorBelow !== undefined || signedSignal(layer)));
      const copy = { ...structuredClone(layer), id: uid(), letter: nextLetter(doc), color: paired ? layer.color : nextColor(doc) };
      nameLayer(copy, layer.label && `${layer.label} copy`);
      doc.layers.splice(doc.layers.findIndex((item) => item.id === layer.id) + 1, 0, copy);
      copies.push(copy.id);
    }
  });
  toast(originals.length === 1 ? `Duplicated ${labelOf(originals[0])} as ${layerById(copies[0])?.letter}` : `Duplicated ${plural(originals.length, "layer")}`);
  return copies;
}

async function addSeries(name, { newPane = false, pane: into = /** @type {string | null} */ (null), bubbles = false } = {}) {
  const target = chart;
  retryInfo(name);
  indexCatalog();
  const meta = await info(name).catch(() => null);
  await presetsSettled();
  // Another chart opened meanwhile, it's not added; the same one made anew (a preset refreshed), it is.
  if (chart?.id !== target?.id) return;
  if (meta && !datedIntervals(meta).length) return void blockOnly(name, (day) => addSeries(day, { newPane, pane: into, bubbles }));
  const unit = meta ? unitOf(meta.type) : units.get(name) ?? "number";
  units.set(name, unit);
  // A pane of its own wanted, every pane taken and the last a stack (where it would go): said, not summed into it.
  const own = into ? null : newPane || !chart.layers.length ? undefined : paneShowing(chart, unit);
  if (!bubbles && !into && !own && chart.layers.length && chart.panes.length >= MAX_PANES && summed(chart.panes.at(-1))) return void notice(`A chart holds up to ${MAX_PANES} panes, the last a stack: add it to a pane from ⌘K's Actions`);
  /** @type {Layer | undefined} */
  let added;
  commit((doc) => {
    const look = firstLook(name, new Set(doc.layers.map((layer) => layer.color)));
    const on = bubbles ? markable(doc)[0] : undefined;
    if (on) return void doc.layers.push((added = makeLayer({ series: name, ...look, type: "bubbles", ref: on.id, pane: on.pane, side: on.side }, doc)));
    const found = into ? doc.panes.find((candidate) => candidate.id === into) : newPane || !doc.layers.length ? undefined : paneShowing(doc, unit);
    const sibling = found && doc.layers.find((layer) => layer.pane === found.id && unitFor(layer, resolved?.results.get(layer.id)) === unit);
    // One of its unit stacked: not summed into it unasked, a pane of its own.
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

function addFormula(/** @type {string | null | undefined} */ into = null) {
  const letters = chart.layers.filter((layer) => layer.type !== "cloud").map((layer) => layer.letter);
  openPalette({ formula: true, into, query: letters.length ? "" : EXAMPLE });
}
function addSignal() {
  const on = markable(chart)[0];
  if (!on) return notice("Add a layer first: a signal marks one");
  openPalette({ formula: true, signal: true, query: `${on.letter} > sma(${on.letter}, 200d)` });
}
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
/** @param {string | null | undefined} pane */
const stackedOnly = (pane) => !(pane && chart.panes.some((item) => item.id === pane)) && chart.panes.length >= MAX_PANES && summed(chart.panes.at(-1));
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
 * `over`: this one over the ones before it, all of history (rebase(shift(A, 1halving), halving_epoch): each line broken where it starts anew), as far back as it has values whole, three at most (more cross into a tangle), the oldest first.
 * @param {Layer} layer @param {"cycles" | "years"} which @returns {{ formula: string, label: string }[]}
 */
function periodsOf(layer, which, over = false) {
  const year = (/** @type {number} */ time) => new Date(time * 1000).getUTCFullYear(), cycles = which === "cycles";
  const values = resolved?.results.get(layer.id)?.values, at = values ? values.findIndex(Number.isFinite) : -1, first = resolved && at >= 0 ? resolved.times[at] : GENESIS;
  // Laid over each other, whole ones alone: those it has a value at the first point of, as rebase reads a moved one (one begun later has no 1× at its start). Years not before the first year() takes: a week's or a month's first point can be dated before the genesis block.
  const times = resolved?.times ?? [], whole = (/** @type {number} */ start, /** @type {number} */ end) => {
    const i = lowerBound(times, start);
    return values !== undefined && i < times.length && times[i] < end && (i > 0 || times[0] <= start) && Number.isFinite(values[i]) && values[i] !== 0;
  };
  const now = year(Date.now() / 1000), from = Math.max(PERIODS_TAKEN.year[0], year(first));
  const starts = cycles ? [GENESIS, ...minedHalvings()] : Array.from({ length: now - from + 2 }, (_, k) => Date.UTC(from + k, 0, 1) / 1000);
  const periods = starts.slice(0, cycles ? starts.length : -1).map((start, n) => ({ n, start, whole: whole(start, starts[n + 1] ?? Infinity) }));
  if (over) {
    let before = 0;
    if (periods.at(-1)?.whole) while (before < 3 && before + 1 < periods.length && periods[periods.length - 2 - before].whole) before++;
    const [unit, by, what] = cycles ? ["halving", "halving_epoch", "cycle"] : ["y", "year", "year"];
    if (before < 1) return [];
    return Array.from({ length: before + 1 }, (_, i) => ((k) => ({ formula: k ? `rebase(shift(${layer.letter}, ${k}${unit}), ${by})` : `rebase(${layer.letter}, ${by})`, label: k ? `${k} ${what}${k > 1 ? "s" : ""} before` : `This ${what}` }))(before - i));
  }
  const any = (/** @type {number} */ start, /** @type {number} */ end) => {
    for (let i = lowerBound(times, start); i < times.length && times[i] < end; i++) if (values && Number.isFinite(values[i])) return true;
    return false;
  };
  return periods.filter(({ n, start }) => any(start, starts[n + 1] ?? Infinity)).map(({ n, start }) => (cycles ? { formula: `halving(${layer.letter}, ${n})`, label: `${year(start)}–${starts[n + 1] ? year(starts[n + 1]) : "now"}` } : { formula: `year(${layer.letter}, ${from + n})`, label: String(from + n) }));
}
/** @param {"cycles" | "years"} which */
const overShort = (which) => (which === "cycles" ? "No whole halving cycle before this one" : "No whole year before this one");
const cloudable = () => chart.layers.some((layer) => !layer.from && !placedOn(layer));
// A cloud goes behind them (after the last of them there), a trend in front (a line over what it follows); in a pane whose last is in front, the other way round.
function addFrom(/** @type {"cloud" | "trend"} */ type, /** @type {string[]} */ ids) {
  const members = fromOf(chart, ids, type);
  const short = type === "cloud" && membersOf(chart, ids, true).length < 2 ? "Pick what it's between" : fromShort({ type, from: members });
  if (short) return void notice(short === ONE_SCALE ? `A ${type} reads its layers as one: put them in one pane, on one side` : short === TREND_FLIP ? "A trend flips when another layer touches it: add one (a price)" : type === "cloud" ? "A cloud is between two layers or more" : "A trend is on one of two layers or more");
  /** @type {Layer | undefined} */
  let added;
  commit((doc) => {
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
 * Behind the price's layers, in dollars as they are, on a log scale as the price is; a pane with nothing in it gets the price as its candles (what it's read against).
 * @param {string} cohort @param {string[]} [profile]
 */
function addHeatmap(cohort, profile = undefined) {
  /** @type {Layer | undefined} */
  let added;
  commit((doc) => {
    // A heatmap's axis is in dollars, whatever it measures: another's pane is one.
    const inDollars = (/** @type {Layer} */ layer) => heatLike(layer) || unitFor(layer, resolved?.results.get(layer.id)) === "usd";
    const drawable = (/** @type {string | undefined} */ id) => ((item) => Boolean(item) && paneMode(item) < 2)(doc.panes.find((item) => item.id === id));
    const empty = (/** @type {string | undefined} */ id) => drawable(id) && !doc.layers.some((layer) => layer.pane === id);
    const dollars = (/** @type {string | undefined} */ id) => drawable(id) && doc.layers.some((layer) => layer.pane === id && inDollars(layer)) && !doc.panes.find((item) => item.id === id)?.stack;
    const chosen = doc.layers.find((layer) => layer.id === selected)?.pane, first = doc.panes[0].id;
    const pane = doc.panes.find((item) => item.id === palette.into)?.id ?? (chosen && (empty(chosen) || dollars(chosen)) ? chosen : doc.panes.find((item) => dollars(item.id))?.id ?? (empty(first) ? first : doc.panes.length < MAX_PANES ? ((made) => ((made.mode = 1), made.id))(newPaneIn(doc)) : doc.panes.find((item) => drawable(item.id))?.id ?? first));
    if (!doc.layers.some((layer) => layer.pane === pane)) doc.layers.push(makeLayer({ series: tree.price.ohlc.name, label: "Price", type: "candles", color: "ink", colors: 1, pane }, doc));
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
/** @param {Layer | null} made @param {Layer | null} first @param {(id: string) => boolean} has */
const memberCandidates = (made, first, has) => chart.layers.filter((layer) => (memberable(layer, made, first) || has(layer.id)) && layer.id !== made?.ref);
const EDGE_NAMES = /** @type {Record<string, string>} */ ({ top: "The pane's top", bottom: "The pane's bottom" });
const edgeIcon = (/** @type {string} */ edge) => (edge === "top" ? ICONS.toTop : ICONS.toBottom);
/** @param {"cloud" | "trend"} type @returns {Page} */
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
    /** @param {string} member @param {Record<string, any>} row */
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
/** @param {Layer} layer */
const betweenField = (layer) => {
  const has = (/** @type {string} */ member) => Boolean(layer.from?.includes(member)), edges = edgesOf(layer);
  const chips = [...memberCandidates(layer, placedBy(chart, layer), has).map((item) => ({ id: item.id, icon: letterChip(item), name: labelOf(item), tip: "" })), ...(layer.type === "cloud" ? EDGES.map((edge) => ({ id: edge, icon: edgeIcon(edge), name: capitalized(edge), tip: `${EDGE_NAMES[edge]}: from its layers ${edge === "top" ? "up" : "down"} to it` })) : [])];
  return `<div class="field"><span>Between<span class="color-now">${plural(fromLetters(layer, chart).length, "layer")}${edges.length ? `, the ${edges.join(" and ")}` : ""}</span></span><div class="from-of" role="group" aria-label="What it's between">${chips.map(({ id, icon, name, tip }) => `<button type="button" data-member-of="${id}" aria-pressed="${has(id)}"${tip ? ` data-tip="${escapeHtml(tip)}"` : ""}>${icon}<span class="name">${escapeHtml(name)}</span></button>`).join("")}</div></div>`;
};
const refMenu = (/** @type {HTMLElement} */ button, /** @type {string} */ id) => menuOf(button, refChoices(id));
/** @returns {Action[]} */
function refChoices(/** @type {string} */ id) {
  const trend = layerById(id);
  if (!trend) return [];
  return (placedOn(trend) ? markable(chart) : refsFor(chart, trend).filter((layer) => oneScale(layer, trend) || layer.id === trend.ref)).map((layer) => ({ key: layer.id, icon: letterChip(layer), label: labelOf(layer), current: trend.ref === layer.id, run: () => trend.ref === layer.id || editById(id, (item) => (item.ref = layer.id)) }));
}
/** @returns {Action[]} */
function deriveChoices(/** @type {string} */ id) {
  const layer = layerById(id);
  if (!layer) return [];
  return [
    ...STARTERS.map((starter) => ({ key: starter.id, icon: ICONS.formula, label: starter.label, aside: starter.formulas.length > 1 ? `${starter.formulas.length} layers` : starter.formulas[0].replaceAll("X", layer.letter), mono: starter.formulas.length === 1, ...((why) => (why ? { disabled: why } : {}))(unfitHere(starter.formulas.map((formula) => formula.replaceAll("X", layer.letter)))), run: () => derive(id, starter.id) })),
    { key: "own", icon: ICONS.pencil, label: "Write your own", aside: `From ${layer.letter}`, verb: "Write", apart: true, run: () => derive(id, "own") },
  ];
}
const onStack = (/** @type {Layer} */ layer) => summed(chart.panes.find((pane) => pane.id === layer.pane));
function derive(/** @type {string} */ id, /** @type {string} */ starter) {
  const source = layerById(id);
  if (!source) return;
  if (starter === "own") return ((stacked) => openPalette({ formula: true, into: stacked ? null : source.pane, query: source.letter, side: stacked ? null : source.side }))(onStack(source));
  const formulas = (STARTERS.find((item) => item.id === starter)?.formulas ?? []).map((text) => text.replaceAll("X", source.letter));
  if (!formulas.length) return;
  // Over a stack, it would add to it: a pane of its own then too.
  const over = Boolean(STARTERS.find((item) => item.id === starter)?.over) && !onStack(source);
  // One that needs a pane of its own, at the most panes: not put in the last one, a stack's maybe.
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

/** @returns {Action[]} */
const seriesChoices = (/** @type {string} */ id) => ((layer) => (layer ? turnsInto(layer) : []).map((name) => ({ key: name, icon: ICONS.series, label: name, mono: true, run: () => toSeries(id, name) })))(layerById(id));
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
  acceptSuggestion, addFormula, addFormulaLayer, addFrom, addHeatmap, addLevel, addSeries, addSignal, addSignalLayer,
  addStop, afterPress, applyFormula, applyFormulaLeft, assistKey, barAt, changeSeries, chooseSuggestion,
  closestSeries, cloudable, colorLevels, colorModeChoices, colorModeOf, colorOf, colorSlot, colorSlots, colorSource,
  completeSeries, completion, completionsAt, copyName, dashRow, drawPlanes, drawWhenReady, dropSuggestions,
  duplicateLayer, duplicateLayers, editAdded, editById, editingMany, editLayer, editPicked, 
  fitChoices, fitOf, FITS, fitsAt, flushPick, formatChoices, formatMenu, formulaDrafts, formulaFieldLeft,
  formulaInputs, fromChoices, fromPage, goneSaid, gradientEdit,
  gradientSwatch, hexField, hideLayers, HUES, inHand, insertAtCaret, inspectorDue, isUnfinished,
  layerActions, layerMenu, layersCommands, layersMenu, leaveFormula, levelRows,
  levelValue, lineChoices, lineLookOn, lineMenu, lineStyleGlyph, lineStyleName,
  markTyping, modeName, modesFor, modeSwatch, moveLayer, movers, moveStop, multiEdit, overShort,
  ownColor, paneChoices, paneMenu, penColors, periodsOf, pick, pickAt, pickedAt, pickedLayers,
  pickedOf, pickPoint, pickPress, placeAssist, placeOf, planeHtml, planeOf, planeTarget, pruneDrafts, 
  readNumber, readyName, readyPage, recipesFor, refChoices, refMenu, removeLayer, removeLayers, removeStop,
  renderInspector, renderLayersBar, replaceSeries, resetFor, reverseGradient, rewrite, setColor, setFit,
  setLineLook, setSlider, sharedBy, showEvery, showPicks, showSelection, showStop, showSuggestions, slider,
  sliders, slotName, soloLayer, stackedIn, stackedOnly, suggest, suggestHere,
  TAG_NEEDS_LINE, toggleDescription, toggleHidden, underColorMenu, updateCallHelp, updateFormula, usesOf, widthRow,
  withLevel, withoutLevel,
};
