// @ts-check
// Checks every icon against the style table in icons.js, and the icons drawn inline in the other pages:
// stroke, pixel grid, corner radii, angles, padding, gaps between parts, ink weight.
// Parts that touch count as one: a part that meets another is not measured against it again, so a near miss further along it
// passes. (A part leaving another sideways is the house pattern; telling that from a parallel run is the sheet's job, by eye.)
// Run: bun website/studio/icons-check.mjs [name…]   Exits 1 when any icon fails.
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { ICONS, ICON_EXEMPT, KEY_GLYPHS, STYLE } from "./icons.js";

/** @typedef {[number, number]} Pt */
/** @typedef {{ t: "L" | "A" | "C", a: Pt, b: Pt, r?: number, c?: Pt, sweep?: number, k?: [Pt, Pt] }} Seg */
/** @typedef {{ pts: Pt[], segs: Seg[], closed: boolean, w: number, fill: boolean, alpha: number, circle?: { c: Pt, rx: number, ry: number }, rect?: { rx: number } }} Part */

const EPS = 0.01;
const { stroke: STROKE, pad: PAD, snap: SNAP, radii: RADII, angles: ANGLES, minGap: GAP } = STYLE;
const rad = (/** @type {number} */ deg) => (deg * Math.PI) / 180;
const near = (/** @type {number} */ a, /** @type {number} */ b, eps = EPS) => Math.abs(a - b) < eps;
/** Whether v is a whole number of steps from offset. */
const onGrid = (/** @type {number} */ v, /** @type {number} */ step, offset = 0) => { const r = Math.abs(((v - offset) / step) % 1); return r < EPS / step || 1 - r < EPS / step; };

// ---- parsing: every element becomes a part (a polyline of its centreline, plus its segments) ----

/** @param {string} d @returns {{ segs: Seg[], closed: boolean }[]} */
const parsePath = (d) => {
  const tokens = d.match(/[a-zA-Z]|[+-]?(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?/g) ?? [];
  const out = /** @type {{ segs: Seg[], closed: boolean }[]} */ ([]);
  let sub = null, x = 0, y = 0, sx = 0, sy = 0, cmd = "", i = 0, ctl = /** @type {Pt | null} */ (null);
  const num = () => {
    const t = tokens[i++];
    if (t === undefined || /[a-zA-Z]/.test(t)) throw new Error(`path "${d}": missing number`);
    return Number(t);
  };
  // Arc flags may be written glued to the next number ("0 011 1"): split them off.
  const flag = () => {
    const t = tokens[i];
    if (t.length > 1 && /^[01]/.test(t) && !t.startsWith("0.") ) { tokens[i] = t.slice(1); return Number(t[0]); }
    return num();
  };
  while (i < tokens.length) {
    if (/[a-zA-Z]/.test(tokens[i])) cmd = tokens[i++];
    const up = cmd.toUpperCase(), rel = cmd !== up;
    if (up === "Z") { if (sub) { if (Math.hypot(x - sx, y - sy) > 1e-9) sub.segs.push({ t: "L", a: [x, y], b: [sx, sy] }); sub.closed = true; } [x, y] = [sx, sy]; continue; }
    const ox = rel ? x : 0, oy = rel ? y : 0;
    if (up === "M") { x = num() + ox; y = num() + oy; [sx, sy] = [x, y]; sub = { segs: [], closed: false }; out.push(sub); cmd = rel ? "l" : "L"; continue; }
    if (!sub) throw new Error(`path "${d}": starts without M`);
    if (up !== "S") ctl = null;
    if (up === "L") { const nx = num() + ox, ny = num() + oy; sub.segs.push({ t: "L", a: [x, y], b: [nx, ny] }); [x, y] = [nx, ny]; }
    else if (up === "H") { const nx = num() + ox; sub.segs.push({ t: "L", a: [x, y], b: [nx, y] }); x = nx; }
    else if (up === "V") { const ny = num() + oy; sub.segs.push({ t: "L", a: [x, y], b: [x, ny] }); y = ny; }
    else if (up === "C" || up === "S") {
      // A curve: only its end points are measured, its handles are free.
      const k1 = /** @type {Pt} */ (up === "C" ? [num() + ox, num() + oy] : ctl ? [2 * x - ctl[0], 2 * y - ctl[1]] : [x, y]), k2 = /** @type {Pt} */ ([num() + ox, num() + oy]), nx = num() + ox, ny = num() + oy;
      sub.segs.push({ t: "C", a: [x, y], b: [nx, ny], k: [k1, k2] });
      [x, y] = [nx, ny]; ctl = k2; continue;
    } else if (up === "A") {
      const r = num(), ry = num(); num(); const large = flag(), sweep = flag(), nx = num() + ox, ny = num() + oy;
      if (!near(r, ry)) throw new Error(`path "${d}": elliptical arc`);
      sub.segs.push(arc([x, y], [nx, ny], r, large, sweep));
      [x, y] = [nx, ny];
    } else throw new Error(`path "${d}": ${cmd} is not drawn in icons (lines and circular arcs only)`);
  }
  return out;
};

/** Circular arc from a to b: its centre, its true radius and its signed sweep in radians. @returns {Seg} */
const arc = (/** @type {Pt} */ a, /** @type {Pt} */ b, /** @type {number} */ r, /** @type {number} */ large, /** @type {number} */ sweep) => {
  const [hx, hy] = [(a[0] - b[0]) / 2, (a[1] - b[1]) / 2], d2 = hx * hx + hy * hy;
  const k = (large === sweep ? -1 : 1) * Math.sqrt(Math.max(0, (r * r - d2) / d2));
  const c = /** @type {Pt} */ ([k * hy + (a[0] + b[0]) / 2, -k * hx + (a[1] + b[1]) / 2]);
  const t1 = Math.atan2(a[1] - c[1], a[0] - c[0]);
  let d = Math.atan2(b[1] - c[1], b[0] - c[0]) - t1;
  if (!sweep && d > 0) d -= 2 * Math.PI;
  if (sweep && d < 0) d += 2 * Math.PI;
  return { t: "A", a, b, r: Math.max(r, Math.sqrt(d2)), c, sweep: d };
};

/** The centreline as points, a step apart at most. @returns {Pt[]} */
const flatten = (/** @type {Seg[]} */ segs) => {
  const pts = /** @type {Pt[]} */ ([]);
  for (const s of segs) {
    if (!pts.length) pts.push(s.a);
    const len = s.t === "A" ? /** @type {number} */ (s.r) * Math.abs(/** @type {number} */ (s.sweep)) : Math.hypot(s.b[0] - s.a[0], s.b[1] - s.a[1]) * (s.t === "C" ? 1.3 : 1);
    const n = Math.max(1, Math.ceil(len / 0.2));
    for (let k = 1; k <= n; k++) {
      if (s.t === "C") {
        const t = k / n, u = 1 - t, [k1, k2] = /** @type {[Pt, Pt]} */ (s.k);
        pts.push([u * u * u * s.a[0] + 3 * u * u * t * k1[0] + 3 * u * t * t * k2[0] + t * t * t * s.b[0], u * u * u * s.a[1] + 3 * u * u * t * k1[1] + 3 * u * t * t * k2[1] + t * t * t * s.b[1]]);
      } else if (s.t === "L") pts.push([s.a[0] + ((s.b[0] - s.a[0]) * k) / n, s.a[1] + ((s.b[1] - s.a[1]) * k) / n]);
      else {
        const c = /** @type {Pt} */ (s.c), t = Math.atan2(s.a[1] - c[1], s.a[0] - c[0]) + (/** @type {number} */ (s.sweep) * k) / n;
        pts.push([c[0] + /** @type {number} */ (s.r) * Math.cos(t), c[1] + /** @type {number} */ (s.r) * Math.sin(t)]);
      }
    }
  }
  return pts;
};

/** Elements of an svg body, each with the attributes it inherits from its groups. @returns {{ tag: string, attrs: Record<string, string> }[]} */
const elements = (/** @type {string} */ body) => {
  const out = [], stack = /** @type {Record<string, string>[]} */ ([{}]);
  for (const [, close, tag, raw, self] of body.matchAll(/<(\/?)([a-z]+)((?:[^>"]|"[^"]*")*?)(\/?)>/g)) {
    if (close) { stack.pop(); continue; }
    const attrs = { ...stack[stack.length - 1], ...Object.fromEntries([...raw.matchAll(/([a-z-]+)="([^"]*)"/g)].map((m) => [m[1], m[2]])) };
    if (tag === "g") { if (!self) stack.push(attrs); continue; }
    out.push({ tag, attrs });
  }
  return out;
};

/** @param {string} body the inside of an icon's svg, drawn with the table's stroke unless an element says otherwise @returns {Part[]} */
const parts = (body, stroke = STROKE) => elements(body).flatMap(({ tag, attrs: a }) => {
  const n = (/** @type {string} */ k, def = 0) => (a[k] === undefined ? def : Number(a[k]));
  const fill = a.fill !== undefined && a.fill !== "none";
  const w = a.stroke === "none" || (fill && a.stroke === undefined) ? 0 : n("stroke-width", stroke);
  const alpha = Number(a[fill ? "fill-opacity" : "stroke-opacity"] ?? 1);
  /** @type {string} */
  let d;
  /** @type {Part["circle"]} */
  let circle;
  /** @type {Part["rect"]} */
  let rect;
  if (tag === "path") d = a.d;
  else if (tag === "circle") { const [cx, cy, r] = [n("cx"), n("cy"), n("r")]; circle = { c: [cx, cy], rx: r, ry: r }; d = `M${cx - r} ${cy}a${r} ${r} 0 1 0 ${2 * r} 0a${r} ${r} 0 1 0 ${-2 * r} 0z`; }
  else if (tag === "rect") {
    const [x, y, rw, rh, rx] = [n("x"), n("y"), n("width"), n("height"), n("rx", n("ry"))];
    rect = { rx };
    d = rx ? `M${x + rx} ${y}h${rw - 2 * rx}a${rx} ${rx} 0 0 1 ${rx} ${rx}v${rh - 2 * rx}a${rx} ${rx} 0 0 1 ${-rx} ${rx}h${2 * rx - rw}a${rx} ${rx} 0 0 1 ${-rx} ${-rx}v${2 * rx - rh}a${rx} ${rx} 0 0 1 ${rx} ${-rx}z` : `M${x} ${y}h${rw}v${rh}h${-rw}z`;
  } else if (tag === "ellipse") {
    const [cx, cy, rx, ry] = [n("cx"), n("cy"), n("rx"), n("ry")];
    const pts = Array.from({ length: 96 }, (_, k) => /** @type {Pt} */ ([cx + rx * Math.cos((k * Math.PI) / 48), cy + ry * Math.sin((k * Math.PI) / 48)]));
    return [{ pts: [...pts, pts[0]], segs: [], closed: true, w, fill, alpha, circle: { c: [cx, cy], rx, ry } }];
  } else throw new Error(`<${tag}> is not drawn in icons`);
  return parsePath(d).map(({ segs, closed }) => ({ pts: flatten(segs), segs, closed, w, fill, alpha, circle, rect }));
});

// ---- measuring ----

const segDist = (/** @type {Pt} */ p, /** @type {Pt} */ a, /** @type {Pt} */ b) => {
  const [dx, dy] = [b[0] - a[0], b[1] - a[1]], l2 = dx * dx + dy * dy;
  const t = l2 ? Math.max(0, Math.min(1, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2)) : 0;
  return Math.hypot(p[0] - a[0] - t * dx, p[1] - a[1] - t * dy);
};
const toLine = (/** @type {Pt} */ p, /** @type {Pt[]} */ line) => Math.min(...line.slice(1).map((q, k) => segDist(p, line[k], q)));
/** Nearest the centrelines of two parts come to each other, and where. */
const between = (/** @type {Pt[]} */ A, /** @type {Pt[]} */ B) => {
  let d = Infinity, at = A[0];
  for (const [P, Q] of [[A, B], [B, A]]) for (const p of P) { const e = toLine(p, Q); if (e < d) { d = e; at = p; } }
  return { d, at };
};
/** Where two segments cross, if they do. @returns {Pt | null} */
const crossAt = (/** @type {Pt} */ p, /** @type {Pt} */ q, /** @type {Pt} */ r, /** @type {Pt} */ s) => {
  const [dx1, dy1, dx2, dy2] = [q[0] - p[0], q[1] - p[1], s[0] - r[0], s[1] - r[1]], den = dx1 * dy2 - dy1 * dx2;
  if (Math.abs(den) < 1e-9) return null;
  const t = ((r[0] - p[0]) * dy2 - (r[1] - p[1]) * dx2) / den, u = ((r[0] - p[0]) * dy1 - (r[1] - p[1]) * dx1) / den;
  return t >= 0 && t <= 1 && u >= 0 && u <= 1 ? [p[0] + t * dx1, p[1] + t * dy1] : null;
};
const inside = (/** @type {Pt} */ p, /** @type {Pt[]} */ poly) => {
  let c = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) if ((poly[i][1] > p[1]) !== (poly[j][1] > p[1]) && p[0] < ((poly[j][0] - poly[i][0]) * (p[1] - poly[i][1])) / (poly[j][1] - poly[i][1]) + poly[i][0]) c = !c;
  return c;
};
/** How much of the grid the ink covers, in square units, sampled every 0.25. */
const ink = (/** @type {Part[]} */ list) => {
  let sum = 0;
  for (let y = 0.125; y < STYLE.grid; y += 0.25) for (let x = 0.125; x < STYLE.grid; x += 0.25) {
    let a = 0;
    for (const p of list) {
      if (p.alpha <= a) continue;
      if (p.fill ? inside([x, y], p.pts) : p.w && p.pts.some((q, k) => k && segDist([x, y], p.pts[k - 1], q) <= p.w / 2)) a = p.alpha;
    }
    sum += a * 0.0625;
  }
  return sum;
};
const KEYLINE_INK = ink(parts(`<circle cx="8" cy="8" r="${(STYLE.keyline.circle - STROKE) / 2}"/>`));

// ---- the checks ----

/** @param {string} body @returns {{ rule: string, text: string }[]} */
export const check = (body, exempt = /** @type {string[]} */ ([])) => {
  const found = /** @type {{ rule: string, text: string }[]} */ ([]);
  const add = (/** @type {string} */ rule, /** @type {string} */ text) => { if (!exempt.includes(rule)) found.push({ rule, text }); };
  const list = parts(body);
  const at = (/** @type {Pt} */ p) => `(${+p[0].toFixed(2)}, ${+p[1].toFixed(2)})`;
  list.forEach((p, i) => {
    const label = `part ${i + 1}`;
    if (p.w && !near(p.w, STROKE)) add("stroke", `${label}: stroke ${p.w}, the table's is ${STROKE}`);
    // Every point on the quarter grid; a straight edge also on .25/.75 so a 1.5 stroke lands on whole pixels at 2×.
    const edge = (/** @type {number} */ v) => onGrid(v, 0.5, p.w / 2);
    for (const s of p.segs) {
      for (const q of [s.a, s.b]) if (!onGrid(q[0], SNAP) || !onGrid(q[1], SNAP)) add("grid", `${label}: point ${at(q)} is off the ${SNAP} grid`);
      // A curve is free-handed, unless it is a straight line in disguise.
      const [k1, k2] = s.k ?? [s.a, s.a];
      if (s.t === "C" && [k1, k2].some((k) => Math.abs((k[0] - s.a[0]) * (s.b[1] - s.a[1]) - (k[1] - s.a[1]) * (s.b[0] - s.a[0])) > EPS)) continue;
      if (s.t === "A") { if (!onGrid(/** @type {number} */ (s.r), SNAP)) add("grid", `${label}: arc radius ${+(/** @type {number} */ (s.r)).toFixed(2)} is off the ${SNAP} grid`); continue; }
      const [dx, dy] = [s.b[0] - s.a[0], s.b[1] - s.a[1]];
      if (Math.hypot(dx, dy) < EPS) continue;
      if (near(dx, 0) && !edge(s.a[0])) add("pixels", `${label}: vertical edge at x=${s.a[0]} does not land on whole pixels at 2×`);
      if (near(dy, 0) && !edge(s.a[1])) add("pixels", `${label}: horizontal edge at y=${s.a[1]} does not land on whole pixels at 2×`);
      const deg = (Math.atan2(Math.abs(dy), Math.abs(dx)) * 180) / Math.PI;
      if (!ANGLES.some((g) => near(deg, g, 1))) add("angle", `${label}: a line at ${deg.toFixed(1)}° from ${at(s.a)}, the table allows ${ANGLES.map((g) => +g.toFixed(1)).join("/")}°`);
    }
    if (p.circle) { const { c, rx, ry } = p.circle; for (const v of [c[0] - rx, c[0] + rx, c[1] - ry, c[1] + ry]) if (!edge(v)) { add("pixels", `${label}: circle ${at(c)} r=${rx}${rx === ry ? "" : `×${ry}`} reaches ${v}, off the pixel grid at 2×`); break; } }
    if (p.rect && !RADII.includes(p.rect.rx)) add("radius", `${label}: rect rx=${p.rect.rx}, the table's radii are ${RADII.join(" and ")}`);
    // A quarter-turn arc between two straight edges is a rounded corner.
    p.segs.forEach((s, k) => {
      if (s.t !== "A" || p.circle) return;
      const prev = p.segs[k - 1] ?? (p.closed ? p.segs[p.segs.length - 1] : null), next = p.segs[k + 1] ?? (p.closed ? p.segs[0] : null);
      if (prev?.t === "L" && next?.t === "L" && near(Math.abs(/** @type {number} */ (s.sweep)), Math.PI / 2, 0.05) && !RADII.some((r) => near(r, /** @type {number} */ (s.r)))) add("radius", `${label}: corner radius ${+(/** @type {number} */ (s.r)).toFixed(2)} at ${at(s.a)}, the table's are ${RADII.join(" and ")}`);
    });
  });
  // Gaps: two separate parts either meet or leave ≥ minGap of clear space between their ink.
  const touch = STROKE / 2;
  for (let i = 0; i < list.length; i++) {
    for (let j = i + 1; j < list.length; j++) {
      const need = (list[i].w + list[j].w) / 2 + GAP;
      const { d, at: where } = between(list[i].pts, list[j].pts);
      if (d > touch && d < need - EPS) add("gap", `parts ${i + 1} and ${j + 1} are ${d.toFixed(2)} apart near ${at(where)}: ${(d - (list[i].w + list[j].w) / 2).toFixed(2)} of clear space, the table wants ${GAP}`);
    }
    // A part coming back near itself (a hairpin, a notch) must also leave the gap.
    const p = list[i], pts = p.pts;
    if (!p.w) continue;
    const arcAt = [0];
    pts.forEach((q, k) => k && arcAt.push(arcAt[k - 1] + Math.hypot(q[0] - pts[k - 1][0], q[1] - pts[k - 1][1])));
    const total = arcAt[arcAt.length - 1], need = STROKE + GAP, opening = Math.sin(rad(22.5));
    // Where the part crosses itself, the branches are not a gap: left to the crossing.
    const crossings = /** @type {Pt[]} */ ([]);
    for (let a = 1; a < pts.length; a++) for (let b = a + 2; b < pts.length; b++) {
      const x = crossAt(pts[a - 1], pts[a], pts[b - 1], pts[b]);
      if (x) crossings.push(x);
    }
    const nearCrossing = (/** @type {Pt} */ q) => crossings.some((x) => Math.hypot(q[0] - x[0], q[1] - x[1]) < 2 * need);
    let worst = Infinity, where = pts[0];
    for (let a = 0; a < pts.length; a++) for (let b = a + 1; b < pts.length; b++) {
      if (nearCrossing(pts[a]) && nearCrossing(pts[b])) continue;
      const along = Math.min(arcAt[b] - arcAt[a], p.closed ? total - (arcAt[b] - arcAt[a]) : Infinity);
      const d = Math.hypot(pts[a][0] - pts[b][0], pts[a][1] - pts[b][1]);
      // Two points of a corner opening 45° or more are that far apart by its shape; nearer than that is a hairpin or a notch.
      if (d >= opening * along) continue;
      if (d > touch && d < worst) { worst = d; where = pts[a]; }
    }
    if (worst < need - EPS) add("gap", `part ${i + 1} comes back within ${worst.toFixed(2)} of itself near ${at(where)}: ${(worst - STROKE).toFixed(2)} of clear space, the table wants ${GAP}`);
  }
  // Padding and size: the ink's box.
  const box = list.flatMap((p) => p.pts.map((q) => /** @type {Pt[]} */ ([[q[0] - p.w / 2, q[1] - p.w / 2], [q[0] + p.w / 2, q[1] + p.w / 2]])).flat());
  const [x0, x1, y0, y1] = [Math.min(...box.map((q) => q[0])), Math.max(...box.map((q) => q[0])), Math.min(...box.map((q) => q[1])), Math.max(...box.map((q) => q[1]))];
  const edges = { left: x0, top: y0, right: STYLE.grid - x1, bottom: STYLE.grid - y1 };
  for (const [side, v] of Object.entries(edges)) if (v < PAD - EPS) add("padding", `ink is ${v.toFixed(2)} from the ${side} edge, the table's padding is ${PAD}`);
  const [w, h] = [x1 - x0, y1 - y0];
  if (Math.max(w, h) < STYLE.keyline.min - EPS) add("size", `the drawing is ${Math.max(w, h).toFixed(1)} across, icons fill at least ${STYLE.keyline.min}`);
  // Ink weight against the keyline circle, so no icon looks darker or fainter than its neighbours.
  const weight = ink(list) / KEYLINE_INK;
  if (weight > STYLE.weight[1]) add("heavy", `${weight.toFixed(2)}× the ink of the keyline circle, the table allows up to ${STYLE.weight[1]}×`);
  if (weight < STYLE.weight[0]) add("light", `${weight.toFixed(2)}× the ink of the keyline circle, the table wants at least ${STYLE.weight[0]}×`);
  return found;
};

// ---- the run ----

/** @param {string} svg @returns {string} the inside of an icon's svg */
const bodyOf = (svg) => svg.replace(/^<svg[^>]*>|<\/svg>$/g, "");
/** Icons drawn inline in other pages: a square viewBox, outlined in the text colour, so not a chart or a picture. @param {string} file */
const inlineIcons = (file) => {
  const found = /** @type {{ name: string, svg: string }[]} */ ([]);
  const src = readFileSync(file, "utf8");
  for (const m of src.matchAll(/<(svg|symbol)\b([^>]*\bstroke="currentColor"[^>]*)>((?:(?!<\/(?:svg|symbol)>).)*)<\/\1>/gs)) {
    const box = /viewBox="0 0 (\d+) (\d+)"/.exec(m[2]);
    if (!box) continue;
    const line = src.slice(0, m.index).split("\n").length;
    const problems = [];
    if (box[1] !== String(STYLE.grid) || box[2] !== String(STYLE.grid)) problems.push(`viewBox ${box[1]}×${box[2]}, the table's grid is ${STYLE.grid}`);
    const sw = /stroke-width="([\d.]+)"/.exec(m[2]);
    if (!sw || Number(sw[1]) !== STROKE) problems.push(`stroke-width ${sw?.[1] ?? "unset"}, the table's is ${STROKE}`);
    // (A body made at run time is checked where its drawings are written: ICONS, KEY_GLYPHS, TRENDS.)
    found.push({ name: `${relative(ROOT, file)}:${line}`, svg: problems.length ? `PROBLEM:${problems.join("; ")}` : m[3].includes("${") ? "" : m[3] });
  }
  return found;
};

const ROOT = join(dirname(new URL(import.meta.url).pathname), "..");
const pages = (/** @type {string} */ dir) => readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
  const p = join(dir, e.name);
  if (e.isDirectory()) return ["assets", "modules", "_engine"].includes(e.name) ? [] : pages(p);
  return /\.(html|js|mjs)$/.test(e.name) && e.name !== "icons.js" && e.name !== "icons-check.mjs" ? [p] : [];
});

if (import.meta.main) {
  const only = process.argv.slice(2);
  const jobs = /** @type {{ name: string, body: string, exempt: string[], problems: string[] }[]} */ ([]);
  for (const [name, svg] of Object.entries(ICONS)) if (!only.length || only.includes(name)) jobs.push({ name, body: bodyOf(svg), exempt: ICON_EXEMPT[name]?.rules ?? [], problems: [] });
  for (const [glyph, body] of Object.entries(KEY_GLYPHS)) if (!only.length || only.includes(glyph)) jobs.push({ name: `key ${glyph}`, body, exempt: [], problems: [] });
  // (Compass's trend arrows: its market.js imports from the browser's paths, so its text is read, not loaded.)
  if (!only.length) for (const [, arrow] of readFileSync(join(dirname(new URL(import.meta.url).pathname), "../compass/market.js"), "utf8").matchAll(/arrow: "([^"]+)"/g)) jobs.push({ name: `compass trend ${arrow}`, body: `<path d="${arrow}"/>`, exempt: [], problems: [] });
  const studio = jobs.length;
  if (!only.length) for (const file of pages(ROOT)) for (const { name, svg } of inlineIcons(file)) jobs.push(svg.startsWith("PROBLEM:") ? { name, body: "", exempt: [], problems: svg.slice(8).split("; ") } : { name, body: svg, exempt: [], problems: [] });
  let failed = 0;
  for (const { name, body, exempt, problems } of jobs) {
    const found = [...problems.map((text) => ({ rule: "table", text })), ...(body ? check(body, exempt) : [])];
    if (found.length) { failed++; console.log(`✗ ${name}`); for (const f of found) console.log(`    ${f.rule}: ${f.text}`); }
  }
  console.log(`${jobs.length - failed} of ${jobs.length} icons pass the style table (${studio} in studio/icons.js, ${jobs.length - studio} drawn inline in pages)`);
  process.exit(failed ? 1 : 0);
}
