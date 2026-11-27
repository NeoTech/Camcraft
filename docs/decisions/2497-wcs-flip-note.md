## #2497 - Cache the WCS page's existing post refusal

**Decided:** Keep the display cache in `WcsUi`. Key it by the document revision and the active setup. Read the existing `setup_problem` result first, then `setup_post_problems`, in the same order as the flip status. Share that cached result with the status and the page. Show no note when no axis is flipped. Keep the mirrored note when the existing readers report no refusal.

**Why:** The page must show what the post refuses without loading a post script on each frame. Document commands, undo, redo, generation, New and Open advance the revision. The setup identity also prevents reuse when the active context changes. The cache is display-only. The post and its validation never read it.

**Considered and dropped:** Do not add a second eligibility rule or change a refusal. Do not load scripts in the drawing code. Do not add a general cache framework or change unrelated setup synchronization. The existing large `state.rs` needs only the cache field on `WcsUi`; its other shared regions stay unchanged.

**Verification:** Structural tests read the real WCS page. They cover non-Z rotation, probing, suppression, undo/redo, post selection, active setup identity, New and project replacement. A counter pins cache reuse. A file-backed post test changes the manifest on disk and proves that unchanged frames do not reload it. A real origin edit refreshes the note. Optional `CAMCRAFT_WCS_EVIDENCE_DIR` captures the same page with the existing test renderer, outside snapshot goldens.

**Cost to change later:** A new document writer must advance the revision. If external script edits later become a live document input, that writer must invalidate this display cache too. This change does not watch files or alter post decisions.
