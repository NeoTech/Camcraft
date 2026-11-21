## #2950 - Browser build: load machine (.mch) and post (.cps) files from the user's disk

**What was decided.**

- A session store in `crates/app/src/session_files.rs` keeps the text of each `.mch` and `.cps` that the browser read, keyed by the bare file name, for this run of the app only.
- The store is thread-local. The browser runs the app on one thread. A process-global store leaked the session machines of one test into the machine-pane tests on other threads (quick gate, ui lib red). A reader on another thread does not find a session name, so the disk read fails and the caller refuses that name.
- A file enters the store only when it passes the checks of a file from disk. A machine must validate and parse. A post must compile and run on an empty stream (the `SelectPost` check).
- The readers that read a machine or post file by path consult the store first, then the disk: `posting::script_of`, `posting::resolve_cps`, `SelectPost` validation, `LoadMachine::from_file`, `SetupMount::from_file`, and the machine catalogue reads. So the existing commands (`LoadMachineFile`, `SelectPostFile`, the MCP tools) run unchanged on a session name. Undo works as before.
- The browser picker has three new purposes: `Machine` (keep and mount), `Post` (keep and select) and `Folder` (keep only). Load machine and Load post are no longer refused in the browser. A new `LoadSetupFolderDialog` command opens an `<input webkitdirectory multiple>` and keeps every `.mch` and `.cps` of the folder. The Machine page shows "Load folder (.mch, .cps)" in the browser build only. On the desktop the command says that the desktop lists its folders itself.
- A drop that holds a `.mch` or `.cps` goes to a new module `setup_file_drop`, called before the model drop in `shell.rs`. One dropped file is mounted or selected. Several dropped files only join the lists. One status line counts what landed and names each refused file with its reason. On the desktop, one dropped file on disk goes to `LoadMachineFile` / `SelectPostFile` by its full path; several are refused.
- Session machines join `library::enumerate`. Session posts join the post combo list (`post_pick::scan`).
- A saved project names a session file by its bare name. When that file is not loaded again, the disk read fails and the post or mount is refused by that name (#2319). There is no fallback.

**Why.** The issue asks for the existing commands, bytes from the browser, a folder picker without the File System Access API, and refusals by name. A store behind the existing readers is the smallest change that keeps one code path for the desktop and the browser. A new `PostRef` or `MachineSource` variant would change the project file and every exhaustive match.

**Dropped.**

- A `MachineSource::Session` / `PostRef::Session` variant: a project file change, and more code for the same result.
- Mount or select on a drop of several files: the last file would win silently. The lists are the honest result.
- Several files dropped on the desktop: the desktop has the machine and post folders for that.
- Stacking on PR #3057, which edits `file_drop::take_drop`: the new drop route is a separate function in a new module, so the two PRs do not change the same function.
- Persistence in IndexedDB for the next session. Not built: it is a separate decision for the maintainer. The options:
  1. No persistence (this PR). The user picks the folder again in each session.
  2. IndexedDB copy of each kept file, restored at startup and listed as session entries. A "Forget" button would remove them. About one module in `crates/ui` (wasm only) and a startup hook.
  3. A project-embedded copy of the post or machine text. This changes the project file and is a larger design.

**Cost to change later.** Low. The store is one module with a small API. A persistence layer can fill it at startup without other changes.
