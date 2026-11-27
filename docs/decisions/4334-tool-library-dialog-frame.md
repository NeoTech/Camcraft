## #4334 - Tool library dialog U8: one frame with a common header and footer

**Decided.** The tool library window gets one frame around its pages
(`crates/ui/src/egui_app/tool_library_dialog/frame.rs`):

- A header row: "‹ List" while an editor (the tool editor, the component
  editor) or the read-only holder view is open; the title ("Tool library" on
  a list page, else the tool, holder or component name with a dim
  subtitle such as "End mill · T1"); the Cutting tools | Holders tabs; and a
  "×" close button at the right (AccessKit name "Close the tool library").
- A footer that all pages share: a hint on the left; Close on a list page and
  on the read-only holder view; Duplicate, Cancel and Apply on the tool
  editor; Cancel and "Apply component" on the component editor.
- The header and the footer are `TopBottomPanel`s inside the window, so the
  footer stays at the bottom of the window while a list scrolls. The page
  content does not change; U9 to U12 change the pages.

**Choices taken unattended (the conservative option each time).**

1. *Floating window, not a modal dialog.* The window stays non-modal, at the
   same place (left of the Properties pane) and the same size as before. The
   mockup draws a large centred dialog. A modal dialog would block the
   viewport and Properties, which U1 (#3892) kept free on purpose. At
   1366 × 768 the window keeps the width rule of #3897 (`width`).
2. *No egui title bar.* The frame's header replaces it, so the window has one
   header row, as in the mockup. The window's AccessKit name stays
   "Tool library" (egui takes it from the title, also without a title bar).
   egui moves a window without a title bar by a drag on its background
   (`WindowDrag::Anywhere`); widgets that take a drag keep it.
3. *The tab keeps the label "Cutting tools", not "Tools".* The ribbon has a
   "Tools" button. Two buttons with the same AccessKit name and role are
   ambiguous for a screen reader and for the tests that click the ribbon
   button.
4. *Duplicate is shown, disabled, with its reason.* No command copies a tool
   with its holder and its component stack. `AddTool` with the stored tool
   would give a copy without the holder: a silent difference. The follow-up
   #4340 owns a real duplicate command.
5. *Cancel = drop the draft and go back to the list.* This is what the old
   way back did, so List and Cancel do the same now. When the editor keeps a
   draft over a page change, Cancel can become "revert the draft".
6. *"‹ List", not "← List".* The bundled egui fonts have no "←" glyph (it
   shows as a box); "‹" renders. An arrow icon can replace it in a later
   refit.
7. *The flat look of #4245 with the existing theme tokens.* No shadow, the
   theme's window fill and stroke, hairline separators of the panels, Apply
   filled with the theme accent (the `action_bar` inks). No new theme or kit
   crate in this PR.

**Dropped.** A centred modal (blocks the workspace); a "Tools" tab label
(name clash with the ribbon); Duplicate through `AddTool` (loses the holder);
keeping the egui title bar above the header (two header rows).

**Cost to change later.** Small: the frame is one file. The label constants
(`frame::BACK`, `CLOSE`, `APPLY`, `CANCEL`, `DUPLICATE`) are what the tests
use.
