## #4454 - Windows: the UI config path of `plan_notes` uses one separator

**Decided:** `unit_env::plan_notes` builds the UI directory as
`root.join("crates").join("ui")`. The test builds its expected config path by
components too. A source pin, `plan_notes_join_the_ui_directory_by_components`,
keeps the production join in that form.

**Why:** on Windows, `Path::join("crates/ui")` keeps the `/` in the path. The
note then shows `...\crates/ui\.cargo\config.toml`. The test expected
`...\crates/ui/.cargo\config.toml`. The two strings differ, so the test failed
on Windows only. With joins by components, both sides show only `\`, and the
printed note is a clean native path for the user too.

**Dropped:**
- A fix in the test only (compare with `Path` values, or normalize the
  separators). It hides the mixed separator in the printed note.
- A change to `config::notes` to normalize every path. That is a larger change
  and it touches the code of #4427 and #4428 (the same kind of failure in
  `config.rs` tests). Those issues stay separate.

**Cost to change later:** none. It is a test helper path and one join.
