# #1256 - Escape closes dialogs: pinned, not changed

**Decision (unattended):** no behaviour change. Escape already closes the
modal dialogs the code comments name; this PR pins that with real key-event
tests.

**Reading:** the two comments that say Escape closes a dialog
(`unsaved_prompt.rs`, `fixture_library_dialog.rs`) are both egui `Modal`s.
egui 0.36's `Modal::should_close()` consumes Escape for the topmost modal
with no popup open. The issue's premise ("no Escape key handler exists") is
true of `panes::shortcuts` but not of the modals. Escape enqueues the same
Cancel that the buttons send (`ResolveUnsaved(Cancel)` for the prompt,
`Act::Close` for the fixture dialogs), so nothing is applied.

**Chosen:** pin it. Three kittest tests in
`crates/ui/src/egui_app/escape_tests.rs`: one press closes only the topmost
of two open dialogs; a single fixture dialog closes with no library file
written; Escape with nothing open changes nothing.

**Considered and dropped:**
- Adding a global Escape handler in `panes::shortcuts`. It would duplicate
  egui's modal handling and risk closing dialogs twice.
- Cancelling an in-progress pick or box-select draft on Escape. No code
  comment says so, so it is out of scope for this issue. Filed for the
  maintainer to decide if wanted.
- Updating a keyboard reference page. None exists on `main` yet; the
  reference from #1246 / PR #1255 is not merged. Nothing to update here.
- egui `Window` panels (tool editor, G-code view, post refusal, machine
  editor, debug info). They have no Escape comment and close with their
  own button; left as is.
