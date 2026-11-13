# 1461 - logotype in the packaging

Format: see `docs/decisions/README.md`.

- **Exe icon via `winresource` (new build dependency, needs the maintainer's yes).** Chosen: `winresource` (default features off) as a `cfg(windows)` build-dependency of `camcraft-ui`, called from the existing `crates/ui/build.rs`. Embedding a Windows exe icon needs a resource-compiler step; a hand-written `.rc` still needs `rc.exe`/`windres` and a `build.rs` call, so a small wrapper is the least code. A missing resource compiler only prints a `cargo:warning` and the build goes on (an exe without an icon is cosmetic, never a failed dev build). Dropped: a hand-rolled COFF resource writer (more code, more ways to be wrong).
- **macOS `.icns` from the generated sizes.** Each iconset entry uses the matching `camcraft-<size>.png`; only the 1024 px (512@2x) entry is scaled up from the 512 px one, since no 1024 px size is generated.
- **Linux: 256 px for the pixmap, flatpak, snap and AppImage; the `.deb`/rpm also install the hicolor 16-512 sizes.** Dropped: ImageMagick `convert` in the AppImage script (the 256 px file is already generated).
- **Wasm shell favicon left as is.** `site/static/favicon.png` serves the docs site, not the `/app/` shell, and replacing it is a site change; not part of this PR.
