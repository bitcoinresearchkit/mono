// @ts-check

import { Line } from "/modules/bitview-charts/line.js";
import { Band } from "/modules/bitview-charts/band.js";
import { Levels } from "/modules/bitview-charts/levels.js";
import { STATES, TRENDS } from "./market.js";
/** @typedef {import("./market.js").Days} Days @typedef {import("/modules/bitview-charts/timeline.js").Timeline} Timeline */

const SANS = "Inter, system-ui, sans-serif";
export const FONTS = { tag: `600 11.5px ${SANS}`, dates: `500 11.5px ${SANS}`, small: `500 11px ${SANS}`, strong: `600 11px ${SANS}` };

/** @typedef {{ states: string[], trends: string[], ink: string, faint: string, long: string, short: string, app: string, dark: boolean }} Paint */
const probe = document.createElement("i");
/** A canvas can't read light-dark(): each colour resolves through an element of that scheme. */
export function paletteOf(/** @type {boolean} */ dark) {
  probe.style.colorScheme = dark ? "dark" : "light";
  document.body.append(probe);
  const resolved = (/** @type {string} */ value) => ((probe.style.color = value), getComputedStyle(probe).color);
  /** @type {Paint} */
  const paint = {
    states: STATES.map((s) => resolved(`var(--s-${s.id})`)),
    trends: TRENDS.map((t) => resolved(`var(--s-${t.color})`)),
    ink: resolved("var(--ink)"), faint: resolved("var(--faint)"),
    long: resolved("var(--cloud-long)"), short: resolved("var(--cloud-short)"), app: resolved("var(--app)"),
    dark,
  };
  probe.remove();
  return paint;
}
/** Held to whole: a canvas silently ignores a colour-mix past 100%. */
export const withAlpha = (/** @type {string} */ color, /** @type {number} */ alpha) => `color-mix(in srgb, ${color} ${Math.min(1, alpha) * 100}%, transparent)`;

/** @typedef {{ state: Set<number> }} Lit */
export const FADED = 0.15, EXTREME = 2;
export const noneLit = () => /** @type {Lit} */ ({ state: new Set() });
export const lighting = (/** @type {Lit} */ lit) => lit.state.size > 0;
export const litDay = (/** @type {Days} */ d, /** @type {Lit} */ lit, /** @type {number} */ day) => !lit.state.size || lit.state.has(d.state[day]);
export const litKey = (/** @type {Lit} */ lit) => [...lit.state].sort().join();

/** @typedef {"mood" | "trend"} ColorBy */
/** @type {WeakMap<Days, Partial<Record<ColorBy, { key: string, colors: Uint8Array }>>>} */
const colorsMade = new WeakMap();
const hueAt = (/** @type {Days} */ d, /** @type {number} */ day, /** @type {ColorBy} */ by) => (by === "trend" ? STATES[d.state[day]].trend : d.state[day]);
export function colorsOf(/** @type {Days} */ d, /** @type {Lit} */ lit, /** @type {ColorBy} */ by) {
  const key = litKey(lit), made = colorsMade.get(d) ?? {}, held = made[by];
  if (held?.key === key) return held.colors;
  const colors = new Uint8Array(d.close.length);
  for (let i = d.first; i <= d.last; i++) colors[i] = hueAt(d, i, by) * 3 + (!litDay(d, lit, i) ? 1 : d.extreme[i] ? 2 : 0);
  colorsMade.set(d, { ...made, [by]: { key, colors } });
  return colors;
}
export const huesOf = (/** @type {Paint} */ paint, /** @type {ColorBy} */ by) => (by === "trend" ? paint.trends : paint.states);
export const colorAt = (/** @type {Paint} */ paint, /** @type {Days} */ d, /** @type {number} */ day, /** @type {ColorBy} */ by) => huesOf(paint, by)[hueAt(d, day, by)];
/** Each colour three times, in colorsOf's index order: lit, faded, extreme. */
export const fadedPalette = (/** @type {Paint} */ paint, /** @type {ColorBy} */ by, /** @type {(color: string, strength: number) => any} */ f) => huesOf(paint, by).flatMap((color) => [f(color, 1), f(color, FADED), f(color, EXTREME)]);

const FILL = { dark: [0.42, 0.04], light: [0.34, 0.03] };
/** @type {WeakMap<Days, { paint: Paint, key: string, hero: (Line | Band)[] }>} */
const heroesMade = new WeakMap();
export function heroOf(/** @type {Days} */ d, /** @type {Paint} */ paint, /** @type {Lit} */ lit, /** @type {ColorBy} */ by) {
  const key = `${by}|${litKey(lit)}`, made = heroesMade.get(d);
  if (made?.paint === paint && made.key === key) return made.hero;
  const colors = colorsOf(d, lit, by), [near, far] = paint.dark ? FILL.dark : FILL.light;
  const fill = new Band({
    upper: d.close, to: "foot", colors, curve: true,
    palette: fadedPalette(paint, by, (color, k) => ({ color: withAlpha(color, near * k), fade: withAlpha(color, far * k) })),
  });
  const price = new Line({ values: d.close, colors, palette: fadedPalette(paint, by, (color, k) => (k < 1 ? paint.ink : color)), width: 2, curve: true, marker: 4.5 });
  // The clouds go under the moods' wash: over it, their grey takes its colour just under the line.
  const hero = [new Cloud(d.cloud.long, paint.long), new Cloud(d.cloud.short, paint.short), fill, price];
  heroesMade.set(d, { paint, key, hero });
  return hero;
}
const QUIET = 0.6;
const quietAt = (/** @type {number} */ days) => 1 - (1 - QUIET) * Math.min(1, Math.max(0, Math.log(days / 365) / Math.log(8)));
/** Recoloured only as the strength crosses a step, so a pan moves its picture as it is. */
class Cloud extends Band {
  #color;
  #strength = -1;
  constructor(/** @type {Float64Array[]} */ [lower, upper], /** @type {string} */ color) {
    super({ lower, upper, color });
    this.#color = color;
  }
  /** @param {Timeline} time */
  viewed(time) {
    const strength = Math.round(quietAt(time.to - time.from) * 50) / 50;
    if (strength !== this.#strength) (this.#strength = strength), this.set({ color: withAlpha(this.#color, strength) });
  }
}

export const INDEX_TICKS = [0, 25, 50, 75, 100];
/** @type {WeakMap<Days, { paint: Paint, key: string, line: Line, ticks: Levels }>} */
const indexMade = new WeakMap();
export function indexOf(/** @type {Days} */ d, /** @type {Paint} */ paint, /** @type {Lit} */ lit) {
  const key = litKey(lit), made = indexMade.get(d);
  if (made?.paint === paint && made.key === key) return made;
  const line = new Line({ values: d.index, colors: colorsOf(d, lit, "trend"), palette: fadedPalette(paint, "trend", (color, k) => (k < 1 ? withAlpha(paint.ink, 0.22) : color)), width: 1.75, marker: 4.5 });
  const ticks = new Levels({ lines: INDEX_TICKS.map((value) => ({ value, color: withAlpha(paint.ink, value === 50 ? 0.16 : 0.08) })) });
  const all = { paint, key, line, ticks };
  indexMade.set(d, all);
  return all;
}

const CELL = 200, GAP = 6;
const REACH = 110;
const SIDES = [
  { cloud: "long", title: "long-term cloud", of: "cycle" },
  { cloud: "short", title: "short-term cloud", of: "local" },
];
export const SIDE_WORDS = { 1: "above", 0: "inside", [-1]: "below" };
const COST = { 1: "in profit", 0: "about even", [-1]: "at a loss" };
export const coinsWords = (/** @type {{ cycle: number, local: number }} */ s) =>
  s.cycle === s.local ? `old and new holders ${COST[s.cycle]}` : `old holders ${COST[s.cycle]}, new ${COST[s.local]}`;
const at = (/** @type {number} */ v) => 300 + v * 0.97 * 300;
export const compassPoint = (/** @type {Days} */ d, /** @type {number} */ i) => [at(d.x[i]), at(-d.y[i])];
/** Pieces are calendar weeks, so a piece's days stay its own as the day moves. */
const TRAIL = 365, WEEK = 7, PIECES = Math.ceil(TRAIL / WEEK) + 1, TAIL = 7;

/** `dot`: false before the days are read (no dot or ring). */
export function buildCompass(/** @type {SVGSVGElement} */ svg, /** @type {boolean} */ dot) {
  const cells = [], words = [];
  for (const [i, s] of STATES.entries()) {
    const col = 1 - s.local, row = 1 - s.cycle, cx = (2 - col) * CELL, cy = row * CELL;
    cells.push(`<g class="cell" data-i="${i}" data-light="${i}" role="button" tabindex="0" aria-label="${s.name}: light its days" aria-pressed="false" style="--c: var(--s-${s.id})"><rect x="${cx + GAP / 2}" y="${cy + GAP / 2}" width="${CELL - GAP}" height="${CELL - GAP}" rx="14" /></g>`);
    words.push(`<g class="cell" data-i="${i}" data-light="${i}" style="--c: var(--s-${s.id})"><text class="name" x="${cx + 18}" y="${cy + 46}">${s.name}</text><g class="icon" fill="none" stroke="var(--s-${TRENDS[s.trend].color})" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path data-x="${cx + 18}" data-y="${cy + 24}" d="${TRENDS[s.trend].arrow}" /></g><text class="share" x="${cx + 18}" y="${cy + 78}">–</text><text class="move" x="${cx + 18}" y="${cy + 180}">${s.move}</text></g>`);
  }
  const sides = SIDES.map(({ cloud, title, of }) => {
    const west = of === "cycle", along = (/** @type {number} */ v) => (west ? (1 - v) * CELL : (1 + v) * CELL) + CELL / 2;
    const placed = (/** @type {number} */ out, /** @type {number} */ v) => (west ? `transform="translate(${-out} ${v}) rotate(-90)"` : `x="${v}" y="${-out}"`);
    const parts = /** @type {(1 | 0 | -1)[]} */ (west ? [1, 0, -1] : [-1, 0, 1]).map((v) => {
      const key = `${STATES.flatMap((s, i) => (s[/** @type {"cycle" | "local"} */ (of)] === v ? [i] : []))}`, c = along(v), words = `${SIDE_WORDS[v]} the ${title}`;
      const [a, b] = [c - CELL / 2 + GAP / 2 + 14, c + CELL / 2 - GAP / 2 - 14];
      // Reaches past the margin, unseen, toward a finger's 44px.
      const hit = west ? `x="-${REACH}" y="${a - 14}" width="${REACH - 2}" height="${b - a + 28}"` : `x="${a - 14}" y="-${REACH}" width="${b - a + 28}" height="${REACH - 2}"`;
      const bar = west ? `x1="-6" y1="${a}" x2="-6" y2="${b}"` : `x1="${a}" y1="-6" x2="${b}" y2="-6"`;
      const focus = west ? `x="-46" y="${a - 10}" width="46" height="${b - a + 20}"` : `x="${a - 10}" y="-46" width="${b - a + 20}" height="46"`;
      return `<g class="side" data-light="${key}" role="button" tabindex="0" aria-label="${words[0].toUpperCase()}${words.slice(1)}: light its days" aria-pressed="false"><rect class="hit" ${hit} /><rect class="focus" ${focus} rx="8" /><line class="bar" ${bar} /><text class="axis" ${placed(20, c)} text-anchor="middle">${SIDE_WORDS[v]} · <tspan class="share">–</tspan></text></g>`;
    });
    const named = west ? `transform="translate(-60 ${CELL * 1.5}) rotate(-90)"` : `transform="translate(${CELL * 1.5} -60)"`;
    return `${parts.join("")}<g class="title" data-cloud="${cloud}" ${named}><text class="axis" text-anchor="middle">${title[0].toUpperCase()}${title.slice(1)}</text></g>`;
  });
  // Trail under the cells' words, its tail and the dot over them, so a trail never ends lost under a word.
  // The dot last, over every line, a side's bar too.
  svg.innerHTML = `${cells.join("")}<g class="trail" id="trail">${"<path />".repeat(PIECES)}</g>${words.join("")}${sides.join("")}${dot ? '<linearGradient id="head" gradientUnits="userSpaceOnUse"><stop offset="0" /><stop offset="1" /></linearGradient><path class="tail" id="tail" /><circle class="ring" id="ring" r="13" /><circle class="dot" id="dot" r="8" />' : ""}`;
  fitCompass(svg);
}
/** Call again on resize and font load: each arrow follows its name's drawn width. */
export function fitCompass(/** @type {SVGSVGElement} */ svg) {
  const drawn = svg.getBoundingClientRect().width;
  const px = svg.viewBox.baseVal.width / drawn;
  if (drawn) svg.style.setProperty("--px", `${px.toFixed(3)}px`), svg.style.setProperty("--axis", `${(12 * px).toFixed(2)}px`);
  for (const icon of svg.querySelectorAll("g.icon")) {
    const name = /** @type {SVGTextElement} */ (icon.previousElementSibling), arrow = /** @type {SVGPathElement} */ (icon.firstElementChild);
    arrow.setAttribute("transform", `translate(${(Number(arrow.dataset.x) + name.getComputedTextLength() + 10).toFixed(1)} ${arrow.dataset.y}) scale(1.5)`);
  }
}

export function compassParts(/** @type {SVGSVGElement} */ svg) {
  return {
    dot: /** @type {SVGCircleElement} */ (svg.querySelector("#dot")),
    ring: /** @type {SVGCircleElement} */ (svg.querySelector("#ring")),
    trail: /** @type {SVGGElement} */ (svg.querySelector("#trail")),
    tail: /** @type {SVGPathElement} */ (svg.querySelector("#tail")),
    head: /** @type {SVGLinearGradientElement} */ (svg.querySelector("#head")),
    cells: [...svg.querySelectorAll(".cell[data-i]")].map((e) => /** @type {[SVGGElement, number]} */ ([/** @type {SVGGElement} */ (e), Number(/** @type {SVGGElement} */ (e).dataset.i)])),
  };
}
/**
 * Written only when changed: writing the same value still restyles the page.
 * @type {WeakMap<Element, { days: Days, drawn: string[], weeks: number[], newest: number }>}
 */
const trails = new WeakMap();
function drawTrail(/** @type {SVGGElement} */ trail, /** @type {SVGPathElement} */ tail, /** @type {SVGLinearGradientElement} */ head, /** @type {Days} */ d, /** @type {number} */ day) {
  const pieces = /** @type {HTMLCollectionOf<SVGPathElement>} */ (trail.children), from = Math.max(d.first, day - TRAIL) + 1, newest = Math.floor(day / WEEK);
  let held = trails.get(trail);
  if (!held) trails.set(trail, (held = { days: d, drawn: Array(PIECES).fill(""), weeks: Array(PIECES).fill(NaN), newest: NaN }));
  else if (held.days !== d) (held.days = d), held.drawn.fill("");
  // From the day before its first, so the pieces meet.
  const through = (/** @type {number} */ a, /** @type {number} */ b) => {
    const points = [];
    for (let i = a - 1; i <= b; i++) points.push(compassPoint(d, i).map((v) => v.toFixed(1)).join(" "));
    return `M${points.join("L")}`;
  };
  for (let j = 0; j < PIECES; j++) {
    const week = newest - ((((newest - j) % PIECES) + PIECES) % PIECES), a = Math.max(week * WEEK, from), b = Math.min(week * WEEK + WEEK - 1, day - TAIL), piece = pieces[j];
    const days = a <= b ? `${a}-${b}` : "";
    if (held.drawn[j] !== days) (held.drawn[j] = days) ? piece.setAttribute("d", through(a, b)) : piece.removeAttribute("d");
    if (held.weeks[j] !== week) piece.style.setProperty("--week", String((held.weeks[j] = week)));
  }
  if (held.newest !== newest) trail.style.setProperty("--newest", String((held.newest = newest)));
  const first = Math.max(from, day - TAIL + 1);
  day >= from ? tail.setAttribute("d", through(first, day)) : tail.removeAttribute("d");
  // Ink rises from the body's newest (--top) to whole at the dot: no step where they meet.
  const [[x1, y1], [x2, y2]] = [compassPoint(d, first - 1), compassPoint(d, day)];
  for (const [k, v] of /** @type {const} */ ([["x1", x1], ["y1", y1], ["x2", x2], ["y2", y2]])) head.setAttribute(k, v.toFixed(1));
}
export function drawCompassDay(/** @type {ReturnType<typeof compassParts>} */ parts, /** @type {Days} */ d, /** @type {number} */ day) {
  const { dot, ring, trail, tail, head, cells } = parts, s = STATES[d.state[day]];
  const [x, y] = compassPoint(d, day), cx = x.toFixed(1), cy = y.toFixed(1);
  drawTrail(trail, tail, head, d, day);
  dot.setAttribute("cx", cx);
  dot.setAttribute("cy", cy);
  dot.style.fill = `var(--s-${s.id})`;
  ring.setAttribute("cx", cx);
  ring.setAttribute("cy", cy);
  for (const [cell, i] of cells) cell.classList.toggle("on", d.state[day] === i);
}
