## #1450 - Drop a model file on the window

**Decided.** A drop is read window-wide each frame in `shell_layout`; one file with an `stl`/`obj`/`step`/`stp` extension (the Model dialog's own filter, case-insensitive) queues `CommandKind::ImportFile`, the command the dialog and recent list already end in, so the worker, the update-or-spawn rule for an existing name and the single undoable import are shared. The hint "Drop to load model" is painted over the viewport rect while egui reports a hovered file. Refusals (several files, wrong extension, a path that is no file, which is also what a browser drop carries) are said on the status line by file name and touch nothing; a corrupt file is refused by the import worker's own "import refused" message.

**Why.** No second import path (issue rule); `ImportFile` already exists, so the ui crate needs no world access.

**Considered and dropped.** Importing several dropped files in turn (the issue asks for a refusal); restricting the drop to the viewport's rect (hiding the viewport tab would make drops silently dead; the hint still sits on the viewport); a new `DropFile` command (it would duplicate `ImportFile`); an MCP tool (the import command already has one, the drop is only a GUI gesture).

**Cost to change.** Low: `crates/ui/src/egui_app/file_drop.rs` is the only file with the rule. Browser drops (bytes, no path) wait for issue 1431.
