// @ts-check
// Drawing on the chart: lines, boxes, text and strokes.
import { capitalized } from "/studio/words.js";
import {
  beltStop, chart, chosen, commit, engine, hideBelt, magnifying, MOTION, phone, prefs, reduced, renderBelt, resolved,
  savePrefs, selected, selectingArea, shownDrawings, takeUp, timeAt, tool,
} from "/studio/app.js";
import { $, all$, escapeHtml, keyText, MAC } from "/studio/dom.js";
import { clamp, uid } from "/studio/math.js";
import { glyphSvg, ICONS } from "/studio/icons.js";
import { placedOn } from "/studio/vocabulary.js";
import {
  BACKS, ENDS, LENS_SAYS, LINES, lookOfDrawing, MAX_DRAWINGS, pointOf, THROUGH, TOOLS,
} from "/studio/tools.js";
import {
  built, cssColor, drawn, fitStrip, seriesValue, seriesY, shownIndex,
} from "/studio/chart.js";
import { paneAt } from "/studio/formulas.js";
import { slider } from "/studio/inspector.js";
import { showMenu } from "/studio/menus.js";
import { refocus } from "/studio/presets-edit.js";
import { sketches, timeLine, contrastOn, plotBox, calendarAt, PULL, REACH } from "/studio/sketch.js";
/** @import { End } from "/studio/tools.js" */
/** @import { Drawing, DrawingKind } from "/studio/vocabulary.js" */

const endsGlyph = (/** @type {End | "" | undefined} */ start, /** @type {End | "" | undefined} */ end) => {
  const at = [start === "extend" ? 5 : 3.5, end === "extend" ? 11 : 12.5], from = start === "extend" ? 0.5 : start === "arrow" ? 5 : at[0], to = end === "extend" ? 15.5 : end === "arrow" ? 11 : at[1];
  const tip = (/** @type {number} */ x, /** @type {number} */ way) => `<path d="M${x} 6 ${x - way * 4.5} 3.2v5.6Z" fill="currentColor"/>`;
  return glyphSvg(`<path d="M${from} 6H${to}" stroke="currentColor" stroke-width="1.5"/>${start === "arrow" ? tip(1, -1) : `<circle cx="${at[0]}" cy="6" r="1.75" fill="currentColor"/>`}${end === "arrow" ? tip(15, 1) : `<circle cx="${at[1]}" cy="6" r="1.75" fill="currentColor"/>`}`);
};

const reachOf = (event) => (event.pointerType === "touch" ? 14 : REACH);
const drawingById = (id) => (id ? chart.drawings?.find((drawing) => drawing.id === id) : undefined);
function drawingNames(doc = chart) {
  const all = doc.drawings ?? [], named = (drawing) => (drawing.kind === "text" ? `“${(drawing.text ?? "").trim().split("\n")[0].slice(0, 32)}”` : TOOLS[drawing.kind].name);
  const counts = new Map(), seen = new Map();
  for (const drawing of all) counts.set(named(drawing), (counts.get(named(drawing)) ?? 0) + 1);
  return new Map(all.map((drawing) => {
    const name = named(drawing), n = (seen.get(name) ?? 0) + 1;
    seen.set(name, n);
    return [drawing.id, counts.get(name) > 1 ? `${name} ${n}` : name];
  }));
}
/** @param {DrawingKind} kind @returns {Drawing} */
function newDrawing(kind, layer, points) {
  return { id: uid(), kind, layer, points, ...lookOfDrawing(kind, prefs.pen), ...(kind === "text" ? { text: "" } : {}) };
}

function timeOfBar(bar) {
  const times = resolved?.times ?? [], n = times.length;
  if (bar >= 0 && bar <= n - 1) {
    const i = Math.floor(bar), share = bar - i;
    return share ? times[i] + share * (times[i + 1] - times[i]) : times[i];
  }
  const edge = bar > n - 1 ? n - 1 : 0, at = calendarAt(times[edge]) + bar - edge, i = Math.max(0, Math.floor(at)), index = shownIndex(), from = timeAt(index, i);
  return from + (at - i) * (timeAt(index, i + 1) - from);
}

function shareAlong(/** @type {Drawing} */ drawing, points, size, x, y) {
  const [a, b] = points;
  if (drawing.kind === "hline") return clamp(x / size.width, 0, 1);
  if (drawing.kind === "vline") return clamp(y / size.height, 0, 1);
  const dx = b.x - a.x, dy = b.y - a.y, length = dx * dx + dy * dy;
  return length ? clamp(((x - a.x) * dx + (y - a.y) * dy) / length, 0, 1) : 0.5;
}

const LENS = 168, LENS_ZOOM = 2;
const putDown = () => takeUp("");
const boxKey = (/** @type {PointerEvent} */ event) => (MAC ? event.metaKey : event.ctrlKey);
const upright = (/** @type {string} */ side) => side === "left" || side === "right";
const beltTool = (/** @type {string} */ each) =>
  each === "cursor" ? { name: "Cursor", key: "", kbd: "esc", icon: ICONS.cursor }
  : each === "select" ? { name: "Select", tip: `Select: drag a box around drawings, or ${keyText("⌘")} and drag anytime`, key: "S", kbd: "D S", icon: ICONS.select }
  : each === "magnifier" ? { name: "Magnifier", tip: `Magnifier: shows ${LENS_SAYS}`, key: "Z", kbd: "D Z", icon: ICONS.magnifier }
  : { name: TOOLS[each].name, key: TOOLS[each].key, kbd: each === "ruler" ? "M" : `D ${TOOLS[each].key}`, icon: TOOLS[each].icon };
function rovingBelt() {
  const buttons = /** @type {HTMLButtonElement[]} */ (all$("button", $("belt"))).filter((button) => !button.disabled && button.checkVisibility());
  const stop = buttons.find((button) => button.id && button.id === beltStop) ?? buttons.find((button) => button.dataset.tool && button.getAttribute("aria-pressed") === "true") ?? buttons[0];
  for (const button of buttons) button.tabIndex = button === stop ? 0 : -1;
}
// One motion, nothing jumping or blinking: each part moves from where it was; fading all in again blinked the tools.
/** @type {Animation | null} */
let beltMotion = null;
function settleBelt() {
  const belt = $("belt");
  beltMotion = null;
  for (const motion of belt.getAnimations({ subtree: true })) motion.cancel();
  for (const child of all$(":scope > :not(.belt-row), :scope > .belt-row > *", belt)) Object.assign(child.style, { minWidth: "", minHeight: "" });
}
function reshapeBelt(/** @type {DOMRect} */ was, /** @type {Map<string, DOMRect>} */ partsWere) {
  const belt = $("belt");
  settleBelt();
  const now = belt.getBoundingClientRect();
  if (Math.abs(now.left - was.left) + Math.abs(now.top - was.top) + Math.abs(now.width - was.width) + Math.abs(now.height - was.height) < 2) return;
  const timing = { duration: MOTION.small, easing: MOTION.ease }, left = parseFloat(belt.style.left), top = parseFloat(belt.style.top);
  // Measured all first, then moved: one layout, the box's own motion not in it.
  const upright = belt.classList.contains("upright"), parts = all$(":scope > .grip, .belt-row :is(button, .belt-rule), .belt-caption", belt).map((part) => /** @type {[HTMLElement, DOMRect, DOMRect | undefined]} */ ([part, part.getBoundingClientRect(), part.id ? partsWere.get(part.id) : undefined]));
  // Held at their size as the box reshapes: narrower, the row would wrap and a tool jump to a second line.
  const held = all$(":scope > :not(.belt-row), :scope > .belt-row > *", belt).map((child) => /** @type {[HTMLElement, DOMRect]} */ ([child, child.getBoundingClientRect()]));
  for (const [child, box] of held) Object.assign(child.style, { minWidth: `${box.width}px`, minHeight: `${box.height}px` });
  const motion = belt.animate({ left: [`${left + was.left - now.left}px`, `${left}px`], top: [`${top + was.top - now.top}px`, `${top}px`], width: [`${was.width}px`, `${now.width}px`], height: [`${was.height}px`, `${now.height}px`], overflow: ["hidden", "hidden"] }, timing);
  beltMotion = motion;
  const release = () => beltMotion === motion && held.forEach(([child]) => Object.assign(child.style, { minWidth: "", minHeight: "" }));
  motion.finished.then(release, release);
  for (const [part, at, from] of parts) {
    if (from) part.animate({ translate: [`${from.left - at.left - (was.left - now.left) - (upright ? (was.width - now.width) / 2 : 0)}px ${from.top - at.top - (was.top - now.top) - (upright ? 0 : (was.height - now.height) / 2)}px`, "0 0"] }, timing);
    else part.animate({ opacity: [0, 1] }, timing);
  }
}

const dockChoices = () => /** @type {const} */ (["top", "bottom", "left", "right"]).map((side) => ({ key: side, label: capitalized(side), current: prefs.belt.dock === side, run: () => dockBelt(side) }));
function dockBelt(/** @type {"top" | "bottom" | "left" | "right"} */ side) {
  Object.assign(prefs.belt, { dock: side, at: 0.5 });
  savePrefs();
  turnBelt(upright(side));
  placeBelt(true);
}
let beltSize = { width: 0, height: 0 };
let beltPressedSeen = "";
function placeBelt(glide = false, stay = false) {
  // Where the plot will be: through a layout's motion the belt goes along with the chart's box, not the chart held meanwhile (it'd jump at the motion's end).
  const belt = $("belt"), plot = belt.hidden || belt.classList.contains("held") ? null : plotBox(true);
  if (!plot) return;
  // Measured at rest: a glide still going ends where it was going (its coming in, a fade and 4px, isn't one: it plays).
  belt.getAnimations().forEach((animation) => animation instanceof CSSAnimation || animation.finish());
  if (phone.matches) {
    const foot = $("stage-foot").getBoundingClientRect(), canvas = $("canvas").getBoundingClientRect();
    belt.classList.remove("upright");
    belt.dataset.dock = "bottom";
    Object.assign(belt.style, { maxWidth: "", width: `${foot.width}px`, height: `${foot.height}px`, left: `${foot.left - canvas.left}px`, top: `${foot.top - canvas.top}px` });
    const pressed = belt.querySelector('[aria-pressed="true"]');
    if (pressed?.id !== beltPressedSeen) pressed?.scrollIntoView({ inline: "nearest", block: "nearest" });
    beltPressedSeen = pressed?.id ?? "";
    const row = /** @type {HTMLElement | null} */ (belt.querySelector(".belt-row"));
    if (row) fitStrip(row);
    const caption = /** @type {HTMLElement | null} */ (belt.querySelector(".belt-caption"));
    if (caption) Object.assign(caption.style, { position: "fixed", top: "auto", right: "auto", left: `${foot.left + 10}px`, bottom: `${innerHeight - $("chart").getBoundingClientRect().bottom + engine.timeHeight + 6}px`, translate: "none", transform: "none" });
    return;
  }
  belt.style.width = belt.style.height = "";
  belt.querySelector(".belt-caption")?.removeAttribute("style");
  let side = prefs.belt.dock, up = upright(side);
  belt.classList.toggle("upright", up);
  if (up && belt.offsetHeight > plot.height - 16) {
    [side, up] = ["bottom", false];
    belt.classList.remove("upright");
  }
  belt.style.maxWidth = up ? "" : `${plot.width - 16}px`;
  // Wider than the plot it wraps into even rows: a tool alone on a row reads as dropped there.
  if (!up && belt.scrollHeight > belt.querySelector(".cap")?.getBoundingClientRect().height * 1.5 + 8) {
    const rows = belt.offsetHeight;
    let low = 0, high = belt.offsetWidth;
    while (high - low > 2) {
      const mid = (low + high) / 2;
      belt.style.maxWidth = `${mid}px`;
      belt.offsetHeight > rows ? (low = mid) : (high = mid);
    }
    belt.style.maxWidth = `${high}px`;
  }
  const width = belt.offsetWidth, height = belt.offsetHeight;
  const along = (/** @type {number} */ from, /** @type {number} */ room, /** @type {number} */ size) => clamp(from + prefs.belt.at * room - size / 2, from + 8, from + room - size - 8);
  const was = { left: parseFloat(belt.style.left), top: parseFloat(belt.style.top) }, kept = stay && belt.dataset.dock === side;
  const left = up ? (side === "left" ? plot.left + 10 : plot.left + plot.width - width - 10) : kept && Number.isFinite(was.left) ? clamp(was.left + (beltSize.width - width) / 2, plot.left + 8, plot.left + plot.width - width - 8) : along(plot.left, plot.width, width);
  const top = up ? (kept && Number.isFinite(was.top) ? clamp(was.top + (beltSize.height - height) / 2, plot.top + 8, plot.top + plot.height - height - 8) : along(plot.top, plot.height, height)) : side === "top" ? plot.top + 10 : plot.top + plot.height - height - 10;
  beltSize = { width, height };
  belt.dataset.dock = side;
  if (glide && !reduced()) belt.animate({ left: [belt.style.left, `${left}px`], top: [belt.style.top, `${top}px`] }, { duration: MOTION.small, easing: MOTION.ease });
  Object.assign(belt.style, { left: `${left}px`, top: `${top}px` });
}
function sideAt(/** @type {number} */ x, /** @type {number} */ y, plot) {
  const edge = Math.min(120, plot.width * 0.15);
  return x < plot.left + edge ? "left" : x > plot.left + plot.width - edge ? "right" : y < plot.top + plot.height / 2 ? "top" : "bottom";
}
function turnBelt(/** @type {boolean} */ up) {
  const belt = $("belt");
  if (belt.classList.contains("upright") === up) return;
  const parts = all$(".grip, .cap, .belt-rule", belt), from = belt.getBoundingClientRect(), before = parts.map((part) => part.getBoundingClientRect());
  settleBelt();
  belt.classList.toggle("upright", up);
  if (reduced()) return;
  const to = belt.getBoundingClientRect(), timing = { duration: MOTION.small, easing: MOTION.ease };
  parts.forEach((part, i) => {
    const now = part.getBoundingClientRect(), dx = before[i].left - from.left - (now.left - to.left), dy = before[i].top - from.top - (now.top - to.top);
    if (dx || dy) part.animate({ transform: [`translate(${dx}px, ${dy}px)`, "none"] }, timing);
  });
  belt.animate({ width: [`${from.width}px`, `${to.width}px`], height: [`${from.height}px`, `${to.height}px`] }, timing);
}
function showBelt() {
  prefs.belt.on = true;
  savePrefs();
  renderBelt();
}
const toggleBelt = () => (prefs.belt.on ? hideBelt() : showBelt());
const beltUp = () => !$("belt").hidden;
function pickTool(/** @type {string} */ each) {
  if (each === "select") return takeUp(selectingArea ? "" : "select");
  if (each === "magnifier") return takeUp(magnifying ? "" : "magnifier");
  takeUp(each === "cursor" || each === tool ? "" : /** @type {DrawingKind} */ (each));
}
const hasLook = (/** @type {string | undefined} */ kind, /** @type {string} */ part) => !!kind && kind !== "ruler" && (part === "width" || part === "dash" ? kind !== "text" : part === "ends" || part === "start" || part === "end" ? THROUGH.has(kind) : part === "size" || part === "back" ? kind === "text" || LINES.has(kind) : true);
const wordsIn = (/** @type {(Drawing | undefined)[]} */ drawings) => drawings.find((item) => item?.kind === "text" || (item && LINES.has(item.kind) && item.label));
const backOf = (/** @type {string | undefined} */ back) => BACKS.find(([each]) => each === (back ?? "")) ?? BACKS[0];
const backGlyph = (/** @type {string} */ back) => glyphSvg(`${back ? `<rect x="1" y="2" width="14" height="8" rx="2.5" ${back === "outline" ? 'fill="none"' : `fill="currentColor" fill-opacity="${back === "solid" ? 1 : 0.4}"`} ${back.includes("outline") ? 'stroke="currentColor" stroke-width="1.2"' : 'stroke="none"'}/>` : ""}<path d="M4.5 6h7" stroke-width="1.6" stroke-linecap="round" ${back === "solid" ? 'style="stroke: var(--card)"' : 'stroke="currentColor"'}/>`);
const backChip = (/** @type {string} */ back, /** @type {string} */ color) => `<span class="back-chip" data-back="${back}" style="--c:${escapeHtml(cssColor(color))};--on:${contrastOn(color)}">Aa</span>`;
const lookFor = (/** @type {string} */ part, drawings = /** @type {Drawing[]} */ ([...chosen].map(drawingById).filter(Boolean))) => (tool ? prefs.pen : drawings.find((drawing) => hasLook(drawing.kind, part)) ?? drawings[0]);
const endName = (/** @type {End | "" | undefined} */ end) => (ENDS.find(([each]) => each === (end ?? "")) ?? ENDS[0])[1];
function endsPopover(/** @type {HTMLElement} */ button) {
  const look = lookFor("ends");
  if (!look) return;
  const row = (/** @type {"start" | "end"} */ side) => `<div class="ends-row"><span aria-hidden="true">${side === "start" ? "Start" : "End"}</span>${beltTabs(side === "start" ? "Start" : "End", ENDS.map(([end, text, tip]) => `<button type="button" data-pick-${side}="${end}" aria-pressed="${(look[side] ?? "") === end}" aria-label="${side === "start" ? "Start" : "End"}: ${text}" data-tip="${text}: ${tip.toLowerCase()}">${side === "start" ? endsGlyph(end, "") : endsGlyph("", end)}</button>`))}</div>`;
  beltPopover(button, "Ends", `${row("start")}${row("end")}`, () => {});
}
function setLook(/** @type {string} */ part, value, coalesce = "", keepMenu = false) {
  const apply = (/** @type {Record<string, any>} */ look) => {
    if (value === undefined) delete look[part];
    else look[part] = value;
  };
  apply(prefs.pen);
  savePrefs();
  if (tool) return renderBelt();
  const ids = [...chosen].filter((id) => hasLook(drawingById(id)?.kind, part));
  if (!ids.length) return;
  commit((doc) => doc.drawings?.forEach((drawing) => ids.includes(drawing.id) && apply(drawing)), { keepMenu, ...(coalesce ? { coalesce: `${coalesce}:${ids.join()}` } : {}) });
}
const beltSide = () => ({ bottom: "above", top: "below", left: "right", right: "left" })[$("belt").dataset.dock ?? ""] ?? "";
function beltPopover(/** @type {HTMLElement} */ button, /** @type {string} */ title, /** @type {string} */ body, onPick) {
  showMenu(button, `<div class="field belt-pop"><span>${title}</span>${body}</div>`, (data) => {
    onPick(data);
    refocus(`#${button.id}`);
  }, null, beltSide());
}
const beltTabs = (/** @type {string} */ label, /** @type {string[]} */ items) => `<div class="tabs ${items.some((item) => item.includes("<svg")) ? "glyphs" : ""}" role="group" aria-label="${label}">${items.join("")}</div>`;
function setDrawSlider(/** @type {"draw-width" | "draw-opacity"} */ id, /** @type {number | undefined} */ value) {
  const part = id === "draw-width" ? "width" : "opacity", look = lookFor(part), element = $(id);
  if (value === undefined || !look) return;
  setLook(part, part === "opacity" && value === 1 ? undefined : value, part, true);
  const now = lookFor(part);
  if (element && now) {
    element.outerHTML = slider(id, now);
    if ($("menu").contains(document.activeElement) || document.activeElement === document.body) $(id)?.focus();
  }
}

const plotPoint = (clientX, clientY, rect) => ({ x: clientX - engine.element.getBoundingClientRect().left - engine.left, y: clientY - rect.top });
function drawingAt(clientX, clientY, reach = REACH) {
  const pane = shownDrawings.length ? paneAt(clientY) : null, id = pane && chart.panes[pane.index]?.id;
  if (!pane || !id) return null;
  const at = plotPoint(clientX, clientY, pane.rect);
  if (at.x < 0 || at.x > engine.time.width) return null;
  // Top as painted: the pane's first layer over the others, then the later added; a point of a drawing (a corner, an end) over any body.
  const inPane = chart.layers.filter((layer) => layer.pane === id);
  const order = new Map((chart.panes[pane.index].reverse ? inPane : inPane.reverse()).map((layer, i) => [layer.id, i])), score = (hit) => (hit.handle >= 0 && hit.over !== "ew" && hit.over !== "ns" ? MAX_DRAWINGS * (chart.layers.length + 1) : 0) + (order.get(hit.drawing.layer) ?? 0) * MAX_DRAWINGS + shownDrawings.indexOf(hit.drawing);
  let found = null;
  for (const layer of chart.layers) {
    const hit = layer.pane === id ? sketches.get(drawn.get(layer.id))?.hit(at.x, at.y, reach) : null;
    if (hit && (!found || score(hit) > score(found))) found = hit;
  }
  return found;
}
function hostIn(index) {
  // Never a signal: its own series isn't seen, its marks are its layer's.
  const id = chart.panes[index]?.id, mates = chart.layers.filter((layer) => layer.pane === id && layer.type !== "signal" && drawn.has(layer.id));
  return mates.find((layer) => layer.id === selected && !layer.hidden) ?? mates.find((layer) => !layer.hidden) ?? mates[0] ?? null;
}
/** @returns {[number, number] | null} */
function pointFrom(at, layer, { free = false, pull = false } = {}) {
  const line = timeLine(), series = drawn.get(layer.id);
  if (!line || !series) return null;
  const exact = (at.x - line.at) / line.step, near = pull ? nearestValue(Math.round(exact), at.y, layer.pane) : null;
  const value = near && near.layer === layer.id ? near.value : seriesValue(series, near?.y ?? at.y);
  return value === null || !Number.isFinite(value) ? null : pointOf(timeOfBar(free ? exact : Math.round(exact)), value);
}
function nearestValue(bar, y, pane) {
  if (bar < 0 || bar >= (resolved?.times.length ?? 0)) return null;
  let best = null;
  for (const layer of chart.layers) {
    const made = built.get(layer.id), series = drawn.get(layer.id);
    if (layer.pane !== pane || layer.hidden || placedOn(layer) || !made || !series) continue;
    for (const value of /** @type {number[]} */ (layer.type === "candles" && !made.stacked && made.opens && Number.isFinite(made.shown?.[bar]) ? [made.opens[bar], made.highs?.[bar], made.lows?.[bar], made.shown?.[bar]] : made.low ? [made.low[bar], made.shown?.[bar]] : [made.shown?.[bar]])) {
      const at = Number.isFinite(value) ? seriesY(series, value) : null;
      if (at !== null && Math.abs(at - y) <= PULL && (!best || Math.abs(at - y) < best.off)) best = { layer: layer.id, value, y: at, off: Math.abs(at - y) };
    }
  }
  return best;
}
export {
  backChip, backGlyph, backOf, beltPopover, beltSide, beltTabs, beltTool, beltUp, boxKey, dockChoices,
  drawingAt, drawingById, drawingNames, endName, endsGlyph, endsPopover, hasLook, hostIn, 
  LENS, LENS_ZOOM, lookFor, newDrawing, pickTool,
  placeBelt, plotPoint, pointFrom, putDown, reachOf, reshapeBelt, rovingBelt,
  setDrawSlider, setLook, shareAlong, showBelt, sideAt, 
  timeOfBar, toggleBelt, turnBelt, upright, wordsIn, 
};
