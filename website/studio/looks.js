// @ts-check
// How a layer and a chart read and draw, the same wherever they show (the chart, its legends, the layers panel, the
// inspector, the grid's tiles): a layer's name, look, fill and stack, a pane's scale, a chart's title, a series' mark.
import { capitalized, inSentence } from "/studio/words.js";
import { cohortWho, HEAT_AGES, chart, ownerOf, fromLetters, shownOklch, light, thumbs, profileColor, css } from "/studio/app.js";
import { HEAT_MEASURES, measureOf, HEAT_WEIGHTS, summed, placedOn, heatLike, edgesOf, LINE_STYLE, gradientOfLayer, belowOf, LINED, dashOf, DASHES } from "/studio/vocabulary.js";
import { humanize } from "/studio/format.js";
import { stack as stackColumns, placeIndex, gradientTable as mixedTable } from "/modules/bitview-charts/index.js";
import { idOf, SIDES, oklchOf, ink, viewName, colorNow } from "/studio/chart.js";
import { hexToOklch, ofPalette, oklchToHex } from "/studio/color.js";
import { markBody } from "/studio/icons.js";
import { escapeHtml } from "/studio/dom.js";
/** @import { Vector } from "/studio/formulas.js" */
/** @import { Layer } from "/studio/vocabulary.js" */
/** @import { Chart, Pane } from "/studio/vocabulary.js" */

/** @param {Layer} layer */
const measureName = (layer) => HEAT_MEASURES.find(([id]) => id === measureOf(layer))?.[1];
const weightName = (/** @type {Layer} */ layer) => HEAT_WEIGHTS.find(([id]) => id === (layer.weight ?? ""))?.[1];
/** @param {Layer} layer */
const weightSaid = (layer) => (layer.weight ? `, ${/** @type {string} */ (weightName(layer)).toLowerCase()}` : "");
/** @param {Layer} layer */
const heatName = (layer) => layer.type === "profile" ? profileName(layer) : `${measureName(layer)} by cost basis${weightSaid(layer)}: ${cohortWho(layer.heatmap ?? "all")}`;
/** @param {Layer} layer */
const profileName = (layer, grouped = true) => ((groups) => `${measureName(layer)} profile${weightSaid(layer)}${!grouped || !groups.length ? "" : `: ${groups.length > 2 ? capitalized(groups.every((cohort) => HEAT_AGES.includes(cohort)) ? (groups.length === HEAT_AGES.length ? "every age band" : `${groups.length} age bands`) : `${groups.length} holder groups`) : groups.map((cohort, k) => (k ? inSentence(cohortWho(cohort)) : cohortWho(cohort))).join(" and ")}`}`)(layer.profile ?? []);
// @ts-check
// How a layer and a chart read and draw, the same wherever they show (the chart, its legends, the layers panel, the
// inspector, the grid's tiles): a layer's name, look, fill and stack, a pane's scale, a chart's title, a series' mark.
const STACK_FILL = 0.35, STACK_WIDTH = 1;
/**
 * Stacked or filled above is even: a band fading by price would read lighter the lower it sits, and above a high line all but vanish.
 * @param {Layer} layer @param {boolean} stacked @returns {[number, number]}
 */
function fillOf(layer, stacked) {
  const regime = Boolean(layer.colorFrom || layer.byLevels), above = layer.fill === "above";
  const top = layer.fillOpacity ?? (stacked || above ? (regime ? 0.3 : above ? 0.1 : STACK_FILL) : regime ? 0.5 : 0.2);
  return [top, stacked || above || layer.solid ? top : regime ? top * 0.08 : 0];
}
let stacksMade = { key: "", stacked: /** @type {Map<string, { values: Float64Array, foot: Float64Array, scale: Float64Array | null, group: string, rank: number }>} */ (new Map()), times: /** @type {unknown} */ (null) };
/**
 * `foot`: the stack under it (a share, in a 100% one); `scale`: the factor a 100% one's sums took, for its own values there (a candle's open, high, low, close); `group`: its stack (pane and side); `rank`: its place there (foot 0).
 * @param {Chart} doc
 * @param {{ results: Map<string, any>, times?: number[] }} data
 * @returns {Map<string, { values: Float64Array, foot: Float64Array, scale: Float64Array | null, group: string, rank: number }>}
 */
function stackOf(doc, data, kept = true) {
  // Kept while what they're made of is the same, so an edit elsewhere (a color, a width) finds the same totals and their series aren't set again. Marks' own, from other data, aren't kept.
  const key = kept && doc.id === chart?.id ? doc.panes.filter(summed).map((pane) => `${pane.id}:${pane.stack}:${pane.reverse ? 1 : 0}:${doc.layers.filter((layer) => layer.pane === pane.id).map((layer) => {
    const shown = data.results.get(layer.id)?.values;
    return `${layer.id}.${layer.side}.${layer.hidden ? 0 : 1}.${layer.from ?? ""}.${placedOn(layer) ? 1 : 0}.${shown ? idOf(shown) : "-"}`;
  }).join()}`).join("|") : "";
  if (key && key === stacksMade.key) return stacksMade.stacked;
  const stacked = new Map();
  for (const pane of doc.panes) {
    if (!summed(pane)) continue;
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
  for (const [id, item] of stacksMade.times === data.times ? stacked : []) {
    const was = stacksMade.stacked.get(id);
    if (was && was.group === item.group && was.rank === item.rank && sameVector(was.values, item.values) && sameVector(was.foot, item.foot) && (was.scale === item.scale || (was.scale && item.scale && sameVector(was.scale, item.scale)))) stacked.set(id, was);
  }
  stacksMade = { key, stacked, times: data.times };
  return stacked;
}
/** @param {ArrayLike<number>} a @param {ArrayLike<number>} b */
function sameVector(a, b) {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) if (a[i] !== b[i] && (a[i] === a[i] || b[i] === b[i])) return false;
  return true;
}
/** @param {Chart} doc */
const modeOf = (doc, layer) => paneMode(doc.panes.find((item) => item.id === layer.pane));
/**
 * A stack is linear (on log a band's height says nothing of its amount); a heatmap linear or log (percent change or indexed would start each row from its own first value). The kept scale never changes with the stack, so switching back and forth leaves the chart as it was.
 * @param {Pane} [pane]
 */
const paneMode = (pane) => (!pane || summed(pane) || (pane.stack && pane.mode > 1) ? 0 : pane.mode);
const labelOf = (layer) => layer.label || (layer.series ? humanize(layer.series) : layer.from ? fromName(layer) : heatLike(layer) ? heatName(layer) : layer.formula || "Formula");
/** @param {{ from?: string[], type?: string }} layer @param {Chart | null} [doc] */
const fromName = (layer, doc = ownerOf(/** @type {Layer} */ (layer))) => ((noun, letters, edges) => (edges.length > 1 && !letters.length ? `${noun} of the pane` : `${!letters.length ? noun : letters.length > 3 ? `${noun} of ${letters.length} layers` : `${noun} of ${letters.join(", ")}`}${edges.length ? ` to the ${edges.join(" and ")}` : ""}`))(layer.type === "trend" ? "Trend" : "Cloud", fromLetters(layer, doc), edgesOf(layer));
const layerById = (id) => chart.layers.find((layer) => layer.id === id);
const lineAt = (layer, key) => (key === "base" ? layer.baseLine ?? {} : layer.levels?.[Number(key)]);
const lineDefaults = (layer, key) => (key === "base" ? { color: "gray", dash: LINE_STYLE.SparseDotted } : { color: layer.color, dash: LINE_STYLE.LargeDashed });
const lineLook = (layer, key) => ({ ...lineDefaults(layer, key), ...lineAt(layer, key) });
/** @param {Layer} layer @param {number} value @returns {{ name: string, color: string } | null} */
function bandOf(layer, value) {
  const levels = layer.levels;
  if (!levels?.length || !Number.isFinite(value) || !(layer.underName || levels.some((level) => level.name))) return null;
  let at = -1;
  levels.forEach((level, i) => level.value <= value && (at < 0 || level.value > levels[at].value) && (at = i));
  if (at < 0) return layer.underName ? { name: layer.underName, color: layer.color } : null;
  return levels[at].name ? { name: /** @type {string} */ (levels[at].name), color: lineLook(layer, at).color } : null;
}
/** @param {Layer} layer */
const named = (layer) => Boolean(layer.levels?.length && (layer.underName || layer.levels.some((level) => level.name)));
/**
 * Sampled, not CSS's own mixing: between its stops that goes through gray in sRGB.
 * @param {[number, string][]} stops
 */
const gradientCss = (stops) => ((table) => `linear-gradient(to right, ${Array.from({ length: 17 }, (_, k) => `${table[placeIndex(k / 16, table.length)]} ${((k / 16) * 100).toFixed(2)}%`).join(", ")})`)(gradientTable(stops));
/** @type {Map<string, string[]>} */
const gradientTables = new Map();
// `--ink`'s dark side, which the light theme can't read.
const INK_DARK = "#f7f8f8";
const GRADIENT_STEPS = 64;
/**
 * Toned in the theme after mixing: the curve bends, so toning the stops first would draw another gradient. With any other color than the palette's on either side, each stop is as the theme shows it and the mix a hex.
 * @param {[number, string][]} stops @returns {string[]}
 */
function mixedOf(stops) {
  const lch = (/** @type {number[]} */ values) => `oklch(${values.join(" ")})`, given = (/** @type {[number, string]} */ [, color]) => !ofPalette(color);
  const forms = stops.map(([at, color]) => /** @type {[number, string]} */ ([at, !ofPalette(color) ? color : lch(color === "ink" ? hexToOklch(INK_DARK) : oklchOf(color))]));
  const kept = mixedTable(forms, GRADIENT_STEPS);
  if (!stops.some(given)) return kept;
  const shown = mixedTable(forms.map(([at, color]) => [at, lch(shownOklch(color))]), GRADIENT_STEPS);
  return kept.map((mixed, k) => {
    const next = stops.findIndex(([where]) => where >= (k / (GRADIENT_STEPS - 1)) * 100);
    const own = next < 0 ? !given(stops[stops.length - 1]) : !given(stops[next]) && (next === 0 || !given(stops[next - 1]));
    return own ? mixed : oklchToHex(...oklchOf(shown[k]));
  });
}
/** @param {[number, string][]} stops @returns {string[]} */
function gradientTable(stops) {
  const key = `${light}:${JSON.stringify(stops)}`, had = gradientTables.get(key);
  if (had) return had;
  const table = mixedOf(stops).map(ink);
  if (gradientTables.size > 200) gradientTables.clear();
  gradientTables.set(key, table);
  return table;
}
/** @param {Layer} layer */
const legendEntry = (doc, layer, after = "") => `<span style="--c:${escapeHtml(((stops) => (stops ? gradientCss(stops) : ink(layer.color)))(gradientOfLayer(doc, layer)))}">${escapeHtml(viewName(doc, layer))}${after}</span>`;
function runs(ys, x, y) {
  const parts = /** @type {string[][]} */ ([[]]);
  ys.forEach((value, i) => (value === null ? parts.push([]) : parts[parts.length - 1].push(`${x(i).toFixed(1)},${y(value).toFixed(1)}`)));
  return parts.filter((part) => part.length > 1);
}
/** @param {string[]} run */
function sides(run, y0) {
  const out = /** @type {{ above: { part: string[], from: number }[], below: { part: string[], from: number }[] }} */ ({ above: [], below: [] });
  const points = run.map((point) => point.split(",").map(Number));
  let part = /** @type {string[]} */ ([]), side = /** @type {"above" | "below"} */ ((points.find(([, y]) => y !== y0)?.[1] ?? y0) > y0 ? "below" : "above"), along = 0, from = 0;
  points.forEach(([x, y], i) => {
    const now = y < y0 ? "above" : y > y0 ? "below" : side;
    if (i) {
      const [px, py] = points[i - 1];
      if (now !== side) {
        const t = (y0 - py) / (y - py), cx = px + (x - px) * t;
        along += Math.hypot(cx - px, y0 - py);
        part.push(`${cx.toFixed(1)},${y0.toFixed(1)}`);
        if (part.length > 1) out[side].push({ part, from });
        part = [`${cx.toFixed(1)},${y0.toFixed(1)}`];
        from = along;
        along += Math.hypot(x - cx, y - y0);
      } else along += Math.hypot(x - px, y - py);
    }
    side = now;
    part.push(`${x},${y}`);
  });
  if (part.length > 1) out[side].push({ part, from });
  return out;
}
const barsOf = (ys, n, base, place) => Array.from({ length: n }, (_, k) => {
  let far = -1;
  for (let i = Math.floor((k * ys.length) / n); i < Math.floor(((k + 1) * ys.length) / n); i++) if (ys[i] !== null && (far < 0 || Math.abs(ys[i] - base) > Math.abs(ys[far] - base))) far = i;
  return far < 0 ? "" : place(k, ys[far], far);
}).join("");
/** @param {string[]} part @param {number} edge */
const areaPoints = (part, edge) => `${part[0].split(",")[0]},${edge} ${part.join(" ")} ${part.at(-1)?.split(",")[0]},${edge}`;
const areaOf = (/** @type {string[]} */ part, /** @type {number} */ edge, /** @type {string} */ color, opacity = ".2") => `<polygon points="${areaPoints(part, edge)}" fill="${color}" fill-opacity="${opacity}"/>`;
/** @param {string[]} colors */
const profileMark = (colors) => [[7, 1.5], [12, 4.75], [5, 8]].map(([length, at]) => colors.map((color, c) => ((part) => `<rect x="${(15 - part * (c + 1)).toFixed(1)}" y="${at}" width="${part.toFixed(1)}" height="2.5" fill="${escapeHtml(color)}"/>`)(length / colors.length)).join("")).join("");
// Kept from the last drawing while a new one is made, so an edit doesn't blank it.
function seriesMark(layer) {
  const thumb = thumbs.get(chart.id), ys = thumb && Object.hasOwn(thumb.marks, layer.id) ? thumb.marks[layer.id] : undefined;
  const level = thumb?.bases && Object.hasOwn(thumb.bases, layer.id) ? thumb.bases[layer.id] : 0;
  const now = layer.type === "histogram" && layer.colors === 4 ? null : colorNow(layer), color = escapeHtml(ink(now ?? layer.color)), below = now ? color : escapeHtml(ink(belowOf(layer)));
  const span = (ys?.length ?? 2) - 1, x = (i) => 0.8 + (i / span) * 14.4, y = (value) => 11 - value * 10;
  const [, , dash, gap] = LINED.has(layer.type) ? dashOf(layer.dash) : DASHES[0];
  const dashes = gap ? `stroke-dasharray="${+(dash * 0.8).toFixed(1)} ${+(gap * 0.8).toFixed(1)}" stroke-linecap="butt"` : 'stroke-linecap="round"';
  let body = `<path d="M1.5 6.5h13" stroke="${color}" stroke-width="1.4" ${dashes}/>`;
  const stops = gradientOfLayer(chart, layer);
  if (stops) body = ((table) => Array.from({ length: 8 }, (_, k) => `<rect x="${(1 + k * 1.75).toFixed(2)}" y="3.5" width="1.85" height="6" fill="${escapeHtml(table[placeIndex(k / 7, table.length)])}"/>`).join(""))(gradientTable(stops));
  else if (layer.type === "cloud") body = `<path d="M1 4.5c2.4-2 4.6-2.4 7-1.3s4.6 1 7-1.6v6.2c-2.4 2.6-4.6 2.6-7 1.5S3.4 8.1 1 10Z" fill="${color}" fill-opacity=".35"/>`;
  else if (layer.type === "signal") body = markBody(layer.place, color);
  else if (layer.type === "bubbles") body = `<circle cx="4.5" cy="7.5" r="2.5" fill="${below}" fill-opacity=".4" stroke="${below}"/><circle cx="11" cy="5.5" r="4" fill="${color}" fill-opacity=".4" stroke="${color}"/>`;
  else if (layer.type === "profile") body = profileMark(layer.profile?.length ? layer.profile.slice(0, 2).map((cohort) => ink(profileColor(layer, cohort))) : [ink("gray")]);
  else if (layer.type === "heatmap") body = [[0.3, 0.6, 1, 0.5], [0.6, 1, 0.45, 0.25], [1, 0.45, 0.2, 0.6]].map((row, r) => row.map((opacity, k) => `<rect x="${(1 + k * 3.6).toFixed(1)}" y="${(2 + r * 3.3).toFixed(1)}" width="3" height="2.7" rx=".5" fill="${layer.measure === "pnl" && !r ? below : color}" fill-opacity="${opacity}"/>`).join("")).join("");
  else if (ys && layer.type === "histogram" && modeOf(chart, layer) === 0) {
    body = barsOf(ys, 6, level, (k, value) => `<rect x="${(0.5 + k * 2.6).toFixed(1)}" y="${Math.min(y(value), y(level)).toFixed(1)}" width="1.8" height="${Math.max(1.2, Math.abs(y(value) - y(level))).toFixed(1)}" rx=".4" fill="${value >= level ? color : below}"/>`);
  } else if (ys && layer.type === "dots") {
    body = ys.map((value, i) => (value === null || i % 4 !== 1 ? "" : `<circle cx="${x(i).toFixed(1)}" cy="${y(value).toFixed(1)}" r="1.2" fill="${color}"/>`)).join("");
  } else if (ys) {
    const stepped = (part) => (layer.shape === "steps" && LINED.has(layer.type) ? part.flatMap((point, i) => (i ? [`${point.split(",")[0]},${part[i - 1].split(",")[1]}`, point] : [point])) : part);
    const edge = layer.fill === "above" ? 0.5 : 11.5;
    const line = (part, stroke, from = 0) => `<polyline points="${part.join(" ")}" fill="none" stroke="${stroke}" stroke-width="1.4" stroke-linejoin="round" ${dashes}${gap && from ? ` stroke-dashoffset="${from.toFixed(1)}"` : ""}/>`;
    body = runs(ys, x, y).map(stepped).map((part) => {
      if (layer.type === "baseline") {
        const { above, below: under } = sides(part, y(level));
        return [...above.map((piece) => line(piece.part, color, piece.from)), ...under.map((piece) => line(piece.part, below, piece.from))].join("");
      }
      return (layer.type === "area" ? areaOf(part, edge, color, ".22") : "") + line(part, color);
    }).join("");
  }
  return `<svg class="series-mark" viewBox="0 0 16 12" aria-hidden="true">${body}</svg>`;
}
const titleOf = (doc) => doc.title || derivedTitle(doc);
const derivedTitle = (/** @type {Chart} */ doc) => {
  const names = (doc.layers ?? []).filter((layer) => layer.type !== "signal").map(labelOf);
  return !names.length ? "Untitled chart" : names.length === 1 ? names[0] : names.length === 2 ? `${names[0]} & ${names[1]}` : `${names[0]} & ${names.length - 1} more`;
};
function faintOf(/** @type {string} */ color) {
  const [l, c, h] = shownOklch(color), [card] = oklchOf(css("--card"));
  return oklchToHex(l + (card - l) * 0.5, c * 0.5, h);
}
/** @param {Layer} layer */
const fadingOf = (layer, /** @type {boolean} */ below) => (below ? layer.colorBelowFading : layer.colorFading) ?? faintOf(below ? belowOf(layer) : layer.color);
/** @param {Vector | number[]} values @returns {[number, number]} */
function rangeOf(values) {
  let low = Infinity, high = -Infinity;
  // Hot: by index, finite by a subtraction; zeros by Math's own, a negative one kept as it was.
  for (let i = 0; i < values.length; i++) {
    const value = values[i];
    if (value === 0) (low = Math.min(low, value)), (high = Math.max(high, value));
    else if (value - value === 0) (low = value < low ? value : low), (high = value > high ? value : high);
  }
  return [low, high];
}
/**
 * No hindsight: each place is what that day would have shown. None where it has none.
 * @param {Vector} values @returns {Float64Array}
 */
function upToDatePlaces(values) {
  const places = new Float64Array(values.length).fill(NaN);
  let low = Infinity, high = -Infinity;
  for (let i = 0; i < values.length; i++) {
    const value = values[i];
    if (!Number.isFinite(value)) continue;
    (low = Math.min(low, value)), (high = Math.max(high, value));
    places[i] = high > low ? (value - low) / (high - low) : 0.5;
  }
  return places;
}

/** @param {Layer} layer */
function levelSteps(layer) {
  const steps = (layer.levels ?? []).map((level, k) => ({ value: level.value, color: lineLook(layer, k).color, k })).sort((a, b) => a.value - b.value || a.k - b.k);
  return steps.filter((step, i) => !i || step.value !== steps[i - 1].value);
}
export {
  heatName, profileName, measureName, weightSaid, weightName, fillOf, STACK_FILL, STACK_WIDTH, lineAt, lineLook, lineDefaults, bandOf, named, modeOf, paneMode, stackOf, stacksMade, sameVector, labelOf, fromName, layerById, gradientTable, gradientCss, gradientTables, mixedOf, INK_DARK, GRADIENT_STEPS, derivedTitle, titleOf, seriesMark, profileMark, barsOf, runs, sides, areaOf, areaPoints, legendEntry, rangeOf, upToDatePlaces, levelSteps, fadingOf, faintOf,
};
