## #2292 - Show the post script file path in the Post dialog

**Decided.** The desktop Post dialog's scope line (the weak small label at
the top right of the "Post" window) names the post script, and the full
path of a shipped script shows as a tool tip when the label is hovered:
`C:\...\assets\posts\fanuc.cps`.
It reuses the existing `posting::builtin_post_path` (which resolves
`assets/posts/<name>.cps` at run time from `camcraft_post::data_dir`, and
returns `None` on the browser build) through a new `posting::post_path`
helper. A user file already names itself through `posting::post_ref_label`
(the label is the path), so `post_path` returns `None` for it and only
registry entries (built-in / named) get the tool tip; a registry entry
that is gone gets none, since `script_source` refuses it anyway.

**Why.** Issue #2292 asks that a user can see which post script will run.
The path is already available (`posting::builtin_post_path`) but was never
shown. Showing it in the dialog scope line keeps the change small, touches
no posted output, and leaves the program header byte-identical so no
goldens change.

**Considered and dropped.**
- A dedicated "Script path" row in the Setup -> Post section
  (`crates/ui/src/egui_app/pages.rs`): redundant, since the post dialog is
  where a user reviews what will post, and a row there would duplicate the
  file name a user file already shows as its label.
- Showing the path for user files too: `post_ref_label` already returns
  the file path as the label, so `post_path` returns `None` for a user
  file and the tool tip stays unset.
- Computing the path from `BUILTINS` text (`assets/posts/<name>.cps`)
  instead of `posting::builtin_post_path`: the disk-resolved path is the
  single source of truth and already handles the wasm `None` case.

**Scope notes.** No command, MCP tool, project file, or posted output
changes. `posting::post_path` is `pub` so other panes can reuse it.

**Cost to change later.** Low. One small pure helper in `posting.rs` and
one label line in `post_dialog.rs`.
