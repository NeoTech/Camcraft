## #2074 - MCP post tools: list_posts, select_post, preview_post, save_post_script

**Decided.** The MCP server gets four post tools in
`crates/app/src/mcp/post_tools.rs`, with the user posts folder in
`crates/app/src/mcp/post_files.rs`:

- `list_posts` lists each built-in post (registry name, the machine kinds
  that `posting::post_serves` gives it, and the manifest fields
  `description`, `programUnits`, `extension`, `declaredOptions`) and each
  `.cps` file of the user posts folder (the same fields, and `serves` as the
  manifest states it, or `null`). A script that does not load gives a
  `problem`, never a guessed field.
- `select_post` runs `SelectPost` through the session History: one undo
  step, with the command's own front-door checks.
- `preview_post` runs the engine on a built-in preview sample (on the
  default VMC) or on the document stream (on the document's machine). It
  writes no file. A script failure is an `ok: false` answer with `errors`,
  not a tool error, so that an author sees the 1-based line in the script.
- `save_post_script` writes a plain file name into the user posts folder
  with `machine::store::write_atomic`.

**The user posts folder** is `CAMCRAFT_POSTS`, else
`<platform data dir>/camcraft/posts`. This is the order of the user
machines folder (`CAMCRAFT_MACHINES`, the #287 decision). No folder existed
before: the GUI reads `.cps` files from any path. A later GUI picker can
use the same folder.

**A save must load and run first** (on an empty stream, as `SelectPost`
validates a file). A broken script is not written. Dropped: save any text.
That would put files in the folder that `select_post` then refuses.

**Live sessions ask first.** `save_post_script` is a `FileWrite` and
`select_post` is a `LimitEdit` in `mcp::live::requires_approval`, as
`save_project` and `set_setup_post` are. `preview_post` with `path` is a
`FilePathAccess` by the general rule. `preview_post` with `source` and
`list_posts` change nothing and need no approval.

**Tests point the folder at a temp folder** through a hidden
`Server::set_user_posts_dir` hook (an `Env` field), as
`Server::hold_generations` does for its test. Dropped: set `CAMCRAFT_POSTS`
in the test process. `std::env::set_var` is `unsafe` in edition 2024, and
the tests of one binary run in parallel.

**An empty document refuses a document preview** ("no operations to
post"), the same refusal that `post` gives. The samples are the way to
preview a script without a job.

**Cost to change later.** The folder and the file-name rule are in one
small file. A GUI that saves posts can move `post_files.rs` out of `mcp/`
with no change to the tools.
