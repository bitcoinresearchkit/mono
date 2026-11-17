# Splits each of the pages' text fonts in two, each with the axes the pages draw at, the features they reach and every
# glyph those reach from its characters: a core file for what the pages write (Latin with its accents, punctuation,
# arrows, math, the keys' and boxes' symbols), and the rest, which a browser fetches only for a character of its own (a
# name or a note in another script). Writes each file's unicode-range into every page's @font-face rule that names it,
# so the pages' text renders as before and a page's first visit fetches about a third of the bytes; stops, naming the
# file, if the pages' sources (TEXT, as written: escapes, live data and typed text aren't seen) hold a character outside
# CORE or ask for a feature outside FEATURES. What a split can't keep: kerning or a mark's place between a core and a
# rest character (a Greek word with Σ, a Cyrillic letter with an accent, a combining mark typed apart from its letter);
# all of Greek in the core would keep Greek words whole for +20 KB on Inter's core and +5 KB on Lilex's.
#
#   uvx --from 'fonttools[woff]==4.68.0' python showcases/assets/fonts/subset.py
#
# Run again when a source font (NOTICE.md says where each comes from), CORE, FEATURES, FLOOR or a page's text changes.
import re
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

HERE = Path(__file__).parent
SHOWCASES = HERE.parents[1]
ROOT = SHOWCASES.parent
# Each font, and the axis it no longer varies on under its default, as no page draws there: both under 400 (Studio's
# Lilex starts at 500), -23 KB on Inter's core and -12 KB on Lilex's, their deltas over 400 kept as they are (fontTools'
# instancer would round them, up to 0.7 units off).
FLOOR = {"InterVariable": "wght", "Lilex": "wght"}
# Every character the pages and their modules write, by range, with Latin Extended-A: a name's letters and accents
# kerned within one file (+8 KB on Inter's core, +4 KB on Lilex's). Combining marks are left to the rest: no page writes
# its accents apart (é, not e + ◌́), and text that does still draws as before, Chromium and WebKit composing it into the
# core's é once the rest is in; in the core, the marks would bring the ccmp-composed letters of every other Latin block
# (Vietnamese, Latin Extended-B and Additional) with them, +24 KB on Inter's.
CORE = [
    (0x0000, 0x017F), (0x0192, 0x0192), (0x02BB, 0x02BC), (0x02C6, 0x02C6), (0x02DA, 0x02DA), (0x02DC, 0x02DC),
    (0x0394, 0x0394), (0x03A3, 0x03A3), (0x03C3, 0x03C3), (0x2000, 0x206F), (0x20AC, 0x20AC), (0x20BF, 0x20BF),
    (0x2122, 0x2122), (0x2190, 0x23FF), (0x2500, 0x25FF), (0x2713, 0x2715), (0xFE0E, 0xFE0F), (0xFEFF, 0xFEFF),
    (0xFFFD, 0xFFFD),
]
# The OpenType features kept: those browsers apply by default (fontTools' list: kerning, contextual alternates, marks,
# local forms...) and those the pages ask for, by font-feature-settings or a font-variant value (VARIANTS); the
# other alternates (Inter's cv01-cv13, ss02-ss08, case, sups, dlig...) are 29 KB of Inter's core, 5 KB of Lilex's.
FEATURES = [*subset.Options().layout_features, "tnum", "cv11", "ss01"]
VARIANTS = {"normal": [], "tabular-nums": ["tnum"]}
# Where the pages' words come from: their own files, and the modules that write text on them (series names, chart
# labels, colour names; the rest of modules/ draws none: search, QR codes, the API reference's own fonts).
TEXT = [
    *(p for p in SHOWCASES.rglob("*") if p.suffix in {".html", ".js", ".mjs", ".css"} and "modules" not in p.relative_to(SHOWCASES).parts),
    *(ROOT / "modules/bitview-client").glob("*.js"),
    *(ROOT / "modules/bitview-charts").glob("*.js"),
    *(ROOT / "modules/xkcd-colors").glob("*.js"),
]


def in_core(cp):
    return any(a <= cp <= b for a, b in CORE)


def css(ranges):
    return ", ".join(f"U+{a:04X}" if a == b else f"U+{a:04X}-{b:04X}" for a, b in ranges)


def runs(cps):
    """Codepoints as ranges, a gap of a few bridged where it holds no core character (the font has nothing there); a
    longer one kept out, as a character the font lacks would fetch the file for nothing."""
    out = []
    for cp in sorted(cps):
        if out and cp - out[-1][1] <= 16 and not any(in_core(c) for c in range(out[-1][1] + 1, cp)):
            out[-1][1] = cp
        else:
            out.append([cp, cp])
    return out


texts = {path.relative_to(ROOT): path.read_text("utf-8") for path in TEXT}
outside = sorted({f"{path}: {ch} U+{ord(ch):04X}" for path, text in texts.items() for ch in text if ord(ch) > 127 and not in_core(ord(ch))})
if outside:
    raise SystemExit("Characters the pages write outside CORE (add them to it):\n  " + "\n  ".join(outside))
# (CSS and JS spellings: font-feature-settings: 'tnum'; featureSettings: '"tnum" 1'; style.fontVariantNumeric = "…".)
SETTINGS = r"(?:font-feature-settings|[fF]eatureSettings)\s*[:=]\s*([^;}\n]*)"
VARIANT = r"(?:font-variant[\w-]*|fontVariant\w*)\s*[:=]\s*[\"']?([\w -]+?)(?:[\"'](?!\w)|\s*(?=[;},\n]))"
asked = [
    *((path, tag) for path, text in texts.items() for rule in re.findall(SETTINGS, text) for tag in re.findall(r"[\"'](\w{4})[\"']", rule)),
    *((path, tag) for path, text in texts.items() for value in re.findall(VARIANT, text) for word in value.split() for tag in VARIANTS.get(word, [f"font-variant {word}"])),
]
outside = sorted({f"{path}: {tag}" for path, tag in asked if tag not in FEATURES})
if outside:
    raise SystemExit("Features the pages ask for outside FEATURES (add them to it, a font-variant value to VARIANTS):\n  " + "\n  ".join(outside))

ranges = {}
for name, floor in FLOOR.items():
    source = HERE / f"{name}.woff2"
    cmap = TTFont(source).getBestCmap()
    parts = {"core": [cp for cp in cmap if in_core(cp)], "rest": [cp for cp in cmap if not in_core(cp)]}
    ranges[f"{name}.core.woff2"], ranges[f"{name}.rest.woff2"] = css(CORE), css(runs(parts["rest"]))
    for part, cps in parts.items():
        font = TTFont(source, recalcTimestamp=False)  # (the same bytes on every run)
        subsetter = subset.Subsetter(subset.Options(layout_features=FEATURES, notdef_outline=True))
        subsetter.populate(unicodes=cps)
        subsetter.subset(font)
        if floor:
            axis = next(axis for axis in font["fvar"].axes if axis.axisTag == floor)
            axis.minValue = axis.defaultValue
            font["fvar"].instances = [i for i in font["fvar"].instances if i.coordinates[floor] >= axis.defaultValue]
            for variations in font["gvar"].variations.values():
                variations[:] = [v for v in variations if v.axes.get(floor, (0, 0, 0))[1] >= 0]
        target = HERE / f"{name}.{part}.woff2"
        font.flavor = "woff2"
        font.save(target)
        print(f"{target.name}: {target.stat().st_size} bytes, {len(cps)} characters")


def ranged(rule):
    """An @font-face rule naming one of the files, its unicode-range that file's."""
    rule = rule.group(0)
    named = [file for file in ranges if f"/{file}" in rule]
    if not named:
        return rule
    if len(re.findall(r"unicode-range:[^;]*;", rule)) != 1:
        raise SystemExit(f"An @font-face rule for {named[0]} without one unicode-range: {rule}")
    return re.sub(r"unicode-range:[^;]*;", f"unicode-range: {ranges[named[0]]};", rule)


for page in (p for p in TEXT if p.suffix == ".html"):
    text = page.read_text("utf-8")
    written = re.sub(r"@font-face\s*\{[^}]*\}", ranged, text)
    if written != text:
        page.write_text(written, "utf-8")
        print(f"ranges written: {page.relative_to(ROOT)}")
