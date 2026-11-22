## #3338 - Separate inside and outside boundary offsets

**Decided.** `AreaControls` keeps `offset` for the inside boundary (the
selection's area, or the line that the user picked for an open boundary)
and gets a new `outer_offset` for the outside boundary (the stock side of
an open boundary). Both default to 0. The Boundary group shows two rows,
"Inside boundary offset" and "Outside boundary offset", each with its own
hint. The MCP tool `set_operation_area` reads and shows `outer_offset`.

**The law.** When the two values are equal, the area law runs the old
code path exactly (`grow` by the one value), so an old project and its
output stay byte-identical. When they differ on an open boundary, the
kept side grows by the larger value, the stock reach grown by
`outer_offset` clips the outer line, and the far side (shrunk by
`offset`, as before) cuts the picked line. A closed area has no outer
line and reads only `offset`. The summary says so in a notice when an
outside offset is set without an open boundary.

**The file.** No version bump. The `area` DTO gets an optional
`outer_offset` key. An absent key means "the same as `offset`". Thus an
old file with one `offset` loads that value into both, and a save of
equal values writes the same bytes. A save of different values writes
the key, also when it is 0. The stamp hashes `outer_offset` only when it
differs from `offset`, so the stamp of an old operation does not change.
This follows the `area` key's own precedent (no bump for an additive
key with a default).

**Dropped.**
- A project-file version bump with a migration step (the issue text
  asked for one). The optional key gives the same result: an old file
  loads into both values and saves the same bytes. A bump would change
  the bytes of every saved project for no gain.
- `outer_offset: Option<f64>` in `AreaControls` ("follows `offset`" when
  unset). The UI would then move both lines when the user edits only the
  inside row, which is the bug.
- Renaming `offset` to `inside_offset`. It would touch many call sites
  and the MCP and file keys, for no behaviour change.
- The `camcraft-mcp` skill text still lists the old `area` keys. It is a
  process file. It is not changed here, and the tool schema describes the
  new key.

**Cost to change later.** The file key is additive. A later rename needs
a DTO alias only.
