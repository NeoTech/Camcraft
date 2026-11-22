## #3261 - Stock From model: the draft holds the part that Source shows

**Decided.** Each frame, the Source picker of the Stock page writes the part
that it shows into the stock draft (`stock_ui.source`), when the two differ.
Before, it wrote the name only when the selection index changed. With one
model, or with the default selection, Apply submitted an empty source.

**Why.** The name that Apply submits must be the name that the user sees.
The picker already falls back to the last eligible model when the draft
names no model. Now the draft takes that same fallback. A part that is
deleted or renamed also moves the draft to the shown part.

**Dropped.**
- Fill the draft once when the page opens. The page has no open event, and
  a deleted source part would leave a stale draft again.
- Default to the first model instead of the last. That changes what the
  picker shows today. The issue asks only that the shown name is applied.

**Cost to change later.** Small. The change is three lines in
`source_fields` (`crates/ui/src/egui_app/stock_page.rs`).
