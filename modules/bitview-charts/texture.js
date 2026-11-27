// @ts-check
// A tile is made once per texture, color and density, then only moved: anchored to the data, so a pan moves it with
// the picture and the strip drawn beside it meets it seamlessly.

/** @typedef {"grain" | "speckle" | "current" | "shale" | "sandstone" | "conglomerate" | "granite"} TextureKind */
/**
 * Its tile is kept by identity: pass the same object again, not a new one each time.
 * @typedef {{ kind: TextureKind, ink: string, light?: string, seed?: number, amount?: number }} Texture
 */
/** @typedef {{ ink: string, light: string, r: () => number, amount: number, fade: number }} Pen */

/** Wider than a phone's chart where the marks run across, so no repeat shows there. */
const SIZE = /** @type {const} */ ({ grain: [64, 64], speckle: [96, 96], current: [400, 96], shale: [400, 64], sandstone: [400, 96], conglomerate: [192, 192], granite: [160, 160] });

/** Seeded, so the same tile each time. @param {number} seed */
function random(seed) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/**
 * Drawn once more past each edge a mark crosses, so the tile repeats without a seam.
 * @param {number} w @param {number} h @param {number} x @param {number} y @param {number} reachX @param {number} reachY
 * @param {(dx: number, dy: number) => void} draw
 */
function wrapped(w, h, x, y, reachX, reachY, draw) {
  for (const dx of [-w, 0, w]) for (const dy of [-h, 0, h]) if (x + dx + reachX >= 0 && x + dx - reachX <= w && y + dy + reachY >= 0 && y + dy - reachY <= h) draw(dx, dy);
}

/** Meets itself at the tile's edges. @param {number} w @param {number} size @param {() => number} r */
function bend(w, size, r) {
  const a = r() * 6.283, b = r() * 6.283, k = 6.283 / w;
  return (/** @type {number} */ x) => size * (0.7 * Math.sin(k * x + a) + 0.3 * Math.sin(2 * k * x + b));
}

/** @param {CanvasRenderingContext2D} g @param {number} x0 @param {number} x1 @param {number} y @param {(x: number) => number} rise */
function along(g, x0, x1, y, rise, step = 4) {
  g.beginPath();
  g.moveTo(x0, y + rise(x0));
  for (let x = x0 + step; x < x1 + step; x += step) g.lineTo(Math.min(x, x1), y + rise(Math.min(x, x1)));
  g.stroke();
}

/** @param {CanvasRenderingContext2D} g @param {number} w @param {number} h @param {Pen} p */
function fracture(g, w, h, p) {
  const r = p.r, x = r() * w, y = r() * h, length = 5 + r() * 9, turn = 1.25 + r() * 0.65, steps = 3;
  const points = [[x, y]];
  for (let k = 1; k <= steps; k++) points.push([x + Math.cos(turn) * (length * k) / steps + (r() - 0.5) * 2, y + Math.sin(turn) * (length * k) / steps]);
  wrapped(w, h, x, y + length / 2, length, length, (dx, dy) => {
    for (const [ink, alpha, shift, width] of /** @type {const} */ ([[p.ink, 0.42, 0, 0.7], [p.light, 0.16, 0.8, 0.5]])) {
      g.globalAlpha = alpha * p.fade;
      g.strokeStyle = ink;
      g.lineWidth = width;
      g.beginPath();
      points.forEach(([px, py], k) => (k ? g.lineTo(px + dx + shift, py + dy) : g.moveTo(px + dx + shift, py + dy)));
      g.stroke();
    }
  });
}

/** @type {Record<TextureKind, (g: CanvasRenderingContext2D, w: number, h: number, p: Pen) => void>} */
const MARKS = {
  grain(g, w, h, p) {
    const r = p.r;
    for (let k = 0, n = Math.round(w * h * 0.22 * p.amount); k < n; k++) {
      g.globalAlpha = (0.05 + r() * 0.16) * p.fade;
      g.fillStyle = r() < 0.6 ? p.ink : p.light;
      g.fillRect(Math.floor(r() * w), Math.floor(r() * h), 1, 1);
    }
  },
  speckle(g, w, h, p) {
    MARKS.grain(g, w, h, p);
    const r = p.r;
    for (let k = 0, n = Math.round(((w * h) / 70) * p.amount); k < n; k++) {
      const x = r() * w, y = r() * h, big = r() < 0.12, rx = big ? 1.4 + r() * 1.4 : 0.5 + r() * 0.9, ry = rx * (0.55 + r() * 0.4), turn = r() * Math.PI, alpha = (0.18 + r() * 0.35) * p.fade, lit = r() < 0.25;
      wrapped(w, h, x, y, rx + 1, rx + 1, (dx, dy) => {
        g.globalAlpha = alpha;
        g.fillStyle = lit ? p.light : p.ink;
        g.beginPath();
        g.ellipse(x + dx, y + dy, rx, ry, turn, 0, Math.PI * 2);
        g.fill();
      });
    }
  },
  current(g, w, h, p) {
    const r = p.r;
    g.lineCap = "round";
    for (let k = 0, n = Math.round(((w * h) / 650) * p.amount); k < n; k++) {
      const x = r() * w, y = r() * h, length = 40 + r() * r() * w * 0.75, rise = 0.3 + r() * 0.8, waves = 1 + Math.floor(r() * 3), phase = r() * 6.283;
      const dark = r() < 0.4, alpha = (0.1 + r() * 0.2) * p.fade;
      wrapped(w, h, x + length / 2, y, length / 2 + 1, rise + 1, (dx, dy) => {
        g.strokeStyle = dark ? p.ink : p.light;
        g.lineWidth = dark ? 0.8 : 0.6;
        for (let s = 0; s < length; s += 4) {
          const t0 = s / length, t1 = Math.min(s + 4, length) / length;
          g.globalAlpha = alpha * Math.sin(Math.PI * (t0 + t1) * 0.5);
          g.beginPath();
          g.moveTo(x + dx + s, y + dy + rise * Math.sin(phase + t0 * waves * 6.283));
          g.lineTo(x + dx + Math.min(s + 4, length), y + dy + rise * Math.sin(phase + t1 * waves * 6.283));
          g.stroke();
        }
      });
    }
    for (let k = 0, n = Math.round(((w * h) / 3000) * p.amount * p.amount); k < n; k++) {
      const x = r() * w, y = r() * h, length = 2 + r() * 6, alpha = (0.25 + r() * 0.35) * p.fade;
      wrapped(w, h, x + length / 2, y, length / 2 + 1, 1, (dx, dy) => {
        g.globalAlpha = alpha;
        g.strokeStyle = p.light;
        g.lineWidth = 0.8;
        g.beginPath();
        g.moveTo(x + dx, y + dy);
        g.lineTo(x + dx + length, y + dy);
        g.stroke();
      });
    }
  },
  shale(g, w, h, p) {
    const r = p.r, rise = bend(w, 1.1, r);
    let y = r() * 4;
    const planes = [];
    while (y < h) planes.push(y), (y += 2.5 + r() * r() * 12);
    planes.forEach((top, k) => {
      const bottom = planes[k + 1] ?? h + planes[0];
      g.globalAlpha = (0.04 + r() * 0.1) * p.fade;
      g.fillStyle = r() < 0.5 ? p.light : p.ink;
      for (const dy of [-h, 0, h]) {
        g.beginPath();
        for (let x = 0; x <= w; x += 8) g.lineTo(x, top + dy + rise(x));
        for (let x = w; x >= 0; x -= 8) g.lineTo(x, bottom + dy + rise(x));
        g.fill();
      }
    });
    g.lineCap = "round";
    for (let y = r() * 2; y < h; y += 3 + r() * 7) {
      const dark = r() < 0.6, alpha = (0.05 + r() * 0.1) * p.fade, width = 0.4 + r() * 0.3;
      for (let x = r() * 30; x < w; ) {
        const run = 40 + r() * w * 0.7, end = Math.min(x + run, w);
        g.globalAlpha = alpha;
        g.strokeStyle = dark ? p.ink : p.light;
        g.lineWidth = width;
        for (const dy of [-h, 0, h]) along(g, x, end, y + dy, rise);
        x = end + 2 + r() * 10;
      }
    }
    for (const top of planes.slice(0, 1 + Math.floor(r() * 2))) {
      for (const dy of [-h, 0, h]) {
        (g.globalAlpha = 0.3 * p.fade), (g.strokeStyle = p.ink), (g.lineWidth = 1.1);
        along(g, 0, w, top + dy, rise);
        (g.globalAlpha = 0.16 * p.fade), (g.strokeStyle = p.light), (g.lineWidth = 0.5);
        along(g, 0, w, top + dy + 1.2, rise);
      }
    }
    if (r() < 0.7 * p.amount) fracture(g, w, h, p);
  },
  sandstone(g, w, h, p) {
    MARKS.grain(g, w, h, { ...p, amount: p.amount * 0.7 });
    const r = p.r, beds = [];
    for (let total = 0; total < h; ) beds.push(12 + r() * 20), (total += beds[beds.length - 1]);
    const scale = h / beds.reduce((a, b) => a + b, 0);
    let top = 0;
    for (const thick of beds.map((t) => t * scale)) {
      const rise = bend(w, 0.8, r), bottom = top + thick;
      g.globalAlpha = (0.03 + r() * 0.07) * p.fade;
      g.fillStyle = r() < 0.5 ? p.light : p.ink;
      g.fillRect(0, top, w, thick);
      for (const dy of [-h, 0, h]) {
        (g.globalAlpha = (0.16 + r() * 0.12) * p.fade), (g.strokeStyle = p.ink), (g.lineWidth = 0.8);
        along(g, 0, w, top + dy, rise);
        (g.globalAlpha = 0.12 * p.fade), (g.strokeStyle = p.light), (g.lineWidth = 0.5);
        along(g, 0, w, top + dy + 1, rise);
      }
      for (let x = r() * 40; x < w; x += 40 + r() * 80) {
        const lines = 3 + Math.floor(r() * 4), slope = 0.22 + r() * 0.25, gap = 3 + r() * 2.5, length = Math.min(thick * 2.6, 16 + r() * 22);
        for (let k = 0; k < lines; k++) {
          const x0 = x + k * gap, alpha = (0.12 + r() * 0.12) * p.fade;
          wrapped(w, h, x0 + length / 2, (top + bottom) / 2, length / 2 + 1, thick, (dx, dy) => {
            g.globalAlpha = alpha;
            g.strokeStyle = p.ink;
            g.lineWidth = 0.6;
            g.beginPath();
            g.moveTo(x0 + dx, top + dy + 1.5);
            g.quadraticCurveTo(x0 + dx + length * 0.45, bottom + dy - 1, x0 + dx + length, bottom + dy - 0.5 - (thick - 2) * slope * 0.15);
            g.stroke();
          });
        }
      }
      top = bottom;
    }
    if (r() < 0.6 * p.amount) fracture(g, w, h, p);
  },
  conglomerate(g, w, h, p) {
    MARKS.grain(g, w, h, p);
    const r = p.r, placed = /** @type {number[][]} */ ([]);
    for (let k = 0, n = Math.round(((w * h) / 300) * p.amount); k < n; k++) {
      let x = 0, y = 0, rx = 0;
      for (let tries = 0; tries < 6; tries++) {
        (x = r() * w), (y = r() * h), (rx = 1.4 + r() * r() * 6);
        if (!placed.some(([px, py, pr]) => Math.hypot(((x - px + w * 1.5) % w) - w / 2, ((y - py + h * 1.5) % h) - h / 2) < rx + pr + 1)) break;
      }
      placed.push([x, y, rx]);
      const ry = rx * (0.6 + r() * 0.35), turn = (r() - 0.5) * 1.2, dark = r() < 0.55, alpha = (0.12 + r() * 0.22) * p.fade;
      wrapped(w, h, x, y, rx + 1, rx + 1, (dx, dy) => {
        g.globalAlpha = alpha;
        g.fillStyle = dark ? p.ink : p.light;
        g.beginPath();
        g.ellipse(x + dx, y + dy, rx, ry, turn, 0, Math.PI * 2);
        g.fill();
        (g.globalAlpha = 0.32 * p.fade), (g.strokeStyle = p.ink), (g.lineWidth = 0.6);
        g.beginPath();
        g.ellipse(x + dx, y + dy, rx, ry, turn, 0.15, Math.PI - 0.15);
        g.stroke();
        (g.globalAlpha = 0.22 * p.fade), (g.strokeStyle = p.light), (g.lineWidth = 0.5);
        g.beginPath();
        g.ellipse(x + dx, y + dy, rx * 0.8, ry * 0.75, turn, Math.PI + 0.4, Math.PI * 2 - 0.4);
        g.stroke();
      });
    }
  },
  granite(g, w, h, p) {
    MARKS.grain(g, w, h, p);
    const r = p.r;
    for (let k = 0, n = Math.round(((w * h) / 16) * p.amount); k < n; k++) {
      const x = r() * w, y = r() * h, kind = r(), feldspar = kind >= 0.6, size = feldspar ? 1.2 + r() * 2.4 : 0.5 + r() * r() * 2.2;
      const ink = kind < 0.3 ? p.ink : kind < 0.6 ? p.light : r() < 0.5 ? p.ink : p.light;
      const alpha = (kind < 0.3 ? 0.32 + r() * 0.3 : kind < 0.6 ? 0.12 + r() * 0.18 : 0.05 + r() * 0.07) * p.fade;
      const sides = 3 + Math.floor(r() * 3), turn = r() * 6.283, corners = Array.from({ length: sides }, (_, s) => [turn + (s / sides) * 6.283 + (r() - 0.5) * 0.8, size * (0.6 + r() * 0.5)]);
      wrapped(w, h, x, y, size + 1, size + 1, (dx, dy) => {
        g.globalAlpha = alpha;
        g.fillStyle = ink;
        g.beginPath();
        for (const [a, d] of corners) g.lineTo(x + dx + Math.cos(a) * d, y + dy + Math.sin(a) * d);
        g.fill();
      });
    }
  },
};

/**
 * Whole canvas pixels, so it repeats without a seam.
 * @param {Texture} texture @param {string} color @param {number} dpr
 */
function tile(texture, color, dpr) {
  const [w, h] = SIZE[texture.kind], made = document.createElement("canvas");
  (made.width = Math.max(1, Math.round(w * dpr))), (made.height = Math.max(1, Math.round(h * dpr)));
  const g = /** @type {CanvasRenderingContext2D} */ (made.getContext("2d"));
  g.scale(made.width / w, made.height / h);
  g.fillStyle = color;
  g.fillRect(0, 0, w, h);
  const amount = texture.amount ?? 1;
  MARKS[texture.kind](g, w, h, { ink: texture.ink, light: texture.light ?? texture.ink, r: random(texture.seed ?? 1), amount, fade: 0.5 + amount / 2 });
  return made;
}

/** A context of its own, so any context can fill with the patterns. */
let pen = /** @type {CanvasRenderingContext2D | null} */ (null);
/**
 * Made once and shared by every fill in that look.
 * @type {WeakMap<Texture, Map<string, { pattern: CanvasPattern | null, a: number, d: number }>>}
 */
const patterns = new WeakMap();

/**
 * `a` and `d` scale its tile's pixels to CSS pixels.
 * @param {Texture} texture @param {string} color @param {number} dpr
 */
export function patternOf(texture, color, dpr) {
  let looks = patterns.get(texture);
  if (!looks) patterns.set(texture, (looks = new Map()));
  const key = `${color}|${dpr}`;
  let made = looks.get(key);
  if (!made) {
    const [w, h] = SIZE[texture.kind], picture = tile(texture, color, dpr);
    pen ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
    looks.set(key, (made = { pattern: pen.createPattern(picture, "repeat"), a: w / picture.width, d: h / picture.height }));
  }
  return made;
}

/** @param {number} n @param {number} m */
const mod = (n, m) => n - Math.floor(n / m) * m;

/** Its pattern is made again only as its texture, color or density change, so a draw makes none. */
export class TextureFill {
  constructor() {
    /** @type {CanvasPattern | null} */
    this.pattern = null;
    /** @type {Texture | null} */
    this.texture = null;
    this.color = "";
    this.dpr = 0;
    this.matrix = new DOMMatrix();
  }

  /**
   * On a recording context (see recorder.js) the color: only the shape counts there, and the screen's tile stays made.
   * @param {CanvasRenderingContext2D} g @param {Texture} texture @param {string} color @param {number} dpr
   * @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @returns {CanvasPattern | string}
   */
  style(g, texture, color, dpr, time, scale) {
    if (!(g instanceof CanvasRenderingContext2D)) return color;
    // Read by index: without a JIT, destructuring walks an iterator made each draw.
    const size = SIZE[texture.kind], w = size[0], h = size[1];
    if (this.texture !== texture || this.color !== color || this.dpr !== dpr) {
      (this.texture = texture), (this.color = color), (this.dpr = dpr);
      const made = patternOf(texture, color, dpr);
      (this.pattern = made.pattern), (this.matrix.a = made.a), (this.matrix.d = made.d);
    }
    if (!this.pattern) return color;
    (this.matrix.e = mod(time.x(0), w)), (this.matrix.f = mod(scale.y(scale.mode === "log" ? 1 : 0), h));
    this.pattern.setTransform(this.matrix);
    return this.pattern;
  }
}
