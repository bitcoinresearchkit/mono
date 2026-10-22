#!/bin/sh
# Studio's fonts, cut to what its page writes (ranges.txt: Latin, punctuation, currency, arrows, math, keys, boxes; any
# other script in a title, the system's font), every feature, axis, glyph name and name record kept (the license with
# them; without the glyph names, macOS draws a canvas's text a little differently). The build inlines these, and stops
# on a character of the page outside the ranges: add its block there and run this again. Needs fontTools
# (`python3 -m pip install fonttools brotli`).
set -e
cd "$(dirname "$0")"
for font in InterVariable Lilex; do
  pyftsubset "../../assets/fonts/$font.woff2" --unicodes-file=ranges.txt --layout-features='*' --glyph-names --name-IDs='*' --flavor=woff2 --output-file="$font.woff2"
done
