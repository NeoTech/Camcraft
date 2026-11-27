## #4336 - Tool library dialog U10: the tool editor fields, the clamping check and the side drawing

**What was decided.**

1. **Tool tab order.** The Tool tab reads as the #3663 mock-up: Name, Type,
   Tool number; a CUTTER group (Diameter, Flute length, Corner radius,
   Flutes, Helix angle, Tool material, Centre cutting, then the rows that
   only the kind reads); a SHANK group (Shank diameter, Shank length); then
   the assembly group. The rows are the same row pieces that Properties
   draws (`tool_settings_grid::{type_row, diameter_row, corner_row,
   flute_row, shank_rows, kind_specific, number_row, centre_cutting_row}`).
   Properties keeps its old order through `geometry()`, which now composes
   the same pieces. The group headings are small dimmed capitals over a
   hairline, in the existing theme inks (#4245: structure from type, no
   boxes).
2. **Name is shown, not typed.** No command renames a tool. Operations,
   scripts and the MCP tools address a tool by its name. A text field that
   does not land would break "every edit is an undoable command". The row
   shows the name with a hover that says why. Follow-up #4370 asks for a rename
   command with its MCP tool.
3. **Moved rows.** The tool number and centre cutting moved from the Details
   and Recipes tabs to the Tool tab; the helix angle moved from Details to
   the CUTTER group ("Helix angle", unit after the field; still text, so a
   half-typed number is refused by name at Apply). Flutes and Tool material
   show in the CUTTER group and stay on the Recipes tab, where they are the
   inputs of the feed per tooth. The two tabs never draw at the same time
   and edit the same draft field, so they cannot disagree.
4. **Assembly group.** Heading "ASSEMBLY: HOW THIS TOOL SITS IN A HOLDER".
   Rows: Holder (the pick), Stick-out (the projection), Holder gauge length
   (the holder's own length, still typed, so no edit path is lost), then
   "Gauge length" read-only with the sum under it ("= 60 mm of the holder +
   55 mm stick-out.") and the link "Open holder "<name>"", which opens the
   holder view and keeps the tool's draft.
5. **Clamping check line.** One line judges the draft shank against the
   resolved holder: "Fits: the shank Ø8 is in the clamping range 1 to 13 mm
   (ER20 collets)." or "Refused by <holder>: <the library's reason>". The
   reason is `HolderEntry::fits_shank`'s own text, so the check and Apply
   holder cannot drift. The command's verdict on the stored tool still
   shows when it says something else (another stored shank, the lengths,
   the fit). The fit line uses the body ink: the theme has no "good" token,
   and #4245 asks for plain text tags.
6. **Apply holder stays its own button.** The footer's Apply lands the tool
   (`ChangeTool`, `SetToolNumber`); Apply holder lands `SetToolBody::library`.
   One Apply for both needs a grouped command in `crates/app` and is out
   of this page's scope.
7. **Side drawing.** The side view is the default view and comes first in
   the toggle; 3D stays as the option. The drawing adds: the visible flute
   lines (the projected groove centre lines of the helix, `flutes::
   side_lines`, built with the model; the Flutes check box shows or hides
   them in both views), a dashed spindle nose outline above the gauge line
   with its label and the holder's name, and a "to scale · mm" note. The
   gauge dimension says "gauge length". The spindle nose is not to scale
   (its width is 1.6 times the holder's): only its face, the gauge line, is.

**Why.** The issue's mock-up and its Done-means; the maintainer's flat
design rules on #4245; the refit rule of `crates/ui/AGENTS.md` (one page,
each edit an undoable command, frame cost).

**Dropped.**
- A typed Name field that renames on Apply: there is no rename command.
- Moving Flutes and Tool material off the Recipes tab: the feed per tooth
  needs them in view there.
- A green "fits" colour: no theme token exists; a new colour is out of the
  rules.
- Painting the flute lines per frame from the tool numbers: they are built
  with the cached model instead, so a still frame builds nothing.

**Cost to change later.** Low. The row pieces are small functions; the
check text is one function (`assembly_group::clamp_check`) with its own
test; the side drawing additions are display only.
