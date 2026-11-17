## #2189 - Per-post `endUnits` manifest, refusal on undeclared scripts, and UI picker gating

**What was decided.**

- Each post script declares its `postOptions.EndUnits` contract with a top-level manifest string `endUnits` parsed by `camcraft_post::post::end_units`:
  - `"G20 G21"` on RS-274 milling posts (`tormach.cps`, `grblhal.cps`, `tinyg.cps`, `fanuc.cps`, `mazak.cps`, `doosan-dnm.cps`) and `"G700 G710"` on `sinumerik.cps`, which emit the matching unit word after the final retract and right before `M30` / `M2` whenever `EndUnits` differs from the program unit.
  - `"note"` on milling posts whose control/firmware cannot switch units at program end (`haas.cps` because Setting 9 alarms on G20/G21 mismatch; `carvera.cps` and `heidenhain.cps` because they are mm-only), which emit a one-time `END UNITS NOT WRITTEN ...` note via `note()` when asked to leave a different unit (never `G20`/`G70`).
  - Absent on non-mill posts (`fanuc-lathe.cps`, `tormach-lathe.cps`, `tormach-plasma.cps`, `wire-iso.cps`, `sodick-wire.cps`, `sinker-iso.cps`) and undeclared user `.cps` files. `engine::run_program_with` calls `end_units::refusal` before any hook runs and refuses by post name when `EndUnits` is `"inch"` or `"mm"` on a script without `endUnits`.
- `camcraft_app::posting::end_units_words` exposes `end_units::words(&cps)` so the Post dialog (`post_dialog.rs` / `post_dialog_tabs.rs`) and `Setup -> Post` (`post_options.rs::end_units_picker`) offer the End units selector only when a post declares two unit words, using those words in the labels (`Inch (G700)` / `mm (G710)` vs `Inch (G20)` / `mm (G21)`). When an unsupported post inherits a non-`Leave` `end_units` value (from an earlier post or MCP), the UI shows a one-click reset (`Clear to Leave modal` / `Reset to Program's unit`) instead of silently dropping or offering impossible choices.
- `set_post_options` on MCP continues to accept `end_units` across posts so callers switching posts or targeting `"note"` posts are not blocked before posting, while `post` writes, notes, or refuses by name.

**Why.** Offering `"Inch (G20)"` / `"mm (G21)"` unconditionally in the UI while 7 of 10 milling posts ignored `postOptions.EndUnits` silently violated the engine's "honor, note once, or refuse by name" rule, and hardcoding `"Tormach PathPilot"` in `end_units_picker` hid a working option on `grblhal` and `tinyg` while `post_dialog_tabs` leaked `"G20"`/`"G21"` labels even on `sinumerik` (`G700`/`G710`). Putting the contract in the `.cps` manifest matches `programUnits` and `explains` and works for user `.cps` files without hardcoding post names in UI or app code.

**Considered and dropped.**

- Making `haas.cps` emit `G20`/`G21` or refusing Haas/Carvera/Heidenhain instead of noting. Dropped: Haas alarms when G20/G21 conflicts with Setting 9, and Carvera/Heidenhain mm-only posts work normally for the program itself; noting once before program end explains why the modal was left alone without risking a machine alarm or blocking a valid program.
- Keeping `set_post_options` / `SetPostOptions` as a setter refusal on `"note"` or undeclared posts. Dropped: changing posts temporarily or setting document defaults would either block legitimate post switches or require hidden clearing; gating the UI picker to writing posts while refusing undeclared scripts at post time keeps both interactive and scripted workflows predictable.
- Adding a hard coded list of post names in `crates/ui`. Dropped: violates clean-layer `.cps` self-description and fails for user `.cps` scripts.

**Cost to change later.** Self-contained across `camcraft_post::post::end_units`, one helper `posting::end_units_words`, the `.cps` `endUnits` lines + footer hooks, and two UI call sites; switching a `"note"` post to emit words later is adding `endUnits = "<words>"` and its footer line.
