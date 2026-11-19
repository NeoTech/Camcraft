#!/usr/bin/env python3
"""Build the bundled Simplified Chinese UI font (#853).

Downloads (or takes) Noto Sans SC, pins it to Regular 400, and cuts it down to
the glyphs the interface can need:

  ASCII, Latin-1, general punctuation, CJK punctuation, fullwidth forms,
  letterlike symbols, arrows, geometric shapes, and the 6 763 hanzi of GB2312.

That is roughly 7 000 glyphs, about a third of the full font's file. A user
typing a rare hanzi outside GB2312 gets the fallback box; the catalogue and
every shipped string are inside the set, which the coverage test checks.

Usage:
    python3 scripts/fonts/subset_sc.py --source NotoSansSC-VF.ttf \
        --out assets/ui/fonts/NotoSansSC-Regular-Subset.ttf

Requires: pip install fonttools brotli
Provenance: our own build script. The font is Noto Sans SC, SIL OFL 1.1;
its licence rides beside it as assets/ui/fonts/OFL.txt.
"""
import argparse
import pathlib
import sys

RANGES = [
    (0x0020, 0x007E),  # ASCII
    (0x00A0, 0x00FF),  # Latin-1 (degree, plus-minus, micro, diameter sign)
    (0x2000, 0x206F),  # general punctuation (en/em dash, quotes, ellipsis)
    (0x20A0, 0x20CF),  # currency
    (0x2100, 0x214F),  # letterlike (numero, ohm)
    (0x2190, 0x21FF),  # arrows
    (0x2200, 0x22FF),  # maths operators
    (0x2500, 0x257F),  # box drawing
    (0x25A0, 0x25FF),  # geometric shapes (bullets, triangles)
    (0x3000, 0x303F),  # CJK punctuation
    (0xFF00, 0xFFEF),  # fullwidth forms
]


def gb2312_hanzi():
    out = []
    for hi in range(0xB0, 0xF8):
        for lo in range(0xA1, 0xFF):
            try:
                out.append(bytes([hi, lo]).decode("gb2312"))
            except UnicodeDecodeError:
                pass
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--source", required=True, help="Noto Sans SC, .ttf or .otf")
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    from fontTools.ttLib import TTFont
    from fontTools.varLib import instancer
    from fontTools import subset

    src = pathlib.Path(args.source)
    out = pathlib.Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)

    font = TTFont(src, lazy=True)
    if "fvar" in font:
        font = instancer.instantiateVariableFont(font, {"wght": 400}, inplace=True)

    unicodes = set()
    for lo, hi in RANGES:
        unicodes.update(range(lo, hi + 1))
    unicodes.update(ord(c) for c in gb2312_hanzi())

    options = subset.Options()
    options.layout_features = ["*"]       # keep the Latin kerning and ligatures
    options.name_IDs = ["*"]              # keep the name table intact
    options.notdef_outline = True
    options.recalc_bounds = True
    options.drop_tables += ["DSIG"]
    subsetter = subset.Subsetter(options=options)
    subsetter.populate(unicodes=unicodes)
    subsetter.subset(font)

    font.flavor = None
    font.save(out)
    print(f"{out}: {len(unicodes)} code points, {out.stat().st_size / 1e6:.2f} MB")
    return 0


if __name__ == "__main__":
    sys.exit(main())
