## #1428 - Restore `assets/ui/cube.png` for packaging; pin packaging paths with an xtask test

**What was decided.** `assets/ui/cube.png` is restored byte for byte from
`v0.18.0` (our own 240 x 240 original, 4584 bytes). No packaging script,
`release.yml` line or manifest changes: `package-appimage.sh`,
`package-flatpak.sh`, `package-linux.sh`, `package-macos.sh` and
`package-snap.sh` keep reading the same path. A new xtask test
(`xtask/src/package_paths_tests.rs`) reads those scripts and
`release.yml` as text and fails on any OS when a literal `assets/...` path
they name, or an asset directory they copy, is missing from the tree.
`PROCESS_VERSION` goes 56 -> 57 (PR #1426 also takes 57; whichever merges
second takes the next number).

**Why.** Commit 6792a8a3f (#661 slice 2, merged by PR #1179, in v0.19.0)
deleted the file when the tree icons were redrawn, but the application
icon for the desktop packages was still read from it; the v0.19.0 linux and
macos jobs failed on it. The app itself does not load the file, and no other
icon in `assets/ui` is above 32 px, so there is nothing larger to point the
scripts at; upscaling a 32 px tree glyph to a 256 px launcher icon would be
a worse product icon than the original.

**Considered and dropped.** Pointing the scripts at `tree-part.png` (32 px,
blurry at 256 px, and a model glyph, not a product icon). Drawing a new SVG
master and rasterising it (new art, larger than this P2 fix; can follow as
its own issue if the maintainer wants a redesigned launcher icon). Checking
by running the packaging scripts: they need Linux/macOS tools, so the text
scan is the check that runs everywhere.
