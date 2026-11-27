## #4335 - Tool library dialog U9: tool list with one header row and a search

**Decided.** The tool list of the tool library window
(`crates/ui/src/egui_app/tool_library_dialog/list.rs`, `list_header.rs`,
`search.rs`; the cache in `crates/ui/src/state_tool_list.rs`) has:

- One header row: the source tabs in the mockup order (Document, My library,
  Catalog), one search box with the placeholder "Filter by name, diameter or
  part number" (AccessKit name "Search tools"), then New tool…, Import… and
  Export… at the right.
- One search for the three sources. The text is `State::tool_search`, so it
  holds when the source changes and between two openings.
- "N of M tools" (My library: "N of M components") above the rows.
- A category rail that lists every category in every group. The count of a
  category is the count of the rows that match the search; an empty
  category shows no count.
- The same columns for Document and Catalog rows: drawing, T, Name, Type,
  Ø, Flute, Flutes, Holder, Gauge, Recipes (the count of cutting data rows)
  and the mark. The heads carry the document's unit ("Ø mm"). A Catalog row
  ends with "in document" or Add; a My library row that the document holds
  shows "in document".

**The matcher.** The text is split into terms at white space. A row matches
when each term matches (AND), so "6 ball" finds the 6 mm ball nose mills.

- A text term matches a part of the name, the type, the category, the holder
  name, the part number or the vendor (#898), without case.
- A number term ("6", "6.35", "Ø8", "10mm", "0,5") matches the cutting or
  the shank diameter within 0.05 mm, in the document's unit (an inch
  document types inches). It matches the tool number when it is a whole
  number. It matches the text only when it has four or more characters (a
  numeric part number), so "6" does not find each name that holds a 6.
- "T5" matches the tool number 5.
- 0.05 mm is finer than the step of any common series (0.1 mm; 1/64 in), so
  "6" does not find a 1/4 in (6.35 mm) tool.

**Frame cost.** State reads the rows of the shown source once for each
document version and keeps a revision that moves only when that read
differs. The hits are built again only when the revision, the text or the
unit changes; the My library hits only when the library read, the text or
the unit changes. A still frame builds nothing (`ListBuilds`, pinned by
`a_still_frame_builds_no_hits_and_a_change_builds_once`).

**Choices taken unattended.**

1. *The Tool type combo is gone.* The category rail and the search (it
   matches the type) do its work; the mockup has no combo. The old filter
   field `State::tool_catalog_kind` is removed.
2. *The import options move into an Import… menu.* The mode (#899) and
   "Import tool holders" (#889) stay, with their texts, in the menu that
   Import… opens, above a "Choose a library file…" button. A click on a mode
   or on the check box keeps the menu open. This keeps the header to one row
   and loses no option. "Import library…" and "Export library…" become
   "Import…" and "Export…", as in the mockup.
3. *A click on a Document row's name opens the tool.* egui's grid has no row
   click; the name is the click target, and the "Edit" button is gone. The
   footer hint says "Click a row to edit the tool."
4. *My library keeps its own page under the shared header.* It lists
   components, not tools, so the same box filters the component rows (name,
   id, kind, the machine and tool sides; a solid tool's diameters). Its own
   "Component filter" box is gone (and `ComponentsView::filter`).
5. *Catalog rows draw the row icon, not the cached drawing.* The drawing
   cache is keyed by the tool name; a catalog tool of the same name as a
   document tool would rebuild it on each source change.
6. *Rows have no stripes.* The design rules of #4245 ask for flat panes and
   structure from type; the heads are small and dim.

**Dropped.** A row-wide click target (needs a custom table widget); matching
the coating and the notes (not asked for; one line in `ToolRow::new` adds
them); a fuzzy match (a CAM user types exact sizes).
