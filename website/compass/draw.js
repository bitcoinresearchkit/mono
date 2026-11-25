// @ts-check
// Compass's pictures, drawn from the days market.js computes: the price chart's series (its clouds, moods and
// price), the index's lines, and the compass with its year's trail. Each a function of what it's
// drawn on, its view, the days, the colours and what's lit, nothing of a page's own: the Compass page draws with
// these, and so can any page that shows its pictures (draw.css beside it for the colours and the compass's look).

import { Line } from "/modules/bitview-charts/line.js";
import { Band } from "/modules/bitview-charts/band.js";
import { Levels } from "/modules/bitview-charts/levels.js";
import { STATES, TRENDS } from "./market.js";
/** @typedef {import("./market.js").Days} Days @typedef {import("/modules/bitview-charts/timeline.js").Timeline} Timeline */

/** The canvases' words, in the page's sans: a tag's, the dates', and the small ones (a scale's, a legend's), plain and
 * strong. */
const SANS = "Inter, system-ui, sans-serif";
export const FONTS = { tag: `600 11.5px ${SANS}`, dates: `500 11.5px ${SANS}`, small: `500 11px ${SANS}`, strong: `600 11px ${SANS}` };

// ── Colours, as a scheme resolves them (a canvas can't read light-dark()) ──

/** @typedef {{ states: string[], trends: string[], ink: string, faint: string, long: string, short: string, app: string, dark: boolean }} Paint */
const probe = document.createElement("i");
/** The colours of draw.css for the dark scheme or the light, whichever the screen's set to (its contrast the
 * screen's): each as the browser resolves it, through an element of that scheme. */
export function paletteOf(/** @type {boolean} */ dark) {
  probe.style.colorScheme = dark ? "dark" : "light";
  document.body.append(probe);
  const resolved = (/** @type {string} */ value) => ((probe.style.color = value), getComputedStyle(probe).color);
  /** @type {Paint} */
  const paint = {
    states: STATES.map((s) => resolved(`var(--s-${s.id})`)),
    trends: TRENDS.map((t) => resolved(`var(--s-${t.color})`)),
    // (Cumulative curves shift lightness while retaining each trend's hue and chroma.)
    ink: resolved("var(--ink)"), faint: resolved("var(--faint)"),
    long: resolved("var(--cloud-long)"), short: resolved("var(--cloud-short)"), app: resolved("var(--app)"),
    dark,
  };
  probe.remove();
  return paint;
}
/** A colour at an opacity, as a canvas reads it (whatever form the browser resolved it to): held to whole, since a
 * canvas silently ignores a colour-mix past 100%. */
export const withAlpha = (/** @type {string} */ color, /** @type {number} */ alpha) => `color-mix(in srgb, ${color} ${Math.min(1, alpha) * 100}%, transparent)`;

// ── What's lit: moods, as many as wanted; a day's lit if it's in any mood lit (or none is) ──

/** The moods lit, by index (empty: none). @typedef {{ state: Set<number> }} Lit */
/** How strongly a day not lit is drawn, against its own colour's usual strength; and an extreme day lit (see
 * market.js extremeOf). */
export const FADED = 0.15, EXTREME = 2;
/** Nothing lit: a new, empty Lit (a page's own to light things in, or a picture with every day drawn whole). */
export const noneLit = () => /** @type {Lit} */ ({ state: new Set() });
export const lighting = (/** @type {Lit} */ lit) => lit.state.size > 0;
/** Whether a day's lit: every day while nothing is. */
export const litDay = (/** @type {Days} */ d, /** @type {Lit} */ lit, /** @type {number} */ day) => !lit.state.size || lit.state.has(d.state[day]);
/** What's lit, for the drawings kept (made again when it changes). */
export const litKey = (/** @type {Lit} */ lit) => [...lit.state].sort().join();

// ── The moods by day ──

/** What a drawing's colours follow: each day's mood (nine colours) or its mood's trend (three). @typedef {"mood" | "trend"} ColorBy */
/** Each days' colours as last made, by what they follow, and what was lit then. @type {WeakMap<Days, Partial<Record<ColorBy, { key: string, colors: Uint8Array }>>>} */
const colorsMade = new WeakMap();
/** A day's colour's place among huesOf's: its mood's or its trend's. */
const hueAt = (/** @type {Days} */ d, /** @type {number} */ day, /** @type {ColorBy} */ by) => (by === "trend" ? STATES[d.state[day]].trend : d.state[day]);
/** Each day's colour (its mood's or its trend's) and how strongly it's drawn, as an index into the drawings' palettes
 * (a colour's three times: lit, not lit, lit and extreme; see fadedPalette), made again as the days or what's lit
 * change. */
export function colorsOf(/** @type {Days} */ d, /** @type {Lit} */ lit, /** @type {ColorBy} */ by) {
  const key = litKey(lit), made = colorsMade.get(d) ?? {}, held = made[by];
  if (held?.key === key) return held.colors;
  const colors = new Uint8Array(d.close.length);
  for (let i = d.first; i <= d.last; i++) colors[i] = hueAt(d, i, by) * 3 + (!litDay(d, lit, i) ? 1 : d.extreme[i] ? 2 : 0);
  colorsMade.set(d, { ...made, [by]: { key, colors } });
  return colors;
}
/** The colours colorsOf indexes, in their order: the moods' or the trends'. */
export const huesOf = (/** @type {Paint} */ paint, /** @type {ColorBy} */ by) => (by === "trend" ? paint.trends : paint.states);
/** A day's colour: its mood's or its trend's. */
export const colorAt = (/** @type {Paint} */ paint, /** @type {Days} */ d, /** @type {number} */ day, /** @type {ColorBy} */ by) => huesOf(paint, by)[hueAt(d, day, by)];
/** Those colours as colorsOf indexes them, each three times: lit, faded, extreme (`f` given a colour and a strength). */
export const fadedPalette = (/** @type {Paint} */ paint, /** @type {ColorBy} */ by, /** @type {(color: string, strength: number) => any} */ f) => huesOf(paint, by).flatMap((color) => [f(color, 1), f(color, FADED), f(color, EXTREME)]);

// ── The price chart ──

/** The moods under the price, strongest at the pane's top and fading to its foot: on a dark screen, and on a light one. */
const FILL = { dark: [0.42, 0.04], light: [0.34, 0.03] };
/** Each days' chart series as last made, for its colours, what they follow and what was lit. @type {WeakMap<Days, { paint: Paint, key: string, hero: (Line | Band)[] }>} */
const heroesMade = new WeakMap();
/**
 * The price chart's series for the days, the colours, what they follow and what's lit (made again when any changes),
 * drawn in this order on a log scale fitted to them: the two clouds (see Cloud); the days' colours under the price (an
 * extreme day's stronger); the price over all of it, in its days' colours (a day not lit, in ink).
 */
export function heroOf(/** @type {Days} */ d, /** @type {Paint} */ paint, /** @type {Lit} */ lit, /** @type {ColorBy} */ by) {
  const key = `${by}|${litKey(lit)}`, made = heroesMade.get(d);
  if (made?.paint === paint && made.key === key) return made.hero;
  const colors = colorsOf(d, lit, by), [near, far] = paint.dark ? FILL.dark : FILL.light;
  const fill = new Band({
    upper: d.close, to: "foot", colors, curve: true,
    palette: fadedPalette(paint, by, (color, k) => ({ color: withAlpha(color, near * k), fade: withAlpha(color, far * k) })),
  });
  const price = new Line({ values: d.close, colors, palette: fadedPalette(paint, by, (color, k) => (k < 1 ? paint.ink : color)), width: 2, curve: true, marker: 4.5 });
  // (The clouds under the moods' wash: drawn over it, their grey took its colour just under the line, where it reads.)
  const hero = [new Cloud(d.cloud.long, paint.long), new Cloud(d.cloud.short, paint.short), fill, price];
  heroesMade.set(d, { paint, key, hero });
  return hero;
}
/** How strongly a cloud is drawn over a view of `days`: whole up to a year, fading with the span's doublings to QUIET
 * at eight years and past, so a long view reads as the price and its moods. */
const QUIET = 0.6;
const quietAt = (/** @type {number} */ days) => 1 - (1 - QUIET) * Math.min(1, Math.max(0, Math.log(days / 365) / Math.log(8)));
/** A cloud between its two edges, as strong as its view's span makes it (see quietAt): its colour set again only as
 * that crosses a fiftieth (a zoom, which draws it whole anyway), so a pan moves its picture as it is. */
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

// ── The index's lines ──

/** Where its scale's numbers sit: a faint line at each, the middle a little stronger. */
export const INDEX_TICKS = [0, 25, 50, 75, 100];
/** The index chart's drawings as last made, by its days: made again only as the colours or what's lit change. @type {WeakMap<Days, { paint: Paint, key: string, line: Line, ticks: Levels }>} */
const indexMade = new WeakMap();
/**
 * The index chart's drawings, 0 to 100 on its scale: its line, in its days' trends' colours, each day not lit one faint
 * ink, so the lit days alone have colour; a faint line at each quarter (drawn by the page over the veil on the days
 * after the day told, so a new day draws the line none again and the quarters run whole).
 */
export function indexOf(/** @type {Days} */ d, /** @type {Paint} */ paint, /** @type {Lit} */ lit) {
  const key = litKey(lit), made = indexMade.get(d);
  if (made?.paint === paint && made.key === key) return made;
  const line = new Line({ values: d.index, colors: colorsOf(d, lit, "trend"), palette: fadedPalette(paint, "trend", (color, k) => (k < 1 ? withAlpha(paint.ink, 0.22) : color)), width: 1.75, marker: 4.5 });
  const ticks = new Levels({ lines: INDEX_TICKS.map((value) => ({ value, color: withAlpha(paint.ink, value === 50 ? 0.16 : 0.08) })) });
  const all = { paint, key, line, ticks };
  indexMade.set(d, all);
  return all;
}

// ── The compass: nine cells, north the cycle's best, east the moment's, each mood's trend after its name; the day a
// dot in its cell, the year before it a trail into it ──

/** A compass cell's size, the gap between cells. */
const CELL = 200, GAP = 6;
/** How far out a side's tap reaches from the cells: past the drawing's 76-unit margin, unseen, into the page's own. */
const REACH = 110;
/** Its sides: each cloud's three parts, the long-term one's rows down the west side, the short-term one's columns
 * along the top; each part's share the sum of its row's or column's. */
const SIDES = [
  { cloud: "long", title: "long-term cloud", of: "cycle" },
  { cloud: "short", title: "short-term cloud", of: "local" },
];
/** Where price is against a cloud, in words, by its side (1 above, 0 inside, -1 below). */
export const SIDE_WORDS = { 1: "above", 0: "inside", [-1]: "below" };
/** Where the price is against a cloud, said of its coins' holders: a cloud is their cost basis (the basis chosen at the
 * page's foot: percentiles, medians or averages, by dollar), so above it they're in profit, inside it about even,
 * below it at a loss, whichever basis (profit and loss the how's words too). */
const COST = { 1: "in profit", 0: "about even", [-1]: "at a loss" };
/** Where a mood's price is against the two clouds, said of their holders (the long-term cloud's coins held over 4 to 6
 * months; the short-term's, new): "old and new holders at a loss", "old holders in profit, new about even". */
export const coinsWords = (/** @type {{ cycle: number, local: number }} */ s) =>
  s.cycle === s.local ? `old and new holders ${COST[s.cycle]}` : `old holders ${COST[s.cycle]}, new ${COST[s.local]}`;
/** A compass point (-1 to 1 each way) on the svg, which is 600 across inside its margin. */
const at = (/** @type {number} */ v) => 300 + v * 0.97 * 300;
/** A day's point on the compass's drawing (its svg's units): across by its moment, up by its cycle. The dot's and
 * the trail's, and a page's own. */
export const compassPoint = (/** @type {Days} */ d, /** @type {number} */ i) => [at(d.x[i]), at(-d.y[i])];
/** The days the trail runs back from the day; its pieces, a calendar week of days since the first each, so a piece's
 * days stay its own as the day moves; its tail, the last days into the dot, over the words. */
const TRAIL = 365, WEEK = 7, PIECES = Math.ceil(TRAIL / WEEK) + 1, TAIL = 7;

/**
 * The compass built into an svg (its viewBox "-76 -76 676 692"): its cells, each a button lighting its mood's days
 * (data-light), each name followed by its trend's arrow in the trend's colour, their shares dashes till they're written; the trail; the day's dot and its ring (`dot`:
 * before the days are read, none); each cloud's sides along it. Fitted to its text (see fitCompass).
 */
export function buildCompass(/** @type {SVGSVGElement} */ svg, /** @type {boolean} */ dot) {
  const cells = [], words = [];
  for (const [i, s] of STATES.entries()) {
    const col = 1 - s.local, row = 1 - s.cycle, cx = (2 - col) * CELL, cy = row * CELL;
    cells.push(`<g class="cell" data-i="${i}" data-light="${i}" role="button" tabindex="0" aria-label="${s.name}: light its days" aria-pressed="false" style="--c: var(--s-${s.id})"><rect x="${cx + GAP / 2}" y="${cy + GAP / 2}" width="${CELL - GAP}" height="${CELL - GAP}" rx="14" /></g>`);
    words.push(`<g class="cell" data-i="${i}" data-light="${i}" style="--c: var(--s-${s.id})"><text class="name" x="${cx + 18}" y="${cy + 46}">${s.name}</text><g class="icon" fill="none" stroke="var(--s-${TRENDS[s.trend].color})" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path data-x="${cx + 18}" data-y="${cy + 24}" d="${TRENDS[s.trend].arrow}" /></g><text class="share" x="${cx + 18}" y="${cy + 78}">–</text><text class="move" x="${cx + 18}" y="${cy + 180}">${s.move}</text></g>`);
  }
  // (Each side: its cloud named on the outside, its three parts inside, each over its row or column, pressed
  // anywhere in that stretch of the margin, the cloud's name too: a press on it lights the days inside its cloud.
  // Rows read upward, west of the compass; columns along its top.)
  const sides = SIDES.map(({ cloud, title, of }) => {
    const west = of === "cycle", along = (/** @type {number} */ v) => (west ? (1 - v) * CELL : (1 + v) * CELL) + CELL / 2;
    const placed = (/** @type {number} */ out, /** @type {number} */ v) => (west ? `transform="translate(${-out} ${v}) rotate(-90)"` : `x="${v}" y="${-out}"`);
    // (In reading order: the west side top to bottom, the top left to right.)
    const parts = /** @type {(1 | 0 | -1)[]} */ (west ? [1, 0, -1] : [-1, 0, 1]).map((v) => {
      const key = `${STATES.flatMap((s, i) => (s[/** @type {"cycle" | "local"} */ (of)] === v ? [i] : []))}`, c = along(v), words = `${SIDE_WORDS[v]} the ${title}`;
      const [a, b] = [c - CELL / 2 + GAP / 2 + 14, c + CELL / 2 - GAP / 2 - 14];
      // (Its reach: the margin's depth and on past it, unseen, toward a finger's 44px.)
      const hit = west ? `x="-${REACH}" y="${a - 14}" width="${REACH - 2}" height="${b - a + 28}"` : `x="${a - 14}" y="-${REACH}" width="${b - a + 28}" height="${REACH - 2}"`;
      const bar = west ? `x1="-6" y1="${a}" x2="-6" y2="${b}"` : `x1="${a}" y1="-6" x2="${b}" y2="-6"`;
      const focus = west ? `x="-46" y="${a - 10}" width="46" height="${b - a + 20}"` : `x="${a - 10}" y="-46" width="${b - a + 20}" height="46"`;
      return `<g class="side" data-light="${key}" role="button" tabindex="0" aria-label="${words[0].toUpperCase()}${words.slice(1)}: light its days" aria-pressed="false"><rect class="hit" ${hit} /><rect class="focus" ${focus} rx="8" /><line class="bar" ${bar} /><text class="axis" ${placed(20, c)} text-anchor="middle">${SIDE_WORDS[v]} · <tspan class="share">–</tspan></text></g>`;
    });
    const named = west ? `transform="translate(-60 ${CELL * 1.5}) rotate(-90)"` : `transform="translate(${CELL * 1.5} -60)"`;
    return `${parts.join("")}<g class="title" data-cloud="${cloud}" ${named}><text class="axis" text-anchor="middle">${title[0].toUpperCase()}${title.slice(1)}</text></g>`;
  });
  // (The trail under the cells' words, each glyph haloed in its cell's colour: see draw.css; its tail into the dot
  // over them with the dot, so the dot never ends a trail lost under a word. The dot last, over every line: a side's
  // bar too, which its ring reaches at the compass's edge.)
  svg.innerHTML = `${cells.join("")}<g class="trail" id="trail">${"<path />".repeat(PIECES)}</g>${words.join("")}${sides.join("")}${dot ? '<linearGradient id="head" gradientUnits="userSpaceOnUse"><stop offset="0" /><stop offset="1" /></linearGradient><path class="tail" id="tail" /><circle class="ring" id="ring" r="13" /><circle class="dot" id="dot" r="8" />' : ""}`;
  fitCompass(svg);
}
/** What follows the drawing's width and its text's size as its font draws it: the page's px and the axis words at
 * its 12px (in the drawing's units, so at any width), each mood's trend's arrow after its name. (Again as it's resized, or
 * its fonts load.) */
export function fitCompass(/** @type {SVGSVGElement} */ svg) {
  const drawn = svg.getBoundingClientRect().width;
  const px = svg.viewBox.baseVal.width / drawn;
  if (drawn) svg.style.setProperty("--px", `${px.toFixed(3)}px`), svg.style.setProperty("--axis", `${(12 * px).toFixed(2)}px`);
  for (const icon of svg.querySelectorAll("g.icon")) {
    const name = /** @type {SVGTextElement} */ (icon.previousElementSibling), arrow = /** @type {SVGPathElement} */ (icon.firstElementChild);
    arrow.setAttribute("transform", `translate(${(Number(arrow.dataset.x) + name.getComputedTextLength() + 10).toFixed(1)} ${arrow.dataset.y}) scale(1.5)`);
  }
}

/** The compass's parts a day changes, found once it's built with its dot: the dot, its ring, the trail and its tail, and each
 * cell (its rect's) with the mood it's of. */
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
/** What each trail piece was last drawn for: the days it holds, of which days (a new day redraws only the piece the tail
 * left and the oldest, so a day told often, as Play does, builds few paths); and the week it holds and the newest,
 * which give its strength and width (draw.css), written only when they change (a write of the same still styles the
 * page again).
 * @type {WeakMap<Element, { days: Days, drawn: string[], weeks: number[], newest: number }>}
 */
const trails = new WeakMap();
/** The trail into the day's dot: every day of the year before where it was, never smoothed or curved (the past as it
 * was), fainter and thinner with age by its week (see draw.css). */
function drawTrail(/** @type {SVGGElement} */ trail, /** @type {SVGPathElement} */ tail, /** @type {SVGLinearGradientElement} */ head, /** @type {Days} */ d, /** @type {number} */ day) {
  const pieces = /** @type {HTMLCollectionOf<SVGPathElement>} */ (trail.children), from = Math.max(d.first, day - TRAIL) + 1, newest = Math.floor(day / WEEK);
  let held = trails.get(trail);
  if (!held) trails.set(trail, (held = { days: d, drawn: Array(PIECES).fill(""), weeks: Array(PIECES).fill(NaN), newest: NaN }));
  else if (held.days !== d) (held.days = d), held.drawn.fill("");
  // (Days a - 1 to b, straight from each day's place to the next: from the day before its first, so the pieces meet.)
  const through = (/** @type {number} */ a, /** @type {number} */ b) => {
    const points = [];
    for (let i = a - 1; i <= b; i++) points.push(compassPoint(d, i).map((v) => v.toFixed(1)).join(" "));
    return `M${points.join("L")}`;
  };
  for (let j = 0; j < PIECES; j++) {
    // (Its week's days up to the tail's.)
    const week = newest - ((((newest - j) % PIECES) + PIECES) % PIECES), a = Math.max(week * WEEK, from), b = Math.min(week * WEEK + WEEK - 1, day - TAIL), piece = pieces[j];
    const days = a <= b ? `${a}-${b}` : "";
    if (held.drawn[j] !== days) (held.drawn[j] = days) ? piece.setAttribute("d", through(a, b)) : piece.removeAttribute("d");
    if (held.weeks[j] !== week) piece.style.setProperty("--week", String((held.weeks[j] = week)));
  }
  if (held.newest !== newest) trail.style.setProperty("--newest", String((held.newest = newest)));
  const first = Math.max(from, day - TAIL + 1);
  day >= from ? tail.setAttribute("d", through(first, day)) : tail.removeAttribute("d");
  // (Its ink rising from the body's newest (--top) where it starts to whole at the dot, along its chord: no step where
  // they meet.)
  const [[x1, y1], [x2, y2]] = [compassPoint(d, first - 1), compassPoint(d, day)];
  for (const [k, v] of /** @type {const} */ ([["x1", x1], ["y1", y1], ["x2", x2], ["y2", y2]])) head.setAttribute(k, v.toFixed(1));
}
/** The compass on a day: its dot where the day was, the trail of its year before into it, the day's mood's cells
 * stronger. */
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
