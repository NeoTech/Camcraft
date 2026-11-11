## #741 - the starter tool catalog is a JSON asset

**Decided.**
- The catalog is `assets/tools/catalog.json`, a `camcraft-tools/1` library
  (the #300 envelope, the `ToolDto` definition keys), the only source of the
  starter tools. `catalog()` keeps its signature, names and order.
- **Layering: the tools crate reads it itself.** `camcraft-tools` gained a
  small strict reader (`catalog_asset.rs`: serde `deny_unknown_fields`
  definitions, format/version/units/name/duplicate/`is_sane` checks, errors
  name the tool), parsed once into a `OnceLock`. `serde` and `serde_json`
  are new dependencies of that crate (already in the workspace and
  `Cargo.lock`; no new crate downloaded).
- **Shipped embedded** with `include_str!`, like `assets/posts/*.cps`: the
  binary works with no file beside it, and the release zip needs no change
  (`xtask` `ASSET_DIRS` untouched).
- A parse failure of the embedded asset (a build bug the tests pin) is
  logged to stderr and yields an empty catalog - never a panic.

**Why.** Adding or tuning a starter tool no longer needs Rust edits, and
there is one tool format. The app's `parse_tool_library` reads the same file;
a test in `crates/app/tests/tool_library.rs` pins that both readers agree.

**Considered, dropped.**
- Moving `ToolDto`/`tool_from_json` into the tools crate: the DTO carries the
  `.camproj` body wire (`ToolBodyDto`, tool numbers) and is app-coupled; too
  large a move for a data change.
- Load in `app` and pass down / install into `tools` at startup: the
  headless `camcraft-tools` bin and the tools crate's own tests call
  `catalog()` with no app, and an unset hook would be a silent empty catalog.
- Runtime read from `assets/`: breaks "works with no files next to it".

**Equality.** Before the Rust table was deleted a test compared the JSON
catalog to it, all 28 tools field by field (Debug-equal): identical. The
permanent pins are in `crates/tools/src/catalog_asset_tests.rs` (order,
per-kind values, bad entries).

**Cost to change.** A user catalog override (the issue's follow-up) is a
second `parse_catalog` call on a user file; not done here.
