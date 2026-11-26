// @ts-check
// The chart's drawings as they paint on it, with what it marks itself (halvings, a trend's ends): each series' sketch.
/** @import { Drawing, Layer } from "/studio/vocabulary.js" */
/** @import { Resolved, Result } from "/studio/chart.js" */
import { $ } from "/studio/dom.js";
import { FINE } from "/studio/intervals.js";
import { oklchToHex, hexToOklch } from "/studio/color.js";
import { growth, amount, duration, precise, dateLabel } from "/studio/format.js";
import { nearestFinite } from "/studio/formulas.js";
import { layerById } from "/studio/looks.js";
import { THROUGH, TEXT_SIZE, BOXES, POINTED } from "/studio/tools.js";
import { FAINT, inkOn, Custom, indexed } from "/modules/bitview-charts/index.js";
import { lowerBound, clamp } from "/studio/math.js";
import { indexAt, timeAt, resolved, engine, css, SANS, labelTyping, measurer, chart, shownOklch, revision, drawVersion, minedHalvings, nextHalvings, typingIn, capturing, chosen, draft, measuring, drawingsHidden, shownDrawings, reshaped, home, placing, moving, boxing } from "/studio/app.js";
import { shownIndex, shownSpan, seriesY, alpha, ink, dashFor, unitFor, viewName, drawn, fadedNow, seriesPane, seriesScale, layerFormats, firstInView, paneBox } from "/studio/chart.js";

/** A text cut with an ellipsis to fit a width, as the page cuts a long name. */
function fitted(context, text, width) {
  if (context.measureText(text).width <= width + 1) return text;
  let cut = text;
  while (cut.length > 1 && context.measureText(`${cut}…`).width > width) cut = cut.slice(0, -1);
  return `${cut.trimEnd()}…`;
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
      // (Each in its line's color there, as drawn (painted date by date, or by its gradient): a flip's last point is its
      // old side's, as drawn before it.)
      const engine = series?.engine;
      this.endsLook = { color: engine?.color, radius: own.width / 2 + 1, at: engine ? (/** @type {number} */ i) => engine.colorAt(i) : null };
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
/** The drawing under the pointer, lit; its cursor on the chart. */
function hover(hit) {
  const next = hit ? { id: hit.drawing.id, handle: hit.handle, over: hit.over } : { id: "", handle: -1, over: "" };
  if (next.id === hovering.id && next.handle === hovering.handle && next.over === hovering.over) return;
  hovering = next;
  if (next.id) $("chart").dataset.over = hit.over;
  else delete $("chart").dataset.over;
  repaintDrawings();
}
/** How near a press must be to take a drawing (a finger's, further), and to a value to be drawn to it, in pixels. */
const REACH = 6, PULL = 8;
export { fitted, Sketch, sketches, ownLayers, paintHalvings, paintEnds, paintHalvingNames, ownLayer, timeLine, halvingHosts, barOf, segmentEnds, drawingsShown, onScreen, boundsOf, NONE, told, axisTag, colorOfDrawing, paintDrawing, lit, hovering, paintHold, reach, onLabel, handlesOf, boxCorners, touches, sideOf, laidOver, wordsOn, wordsTones, contrastOf, luminance, tickOn, contrastOn, repaintDrawings, dropOwnLayers, plotBox, arrowBack, calendarAt, clipOut, fills, finite, hostsFound, labelPlace, lineThrough, ordinal, paintArrow, paintLabel, paintRuler, pointBars, rulerReading, strokeThrough, textPlace, toSegment, wordsColor, arrowSize, placedBars, tagShape, measureTag, tagShapes, hover, PULL, REACH
};
