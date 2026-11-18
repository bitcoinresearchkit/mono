// @ts-check
// Drawing on the chart: lines, boxes, text and strokes.
import { Custom, FAINT, indexed, inkOn } from "/modules/bitview-charts/index.js";
import { capitalized } from "/studio/words.js";
import {
  beltStop, boxing, capturing, chart, chosen, commit, css, draft, drawingsHidden, drawVersion, engine, hideBelt, home,
  indexAt, labelTyping, magnifying, measurer, measuring, minedHalvings, MOTION, moving, nextHalvings, phone, placing,
  prefs, reduced, renderBelt, reshaped, resolved, revision, SANS, savePrefs, selected, selectingArea, shownDrawings,
  shownOklch, takeUp, timeAt, tool, typingIn,
} from "/studio/app.js";
import { $, all$, escapeHtml, keyText, MAC } from "/studio/dom.js";
import { clamp, lowerBound, uid } from "/studio/math.js";
import { amount, dateLabel, duration, growth, precise } from "/studio/format.js";
import { glyphSvg, ICONS } from "/studio/icons.js";
import { opacityOf, placedOn } from "/studio/vocabulary.js";
import {
  BACKS, BOXES, ENDS, LENS_SAYS, LINES, lookOfDrawing, MAX_DRAWINGS, POINTED, pointOf, TEXT_SIZE, THROUGH, TOOLS,
} from "/studio/tools.js";
import { FINE } from "/studio/intervals.js";
import { hexToOklch, oklchToHex } from "/studio/color.js";
import {
  alpha, built, colorsAt, cssColor, dashFor, drawn, fadedNow, firstInView, fitStrip, ink, layerFormats,
  paintedLayers, paneBox, seriesPane, seriesScale, seriesValue, seriesY, shownIndex, shownSpan, unitFor, viewName,
} from "/studio/chart.js";
import { nearestFinite, paneAt } from "/studio/formulas.js";
import { layerById, slider } from "/studio/inspector.js";
import { fitted, showMenu } from "/studio/menus.js";
import { refocus } from "/studio/presets-edit.js";
/** @import { End } from "/studio/tools.js" */
/** @import { Drawing, DrawingKind, Layer } from "/studio/vocabulary.js" */
/** @import { Resolved, Result } from "/studio/chart.js" */


/** A trend line's ends, drawn at 16×12: its line between its points, each end a dot, an arrowhead, or on past it. */
const endsGlyph = (/** @type {End | "" | undefined} */ start, /** @type {End | "" | undefined} */ end) => {
  const at = [start === "extend" ? 5 : 3.5, end === "extend" ? 11 : 12.5], from = start === "extend" ? 0.5 : start === "arrow" ? 5 : at[0], to = end === "extend" ? 15.5 : end === "arrow" ? 11 : at[1];
  const tip = (/** @type {number} */ x, /** @type {number} */ way) => `<path d="M${x} 6 ${x - way * 4.5} 3.2v5.6Z" fill="currentColor"/>`;
  return glyphSvg(`<path d="M${from} 6H${to}" stroke="currentColor" stroke-width="1.5"/>${start === "arrow" ? tip(1, -1) : `<circle cx="${at[0]}" cy="6" r="1.75" fill="currentColor"/>`}${end === "arrow" ? tip(15, 1) : `<circle cx="${at[1]}" cy="6" r="1.75" fill="currentColor"/>`}`);
};
/** How near a press must be to take a drawing (a finger's, further), and to a value to be drawn to it, in pixels. */
const REACH = 6, PULL = 8;
const reachOf = (event) => (event.pointerType === "touch" ? 14 : REACH);
const drawingById = (id) => (id ? chart.drawings?.find((drawing) => drawing.id === id) : undefined);
/** Each drawing's name, its kind's (a text, its words), numbered where the chart has more than one alike. */
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
/** A new drawing, in the pen's look (the belt shows it, and sets it). @param {DrawingKind} kind @returns {Drawing} */
function newDrawing(kind, layer, points) {
  return { id: uid(), kind, layer, points, ...lookOfDrawing(kind, prefs.pen), ...(kind === "text" ? { text: "" } : {}) };
}

/** A time on the interval's calendar, in bars from its first (within a bar, the share of it gone). */
function calendarAt(time) {
  const index = shownIndex(), i = indexAt(index, time), from = timeAt(index, i);
  return i + (time - from) / (timeAt(index, i + 1) - from);
}
/**
 * A time's place across the chart, in bars, as its points have theirs: between two, the share of the way; past either
 * end of the data, by the calendar, so a drawing holds in the room past the latest (and before the first) too.
 */
function barOf(time) {
  const times = resolved?.times ?? [], n = times.length;
  if (time >= times[0] && time <= times[n - 1]) {
    const i = lowerBound(times, time);
    return times[i] === time ? i : i - 1 + (time - times[i - 1]) / (times[i] - times[i - 1]);
  }
  const edge = time > times[n - 1] ? n - 1 : 0;
  return edge + calendarAt(time) - calendarAt(times[edge]);
}
/** The time at a place across the chart, in bars: `barOf`, read back. */
function timeOfBar(bar) {
  const times = resolved?.times ?? [], n = times.length;
  if (bar >= 0 && bar <= n - 1) {
    const i = Math.floor(bar), share = bar - i;
    return share ? times[i] + share * (times[i + 1] - times[i]) : times[i];
  }
  // (Before the calendar's first bar, its first's length on back, as calendarAt reads it.)
  const edge = bar > n - 1 ? n - 1 : 0, at = calendarAt(times[edge]) + bar - edge, i = Math.max(0, Math.floor(at)), index = shownIndex(), from = timeAt(index, i);
  return from + (at - i) * (timeAt(index, i + 1) - from);
}
/** The time axis as a line: bar 0's x, and a bar's width (the library's x is a bar's index, scaled and moved). */
function timeLine() {
  const at = engine.time.x(0), next = engine.time.x(1);
  return resolved?.times.length && next > at ? { at, step: next - at } : null;
}
/**
 * A trend's segments' ends in view, in pixels: each one's first point (none before it, or a switch just before) and
 * its last (a switch, or none after). @param {Result} result @param {{ at: number, step: number }} line
 */
function segmentEnds(result, line, series) {
  const shown = result.values, breaks = result.breaks, range = shownSpan();
  if (!shown || !range) return [];
  const on = (/** @type {number} */ i) => i >= 0 && i < shown.length && Number.isFinite(shown[i]), ends = [];
  for (let i = Math.max(0, Math.floor(range.from) - 1), to = Math.min(shown.length - 1, Math.ceil(range.to) + 1); i <= to; i++) {
    if (!on(i) || (on(i - 1) && !breaks?.[i - 1] && !breaks?.[i] && on(i + 1))) continue;
    const y = seriesY(series, shown[i]);
    if (y !== null) ends.push({ x: line.at + i * line.step, y, i });
  }
  return ends;
}
/** Those ends, dotted in the line's color (faint with it), a little wider than it. */
function paintEnds(context, /** @type {{ x: number, y: number, i: number }[]} */ ends, /** @type {{ color: string, radius: number, at?: ((i: number) => string) | null }} */ look, faint = false) {
  context.globalAlpha = faint ? FAINT : 1;
  for (const { x, y, i } of ends) {
    context.fillStyle = look.at?.(i) ?? look.color;
    context.beginPath();
    context.arc(x, y, look.radius, 0, Math.PI * 2);
    context.fill();
  }
  context.globalAlpha = 1;
}
/** A drawing's points on its pane, in pixels (NaN where there's none: a value its scale can't show). */
function onScreen(drawing, series, line) {
  const bars = pointBars(drawing.points);
  return drawing.points.map(([, value], i) => ({ x: line.at + bars[i] * line.step, y: seriesY(series, value) ?? NaN }));
}
/**
 * Each point's place across, in bars, kept until the chart's times change (a pan, a zoom need none of it again). A
 * drawing's points are never changed in place, only added to (a stroke being drawn): those added are placed alone.
 */
const placedBars = new WeakMap();
function pointBars(points) {
  let kept = placedBars.get(points);
  const times = resolved?.times, index = shownIndex();
  if (kept?.times !== times || kept.index !== index) placedBars.set(points, (kept = { times, index, bars: [] }));
  for (let i = kept.bars.length; i < points.length; i++) kept.bars.push(barOf(points[i][0]));
  return kept.bars;
}
const finite = (point) => Number.isFinite(point?.x) && Number.isFinite(point?.y);

/**
 * A line through points as drawn (a trend line, a path): to each in turn (those shown), its first and its last on past
 * them (each end its own) to the plot's edges, along its first stretch and its last.
 */
function lineThrough(drawing, points, size) {
  const shown = points.filter(finite), n = shown.length;
  if (n < 2) return shown;
  const first = drawing.start === "extend" ? reach(shown[0], shown[1], size, true, false)[0] : shown[0], last = drawing.end === "extend" ? reach(shown[n - 2], shown[n - 1], size, false, true)[1] : shown[n - 1];
  return [first, ...shown.slice(1, -1), last];
}
/** A line through two points, out to a box's edges: on past the second (`forth`), the first (`back`). */
function reach(a, b, size, back, forth) {
  const dx = b.x - a.x, dy = b.y - a.y;
  if (!dx && !dy) return [a, b];
  const exit = (from, d, edge) => (d > 0 ? (edge - from) / d : d < 0 ? -from / d : Infinity);
  const entry = (from, d, edge) => (d > 0 ? -from / d : d < 0 ? (edge - from) / d : -Infinity);
  const far = forth ? Math.max(1, Math.min(exit(a.x, dx, size.width), exit(a.y, dy, size.height))) : 1;
  const near = back ? Math.min(0, Math.max(entry(a.x, dx, size.width), entry(a.y, dy, size.height))) : 0;
  return [{ x: a.x + near * dx, y: a.y + near * dy }, { x: a.x + far * dx, y: a.y + far * dy }];
}
/** A stroke through its points, as kept (steadied as drawn, simplified within a pixel): as it's hit. */
function strokeThrough(context, points) {
  if (!points.length) return;
  context.moveTo(points[0].x, points[0].y);
  if (points.length === 1) return void context.lineTo(points[0].x + 0.01, points[0].y);
  for (let i = 1; i < points.length; i++) context.lineTo(points[i].x, points[i].y);
}
const ordinal = (/** @type {number} */ n) => `${n}${n % 100 >= 11 && n % 100 <= 13 ? "th" : ["th", "st", "nd", "rd"][n % 10] ?? "th"}`;
/** The halvings across a pane, faint and dashed (named by paintHalvingNames; placed among the series by markOrder). */
function paintHalvings(context, size, /** @type {{ x: number, n: number, next: boolean }[]} */ marks) {
  const soft = css("--soft");
  context.save();
  context.lineWidth = 1;
  for (const { x, next } of marks) {
    if (x < -1 || x > size.width + 1) continue;
    const at = Math.round(x) + 0.5;
    context.strokeStyle = alpha(soft, next ? 0.3 : 0.45);
    context.setLineDash(next ? [1, 4] : [3, 4]);
    context.beginPath();
    context.moveTo(at, 0);
    context.lineTo(at, size.height);
    context.stroke();
  }
  context.restore();
}
/** Their names, up beside them, over the series (a halo in the card's color, as a drawing's words: the data under them stays out of them). */
function paintHalvingNames(context, size, /** @type {{ x: number, n: number, next: boolean }[]} */ marks) {
  const soft = css("--soft");
  context.save();
  context.font = `510 10px ${SANS}`;
  context.textBaseline = "top";
  context.lineJoin = "round";
  context.lineWidth = 3;
  context.strokeStyle = css("--card");
  for (const { x, n, next } of marks) {
    // (Drawn while any of it's in view: it runs from its mark to about 16 pixels right of it, so a pan slides it out.)
    if (x < -16 || x > size.width + 1) continue;
    const name = `${ordinal(n)} halving${next ? " ≈" : ""}`;
    context.save();
    context.translate(Math.round(x) + 4.5, size.height - 8);
    context.rotate(-Math.PI / 2);
    context.fillStyle = alpha(soft, next ? 0.6 : 0.85);
    context.strokeText(name, 0, 0);
    context.fillText(name, 0, 0);
    context.restore();
  }
  context.restore();
}
/** A drawing on its pane (a line's label in a gap of it). */
function paintDrawing(context, drawing, points, size, card) {
  const color = ink(drawing.color), [a, b] = points, words = { color, back: drawing.back ?? "", on: wordsColor(drawing, card), card };
  const label = drawing.label && drawing.id !== labelTyping ? labelPlace(drawing, points, size) : null;
  context.save();
  context.globalAlpha = drawing.kind === "ruler" ? 1 : drawing.opacity ?? 1;
  context.strokeStyle = context.fillStyle = color;
  context.lineWidth = drawing.width;
  context.lineCap = drawing.dash ? "butt" : "round";
  context.lineJoin = "round";
  context.setLineDash(dashFor(drawing.width, drawing.dash));
  if (label) {
    context.save();
    clipOut(context, label, size);
  }
  context.beginPath();
  const through = THROUGH.has(drawing.kind) ? lineThrough(drawing, points, size) : [], n = through.length;
  // (Its dashes from its own first point, whatever's drawn of it: run on to the plot's edge, or across, they move with
  // the chart as it's panned, as a picture moved along has them.)
  if (n > 1 && Number.isFinite(a.x)) context.lineDashOffset = -Math.hypot(a.x - through[0].x, a.y - through[0].y);
  if (n > 1) {
    // (An arrow's line stops at its head's back: its head makes its end.)
    context.moveTo(...(drawing.start === "arrow" ? arrowBack(through[1], through[0], drawing.width) : [through[0].x, through[0].y]));
    for (const point of through.slice(1, -1)) context.lineTo(point.x, point.y);
    context.lineTo(...(drawing.end === "arrow" ? arrowBack(through[n - 2], through[n - 1], drawing.width) : [through[n - 1].x, through[n - 1].y]));
  } else if (drawing.kind === "hline" && Number.isFinite(a.y)) {
    const from = Number.isFinite(a.x) ? a.x : 0;
    context.moveTo(from, a.y);
    context.lineTo(size.width, a.y);
    context.moveTo(from, a.y);
    context.lineTo(0, a.y);
  } else if (drawing.kind === "vline" && Number.isFinite(a.x)) {
    context.moveTo(a.x, 0);
    context.lineTo(a.x, size.height);
  } else if (drawing.kind === "rect" && finite(a) && finite(b)) {
    context.rect(Math.min(a.x, b.x), Math.min(a.y, b.y), Math.abs(b.x - a.x), Math.abs(b.y - a.y));
    context.fillStyle = alpha(color, 0.12);
    context.fill();
  } else if (drawing.kind === "ellipse" && finite(a) && finite(b)) {
    context.ellipse((a.x + b.x) / 2, (a.y + b.y) / 2, Math.abs(b.x - a.x) / 2, Math.abs(b.y - a.y) / 2, 0, 0, Math.PI * 2);
    context.fillStyle = alpha(color, 0.12);
    context.fill();
  } else if (drawing.kind === "ruler" && finite(a) && finite(b)) paintRuler(context, drawing, a, b, size);
  else if (drawing.kind === "brush") strokeThrough(context, points.filter(finite));
  else if (drawing.kind === "text" && finite(a)) paintLabel(context, textPlace(drawing, a), words);
  if (drawing.kind !== "text" && drawing.kind !== "ruler") context.stroke();
  if (n > 1) {
    if (drawing.end === "arrow") paintArrow(context, through[n - 2], through[n - 1], drawing.width);
    if (drawing.start === "arrow") paintArrow(context, through[1], through[0], drawing.width);
  }
  if (label) {
    context.restore();
    paintLabel(context, label, words);
  }
  context.restore();
}
/** Painting kept off a tag's place (a line's label: the line breaks around it). */
function clipOut(context, place, size) {
  const cos = Math.cos(place.angle), sin = Math.sin(place.angle), w = place.width / 2, h = place.height / 2;
  context.beginPath();
  context.rect(-size.width, -size.height, size.width * 3, size.height * 3);
  [[-w, -h], [w, -h], [w, h], [-w, h]].forEach(([x, y], i) => context[i ? "lineTo" : "moveTo"](place.x + x * cos - y * sin, place.y + x * sin + y * cos));
  context.closePath();
  context.clip("evenodd");
}
/** An arrow's head, from a line's first point to its second: as long and as wide as the line is thick asks (no longer than the line). */
const arrowSize = (a, b, /** @type {number} */ width) => Math.min(7 + width * 2.5, Math.hypot(b.x - a.x, b.y - a.y));
function arrowBack(a, b, /** @type {number} */ width) {
  const angle = Math.atan2(b.y - a.y, b.x - a.x), back = arrowSize(a, b, width) * 0.8;
  return [b.x - back * Math.cos(angle), b.y - back * Math.sin(angle)];
}
function paintArrow(context, a, b, /** @type {number} */ width) {
  const angle = Math.atan2(b.y - a.y, b.x - a.x), length = arrowSize(a, b, width), half = length * 0.45;
  const base = { x: b.x - length * Math.cos(angle), y: b.y - length * Math.sin(angle) };
  context.setLineDash([]);
  context.beginPath();
  context.moveTo(b.x, b.y);
  context.lineTo(base.x - half * Math.sin(angle), base.y + half * Math.cos(angle));
  context.lineTo(base.x + half * Math.sin(angle), base.y - half * Math.cos(angle));
  context.closePath();
  context.fillStyle = context.strokeStyle;
  context.fill();
}
/** Tags' shapes as measured, by size and words (measured again once fonts have loaded). */
const tagShapes = new Map();
/** A tag's lines and its box: its words at a size (half of it to each side, a line a third taller). */
function tagShape(/** @type {string} */ text, /** @type {number} */ size) {
  const key = `${size}|${text}`, kept = tagShapes.get(key);
  if (kept) return kept;
  if (tagShapes.size > 500) tagShapes.clear();
  tagShapes.set(key, measureTag(text, size));
  return tagShapes.get(key);
}
function measureTag(/** @type {string} */ text, /** @type {number} */ size) {
  const lines = (text || " ").split("\n");
  measurer.font = `590 ${size}px ${SANS}`;
  return { lines, size, width: Math.max(...lines.map((line) => measurer.measureText(line).width)) + size, height: lines.length * Math.round(size * 1.33) + Math.round(size / 6) };
}
/** Where a text sits: a tag, level, its middle at its point. */
const textPlace = (/** @type {Drawing} */ drawing, point, text = drawing.text ?? "") => ({ x: point.x, y: point.y, angle: 0, ...tagShape(text, drawing.size ?? TEXT_SIZE) });
/**
 * Where a line's label sits (its middle, the angle it reads at, its box): a share of the way along the line (a trend
 * line's, from its first point to its second, upright whichever way it runs; across, across the plot; down, rising
 * down the plot), kept whole in the plot.
 */
function labelPlace(/** @type {Drawing} */ drawing, points, size, text = drawing.label ?? "") {
  const [a, b] = points, at = drawing.at ?? (drawing.kind === "hline" || drawing.kind === "vline" ? 1 : 0.5);
  const shape = tagShape(text, drawing.size ?? TEXT_SIZE), { width } = shape;
  if (drawing.kind === "trend") {
    if (!finite(a) || !finite(b)) return null;
    let angle = Math.atan2(b.y - a.y, b.x - a.x);
    if (angle > Math.PI / 2) angle -= Math.PI;
    else if (angle < -Math.PI / 2) angle += Math.PI;
    return { ...shape, x: a.x + (b.x - a.x) * at, y: a.y + (b.y - a.y) * at, angle };
  }
  if (drawing.kind === "hline") return Number.isFinite(a.y) ? { ...shape, x: clamp(at * size.width, width / 2 + 6, size.width - width / 2 - 6), y: a.y, angle: 0 } : null;
  return Number.isFinite(a.x) ? { ...shape, x: a.x, y: clamp(at * size.height, width / 2 + 6, size.height - width / 2 - 6), angle: -Math.PI / 2 } : null;
}
/**
 * A tag: its words line under line, on what its back asks (nothing: a halo of the card around the letters, so they
 * read over anything; a soft fill over the card; its color, the words in contrast) in a border if it asks.
 */
function paintLabel(context, place, /** @type {{ color: string, back: string, on: string, card: string }} */ words) {
  const { color, back, on, card } = words, filled = back === "soft" || back === "soft-outline" || back === "solid";
  context.save();
  context.translate(place.x, place.y);
  context.rotate(place.angle);
  context.setLineDash([]);
  context.beginPath();
  context.roundRect(-place.width / 2 + 0.5, -place.height / 2 + 0.5, place.width - 1, place.height - 1, 4);
  if (filled) {
    context.fillStyle = back === "solid" ? color : card;
    context.fill();
  }
  if (back === "soft" || back === "soft-outline") {
    context.fillStyle = alpha(color, 0.16);
    context.fill();
  }
  if (back === "outline" || back === "soft-outline") {
    context.lineWidth = 1;
    context.strokeStyle = color;
    context.stroke();
  }
  context.font = `590 ${place.size}px ${SANS}`;
  context.textAlign = "center";
  context.textBaseline = "middle";
  context.lineJoin = "round";
  context.lineWidth = 3;
  context.strokeStyle = card;
  context.fillStyle = on;
  const line = Math.round(place.size * 1.33);
  place.lines.forEach((words, i) => {
    const y = -place.height / 2 + place.size / 12 + (i + 0.5) * line + 0.5;
    if (!filled) context.strokeText(words, 0, y);
    context.fillText(words, 0, y);
  });
  context.restore();
}
/** Whether a point of the pane is on a line's label (in the label's own frame: it may lie at an angle). */
function onLabel(/** @type {Drawing} */ drawing, points, size, x, y) {
  const place = drawing.label ? labelPlace(drawing, points, size) : null;
  if (!place) return false;
  const dx = x - place.x, dy = y - place.y, cos = Math.cos(-place.angle), sin = Math.sin(-place.angle);
  return Math.abs(dx * cos - dy * sin) <= place.width / 2 + 2 && Math.abs(dx * sin + dy * cos) <= place.height / 2 + 2;
}
/** The share of the way along a line a point of the pane is at (where a label slid there goes). */
function shareAlong(/** @type {Drawing} */ drawing, points, size, x, y) {
  const [a, b] = points;
  if (drawing.kind === "hline") return clamp(x / size.width, 0, 1);
  if (drawing.kind === "vline") return clamp(y / size.height, 0, 1);
  const dx = b.x - a.x, dy = b.y - a.y, length = dx * dx + dy * dy;
  return length ? clamp(((x - a.x) * dx + (y - a.y) * dy) / length, 0, 1) : 0.5;
}
/** What a ruler measures: its layer's change from its first date to its last (whichever way it was drawn): in %, in its unit, over how long. */
function rulerReading(/** @type {Drawing} */ drawing) {
  const layer = layerById(drawing.layer), result = layer && resolved?.results.get(layer.id), values = result?.values, n = resolved?.times.length ?? 0;
  if (!layer || !values || !n) return null;
  const [a, b] = drawing.points.map(([time]) => nearestFinite(values, clamp(Math.round(barOf(time)), 0, n - 1)));
  // (Its dates past the data read its nearest; both on one: nothing measured.)
  if (a < 0 || b < 0 || a === b) return null;
  const unit = unitFor(layer, result), [from, to] = a <= b ? [a, b] : [b, a], delta = values[to] - values[from];
  const relative = unit === "percent" ? null : growth(values[from], values[to]), sign = (/** @type {boolean} */ up) => (up ? "+" : "−");
  return {
    rising: relative ? relative.up : delta >= 0,
    name: viewName(chart, layer),
    figures: [...(relative ? [`${relative.text.endsWith("×") ? "" : sign(relative.up)}${relative.text}`] : []), `${sign(delta >= 0)}${amount(delta, unit)}`],
    span: duration(/** @type {Resolved} */ (resolved).times[to] - /** @type {Resolved} */ (resolved).times[from]),
  };
}
/** A ruler: its box tinted (green up, red down), its reading on a label over it (under it, with no room above), in the plot. */
function paintRuler(context, /** @type {Drawing} */ drawing, a, b, size) {
  const reading = rulerReading(drawing), color = css(reading?.rising === false ? "--danger" : "--up");
  const left = Math.min(a.x, b.x), top = Math.min(a.y, b.y), width = Math.abs(b.x - a.x), height = Math.abs(b.y - a.y);
  context.setLineDash([]);
  context.fillStyle = alpha(color, 0.09);
  context.fillRect(left, top, width, height);
  context.strokeStyle = alpha(color, 0.7);
  context.lineWidth = 1;
  context.strokeRect(left + 0.5, top + 0.5, Math.max(0, width - 1), Math.max(0, height - 1));
  if (!reading) return;
  const parts = [[reading.name, css("--soft"), 510], ...reading.figures.map((figure) => [figure, css("--ink"), 590]), [reading.span, css("--muted"), 510]];
  const widths = parts.map(([text, , weight]) => {
    context.font = `${weight} 12px ${SANS}`;
    return context.measureText(String(text)).width;
  });
  const fit = () => widths.reduce((sum, each) => sum + each, 0) + 10 * (parts.length - 1) + 18, label = { width: fit(), height: 26 };
  // (Too wide for its pane: its layer's name cut short, or left out when little of it would be left.)
  if (label.width > size.width - 8) {
    const room = widths[0] - (label.width - (size.width - 8));
    context.font = `510 12px ${SANS}`;
    if (room >= 32) widths[0] = context.measureText(String((parts[0][0] = fitted(context, String(parts[0][0]), room)))).width;
    else {
      parts.shift();
      widths.shift();
    }
    label.width = fit();
  }
  const x = clamp(left + width / 2 - label.width / 2, 4, size.width - label.width - 4), y = clamp(top - label.height - 8 >= 4 ? top - label.height - 8 : top + height + 8, 4, size.height - label.height - 4);
  context.beginPath();
  context.roundRect(x, y, label.width, label.height, 6);
  context.fillStyle = css("--raise");
  context.fill();
  context.strokeStyle = alpha(color, 0.5);
  context.stroke();
  context.textBaseline = "middle";
  let at = x + 9;
  parts.forEach(([text, ink, weight], i) => {
    context.font = `${weight} 12px ${SANS}`;
    context.fillStyle = String(ink);
    context.fillText(String(text), at, y + label.height / 2);
    at += widths[i] + 10;
  });
}
/**
 * Where a drawing is held: its ends (a line's, a path's: selected, its box's corners after them, as a stroke's), a
 * box's corners; a line across, at its point (kept in view).
 */
function handlesOf(drawing, points, size) {
  const [a, b] = points;
  // (A corner on one of its points (a line's ends, a path's at a corner of its box): that point's, none of its own,
  // its place kept.)
  if (THROUGH.has(drawing.kind)) return lit(drawing) ? [...points, ...boxCorners(drawing, points).map((corner) => (points.some((point) => finite(point) && Math.hypot(point.x - corner.x, point.y - corner.y) < 12) ? { x: NaN, y: NaN } : corner))] : points;
  // (An ellipse's corners are off it, on its box: there once it's selected, as a stroke's.)
  if (BOXES.has(drawing.kind) && (drawing.kind !== "ellipse" || lit(drawing))) return [a, { x: b.x, y: a.y }, b, { x: a.x, y: b.y }];
  if (drawing.kind === "hline") return [{ x: clamp(Number.isFinite(a.x) ? a.x : size.width / 2, 10, size.width - 10), y: a.y }];
  if (drawing.kind === "vline") return [{ x: a.x, y: clamp(Number.isFinite(a.y) ? a.y : size.height / 2, 10, size.height - 10) }];
  if (drawing.kind === "brush" && lit(drawing)) return boxCorners(drawing, points);
  return [];
}
/**
 * The corners of the box around a stroke, a line or a path (see boundsOf), from its top left round; none shown, or a
 * dot (no width, no height: nothing to scale, it's moved), none.
 */
function boxCorners(drawing, points) {
  const box = boundsOf(drawing, points);
  return Number.isFinite(box.left) && (box.right > box.left || box.bottom > box.top) ? [{ x: box.left, y: box.top }, { x: box.right, y: box.top }, { x: box.right, y: box.bottom }, { x: box.left, y: box.bottom }] : [];
}
/**
 * The side of a box (a rectangle's, an ellipse's, a ruler's, a selected stroke's, line's or path's: its corners' box)
 * a point is on, between its corners: 4 its top, 5 its right, 6 its foot, 7 its left; -1, none.
 */
function sideOf(/** @type {Drawing} */ drawing, corners, x, y, reach) {
  if (corners.length !== 4 || !(BOXES.has(drawing.kind) || drawing.kind === "brush" || THROUGH.has(drawing.kind))) return -1;
  const left = Math.min(...corners.map((p) => p.x)), right = Math.max(...corners.map((p) => p.x)), top = Math.min(...corners.map((p) => p.y)), bottom = Math.max(...corners.map((p) => p.y));
  const across = x > left + reach && x < right - reach, down = y > top + reach && y < bottom - reach;
  return across && Math.abs(y - top) <= reach ? 4 : across && Math.abs(y - bottom) <= reach ? 6 : down && Math.abs(x - right) <= reach ? 5 : down && Math.abs(x - left) <= reach ? 7 : -1;
}
/**
 * The box around a text (clear of its words), a stroke, an ellipse, a line or a path (on its outermost points: its dots on its
 * edges, a corner on one where it's there, one dot, see handlesOf): its outline, selected.
 */
function boundsOf(drawing, points) {
  if (drawing.kind === "text") {
    const place = textPlace(drawing, points[0]);
    return { left: place.x - place.width / 2 - 2, top: place.y - place.height / 2 - 2, right: place.x + place.width / 2 + 2, bottom: place.y + place.height / 2 + 2 };
  }
  const shown = points.filter(finite);
  return { left: Math.min(...shown.map((p) => p.x)), top: Math.min(...shown.map((p) => p.y)), right: Math.max(...shown.map((p) => p.x)), bottom: Math.max(...shown.map((p) => p.y)) };
}
/** A drawing's color as drawn: a ruler's, up or down; any other's, its own. */
const colorOfDrawing = (/** @type {Drawing} */ drawing) => (drawing.kind === "ruler" ? css(rulerReading(drawing)?.rising === false ? "--danger" : "--up") : ink(drawing.color));
/** A drawing selected, being drawn, or pointed at: its points to take it by, or a box around a text or a stroke. */
function paintHold(context, drawing, points, size, card, lit) {
  const color = colorOfDrawing(drawing);
  context.save();
  // (A box where its handles are off it (a text's, a stroke's, an ellipse's): faint, pointed at, as what a press
  // takes; a line's or a path's, selected only: pointed at, its points say where it's held.)
  if (drawing.kind === "text" || drawing.kind === "brush" || drawing.kind === "ellipse" || (THROUGH.has(drawing.kind) && lit)) {
    const box = boundsOf(drawing, points);
    context.strokeStyle = alpha(color, lit ? 0.9 : 0.45);
    context.lineWidth = 1;
    context.setLineDash([3, 3]);
    if (Number.isFinite(box.left)) context.strokeRect(box.left, box.top, box.right - box.left, box.bottom - box.top);
    context.setLineDash([]);
  }
  for (const [i, point] of handlesOf(drawing, points, size).entries()) {
    // (A level's dot under its label: the label shows.)
    if (!finite(point) || ((drawing.kind === "hline" || drawing.kind === "vline") && onLabel(drawing, points, size, point.x, point.y))) continue;
    context.beginPath();
    context.arc(point.x, point.y, hovering.id === drawing.id && hovering.handle === i ? 5 : 4, 0, Math.PI * 2);
    context.fillStyle = card;
    context.fill();
    context.lineWidth = 1.5;
    context.strokeStyle = color;
    context.stroke();
  }
  context.restore();
}
/** How far a point is from a segment. */
function toSegment(x, y, a, b) {
  const dx = b.x - a.x, dy = b.y - a.y, length = dx * dx + dy * dy;
  const t = length ? clamp(((x - a.x) * dx + (y - a.y) * dy) / length, 0, 1) : 0;
  return Math.hypot(x - a.x - t * dx, y - a.y - t * dy);
}
/** Whether a point of the pane is on a drawing: within reach of its line, inside its box. */
function touches(drawing, points, x, y, size, reach) {
  const [a, b] = points, near = reach + drawing.width / 2;
  if (THROUGH.has(drawing.kind)) return lineThrough(drawing, points, size).some((point, i, through) => i > 0 && toSegment(x, y, through[i - 1], point) <= near);
  if (drawing.kind === "hline") return Math.abs(y - a.y) <= near;
  if (drawing.kind === "vline") return Math.abs(x - a.x) <= near;
  if (drawing.kind === "ellipse") {
    const rx = Math.abs(b.x - a.x) / 2 + near, ry = Math.abs(b.y - a.y) / 2 + near;
    return Math.hypot((x - (a.x + b.x) / 2) / rx, (y - (a.y + b.y) / 2) / ry) <= 1;
  }
  if (BOXES.has(drawing.kind)) return x >= Math.min(a.x, b.x) - near && x <= Math.max(a.x, b.x) + near && y >= Math.min(a.y, b.y) - near && y <= Math.max(a.y, b.y) + near;
  if (drawing.kind === "brush") {
    const shown = points.filter(finite);
    return shown.some((point, i) => toSegment(x, y, point, shown[i + 1] ?? point) <= near);
  }
  if (!finite(a)) return false;
  const box = boundsOf(drawing, points);
  return x >= box.left && x <= box.right && y >= box.top && y <= box.bottom;
}
/** Words on a color as it's named (shown as the theme tones it): white or black, by the chart library's one rule (see its color.js). */
const contrastOn = (/** @type {string} */ color) => inkOn(oklchToHex(...shownOklch(color)));
/** A color's luminance as WCAG reads it (sRGB hex), and two colors' contrast. */
const luminance = (/** @type {string} */ hex) => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255).map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4)).reduce((sum, v, i) => sum + v * [0.2126, 0.7152, 0.0722][i], 0);
const contrastOf = (/** @type {string} */ a, /** @type {string} */ b) => ((x, y) => (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05))(luminance(a), luminance(b));
/** A color laid over another at a share, as a canvas lays it (sRGB hex). */
const laidOver = (/** @type {string} */ base, /** @type {string} */ top, /** @type {number} */ share) => `#${[1, 3, 5].map((i) => Math.round(parseInt(base.slice(i, i + 2), 16) * (1 - share) + parseInt(top.slice(i, i + 2), 16) * share).toString(16).padStart(2, "0")).join("")}`;
const wordsTones = new Map();
/**
 * Words in a color, as they read on their ground: its hue and chroma, its lightness moved (darker on a light ground,
 * lighter on a dark one) as little as reaching 4.5:1 takes; already there, the color itself (sRGB hex, both).
 */
function wordsOn(/** @type {string} */ color, /** @type {string} */ ground) {
  const key = `${color}|${ground}`, wordsKnown = wordsTones.get(key);
  if (wordsKnown) return wordsKnown;
  let tone = color;
  if (contrastOf(color, ground) < 4.5) {
    const [l, c, h] = hexToOklch(color), down = luminance(ground) > 0.18;
    // (The lightness nearest its own that reads, halved toward.)
    let lo = down ? 0 : l, hi = down ? l : 1;
    for (let i = 0; i < 14; i++) {
      const mid = (lo + hi) / 2, reads = contrastOf(oklchToHex(mid, c, h), ground) >= 4.5;
      if (down === reads) lo = mid;
      else hi = mid;
    }
    tone = oklchToHex(down ? lo : hi, c, h);
  }
  if (wordsTones.size > 200) wordsTones.clear();
  wordsTones.set(key, tone);
  return tone;
}
/** A drawing's words' color: on its own color (a solid back), black or white; else its color, as reads on what's under them (the card; a soft back, its tint). */
function wordsColor(/** @type {Drawing} */ drawing, card = css("--card")) {
  if (drawing.back === "solid") return contrastOn(drawing.color);
  const color = ink(drawing.color);
  return wordsOn(color, drawing.back === "soft" || drawing.back === "soft-outline" ? laidOver(card, color, 0.16) : card);
}
/** A tick on a color, in contrast, as a background image. */
const tickOn = (/** @type {string} */ color) => `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Cpath d='m4 8.2 2.6 2.6L12 5.4' fill='none' stroke='${encodeURIComponent(contrastOn(color))}' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E")`;
/** A label on an axis, at `at` along it, in a drawing's color as drawn (see colorOfDrawing: not toned again), its words the chart library's. @param {number} at @param {string} text @param {string} color */
function axisTag(at, text, color) {
  return { at, text, color };
}
/** @typedef {{ at: number, text: string, color: string }} AxisTag */

const NONE = /** @type {AxisTag[]} */ ([]);
/** A layer whose series covers what's behind it: a cloud, an area, a baseline's fill, bars, candles. @param {Layer} layer */
const fills = (layer) => layer.type === "cloud" || ["area", "baseline", "histogram", "candles"].includes(layer.type);
/**
 * Each pane's halving marks' hosts (its frontmost fill, else its first layer drawn and shown; and its last), and the
 * lowest pane with a layer shown: found once for every series' sketch a draw brings up to the view (each found them
 * itself, the chart's layers walked for each), anew after a change of state, a draw, or by the next task.
 */
let hostsFound = /** @type {{ at: string, byPane: Map<string | undefined, { fill: Layer | undefined, host: Layer | null, back: Layer | null }>, lowest: string | undefined } | null} */ (null);
function halvingHosts() {
  const at = `${revision}:${drawVersion}:${drawn.size}`;
  if (hostsFound?.at === at) return hostsFound;
  const byPane = new Map();
  for (const pane of chart.panes) {
    const shown = chart.layers.filter((layer) => drawn.has(layer.id) && !layer.hidden && layer.pane === pane.id), frontFirst = pane.reverse ? shown.reverse() : shown, fill = frontFirst.find(fills);
    byPane.set(pane.id, { fill, host: fill ?? frontFirst[0] ?? null, back: frontFirst.at(-1) ?? null });
  }
  hostsFound = { at, byPane, lowest: chart.panes.filter((pane) => chart.layers.some((layer) => layer.pane === pane.id && !layer.hidden)).at(-1)?.id };
  queueMicrotask(() => (hostsFound = null));
  return hostsFound;
}
/**
 * A layer's drawings, on its series: over its pane (over every series, and the crosshair), a line across's value or
 * date on its axis (a selected one's, each point's), and found under the pointer where they were last drawn. With
 * them its pane's halving marks (if its layer draws them) and a trend's ends: the library's layers on its series'
 * pane (see its constructor).
 */
class Sketch {
  /** @param {string} layer @param {any} series */
  constructor(layer, series) {
    this.layer = layer;
    this.series = series;
    /** The pane and timeline whose drawing geometry is kept (see fresh). @type {{ pane: import("/modules/bitview-charts/pane.js").Pane | undefined, changes: number | undefined, from: number, spacing: number, width: number, times: ArrayLike<number> } | null} */
    this.view = null;
    this.update = () => ((this.view = null), engine.invalidate(true));
    /** @type {{ drawing: Drawing, points: { x: number, y: number }[], box: { left: number, top: number, right: number, bottom: number } | null }[]} */
    this.shown = [];
    this.size = { width: 0, height: 0 };
    /** The halvings in this pane, in pixels across, each its number (`next`: the estimated one), if this layer draws them for its pane. @type {{ x: number, n: number, next: boolean }[]} */
    this.marks = [];
    /** The same, if this layer names them (the lowest pane's backmost layer: its top views drawn first, every drawing over them). @type {{ x: number, n: number, next: boolean }[]} */
    this.names = [];
    /** Where the halvings draw: just over this layer's series (it's the pane's frontmost fill), else under every one. @type {"bottom" | "normal"} */
    this.markOrder = "bottom";
    /** A trend's segments' ends, dotted (where one starts after a switch, or ends before one), as the series draws. @type {{ x: number, y: number, i: number }[]} */
    this.ends = [];
    this.endsLook = /** @type {{ color: string, radius: number, at?: ((i: number) => string) | null }} */ ({ color: "", radius: 1.5 });
    /** Its axes' tags, found once a view (read again as the axes are drawn). @type {AxisTag[] | null} */
    this.priceTags = null;
    /** @type {AxisTag[] | null} */
    this.timeTags = null;
    // Its layers on its series' pane, on its side: the halvings under every series (its pane with no fill to draw
    // them over); just over its series, a trend's ends and the halvings over its fill; over everything (the crosshair
    // too), the halvings' names and its drawings, their values and dates on the axes. Each pane's view the plot's
    // size (see sizeOf). Brought up to the view as first read (see fresh).
    const size = { width: 0, height: 0 }, sizeOf = (/** @type {{ time: { width: number }, scale: { height: number } }} */ at) => ((size.width = at.time.width), (size.height = at.scale.height), size);
    const under = new Custom({ draw: (g, at) => void (this.fresh().markOrder === "bottom" && this.marks.length && paintHalvings(g, sizeOf(at), this.marks)) }, { behind: true });
    const beside = new Custom({
      draw: (g, at) => {
        if (this.fresh().ends.length) paintEnds(g, this.ends, this.endsLook, fadedNow.has(this.layer));
        if (this.markOrder === "normal" && this.marks.length) paintHalvings(g, sizeOf(at), this.marks);
      },
    });
    const side = series.side === "left" ? 1 : 0;
    const over = new Custom(
      {
        draw: (g, at) => {
          if (this.fresh().names.length) paintHalvingNames(g, sizeOf(at), this.names);
          if (this.shown.length) this.draw(g, sizeOf(at));
        },
        axisTags: (on, add) => {
          if (on === side) for (const tag of this.axisTags()) if (Number.isFinite(tag.at)) add(tag.at, tag.text, tag.color);
        },
        timeTags: (add) => {
          for (const tag of this.dateTags()) if (Number.isFinite(tag.at)) add(tag.at, tag.text, tag.color);
        },
      },
      { over: true },
    );
    for (const custom of [under, beside, over]) ownLayer(series, custom);
  }
  /** What it shows as things are now: found once a view, as its layers, its tags or a hit read it. */
  fresh() {
    const time = engine.time, view = this.view, pane = view?.pane ?? engine.panes[seriesPane(this.series)];
    if (view && view.changes === pane?.changes && view.from === time.from && view.spacing === time.spacing && view.width === time.width && view.times === time.times) return this;
    this.view = { pane, changes: pane?.changes, from: time.from, spacing: time.spacing, width: time.width, times: time.times };
    this.priceTags = this.timeTags = null;
    const line = timeLine(), series = this.series;
    // (Once a pane, the lowest pane's with their words: over its frontmost fill, a cloud's, an area's, bars, candles,
    // so none hides them; under the lines in front of it. No fill: under every series.)
    const own = layerById(this.layer), hosts = chart.halvings && line && series ? halvingHosts() : null, at = hosts?.byPane.get(own?.pane);
    const fill = at?.fill, host = at?.host ?? null, back = at?.back ?? null;
    this.markOrder = fill ? "normal" : "bottom";
    const lowest = Boolean(host) && hosts?.lowest === host?.pane;
    const marks = line && (host?.id === this.layer || (lowest && back?.id === this.layer)) ? [...minedHalvings().map((time, i) => ({ x: line.at + barOf(time) * line.step, n: i + 1, next: false })), ...nextHalvings().map(({ n, time }) => ({ x: line.at + barOf(time) * line.step, n, next: true }))] : [];
    this.marks = host?.id === this.layer ? marks : [];
    this.names = lowest && back?.id === this.layer ? marks : [];
    const trend = own?.type === "trend" && !own.hidden && line && series ? resolved?.results.get(this.layer) : null;
    this.ends = trend ? segmentEnds(trend, /** @type {{ at: number, step: number }} */ (line), series) : [];
    if (this.ends.length && own) {
      // (Each in its segment's color, as painted: a flip's last point is its old side's, as drawn before it.)
      const painted = paintedLayers.get(own.id), source = painted && resolved ? colorsAt(chart, own, resolved) : null, op = opacityOf(own), dated = source && ((/** @type {number} */ i) => ink(source(i) ?? own.color));
      this.endsLook = { color: series?.engine.color, radius: own.width / 2 + 1, at: dated ? (/** @type {number} */ i) => (op < 1 ? alpha(dated(i), op) : dated(i)) : null };
    }
    this.shown = line && series ? drawingsShown().filter((drawing) => drawing.layer === this.layer && drawing.id !== typingIn).map((drawing) => {
      const points = onScreen(drawing, series, line);
      // A stroke's box, so the pointer far from it is turned away before its every segment.
      // (A stroke's box, a path's (but one run on to the plot's edges): only a pointer near it is looked at closely.)
      return { drawing, points, box: drawing.kind === "brush" || (drawing.kind === "path" && drawing.start !== "extend" && drawing.end !== "extend") ? boundsOf(drawing, points) : null };
    }) : [];
    return this;
  }
  /** Each line across's value; a selected line's or box's, each point's (and as it's drawn): as the axis reads it. */
  axisTags() {
    if (!this.fresh().shown.length) return NONE;
    if (this.priceTags) return this.priceTags;
    const text = this.valueText();
    return (this.priceTags = this.shown.flatMap(({ drawing, points }) => (drawing.kind === "hline" ? [0] : told(drawing) && POINTED.has(drawing.kind) ? [0, points.length - 1] : []).map((i) => axisTag(points[i].y, text(drawing.points[i][1]), colorOfDrawing(drawing)))));
  }
  /**
   * Values as this series' axis writes them: a % or indexed scale reads each against the series' first value in view,
   * as the chart's own labels do (its percent from it, or 100 at it).
   */
  valueText() {
    const series = this.series, scale = seriesScale(series), write = layerFormats.get(this.layer) ?? precise;
    if (!scale?.relative) return write;
    // (Its first value in view: what its scale reads against.)
    const base = resolved?.times.length ? series.engine.first(firstInView()) : NaN;
    return (/** @type {number} */ value) => {
      if (!base) return "";
      const change = (100 * (value - base)) / Math.abs(base);
      return scale.mode === "percent" ? precise(change, "%") : precise(indexed(change));
    };
  }
  /** Each line down's date; a selected line's or box's, each point's (and as it's drawn). */
  dateTags() {
    if (!this.fresh().shown.length) return NONE;
    if (this.timeTags) return this.timeTags;
    return (this.timeTags = this.shown.flatMap(({ drawing, points }) => (drawing.kind === "vline" ? [0] : told(drawing) && POINTED.has(drawing.kind) ? [0, points.length - 1] : []).map((i) => axisTag(points[i].x, dateLabel(Math.round(drawing.points[i][0]), FINE.has(shownIndex())), colorOfDrawing(drawing)))));
  }
  draw(context, size) {
    this.size = size;
    const card = css("--card");
    for (const { drawing, points } of this.shown) paintDrawing(context, drawing, points, size, card);
    if (capturing) return;
    for (const { drawing, points } of this.shown) if (lit(drawing) || hovering.id === drawing.id) paintHold(context, drawing, points, size, card, lit(drawing));
  }
  /**
   * The drawing at a point of the pane, within `reach`: a point of one before a body, the topmost body; and how it's
   * taken there (`over`: its body, a point, or a box's corner, ↖↘ or ↗↙ as it lies on screen).
   */
  hit(x, y, reach) {
    this.fresh();
    let body = null;
    // (From the last drawn, on top.)
    for (let i = this.shown.length - 1; i >= 0; i--) {
      const { drawing, points, box } = this.shown[i];
      // (Its box on its outermost points: a pointer as near as its line's reach, half its width out.)
      const near = reach + drawing.width / 2 + 1;
      if (box && !(x >= box.left - near && x <= box.right + near && y >= box.top - near && y <= box.bottom + near)) continue;
      // (A line across's or down's label over its dot: the dot only moves it, as its body does.)
      if (!body && (drawing.kind === "hline" || drawing.kind === "vline") && onLabel(drawing, points, this.size, x, y)) body = { drawing, handle: -1, over: "label" };
      // (A line's or a path's box, selected: its corners after its points, its sides after them, see moveTo.)
      const handles = handlesOf(drawing, points, this.size), handle = handles.findIndex((point) => Math.hypot(point.x - x, point.y - y) <= reach + 1);
      const ends = THROUGH.has(drawing.kind) ? points.length : 0, ring = ends && lit(drawing) ? boxCorners(drawing, points) : handles.slice(ends);
      if (handle >= 0 && body?.drawing !== drawing) {
        const middle = { x: ring.reduce((sum, point) => sum + point.x, 0) / ring.length, y: ring.reduce((sum, point) => sum + point.y, 0) / ring.length };
        const corner = BOXES.has(drawing.kind) || drawing.kind === "brush" || (ends > 0 && handle >= ends);
        return { drawing, handle, over: corner ? ((handles[handle].x - middle.x) * (handles[handle].y - middle.y) > 0 ? "nwse" : "nesw") : "handle" };
      }
      // (Under a drawing found above (a side of one too): only its points still win.)
      if (body) continue;
      // (A stroke's, a line's or a path's own, before its box's sides: its box is on its outermost points, all along
      // it where it's flat.)
      const side = (ends || drawing.kind === "brush") && touches(drawing, points, x, y, this.size, reach) ? -1 : sideOf(drawing, ring, x, y, reach);
      if (side >= 0) {
        body = { drawing, handle: ends + side, over: side % 2 ? "ew" : "ns" };
        continue;
      }
      if (onLabel(drawing, points, this.size, x, y)) body = { drawing, handle: -1, over: "label" };
      else if (touches(drawing, points, x, y, this.size, reach)) body = { drawing, handle: -1, over: "drawing" };
    }
    return body;
  }
}
/** A drawing selected, or being drawn (a stroke aside: it shows as it's drawn). */
const lit = (drawing) => !capturing && (chosen.has(drawing.id) || (drawing === draft && drawing.kind !== "brush"));
/** A drawing whose points are told on the axes: the one selected (not one of several), or one being drawn. */
const told = (drawing) => lit(drawing) && (drawing === draft || chosen.size === 1);
/** Each series' sketch. @type {WeakMap<any, Sketch>} */
const sketches = new WeakMap();
/** The library's layers of a drawn series' own (its sketch's, a profile's shares on the axis), gone with it. @type {WeakMap<any, any[]>} */
const ownLayers = new WeakMap();
/** A layer of a series' own, on its pane and side. @param {any} series @param {any} custom */
function ownLayer(series, custom) {
  engine.panes[seriesPane(series)]?.add(custom, series.side);
  (ownLayers.get(series) ?? ownLayers.set(series, []).get(series))?.push(custom);
}
/** A series' own layers gone from its pane, as it goes. @param {any} series */
function dropOwnLayers(series) {
  const pane = engine.panes[seriesPane(series)];
  for (const custom of ownLayers.get(series) ?? []) pane?.remove(custom);
  ownLayers.delete(series);
}
/** The drawings as shown: the chart's (a look's, while one shows) as last built, one being drawn or moved in its place, a measure. */
function drawingsShown() {
  const passing = /** @type {Drawing[]} */ ([measuring, draft].filter(Boolean));
  if (drawingsHidden) return passing;
  return [...shownDrawings.map((drawing) => reshaped.get(drawing.id) ?? drawing), ...passing];
}
/** Every sketch drawn again: the drawings, what's selected or pointed at, changed. */
function repaintDrawings() {
  // (Under the Charts page, none: painted as it's left, see redrawChart.)
  if (!home) for (const series of drawn.values()) sketches.get(series)?.update();
  // (A drawing placed point by point: between its points, the belt's to press (its Done).)
  $("belt").classList.toggle("busy", Boolean(placing?.held || moving?.moved || boxing));
}
/** The drawing under the pointer, its handle there (-1: none), and how it's taken there (a corner's way). */
let hovering = { id: "", handle: -1, over: "" };
const LENS = 168, LENS_ZOOM = 2;
/** Whatever's in hand put down (another chart, the grid). */
const putDown = () => takeUp("");
/** ⌘ (Ctrl elsewhere) and a drag: a box around drawings. */
const boxKey = (/** @type {PointerEvent} */ event) => (MAC ? event.metaKey : event.ctrlKey);
const upright = (/** @type {string} */ side) => side === "left" || side === "right";
/** A belt button's name, its key (after D; the ruler's is its own, M), its icon. */
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
/**
 * The belt from its last shape to this one (its place, its size), each part kept (by its id: a tool, the grip, a
 * look's button) moving from where it was with it, the new ones coming in: one motion, nothing jumping or blinking
 * (all of them faded in again, each change, as the box moved: the tools blinked, and stood in their new places in
 * a box still the old one's, cut).
 */
/** The belt's reshaping under way (see reshapeBelt). @type {Animation | null} */
let beltMotion = null;
/** The belt at rest, at once: its motions cut short, its parts let go of (as laid out by the next measure, not later). */
function settleBelt() {
  const belt = $("belt");
  beltMotion = null;
  for (const motion of belt.getAnimations({ subtree: true })) motion.cancel();
  for (const child of all$(":scope > :not(.belt-row), :scope > .belt-row > *", belt)) Object.assign(child.style, { minWidth: "", minHeight: "" });
}
function reshapeBelt(/** @type {DOMRect} */ was, /** @type {Map<string, DOMRect>} */ partsWere) {
  const belt = $("belt");
  // (One under way (it changed again meanwhile): from where it shows, `was`, the box measured at rest.)
  settleBelt();
  const now = belt.getBoundingClientRect();
  if (Math.abs(now.left - was.left) + Math.abs(now.top - was.top) + Math.abs(now.width - was.width) + Math.abs(now.height - was.height) < 2) return;
  const timing = { duration: MOTION.small, easing: MOTION.ease }, left = parseFloat(belt.style.left), top = parseFloat(belt.style.top);
  // (Measured all first, where they come to rest, then moved: one layout, and the box's own motion not in it.)
  const upright = belt.classList.contains("upright"), parts = all$(":scope > .grip, .belt-row :is(button, .belt-rule), .belt-caption", belt).map((part) => /** @type {[HTMLElement, DOMRect, DOMRect | undefined]} */ ([part, part.getBoundingClientRect(), part.id ? partsWere.get(part.id) : undefined]));
  // (What's in it held at its size as the box reshapes (narrower, its row would wrap: a tool on a second line, cut),
  // the box showing more of it or less.)
  // (Its boxes as laid out: its row's own, the row itself none, see .belt-row.)
  const held = all$(":scope > :not(.belt-row), :scope > .belt-row > *", belt).map((child) => /** @type {[HTMLElement, DOMRect]} */ ([child, child.getBoundingClientRect()]));
  for (const [child, box] of held) Object.assign(child.style, { minWidth: `${box.width}px`, minHeight: `${box.height}px` });
  const motion = belt.animate({ left: [`${left + was.left - now.left}px`, `${left}px`], top: [`${top + was.top - now.top}px`, `${top}px`], width: [`${was.width}px`, `${now.width}px`], height: [`${was.height}px`, `${now.height}px`], overflow: ["hidden", "hidden"] }, timing);
  // (Let go once it's done: one cut short by the next (it changed again), not the next's holds.)
  beltMotion = motion;
  const release = () => beltMotion === motion && held.forEach(([child]) => Object.assign(child.style, { minWidth: "", minHeight: "" }));
  motion.finished.then(release, release);
  for (const [part, at, from] of parts) {
    // (From where it was: less the box's own move, which carries it too, on the same curve, and its centering across
    // the box (its height, upright its width) as that changes.)
    if (from) part.animate({ translate: [`${from.left - at.left - (was.left - now.left) - (upright ? (was.width - now.width) / 2 : 0)}px ${from.top - at.top - (was.top - now.top) - (upright ? 0 : (was.height - now.height) / 2)}px`, "0 0"] }, timing);
    else part.animate({ opacity: [0, 1] }, timing);
  }
}
/**
 * The plot, in the canvas' coordinates: from its first pane's top to its last pane's foot, between the price axes.
 * `boxed`: as the chart will lie in its box once a layout's motion ends (its size held meanwhile: see sizeHeld), the
 * box's own edges less its axes, following the box as it moves; else what shows of it. At rest, the same.
 */
function plotBox(boxed = false) {
  const first = paneBox(0), last = paneBox(engine.panes.length - 1);
  if (!first || !last) return null;
  const canvas = $("canvas").getBoundingClientRect(), chartBox = $("chart").getBoundingClientRect(), element = engine.element.getBoundingClientRect();
  // (A scale not laid out yet: 0 wide.)
  const axis = engine.left, firstTop = first.top, lastBottom = last.bottom;
  // (What shows of it: held larger than its box through a layout's motion, the chart's cut at the box's edges.)
  const [left, right, top, bottom] = boxed
    ? [chartBox.left + axis, chartBox.right - (element.width - axis - engine.time.width), chartBox.top + (firstTop - element.top), chartBox.bottom - (element.bottom - lastBottom)]
    : [Math.max(element.left + axis, chartBox.left), Math.min(element.left + axis + engine.time.width, chartBox.right), Math.max(firstTop, chartBox.top), Math.min(lastBottom, chartBox.bottom)];
  return { left: left - canvas.left, top: top - canvas.top, width: Math.max(0, right - left), height: Math.max(0, bottom - top) };
}
/** The sides the drawing tools dock on, as choices (⌘K's, the grip's by keys). */
const dockChoices = () => /** @type {const} */ (["top", "bottom", "left", "right"]).map((side) => ({ key: side, label: capitalized(side), current: prefs.belt.dock === side, run: () => dockBelt(side) }));
/** The drawing tools docked on a side (⌘K's way to what dragging them does), in the middle of it. */
function dockBelt(/** @type {"top" | "bottom" | "left" | "right"} */ side) {
  Object.assign(prefs.belt, { dock: side, at: 0.5 });
  savePrefs();
  turnBelt(upright(side));
  placeBelt(true);
}
/** The belt's size as last placed (to keep its middle where it was as it grows or shrinks). */
let beltSize = { width: 0, height: 0 };
/** The belt's pressed tool last brought into sight, on a phone. */
let beltPressedSeen = "";
/**
 * The docked belt stays inside the plot (top and foot wrap); `stay` keeps its tools in place as its contents grow
 * or shrink.
 */
function placeBelt(glide = false, stay = false) {
  // (Where the plot will be: through a layout's motion (the sidebar, the panel, full screen), the belt goes along with
  // the chart's box, not with the chart held meanwhile (it'd jump to its place at the motion's end).)
  const belt = $("belt"), plot = belt.hidden || belt.classList.contains("held") ? null : plotBox(true);
  if (!plot) return;
  // Measured at rest: a glide still going ends where it was going (its coming in, a fade and 4px, isn't one: it plays).
  belt.getAnimations().forEach((animation) => animation instanceof CSSAnimation || animation.finish());
  // A phone's: in the stage foot's place while it shows (the interval and range wait under it), one row sliding
  // sideways, its tool in sight, what the tool does over the plot's foot; the belt never over the chart (no grip:
  // nowhere to take it).
  if (phone.matches) {
    const foot = $("stage-foot").getBoundingClientRect(), canvas = $("canvas").getBoundingClientRect();
    belt.classList.remove("upright");
    belt.dataset.dock = "bottom";
    Object.assign(belt.style, { maxWidth: "", width: `${foot.width}px`, height: `${foot.height}px`, left: `${foot.left - canvas.left}px`, top: `${foot.top - canvas.top}px` });
    // (Its pressed tool brought into sight when it's another: the row stays where a finger slid it, else.)
    const pressed = belt.querySelector('[aria-pressed="true"]');
    if (pressed?.id !== beltPressedSeen) pressed?.scrollIntoView({ inline: "nearest", block: "nearest" });
    beltPressedSeen = pressed?.id ?? "";
    // (More past an edge: that edge fades, as a strip's.)
    const row = /** @type {HTMLElement | null} */ (belt.querySelector(".belt-row"));
    if (row) fitStrip(row);
    // (Its caption on the plot's side, as a belt's elsewhere: over the plot, above the dates, which a finger drawing reads.)
    const caption = /** @type {HTMLElement | null} */ (belt.querySelector(".belt-caption"));
    if (caption) Object.assign(caption.style, { position: "fixed", top: "auto", right: "auto", left: `${foot.left + 10}px`, bottom: `${innerHeight - $("chart").getBoundingClientRect().bottom + engine.timeHeight + 6}px`, translate: "none", transform: "none" });
    return;
  }
  belt.style.width = belt.style.height = "";
  belt.querySelector(".belt-caption")?.removeAttribute("style");
  let side = prefs.belt.dock, up = upright(side);
  belt.classList.toggle("upright", up);
  // Standing taller than the plot (a short window, a phone on its side): it lies along the foot, this time.
  if (up && belt.offsetHeight > plot.height - 16) {
    [side, up] = ["bottom", false];
    belt.classList.remove("upright");
  }
  belt.style.maxWidth = up ? "" : `${plot.width - 16}px`;
  // (Lying, wider than the plot, it wraps: into even rows, as narrow as they stay that many (one tool alone on a
  // row of its own read as dropped there).)
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
  // (Kept where it was, it grows or shrinks from its middle: what comes in, on both sides.)
  const left = up ? (side === "left" ? plot.left + 10 : plot.left + plot.width - width - 10) : kept && Number.isFinite(was.left) ? clamp(was.left + (beltSize.width - width) / 2, plot.left + 8, plot.left + plot.width - width - 8) : along(plot.left, plot.width, width);
  const top = up ? (kept && Number.isFinite(was.top) ? clamp(was.top + (beltSize.height - height) / 2, plot.top + 8, plot.top + plot.height - height - 8) : along(plot.top, plot.height, height)) : side === "top" ? plot.top + 10 : plot.top + plot.height - height - 10;
  beltSize = { width, height };
  belt.dataset.dock = side;
  if (glide && !reduced.matches) belt.animate({ left: [belt.style.left, `${left}px`], top: [belt.style.top, `${top}px`] }, { duration: MOTION.small, easing: MOTION.ease });
  Object.assign(belt.style, { left: `${left}px`, top: `${top}px` });
}
/** The side a belt carried to a point goes to: the left or right within the plot's outer edge, else the top or foot. */
function sideAt(/** @type {number} */ x, /** @type {number} */ y, plot) {
  const edge = Math.min(120, plot.width * 0.15);
  return x < plot.left + edge ? "left" : x > plot.left + plot.width - edge ? "right" : y < plot.top + plot.height / 2 ? "top" : "bottom";
}
/** The belt turned to stand (or to lie): each part glides from where it was to its new place, the belt's box with them. */
function turnBelt(/** @type {boolean} */ up) {
  const belt = $("belt");
  if (belt.classList.contains("upright") === up) return;
  const parts = all$(".grip, .cap, .belt-rule", belt), from = belt.getBoundingClientRect(), before = parts.map((part) => part.getBoundingClientRect());
  // (A turn still going is let go from where it is: a quick turn back starts there.)
  settleBelt();
  belt.classList.toggle("upright", up);
  if (reduced.matches) return;
  const to = belt.getBoundingClientRect(), timing = { duration: MOTION.small, easing: MOTION.ease };
  parts.forEach((part, i) => {
    const now = part.getBoundingClientRect(), dx = before[i].left - from.left - (now.left - to.left), dy = before[i].top - from.top - (now.top - to.top);
    if (dx || dy) part.animate({ transform: [`translate(${dx}px, ${dy}px)`, "none"] }, timing);
  });
  belt.animate({ width: [`${from.width}px`, `${to.width}px`], height: [`${from.height}px`, `${to.height}px`] }, timing);
}
/** The belt on (it stays) or off (it shows only while drawing, or with a drawing selected). */
/** The drawing tools on, shown at all times. */
function showBelt() {
  prefs.belt.on = true;
  savePrefs();
  renderBelt();
}
/** The header's button: on, or off and put away. */
const toggleBelt = () => (prefs.belt.on ? hideBelt() : showBelt());
/** The drawing tools in sight (on, or up for what's in hand). */
const beltUp = () => !$("belt").hidden;
/** A tool from the belt: the cursor puts down what's in hand, Select boxes drawings (or stops), a drawing tool's taken (or put down). */
function pickTool(/** @type {string} */ each) {
  if (each === "select") return takeUp(selectingArea ? "" : "select");
  if (each === "magnifier") return takeUp(magnifying ? "" : "magnifier");
  takeUp(each === "cursor" || each === tool ? "" : /** @type {DrawingKind} */ (each));
}
/** Whether a kind of drawing has a look: every one a color and an opacity; a line, its width and style; a trend line, its ends; words (a text's, a line's label's), their size and background. */
const hasLook = (/** @type {string | undefined} */ kind, /** @type {string} */ part) => !!kind && kind !== "ruler" && (part === "width" || part === "dash" ? kind !== "text" : part === "ends" || part === "start" || part === "end" ? THROUGH.has(kind) : part === "size" || part === "back" ? kind === "text" || LINES.has(kind) : true);
/** The first of some drawings with words: a text, a labelled line. */
const wordsIn = (/** @type {(Drawing | undefined)[]} */ drawings) => drawings.find((item) => item?.kind === "text" || (item && LINES.has(item.kind) && item.label));
const backOf = (/** @type {string | undefined} */ back) => BACKS.find(([each]) => each === (back ?? "")) ?? BACKS[0];
/** A background's glyph (⌘K's): a bar of words in a box as it asks. */
const backGlyph = (/** @type {string} */ back) => glyphSvg(`${back ? `<rect x="1" y="2" width="14" height="8" rx="2.5" ${back === "outline" ? 'fill="none"' : `fill="currentColor" fill-opacity="${back === "solid" ? 1 : 0.4}"`} ${back.includes("outline") ? 'stroke="currentColor" stroke-width="1.2"' : 'stroke="none"'}/>` : ""}<path d="M4.5 6h7" stroke-width="1.6" stroke-linecap="round" ${back === "solid" ? 'style="stroke: var(--card)"' : 'stroke="currentColor"'}/>`);
/** A background as the belt shows it: words on it, in their color. */
const backChip = (/** @type {string} */ back, /** @type {string} */ color) => `<span class="back-chip" data-back="${back}" style="--c:${escapeHtml(cssColor(color))};--on:${contrastOn(color)}">Aa</span>`;
/** A look as the belt shows it: with a tool in hand, the pen's; else the first selected drawing that has it's. */
const lookFor = (/** @type {string} */ part, drawings = /** @type {Drawing[]} */ ([...chosen].map(drawingById).filter(Boolean))) => (tool ? prefs.pen : drawings.find((drawing) => hasLook(drawing.kind, part)) ?? drawings[0]);
/** An end's name (plain, none set). */
const endName = (/** @type {End | "" | undefined} */ end) => (ENDS.find(([each]) => each === (end ?? "")) ?? ENDS[0])[1];
/** A trend line's ends to pick, its start's and its end's, each as it'd be drawn (picked in place: both a click away, see below). */
function endsPopover(/** @type {HTMLElement} */ button) {
  const look = lookFor("ends");
  if (!look) return;
  const row = (/** @type {"start" | "end"} */ side) => `<div class="ends-row"><span aria-hidden="true">${side === "start" ? "Start" : "End"}</span>${beltTabs(side === "start" ? "Start" : "End", ENDS.map(([end, text, tip]) => `<button type="button" data-pick-${side}="${end}" aria-pressed="${(look[side] ?? "") === end}" aria-label="${side === "start" ? "Start" : "End"}: ${text}" data-tip="${text}: ${tip.toLowerCase()}">${side === "start" ? endsGlyph(end, "") : endsGlyph("", end)}</button>`))}</div>`;
  beltPopover(button, "Ends", `${row("start")}${row("end")}`, () => {});
}
/**
 * A look set from the belt: the pen's, so the next drawing takes it, whatever its tool; and with drawings selected,
 * each one's that has it, in one step (`coalesce`: one step a drag; `keepMenu`: made from a popover that stays, as
 * its slider).
 */
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
/** Where the belt's popovers open: toward the chart, away from the edge it's docked on (each the same way). */
const beltSide = () => ({ bottom: "above", top: "below", left: "right", right: "left" })[$("belt").dataset.dock ?? ""] ?? "";
/** A choice from a belt button's popover, as the layers panel makes it (its tabs, its slider); focus back on the button. */
function beltPopover(/** @type {HTMLElement} */ button, /** @type {string} */ title, /** @type {string} */ body, onPick) {
  showMenu(button, `<div class="field belt-pop"><span>${title}</span>${body}</div>`, (data) => {
    onPick(data);
    refocus(`#${button.id}`);
  }, null, beltSide());
}
/** A row of tabs, as the layers panel's. */
const beltTabs = (/** @type {string} */ label, /** @type {string[]} */ items) => `<div class="tabs ${items.some((item) => item.includes("<svg")) ? "glyphs" : ""}" role="group" aria-label="${label}">${items.join("")}</div>`;
// The width's or the opacity's slider, in its popover: scrubbed or stepped by keys as the layers panel's, one undo
// step a drag.
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
/** The drawing under the pointer, lit; its cursor on the chart. */
function hover(hit) {
  const next = hit ? { id: hit.drawing.id, handle: hit.handle, over: hit.over } : { id: "", handle: -1, over: "" };
  if (next.id === hovering.id && next.handle === hovering.handle && next.over === hovering.over) return;
  hovering = next;
  if (next.id) $("chart").dataset.over = hit.over;
  else delete $("chart").dataset.over;
  repaintDrawings();
}
/** A point of the chart in its pane's plot, where drawings are drawn, in pixels. */
const plotPoint = (clientX, clientY, rect) => ({ x: clientX - engine.element.getBoundingClientRect().left - engine.left, y: clientY - rect.top });
/** The drawing under a point of the chart: a point of one first, else the last drawn. */
function drawingAt(clientX, clientY, reach = REACH) {
  const pane = shownDrawings.length ? paneAt(clientY) : null, id = pane && chart.panes[pane.index]?.id;
  if (!pane || !id) return null;
  const at = plotPoint(clientX, clientY, pane.rect);
  // Over an axis, the axis'.
  if (at.x < 0 || at.x > engine.time.width) return null;
  // (On top as painted: as the pane's series are, its first layer's over the others (reversed, its last's); then the
  // later added; a point of one (a corner, an end; not a side) over any body.)
  const inPane = chart.layers.filter((layer) => layer.pane === id);
  const order = new Map((chart.panes[pane.index].reverse ? inPane : inPane.reverse()).map((layer, i) => [layer.id, i])), score = (hit) => (hit.handle >= 0 && hit.over !== "ew" && hit.over !== "ns" ? MAX_DRAWINGS * (chart.layers.length + 1) : 0) + (order.get(hit.drawing.layer) ?? 0) * MAX_DRAWINGS + shownDrawings.indexOf(hit.drawing);
  let found = null;
  for (const layer of chart.layers) {
    const hit = layer.pane === id ? sketches.get(drawn.get(layer.id))?.hit(at.x, at.y, reach) : null;
    if (hit && (!found || score(hit) > score(found))) found = hit;
  }
  return found;
}
/** The layer drawn on in a pane: the selected one, if it's there and shown; else its first shown (its first, every one hidden). */
function hostIn(index) {
  // (Never a signal: its own series isn't seen, its marks are its layer's.)
  const id = chart.panes[index]?.id, mates = chart.layers.filter((layer) => layer.pane === id && layer.type !== "signal" && drawn.has(layer.id));
  return mates.find((layer) => layer.id === selected && !layer.hidden) ?? mates.find((layer) => !layer.hidden) ?? mates[0] ?? null;
}
/**
 * A point on a layer's scale, from where the pointer is: its time on the nearest bar (`free`: anywhere), past the
 * data too; its value drawn to a shown layer's at that bar within reach (`pull`: a candle's high, a line's point),
 * else as the scale reads there.
 * @returns {[number, number] | null}
 */
function pointFrom(at, layer, { free = false, pull = false } = {}) {
  const line = timeLine(), series = drawn.get(layer.id);
  if (!line || !series) return null;
  const exact = (at.x - line.at) / line.step, near = pull ? nearestValue(Math.round(exact), at.y, layer.pane) : null;
  const value = near && near.layer === layer.id ? near.value : seriesValue(series, near?.y ?? at.y);
  return value === null || !Number.isFinite(value) ? null : pointOf(timeOfBar(free ? exact : Math.round(exact)), value);
}
/** The value nearest a height at a bar among a pane's shown layers' as drawn (a candle's four; stacked, its total), within reach. */
function nearestValue(bar, y, pane) {
  if (bar < 0 || bar >= (resolved?.times.length ?? 0)) return null;
  let best = null;
  for (const layer of chart.layers) {
    const made = built.get(layer.id), series = drawn.get(layer.id);
    // (Not a signal's: its carrier's drawn nowhere.)
    if (layer.pane !== pane || layer.hidden || placedOn(layer) || !made || !series) continue;
    for (const value of /** @type {number[]} */ (layer.type === "candles" && !made.stacked && made.opens && Number.isFinite(made.shown?.[bar]) ? [made.opens[bar], made.highs?.[bar], made.lows?.[bar], made.shown?.[bar]] : made.low ? [made.low[bar], made.shown?.[bar]] : [made.shown?.[bar]])) {
      const at = Number.isFinite(value) ? seriesY(series, value) : null;
      if (at !== null && Math.abs(at - y) <= PULL && (!best || Math.abs(at - y) < best.off)) best = { layer: layer.id, value, y: at, off: Math.abs(at - y) };
    }
  }
  return best;
}
export {
  backChip, backGlyph, backOf, barOf, beltPopover, beltSide, beltTabs, beltTool, beltUp, boundsOf, boxKey, dockChoices,
  drawingAt, drawingById, drawingNames, dropOwnLayers, endName, endsGlyph, endsPopover, finite, hasLook, hostIn, hover,
  labelPlace, laidOver, LENS, LENS_ZOOM, lineThrough, lookFor, newDrawing, onScreen, ownLayer, ownLayers, pickTool,
  placeBelt, plotBox, plotPoint, pointFrom, PULL, putDown, reachOf, repaintDrawings, reshapeBelt, rovingBelt,
  rulerReading, setDrawSlider, setLook, shareAlong, showBelt, sideAt, Sketch, sketches, tagShapes, textPlace, tickOn,
  timeLine, timeOfBar, toggleBelt, toSegment, turnBelt, upright, wordsColor, wordsIn, wordsOn,
};
