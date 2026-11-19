## #1889 - Deburr: a policy for edges out of the cone's reach

**What was decided**

- `finishing3d::UnreachableEdges` has two values: `SkipWithNote` (the
  default) and `Refuse`. It is a field of `DeburrParams` and of
  `OpParams::Deburr` (`unreachable`).
- An edge is out of reach when it is inside the programmed window and
  climbs steeper than the cone's flank (#303). This is the one reach test
  that the kernel has.
- `SkipWithNote` keeps the behaviour of each earlier Deburr: the edge is
  left, and the note `deburr: N edges climb steeper than the cone's flank
  and were skipped` gives the count. No golden moves.
- `Refuse` returns `GenError::Invalid` before a path is built. The text
  gives the count, names the setting, and names `Skip with note` as the
  way out. It comes before the `Empty` answer.
- The project word is `"unreachable": "skip_with_note" | "refuse"`. It has
  `serde(default)`, so an earlier file reads as `skip_with_note`. No
  `PROJECT_VERSION` step.
- The word is always written, the default too.
- The digest writes one byte (`0x52`) only for `Refuse`. The stamp of each
  earlier Deburr does not change.
- UI: a combo `Unreachable edges` on the Geometry tab of the Deburr page.
- MCP: no new tool. `add_operation` and `edit_operation` take the field in
  the params patch, and `list_operation_types` and `get_job` show it.

**Why**

- The issue asks for the two modes, with skip as the default.
- A `Refuse` that comes after `Empty` would hide the reach failure when
  each candidate edge is steep.
- The word is always written because the MCP patch check
  (`mcp/edit.rs`, `lost`) refuses a key that does not come back from the
  typed round-trip. With the default left out of the file, the patch
  `{"unreachable": "skip_with_note"}` was refused as an unknown field.
  The pin `mcp_selects_the_two_modes_and_lists_the_field` found this.

**Considered and dropped**

- To count an edge that the window clips as unreachable. Dropped: the
  window is the machinist's own selection. With it, `Refuse` would refuse
  each closed part, because the bottom rim is always under the window.
- To count the winding and non-manifold skips as unreachable. Dropped:
  they are mesh defects, not reach. Their notes stay as they were.
- To move the flute-length check of `generate.rs` under the policy.
  Dropped: it is true for each edge of the ride, so it always refuses.
- To leave the default word out of the file (`skip_serializing_if`), with
  a special row in `list_operation_types`. Dropped for the MCP reason
  above. The cost: a Deburr saved by this build has one more key.
- A stricter refusal that names each edge position. Dropped as larger
  than the issue; the count is what the note already gives.

**Cost to change later**

- A third policy value is one enum variant, one DTO word, one combo entry
  and one digest byte.
- To leave the default out of the file later needs a change to the MCP
  patch check first.
