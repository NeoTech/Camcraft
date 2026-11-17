## #2282 - Post scripts and machine files are read from disk at run time

**Decided.** The desktop executable holds no compiled-in post script or
machine file. `camcraft_post::data_dir` finds `assets/<kind>` at run time,
and `posting::builtin_post_text` / `builtin_machine_text` /
`default_mch_text` read each file once into a static cell. The registry
tables (`posting::BUILTINS`, `posting::MACHINES`) keep the names and now
hold the file name (`AssetFile`), not the text.

- Lookup order, first existing directory wins for the whole kind:
  1. `<exe dir>/assets/<kind>` (zip, installer, Linux tarball, deb/rpm,
     AppImage, flatpak, snap);
  2. `<app>/Contents/Resources/assets/<kind>` (macOS bundle);
  3. `<exe dir>/../../assets/<kind>` (dev run from `target/<profile>/`);
  4. `<crates/post>/../../assets/<kind>` (test binaries in
     `target/*/deps/`; a source path, not a compiled-in copy).
- Candidates 3 and 4 apply only when the exe is in a cargo build tree:
  the exe folder (or the folder above `deps` / `examples`) holds cargo's
  `.fingerprint` folder. An installed or packaged exe without its
  `assets/<kind>` folder gets the named refusal. It never reads the build
  machine's checkout (`CARGO_MANIFEST_DIR` is compiled into every
  binary), `/usr/assets` or `C:/assets`.
- No directory, or a file that is missing, not UTF-8 or over 4 MiB, is an
  error that names the file and the directory. There is no compiled-in
  fallback on the desktop: a silent substitute would hide which script
  made the G-code.
- The browser build (`wasm32`) has no disk and keeps a compiled-in copy.
- Each read logs the full path once (`read shipped posts file ...`), so a
  user can see which post file made a program.
- The text lives in static `OnceLock` cells, so callers keep `&'static
  str`. This is not a leak: one bounded copy per shipped file for the
  life of the process, as before with `include_str!`.

**Why.** The maintainer (2026-11-16) asked that the machine files, post
scripts, themes and scripts be read from disk and not compiled in. The
release packages already ship `assets/{machines,posts,scripts,themes,ui}`
beside the executable (checked against the v0.22.0 Windows zip and
`xtask/src/release_pack.rs` `ASSET_DIRS`), so no packaging change is
necessary.

**Considered and dropped.**
- A per-file merge of several directories (a user file overrides one
  shipped file): two sources for one kind make it hard to say which file
  posted. Whole-directory precedence is simpler to explain.
- A compiled-in fallback on the desktop: rejected by the issue.
- `Box::leak` to get `&'static str`: AGENTS.md calls a leak a critical
  bug; static cells give the same lifetime without it.
- An environment override (like `CAMCRAFT_THEMES`): not asked for.
- A per-user `<data>/camcraft/assets/<kind>` folder (review of PR #2293):
  removed. #2282 does not ask for it. It never won for an intact
  package (candidate 1 or 2 always exists there), but it came before the
  dev candidates, so a stale copy silently replaced the repository
  scripts for `cargo run` and for every golden test on that machine. The
  other option (put it last and log at `warn`) keeps a place for a stale
  copy with no use; dropped.
- `cfg(debug_assertions)` to gate the dev candidates: dropped, because
  the release-tier lanes run test binaries in release. The
  `.fingerprint` marker works in both profiles and with a custom
  `CARGO_TARGET_DIR`.

**Scope notes.**
- Themes were already read from disk (`crates/ui/src/theme_dir.rs`). The
  built-in dark and light themes are Rust token constants, not files.
- `assets/scripts` (`cam.d.ts`, `example-batch.js`) was never compiled in.
- `assets/tools/catalog.json` (`crates/tools/src/catalog_asset.rs`) is
  still compiled in: it is the tool library, outside this issue.
- `include_str!` in tests stays: a test is not the shipped executable.
- The user machine folder `<data>/camcraft/machines` is unchanged.

**Cost to change later.** Low. The lookup is one module with pure
`candidates`/`resolve` functions; adding a candidate or an environment
override is a few lines and a test.
