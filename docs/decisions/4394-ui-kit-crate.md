## #4394 - The UI kit crate `camcraft-ui-kit` and the edge `ui -> ui-kit`

**What was decided.** A new crate `crates/ui-kit` (`camcraft-ui-kit`)
holds the design system of the shell. This PR moves three things into
it, unchanged in behaviour:

- the theme token table `Tokens` with its two built-in columns, its
  clamps (`theme::clamp`), its exhaustiveness guard, `DropGeom`,
  `ribbon_h`, `status_h`, `Tokens::visuals`, `Tokens::style` and
  `apply_to`;
- the `camcraft-theme/1` reader (`theme_files`: `Theme`, `Cache`, `load`,
  `discover`, `parse_text`, the caps, `COLOR_NAMES`, `GEOM_NAMES`);
- the contrast arithmetic (`contrast`: `luma`, `ratio`, `on_fill`,
  `follow`, `follow_states`, `readable`, `BODY`, `UI`) and
  `light_ground`, which moved from `ground_ink` because the visuals and
  the file overlay both call it.

The kit depends on `egui` (the workspace pin `=0.36.2`) and
`serde_json` only. It has no `camcraft-*` dependency. The optional
feature `gallery` adds `eframe` (same pin) for the gallery window
`ui-kit-gallery`; tests do not build it.

`camcraft-ui` gets the new edge `camcraft-ui-kit`. It keeps the old
module paths as re-exports (`egui_app::contrast`, `egui_app::theme`,
`egui_app::theme_files`, `ground_ink::light_ground`), so no other file
of the crate changes, and open UI PRs keep compiling.

**Imported items (the edge `ui -> ui-kit`).** `camcraft-ui` uses about
thirty items of the kit: `Tokens` (with `DARK`, `LIGHT`, `visuals`,
`style`, `dropdown`, `strip_fill`), `apply_to`, `ribbon_h`, `status_h`,
`DropGeom`, the `clamp` module and its ranges, the nine `contrast`
items above, and the `theme_files` items above plus `ThemeSource` and
`resolve_source`. That is far above the five-item floor of AGENTS.md.

**Why the caller cannot pass the values in.** These items are not values
that a caller could hand over as a parameter or a closure. They are the
types and the functions that every draw site of the shell reads: the
token table is the type of the theme itself. The maintainer asked for
the UI/UX work in a crate, "one crate for the design system only
(tokens, theme and components)" (#4245, 2026-11-27). The edge is the
point of the change: the dependency graph now proves that the design
system cannot read the document, because the kit cannot see
`camcraft-app`.

**What stays in `camcraft-ui`, and why.**

- The live theme cell (`theme::current`, `theme::set_current`,
  `theme::apply`) and its tests. Its unit-test form is a thread-local
  (#2373), chosen by `#[cfg(test)]`. That `cfg` applies per crate: in the
  kit it would be off while `camcraft-ui` runs its tests, and parallel
  UI harness threads would share one theme again. The cell is shell
  state, not design system, so it stays where the shell runs.
- The map from the document's `ThemeRef` to the kit's `ThemeSource`
  (`theme_files::resolve`). The kit has its own small enum, so it does
  not need `camcraft-app`.

**What rebuilds when the kit changes.** `camcraft-ui-kit` and
`camcraft-ui` (its lib, its binary and its test binaries). No other
crate depends on either, so a kit change rebuilds no engine crate, no
`camcraft-app` and no flecs. A change in `camcraft-ui` does not rebuild
the kit. `cargo test -p camcraft-ui-kit` builds egui and serde_json
only.

**Tests.** The four contrast tests moved from
`crates/ui/tests/theme_files.rs` to `crates/ui-kit/tests/contrast.rs`
unchanged. The kit adds `tests/gallery.rs` (four tests). `camcraft-ui`
adds four tests for the seam (`egui_app/theme_files/resolve_tests.rs`:
the `ThemeRef` map and the old paths), so its test count stays the same. The colour-literal guard of `camcraft-ui` loses the rows of
`contrast.rs` and `theme/tables.rs`, because those files are no longer
in that crate.

**Gate.** The gate has fixed crate lists (`xtask/src/plan.rs`:
`ALL_CRATES`, `BATCH_CRATES`, `CLIPPY`); it does not discover crates.
Without an entry, a diff in `crates/ui-kit` would select no test and no
clippy run. The PR adds `ui-kit` to the three lists, before `ui`
(dependency order). This is a process file change.

**Considered and dropped.**

- Move the live cell into the kit behind a feature that `camcraft-ui`
  turns on as a dev-dependency. Dropped: Cargo unifies the feature for
  all test targets of `camcraft-ui`, so its integration tests would
  change from one process-wide theme to a thread-local one. That is a
  behaviour change in a PR that must have none.
- Move the layout widgets, the ribbon cell and the icons now (research
  note 2 on #4245). Dropped: the issue asks for no big move. Later PRs
  move each component in the PR that touches it.
- Keep `ThemeRef` in the kit's signature. Dropped: it needs
  `camcraft-app`, which the kit must never see.

**Cost to change later.** Small. The re-exports in `camcraft-ui` can go
when the last caller uses the kit paths directly; that is a mechanical
path change. Removing the crate again means moving the files back and
deleting the edge.
