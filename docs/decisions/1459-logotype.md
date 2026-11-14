## #1459 - logotype as application icon

**Decided.** The maintainer's PNG is committed as given (`assets/logo/camcraft-logo.png`); no SVG
trace. `cargo xtask logo` pads it to a square with its own corner colour and box-filters it to
16-512 px PNGs plus a PNG-compressed `camcraft.ico` under `assets/logo/icons/`; a unit test fails when
the committed files are stale. The window icon is the 128 px PNG embedded in the ui crate.

**Why.** resvg's tiny-skia (already in xtask) decodes, encodes and holds the pixels, and the resize and
ICO container are about 60 lines: no new dependency. Padding rather than cropping or stretching keeps
the artwork unaltered.

**Dropped.** ImageMagick/`sips` per packaging script (platform tools, already how packaging works but
not reproducible); the `image` crate in xtask (a new edge for what tiny-skia does).

**Not done here (scripts/, build.rs, .github): Windows exe/installer icon, macOS .icns, Linux
desktop icons.** The packaging scripts still use `assets/ui/cube.png`; follow-up issue filed.
