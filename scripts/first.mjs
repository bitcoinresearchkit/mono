// The face a first screen draws with, cut from the committed core face: only the characters a chart's first screen
// writes (FIRST), every glyph they reach (their alternates, ligatures and components) and the kerning between them,
// the axes as they are. Glyphs keep their numbers (those not reached drawn empty), so the variation data, the advances
// and the substitutions are kept as they are; the kerning is written anew, one pair per pair, each with its variation.
// The page lists it last, by its unicode-range: those characters come from it, any other from the core and the rest.
//
//   bun scripts/first.mjs           writes website/assets/fonts/InterVariable.first.woff2 and prints its unicode-range
//   bun scripts/first.mjs --check   fails if it would be written otherwise (the pre-commit hook runs it)
//
// Run again when the core face or FIRST changes. No subsetter here: WOFF2 read and written by hand (brotli is bun's).
import { brotliCompressSync, brotliDecompressSync, constants } from "node:zlib";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const WEBSITE = join(import.meta.dir, "../website"), FONTS = join(WEBSITE, "assets/fonts");
const SOURCE = join(FONTS, "InterVariable.core.woff2"), OUT = join(FONTS, "InterVariable.first.woff2");
// Basic Latin, and what a chart's first screen writes beyond it: a no-break space, ·, ×, the dashes, ’, …, ₿, ↑ ↓, −.
const FIRST = [[0x20, 0x7e], [0xa0, 0xa0], [0xb7, 0xb7], [0xd7, 0xd7], [0x2013, 0x2014], [0x2019, 0x2019], [0x2026, 0x2026], [0x203a, 0x203a], [0x20bf, 0x20bf], [0x2191, 0x2191], [0x2193, 0x2193], [0x2212, 0x2212], [0x2248, 0x2248], [0x2318, 0x2318], [0x2325, 0x2325]];

// The substitutions a page gets: what browsers apply by default, tabular numbers (the pages' tabular-nums), and every
// feature Studio's styles set (its body's single-storey a, cv11: missed, each a drawn empty).
const STYLES = ["studio/index.html", "studio/rest.css"].map((file) => readFileSync(join(WEBSITE, file), "utf8")).join("\n");
const APPLIED = new Set(["calt", "ccmp", "clig", "liga", "locl", "rclt", "rlig", "rvrn", "tnum", ...[...STYLES.matchAll(/font-feature-settings:([^;}]*)/g)].flatMap(([, list]) => [...list.matchAll(/"(\w{4})"/g)].map(([, tag]) => tag))]);

// ── Reading WOFF2 (https://www.w3.org/TR/WOFF2/) ──
const KNOWN = "cmap head hhea hmtx maxp name OS/2 post cvt_ fpgm glyf loca prep CFF_ VORG EBDT EBLC gasp hdmx kern LTSH PCLT VDMX vhea vmtx BASE GDEF GPOS GSUB EBSC JSTF MATH CBDT CBLC COLR CPAL SVG_ sbix acnt avar bdat bloc bsln cvar fdsc feat fmtx fvar gvar hsty just lcar mort morx opbd prop trak Zapf Silf Glat Gloc Feat Sill".split(" ").map((tag) => tag.replace("_", " "));

/** The font's tables, by tag, as its stream holds them, and which of them are transformed. @param {Uint8Array} file */
function readWoff2(file) {
  const view = new DataView(file.buffer, file.byteOffset, file.byteLength);
  if (view.getUint32(0) !== 0x774f4632) throw new Error("not WOFF2");
  const count = view.getUint16(12);
  let at = 48;
  const base128 = () => {
    let value = 0;
    for (let i = 0; i < 5; i++) {
      const byte = file[at++];
      value = value * 128 + (byte & 0x7f);
      if (!(byte & 0x80)) return value;
    }
    throw new Error("UIntBase128 too long");
  };
  const entries = [];
  for (let i = 0; i < count; i++) {
    const flags = file[at++], tag = (flags & 63) === 63 ? String.fromCharCode(...file.subarray(at, (at += 4))) : KNOWN[flags & 63], version = flags >> 6;
    const length = base128(), transformed = tag === "glyf" || tag === "loca" ? version === 0 : version !== 0;
    entries.push({ tag, length: transformed ? base128() : length, transformed });
  }
  const data = brotliDecompressSync(file.subarray(at, at + view.getUint32(20)));
  const tables = new Map();
  let offset = 0;
  for (const entry of entries) {
    tables.set(entry.tag, data.subarray(offset, offset + entry.length));
    offset += entry.length;
  }
  return { tables, transformed: new Set(entries.filter((entry) => entry.transformed).map((entry) => entry.tag)) };
}

/** A 255UInt16 (WOFF2's), read from a stream. */
const read255 = (/** @type {Uint8Array} */ bytes, /** @type {{ at: number }} */ s) => {
  const code = bytes[s.at++];
  if (code === 253) return (bytes[s.at++] << 8) | bytes[s.at++];
  if (code === 255) return bytes[s.at++] + 253;
  if (code === 254) return bytes[s.at++] + 506;
  return code;
};

/** The glyphs of a transformed glyf table (WOFF2 §5.1), each its OpenType bytes (empty: none). @param {Uint8Array} table @returns {Uint8Array[]} */
function glyphsOf(table) {
  const view = new DataView(table.buffer, table.byteOffset, table.byteLength), options = view.getUint16(2), count = view.getUint16(4);
  let at = 36;
  const stream = (size) => ((part) => ((at += size), part))(table.subarray(at, at + size));
  const contours = stream(view.getUint32(8)), points = stream(view.getUint32(12)), flags = stream(view.getUint32(16)), glyphs = stream(view.getUint32(20)), composites = stream(view.getUint32(24)), boxes = stream(view.getUint32(28)), instructions = stream(view.getUint32(32));
  if (options & 1) throw new Error("overlap bitmap: not handled");
  const contourView = new DataView(contours.buffer, contours.byteOffset), boxView = new DataView(boxes.buffer, boxes.byteOffset), boxBits = 4 * Math.floor((count + 31) / 32);
  const p = { at: 0 }, f = { at: 0 }, g = { at: 0 }, c = { at: 0 }, b = { at: boxBits }, n = { at: 0 };
  const out = [];
  for (let i = 0; i < count; i++) {
    const contourCount = contourView.getInt16(i * 2), boxed = boxes[i >> 3] & (0x80 >> (i & 7));
    const box = () => ((values) => ((b.at += 8), values))([0, 2, 4, 6].map((k) => boxView.getInt16(b.at + k)));
    if (contourCount === 0) {
      out.push(new Uint8Array(0));
      continue;
    }
    if (contourCount < 0) {
      // Components as stored, their length read from their flags; instructions after, if the last says so.
      const start = c.at;
      let more = true, instructed = false;
      while (more) {
        const flag = (composites[c.at] << 8) | composites[c.at + 1];
        c.at += 4 + (flag & 1 ? 4 : 2) + (flag & 8 ? 2 : flag & 0x40 ? 4 : flag & 0x80 ? 8 : 0);
        more = Boolean(flag & 0x20);
        instructed ||= Boolean(flag & 0x100);
      }
      const parts = composites.subarray(start, c.at), code = instructed ? instructions.subarray(n.at, (n.at += read255(glyphs, g))) : null;
      if (!boxed) throw new Error(`glyph ${i}: a composite without its box`);
      const glyph = new Writer();
      glyph.i16(-1);
      for (const v of box()) glyph.i16(v);
      glyph.bytes(parts);
      if (code) glyph.u16(code.length), glyph.bytes(code);
      out.push(glyph.done());
      continue;
    }
    // A simple glyph: its points from the flag and triplet streams, written in the shortest form.
    const ends = [];
    let total = 0;
    for (let k = 0; k < contourCount; k++) ends.push((total += read255(points, p)) - 1);
    const xs = [], ys = [], on = [];
    let x = 0, y = 0;
    for (let k = 0; k < total; k++) {
      const flag = flags[f.at++], low = flag & 0x7f, d = glyphs, a = g.at;
      const sign = (v, bit) => (bit & 1 ? v : -v);
      let dx, dy, size;
      if (low < 10) (dx = 0), (dy = sign(((low & 14) << 7) + d[a], low)), (size = 1);
      else if (low < 20) (dx = sign((((low - 10) & 14) << 7) + d[a], low)), (dy = 0), (size = 1);
      else if (low < 84) {
        const b0 = low - 20, b1 = d[a];
        (dx = sign(1 + (b0 & 0x30) + (b1 >> 4), low)), (dy = sign(1 + ((b0 & 0x0c) << 2) + (b1 & 0x0f), low >> 1)), (size = 1);
      } else if (low < 120) {
        const b0 = low - 84;
        (dx = sign(1 + (Math.floor(b0 / 12) << 8) + d[a], low)), (dy = sign(1 + (((b0 % 12) >> 2) << 8) + d[a + 1], low >> 1)), (size = 2);
      } else if (low < 124) (dx = sign((d[a] << 4) + (d[a + 1] >> 4), low)), (dy = sign(((d[a + 1] & 0x0f) << 8) + d[a + 2], low >> 1)), (size = 3);
      else (dx = sign((d[a] << 8) + d[a + 1], low)), (dy = sign((d[a + 2] << 8) + d[a + 3], low >> 1)), (size = 4);
      g.at += size;
      xs.push((x += dx)), ys.push((y += dy)), on.push(!(flag & 0x80));
    }
    const code = instructions.subarray(n.at, (n.at += read255(glyphs, g)));
    const bounds = boxed ? box() : [Math.min(...xs), Math.min(...ys), Math.max(...xs), Math.max(...ys)];
    const glyph = new Writer();
    glyph.i16(contourCount);
    for (const v of bounds) glyph.i16(v);
    for (const end of ends) glyph.u16(end);
    glyph.u16(code.length), glyph.bytes(code);
    const deltas = (values) => values.map((v, k) => v - (k ? values[k - 1] : 0)), ddx = deltas(xs), ddy = deltas(ys);
    const flagOf = (k) => (on[k] ? 1 : 0) | (ddx[k] === 0 ? 0x10 : Math.abs(ddx[k]) < 256 ? 0x02 | (ddx[k] > 0 ? 0x10 : 0) : 0) | (ddy[k] === 0 ? 0x20 : Math.abs(ddy[k]) < 256 ? 0x04 | (ddy[k] > 0 ? 0x20 : 0) : 0);
    for (let k = 0; k < total; k++) glyph.u8(flagOf(k));
    for (const [d, short] of [[ddx, 0x02], [ddy, 0x04]]) for (let k = 0; k < total; k++) if (d[k] !== 0) flagOf(k) & short ? glyph.u8(Math.abs(d[k])) : glyph.i16(d[k]);
    out.push(glyph.done());
  }
  return out;
}

// ── Writing ──
class Writer {
  bytes_ = [];
  u8(v) { this.bytes_.push(v & 0xff); }
  u16(v) { this.bytes_.push((v >> 8) & 0xff, v & 0xff); }
  i16(v) { this.u16(v & 0xffff); }
  u32(v) { this.u16(Math.floor(v / 65536)), this.u16(v % 65536); }
  tag(s) { for (const ch of s) this.u8(ch.charCodeAt(0)); }
  bytes(b) { for (const v of b) this.bytes_.push(v); }
  get length() { return this.bytes_.length; }
  pad(n = 4) { while (this.bytes_.length % n) this.bytes_.push(0); }
  set16(at, v) { this.bytes_[at] = (v >> 8) & 0xff, this.bytes_[at + 1] = v & 0xff; }
  done() { return Uint8Array.from(this.bytes_); }
}
const view = (bytes) => new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);

// ── What's reached: the characters' glyphs, then what substitutions and components bring ──
/** Each character of the font's cmap (format 4 and 12) to its glyph. */
function cmapOf(cmap) {
  const v = view(cmap), map = new Map();
  for (let i = 0; i < v.getUint16(2); i++) {
    const at = v.getUint32(8 + i * 8), format = v.getUint16(at);
    if (format === 4) {
      const segments = v.getUint16(at + 6) / 2, ends = at + 14, starts = ends + segments * 2 + 2, deltas = starts + segments * 2, ranges = deltas + segments * 2;
      for (let s = 0; s < segments; s++) {
        const end = v.getUint16(ends + s * 2), start = v.getUint16(starts + s * 2), delta = v.getInt16(deltas + s * 2), range = v.getUint16(ranges + s * 2);
        for (let cp = start; cp <= end && cp !== 0xffff; cp++) {
          const glyph = range ? v.getUint16(ranges + s * 2 + range + (cp - start) * 2) : cp;
          if (glyph) map.set(cp, range ? (glyph ? (glyph + delta) & 0xffff : 0) : (cp + delta) & 0xffff);
        }
      }
    } else if (format === 12) {
      for (let g = 0; g < v.getUint32(at + 12); g++) {
        const r = at + 16 + g * 12;
        for (let cp = v.getUint32(r); cp <= v.getUint32(r + 4); cp++) map.set(cp, v.getUint32(r + 8) + cp - v.getUint32(r));
      }
    }
  }
  return map;
}

/** A Coverage table's glyphs, in coverage order. */
function coverage(v, at) {
  const format = v.getUint16(at), out = [];
  if (format === 1) for (let i = 0; i < v.getUint16(at + 2); i++) out.push(v.getUint16(at + 4 + i * 2));
  else for (let i = 0; i < v.getUint16(at + 2); i++) for (let g = v.getUint16(at + 4 + i * 6); g <= v.getUint16(at + 6 + i * 6); g++) out.push(g);
  return out;
}
/** A ClassDef table, as a glyph's class (0 for one it doesn't name). */
function classDef(v, at) {
  const format = v.getUint16(at), map = new Map();
  if (format === 1) for (let i = 0, first = v.getUint16(at + 2); i < v.getUint16(at + 4); i++) map.set(first + i, v.getUint16(at + 6 + i * 2));
  else for (let i = 0; i < v.getUint16(at + 2); i++) for (let g = v.getUint16(at + 4 + i * 6); g <= v.getUint16(at + 6 + i * 6); g++) map.set(g, v.getUint16(at + 8 + i * 6));
  return (glyph) => map.get(glyph) ?? 0;
}
/** A layout table's lookups, each its type and its subtables' offsets (extensions followed). */
function lookupsOf(table, extension) {
  const v = view(table), list = v.getUint16(8), out = [];
  for (let i = 0; i < v.getUint16(list); i++) {
    const at = list + v.getUint16(list + 2 + i * 2);
    const own = v.getUint16(at), subtables = [];
    let type = own;
    for (let s = 0; s < v.getUint16(at + 4); s++) {
      let sub = at + v.getUint16(at + 6 + s * 2);
      if (own === extension) (type = v.getUint16(sub + 2)), (sub += v.getUint32(sub + 4));
      subtables.push(sub);
    }
    out.push({ type, subtables });
  }
  return { v, out };
}
/** Each feature's lookups, by its tag. */
function featuresOf(table) {
  const v = view(table), list = v.getUint16(6), out = new Map();
  for (let i = 0; i < v.getUint16(list); i++) {
    const tag = String.fromCharCode(...table.subarray(list + 2 + i * 6, list + 6 + i * 6)), at = list + v.getUint16(list + 6 + i * 6);
    out.set(tag, [...(out.get(tag) ?? []), ...Array.from({ length: v.getUint16(at + 2) }, (_, k) => v.getUint16(at + 4 + k * 2))]);
  }
  return out;
}
/** The lookups a contextual subtable (GSUB 5, 6) calls. */
function called(v, type, at) {
  const format = v.getUint16(at), out = [];
  const records = (o, count) => { for (let k = 0; k < count; k++) out.push(v.getUint16(o + k * 4 + 2)); };
  // (A rule: its glyphs or classes to skip, then its records.)
  const rule = (o) => {
    if (type === 5) return records(o + 4 + (v.getUint16(o) - 1) * 2, v.getUint16(o + 2));
    o += 2 + v.getUint16(o) * 2;
    o += 2 + (v.getUint16(o) - 1) * 2;
    o += 2 + v.getUint16(o) * 2;
    records(o + 2, v.getUint16(o));
  };
  if (format === 3) {
    if (type === 5) return records(at + 6 + v.getUint16(at + 2) * 2, v.getUint16(at + 4)), out;
    let o = at + 2;
    for (let part = 0; part < 3; part++) o += 2 + v.getUint16(o) * 2;
    return records(o + 2, v.getUint16(o)), out;
  }
  const sets = at + (format === 1 ? 4 : type === 5 ? 6 : 10);
  for (let s = 0; s < v.getUint16(sets); s++) {
    const set = v.getUint16(sets + 2 + s * 2);
    if (!set) continue;
    for (let r = 0; r < v.getUint16(at + set); r++) rule(at + set + v.getUint16(at + set + 2 + r * 2));
  }
  return out;
}
/**
 * The glyphs reached from `glyphs`: through the substitutions of the features a page applies (APPLIED) and those their
 * contexts call, to a fixed point (context only picks which apply), and every component.
 */
function closure(tables, glyphs) {
  const { v, out: lookups } = lookupsOf(tables.get("GSUB"), 7), reached = new Set(glyphs), features = featuresOf(tables.get("GSUB"));
  const used = new Set([...features].flatMap(([tag, list]) => (APPLIED.has(tag) ? list : [])));
  for (const index of used) for (const at of [5, 6].includes(lookups[index].type) ? lookups[index].subtables : []) for (const other of called(v, lookups[index].type, at)) used.add(other);
  for (let size = -1; size !== reached.size; ) {
    size = reached.size;
    for (const [index, { type, subtables }] of lookups.entries()) for (const at of type <= 4 && used.has(index) ? subtables : []) {
      const format = v.getUint16(at), covered = coverage(v, at + v.getUint16(at + 2));
      if (type === 1) covered.forEach((g, i) => reached.has(g) && reached.add(format === 1 ? (g + v.getInt16(at + 4)) & 0xffff : v.getUint16(at + 6 + i * 2)));
      else if (type === 2 || type === 3) covered.forEach((g, i) => {
        if (!reached.has(g)) return;
        const seq = at + v.getUint16(at + 6 + i * 2);
        for (let k = 0; k < v.getUint16(seq); k++) reached.add(v.getUint16(seq + 2 + k * 2));
      });
      else if (type === 4) covered.forEach((g, i) => {
        if (!reached.has(g)) return;
        const set = at + v.getUint16(at + 6 + i * 2);
        for (let k = 0; k < v.getUint16(set); k++) {
          const lig = set + v.getUint16(set + 2 + k * 2), parts = v.getUint16(lig + 2);
          if (Array.from({ length: parts - 1 }, (_, c) => v.getUint16(lig + 4 + c * 2)).every((part) => reached.has(part))) reached.add(v.getUint16(lig));
        }
      });
    }
    for (const g of [...reached]) {
      const glyph = tables.get("glyf")[g];
      if (!glyph?.length || view(glyph).getInt16(0) >= 0) continue;
      const gv = view(glyph);
      for (let at = 10, more = true; more; ) {
        const flag = gv.getUint16(at);
        reached.add(gv.getUint16(at + 2));
        at += 4 + (flag & 1 ? 4 : 2) + (flag & 8 ? 2 : flag & 0x40 ? 4 : flag & 0x80 ? 8 : 0);
        more = Boolean(flag & 0x20);
      }
    }
  }
  return reached;
}

// ── Kerning: every pair of reached glyphs, as the kern feature's pair lookups give it (each lookup's first subtable
// that takes the pair; lookups add up), with its variation (a VariationIndex into GDEF's store, kept as it is) ──
function kerning(tables, kept) {
  const gpos = tables.get("GPOS"), v = view(gpos), features = v.getUint16(6), { out: lookups } = lookupsOf(gpos, 9), kern = new Set();
  for (let i = 0; i < v.getUint16(features); i++) {
    if (String.fromCharCode(...gpos.subarray(features + 2 + i * 6, features + 6 + i * 6)) !== "kern") continue;
    const at = features + v.getUint16(features + 6 + i * 6);
    for (let k = 0; k < v.getUint16(at + 2); k++) kern.add(v.getUint16(at + 4 + k * 2));
  }
  const sizes = (format) => [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80].filter((bit) => format & bit).length * 2;
  /** A ValueRecord's x advance and its variation (outer, inner), if any; its device offset from `base` (a pair set's, a class pair's subtable). */
  const value = (base, at, format) => {
    let x = 0, device = 0, o = at;
    for (const bit of [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80]) {
      if (!(format & bit)) continue;
      if (bit === 4) x = v.getInt16(o);
      if (bit === 0x40) device = v.getUint16(o);
      o += 2;
    }
    const d = device && base + device, variation = d && v.getUint16(d + 4) === 0x8000 ? [v.getUint16(d), v.getUint16(d + 2)] : null;
    if (format & ~0x44) for (const bit of [1, 2, 8, 0x10, 0x20, 0x80]) if (format & bit) throw new Error(`kerning with value format ${format}: only x advances are kept`);
    return { x, variation };
  };
  const pairs = new Map(), glyphs = [...kept].sort((a, b) => a - b), covers = new Map(), classes = new Map();
  const covered = (at) => covers.get(at) ?? covers.set(at, coverage(v, at)).get(at), classOf = (at) => classes.get(at) ?? classes.set(at, classDef(v, at)).get(at);
  for (const index of [...kern].sort((a, b) => a - b)) {
    const { type, subtables } = lookups[index];
    if (type !== 2) throw new Error(`kern lookup ${index} is of type ${type}: only pairs are kept`);
    for (const first of glyphs) for (const second of glyphs) {
      for (const sub of subtables) {
        const format = v.getUint16(sub), i = covered(sub + v.getUint16(sub + 2)).indexOf(first);
        if (i < 0) continue;
        const f1 = v.getUint16(sub + 4), f2 = v.getUint16(sub + 6), size = sizes(f1) + sizes(f2);
        let found = null;
        if (format === 1) {
          const set = sub + v.getUint16(sub + 10 + i * 2);
          for (let k = 0; k < v.getUint16(set); k++) if (v.getUint16(set + 2 + k * (2 + size)) === second) found = value(set, set + 4 + k * (2 + size), f1);
          if (!found) continue;
        } else {
          const c1 = classOf(sub + v.getUint16(sub + 8))(first), c2 = classOf(sub + v.getUint16(sub + 10))(second), count2 = v.getUint16(sub + 14);
          found = value(sub, sub + 16 + (c1 * count2 + c2) * size, f1);
        }
        if (f2) throw new Error("kerning that moves the second glyph: not kept");
        if (found.x || found.variation) {
          const key = first * 65536 + second, had = pairs.get(key);
          if (had && (had.variation || found.variation)) throw new Error(`pair ${first} ${second}: two kerns with variations`);
          pairs.set(key, { x: (had?.x ?? 0) + found.x, variation: found.variation ?? had?.variation ?? null });
        }
        break;
      }
    }
  }
  return pairs;
}

/** A GPOS of one kern lookup: one PairPos (format 1) of the pairs, each x advance with its variation. */
function gposOf(pairs) {
  const firsts = [...new Set([...pairs.keys()].map((key) => Math.floor(key / 65536)))].sort((a, b) => a - b), varied = [...pairs.values()].some((pair) => pair.variation);
  const format = varied ? 0x44 : 0x04;
  // The subtable: its header, its pair sets (each its variation indexes after it, each once: their offsets are the
  // set's own; shared by all, the font is 1.3 KB more), its coverage.
  const sub = new Writer();
  sub.u16(1), sub.u16(0), sub.u16(format), sub.u16(0), sub.u16(firsts.length);
  const setOffsets = sub.length;
  for (const _ of firsts) sub.u16(0);
  firsts.forEach((first, i) => {
    const set = sub.length, devices = new Map(), fixups = [];
    sub.set16(setOffsets + i * 2, set);
    const seconds = [...pairs].filter(([key]) => Math.floor(key / 65536) === first).map(([key, pair]) => [key % 65536, pair]).sort((a, b) => a[0] - b[0]);
    sub.u16(seconds.length);
    for (const [second, pair] of seconds) {
      sub.u16(second), sub.i16(pair.x);
      if (varied) fixups.push([sub.length, pair.variation]), sub.u16(0);
    }
    for (const [at, variation] of fixups) {
      if (!variation) continue;
      const key = variation.join(" ");
      if (!devices.has(key)) devices.set(key, sub.length - set), sub.u16(variation[0]), sub.u16(variation[1]), sub.u16(0x8000);
      sub.set16(at, devices.get(key));
    }
  });
  sub.set16(2, sub.length);
  sub.u16(1), sub.u16(firsts.length);
  for (const first of firsts) sub.u16(first);
  if (sub.length > 65535) throw new Error("kerning over one subtable's reach");
  // Header, ScriptList (DFLT and latn, each its default language: the one feature), FeatureList, LookupList.
  const out = new Writer();
  out.u16(1), out.u16(0), out.u16(10), out.u16(0), out.u16(0);
  const scripts = out.length;
  out.u16(2);
  for (const tag of ["DFLT", "latn"]) out.tag(tag), out.u16(0);
  const scriptAt = [];
  for (let i = 0; i < 2; i++) {
    scriptAt.push(out.length - scripts);
    out.u16(4), out.u16(0); // Script: default LangSys at 4, no others
    out.u16(0), out.u16(0xffff), out.u16(1), out.u16(0); // LangSys: no reordering, no required feature, feature 0
  }
  out.set16(scripts + 2 + 4, scriptAt[0]), out.set16(scripts + 2 + 6 + 4, scriptAt[1]);
  out.set16(6, out.length);
  const features = out.length;
  out.u16(1), out.tag("kern"), out.u16(8), out.u16(0), out.u16(1), out.u16(0);
  out.set16(8, out.length);
  out.u16(1), out.u16(4), out.u16(2), out.u16(0), out.u16(1), out.u16(8);
  out.bytes(sub.done());
  return out.done();
}

// ── The font anew ──
function cmapTable(map) {
  const cps = [...map.keys()].sort((a, b) => a - b), segments = [];
  for (const cp of cps) {
    const last = segments.at(-1);
    if (last && cp === last.end + 1 && map.get(cp) - cp === last.delta) last.end = cp;
    else segments.push({ start: cp, end: cp, delta: map.get(cp) - cp });
  }
  segments.push({ start: 0xffff, end: 0xffff, delta: 1 });
  const n = segments.length, w = new Writer();
  w.u16(0), w.u16(1), w.u16(3), w.u16(1), w.u32(12);
  const length = 16 + n * 8;
  w.u16(4), w.u16(length), w.u16(0), w.u16(n * 2);
  const search = 2 ** Math.floor(Math.log2(n)) * 2;
  w.u16(search), w.u16(Math.log2(search / 2)), w.u16(n * 2 - search);
  for (const s of segments) w.u16(s.end);
  w.u16(0);
  for (const s of segments) w.u16(s.start);
  for (const s of segments) w.i16(s.delta);
  for (const _ of segments) w.u16(0);
  return w.done();
}

/** gvar with only the kept glyphs' variations (the others none). */
function gvarOf(gvar, kept) {
  const v = view(gvar), count = v.getUint16(12), long = v.getUint16(14) & 1, shared = v.getUint32(8), data = v.getUint32(16);
  const at = (i) => (long ? v.getUint32(20 + i * 4) : v.getUint16(20 + i * 2) * 2);
  const head = gvar.subarray(0, 20), sharedTuples = gvar.subarray(shared, shared + v.getUint16(6) * v.getUint16(4) * 2);
  const w = new Writer();
  w.bytes(head);
  const offsets = w.length;
  for (let i = 0; i <= count; i++) w.u32(0);
  const tuplesAt = w.length;
  w.bytes(sharedTuples), w.pad(2);
  const dataAt = w.length;
  const ends = [0];
  for (let i = 0; i < count; i++) {
    if (kept.has(i)) w.bytes(gvar.subarray(data + at(i), data + at(i + 1))), w.pad(2);
    ends.push(w.length - dataAt);
  }
  const out = w.done(), ov = view(out);
  ov.setUint16(14, v.getUint16(14) | 1), ov.setUint32(8, tuplesAt), ov.setUint32(16, dataAt);
  ends.forEach((end, i) => ov.setUint32(offsets + i * 4, end));
  return out;
}

function writeWoff2(tables) {
  const tags = [...tables.keys()].sort(), sfnt = new Writer();
  // The sfnt total as it would be (header, directory, each table padded), as WOFF2's header says it.
  let total = 12 + 16 * tags.length;
  const all = new Writer();
  // (Packed end to end: WOFF2's tables aren't padded in its stream.)
  for (const tag of tags) (total += Math.ceil(tables.get(tag).length / 4) * 4), all.bytes(tables.get(tag));
  const packed = brotliCompressSync(all.done(), { params: { [constants.BROTLI_PARAM_QUALITY]: 11, [constants.BROTLI_PARAM_MODE]: constants.BROTLI_MODE_FONT, [constants.BROTLI_PARAM_SIZE_HINT]: all.length } });
  const dir = new Writer();
  const base128 = (n) => {
    const out = [];
    do out.unshift(n & 0x7f), (n = Math.floor(n / 128)); while (n);
    out.forEach((b, i) => dir.u8(i < out.length - 1 ? b | 0x80 : b));
  };
  for (const tag of tags) {
    const known = KNOWN.indexOf(tag), version = tag === "glyf" || tag === "loca" ? 3 : 0;
    dir.u8((version << 6) | (known < 0 ? 63 : known));
    if (known < 0) dir.tag(tag);
    base128(tables.get(tag).length);
  }
  const header = new Writer();
  header.u32(0x774f4632), header.u32(0x00010000), header.u32(0), header.u16(tags.length), header.u16(0), header.u32(total), header.u32(packed.length), header.u16(1), header.u16(0);
  for (let i = 0; i < 5; i++) header.u32(0);
  sfnt.bytes(header.done()), sfnt.bytes(dir.done()), sfnt.bytes(packed), sfnt.pad(4);
  const file = sfnt.done();
  view(file).setUint32(8, file.length);
  return file;
}

// ── The cut ──
const { tables, transformed } = readWoff2(readFileSync(SOURCE));
// (The core face's glyf is transformed, as fontTools wrote it: undone, its loca with it.)
if (!transformed.has("glyf")) throw new Error(`${SOURCE}: its glyf isn't transformed`);
tables.set("glyf", glyphsOf(tables.get("glyf")));
tables.delete("loca");
const cmap = cmapOf(tables.get("cmap")), wanted = new Map();
for (const [start, end] of FIRST) for (let cp = start; cp <= end; cp++) {
  if (!cmap.has(cp)) throw new Error(`${SOURCE} has no U+${cp.toString(16).toUpperCase()}`);
  wanted.set(cp, cmap.get(cp));
}
const kept = closure(tables, [0, ...wanted.values()]);
const glyphs = tables.get("glyf").map((glyph, i) => (kept.has(i) ? glyph : new Uint8Array(0)));
const glyf = new Writer(), loca = new Writer();
for (const glyph of glyphs) loca.u32(glyf.length), glyf.bytes(glyph), glyf.pad(4);
loca.u32(glyf.length);
const head = Uint8Array.from(tables.get("head"));
view(head).setInt16(50, 1);
view(head).setUint32(8, 0); // checksum adjustment: not kept (no checksums in WOFF2's tables)
const out = new Map(tables);
out.set("head", head), out.set("glyf", glyf.done()), out.set("loca", loca.done()), out.set("cmap", cmapTable(wanted)), out.set("gvar", gvarOf(tables.get("gvar"), kept)), out.set("GPOS", gposOf(kerning(tables, kept)));
const range = FIRST.map(([a, b]) => (a === b ? `U+${a.toString(16).toUpperCase()}` : `U+${a.toString(16).toUpperCase()}-${b.toString(16).toUpperCase()}`)).join(", ");
// (The page's face says FIRST's characters.)
const said = STYLES.match(/InterVariable\.first\.woff2[^}]*?unicode-range:\s*([^;]+);/)?.[1];
if (said !== range) throw new Error(`studio/index.html: the first face's unicode-range is ${said}, FIRST's ${range}`);
if (process.argv.includes("--check")) {
  // (Its tables, not its bytes: another brotli packs the same tables otherwise.)
  const had = readWoff2(readFileSync(OUT)).tables;
  if (had.size !== out.size || [...out].some(([tag, table]) => !had.has(tag) || Buffer.compare(had.get(tag), table))) throw new Error(`${OUT} isn't as first.mjs writes it: run it`);
  console.log(`Up to date: ${OUT}`);
} else {
  const file = writeWoff2(out);
  writeFileSync(OUT, file);
  console.log(`Written: ${OUT}, ${file.length} bytes, ${kept.size} glyphs of ${glyphs.length}, ${[...out.get("GPOS")].length} bytes of kerning; unicode-range: ${range}`);
}
