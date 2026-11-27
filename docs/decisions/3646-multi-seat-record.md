## #3646 - Multi-seat indexable cutters in the document (app slice)

**What was decided.**
- Two fields on the #3644 row `DocumentComponent` (`crates/app/src/schema.rs`):
  - `milling: Option<MillingBody>`: the catalog facts of a multi-seat
    cutter body (`DC`, `KAPR`, `APMX`, centre cutting, seat count). Only
    on a tool item.
  - `running: Option<InsertRunning>`: an insert's own tool material and
    cutting rows. Only on an insert.
  Both stay in the row, next to the catalog fact and the geometry, so
  that one row is still the whole component (the #3644 law).
- `tool_assembly::derive`: when the insert's parent row states cutter
  facts, the record is `milling_cutter::derive(..).assembled(base, nodes)`
  of #4192. This gives the whole milling record (kind, diameter, corner,
  flutes, flute length, centre cutting, the holder enclosed over the
  tool) and checks every seat and the one-source law. A refusal is
  `tool '<name>': <the cutter's refusal>`; the edit command adds the
  component id in front, as #3644 does.
- The base is the tool's current record with the insert row's running
  data over it. The single-seat insert path takes the running data too,
  so one insert gives the same material and rows to each tool that cuts
  with it. A row with no running data changes nothing on that path.
- `ChangeTool`'s laws stay the last word: the #890 seed loads the
  document material's row into rpm and feed when the rows change.
- Commands: `UpsertDocumentComponent::with_row` and
  `EditDocumentComponent::with_row` take the whole row.
  `EditDocumentComponent::new` (catalog fact and geometry) keeps the held
  row's facts and running data. A fact on the wrong kind of component,
  and running rows a milling tool cannot take
  (`cutting_rows::refusal_for_kind`), are refused when the row is made.
- Staleness is still not written: the operations of each changed tool go
  Stale by their stamps (#3644).

**Why.** The issue asks that the shared insert carries the tool material
and the cutting rows, and that an edit of it updates every multi-seat
tool. The #4192 resolver already holds the gouge law of the record; the
document only selects it and lands its result through the #3644 edit
law.

**Considered and dropped.**
- A separate table of cutter facts, keyed by component id. Dropped: two
  tables can drift apart, and the #3644 decision keeps one row for each
  component.
- A field on the #3642 `ToolComponent` kind. Dropped: #4192 keeps
  `ComponentKind` as it is.
- An MCP tool for the new constructors. #3647 owns the component MCP
  tools.
- The project file. The table is not saved yet; #3645 (PR #4233) adds
  the file format. The two new fields must go into that format when the
  two PRs meet.

**Cost to change later.** Low. Only the two commands, `derive` and the
tests read the new fields.
