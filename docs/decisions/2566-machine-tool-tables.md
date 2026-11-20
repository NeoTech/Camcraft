## #2566 - Import and export PathPilot (.csv) and Hurco WinMax (.TCM) tool tables

**Decided:** Add a shared machine tool-table codec layer in `camcraft-tools`
(`crates/tools/src/tool_table*.rs`) with a minimal pure-Rust XML reader/escaper
(`tool_table_xml.rs`, no new crate dependencies) and a document-unit bridge in
`crates/app/src/tool_table.rs`. Extend `LibraryTool` with `number: Option<u32>`
and extend `ImportTools` (`crates/app/src/commands/tool_import/mod.rs`) to preserve
controller `T<n>` numbers, refuse tool-number clashes by name, allow number
swaps and reassignments within Update/Replace imports, and apply everything in one
undo step. Keep `.camtools` JSON (`units: "mm"`, no controller numbers)
unchanged. Default controller table units to the document's active units in GUI
(`state_tool_library.rs`), MCP (`import_tool_library`, `export_tool_library`), and
job scripts (`cam.importTools`, `cam.exportTools`), with optional `format` and
`units` overrides in MCP/scripts.

For Tormach PathPilot `.csv`: require the row-1 `Mill Tool Table` marker, bound
numbers to `1..=1000`, parse optional `Description` tokens (`R:`, `loc:`,
`<n>FL`, tool material, `[part]`), keep `Description` as tool name, ignore
machine `Z offset` placeholders, treat negative `Max RPM` values (such as
`-2.0` on probes) as unstated, skip empty slots, refuse bad rows or non-mill
export kinds by name, and export with zeroed offsets.

For Hurco WinMax `.TCM`: read/write UTF-8 XML with BOM and CRLF under
`<Hurco_Document Version="1.1">`, strictly map documented `#30001` `ToolType`
codes (`0,2,7,11,12,13,14,15,16,18,20,21,22,26`) and refuse unsupported codes
(`1,3,4,5,6,8,9,10,17,19,23` or unknown) by name, map `AbsoluteGaugeLength` onto
holder `gauge_len`, and omit Hurco control state (`UsedTime`, `Flags`,
`GeometryFlags`, probe offset indexes) on export.

**Why:** Keeping parsers/writers in `camcraft-tools` decouples table codecs
from Flecs ECS while sharing tool geometry and sanity rules. Refusing
unsupported Hurco `ToolType` codes and non-mill PathPilot exports by name
prevents silent cutter substitution. Ignoring PathPilot `Z offset` on import
and zeroing exported offsets avoids confusing machine gauge offsets with tool
stick-out or writing stale offsets back to a machine. Omitting WinMax control
state avoids resetting tool life or probe calibration data on import.

**Considered and dropped:**
- Adding an external XML dependency in `Cargo.toml` (dropped for one narrow,
  self-contained XML format).
- Mapping unsupported Hurco cutters such as keyseat or dovetail tools to
  `ToolKind::EndMill` (dropped because wrong cutter geometry is unsafe).
- Adding controller tool numbers to `.camtools` JSON or changing its `units: "mm"`
  contract (dropped to keep native `.camtools` files backwards-compatible).
- Emitting guessed zeroed `UsedTime`/`Flags`/`GeometryFlags` tags in `.TCM`
  exports (dropped so partial exports never clobber controller runtime state).

**Limits and later cost:** Acceptance of exported partial `.TCM` files on a
live Hurco WinMax 9.01 control and live PathPilot Hub UI import still need
machine verification. All format-specific field mappings sit inside
`crates/tools/src/tool_table*.rs`, so adding fields or relaxing version headers
from real machine captures is isolated to that module.

**Decided after review:** Refuse a named slitting saw in PathPilot input until
our tool model can represent its cutting shape. Before PathPilot export,
read the proposed description and rounded diameter through the same reader.
Refuse the export by tool name if the resulting cutter geometry differs.
Keep the description and tool name unchanged. Direct the user to `.camtools`
when PathPilot cannot preserve the geometry. Compare geometry lengths within
`1e-6` mm to allow the seven-decimal CSV diameter rounding.

Use character boundaries for malformed XML entity diagnostics. Limit XML
nesting to 32 elements from the root through the leaf before recursive descent.
Return a diagnostic when the input exceeds this limit.

**Why:** A controller table must not substitute a different cutter or cutting
shape. A malformed table must return an error without a Unicode panic or
stack overflow.

**Considered and dropped:** Rewriting descriptions with new geometry tokens
would change tool names and require a new metadata convention. Preserving
names and refusing a lossy export keeps the existing table contract. Unbounded
XML recursion cannot safely accept untrusted controller tables.

**Limits and later cost:** A future tool model can add slitting saw support.
PathPilot export can accept more descriptions when its reader can preserve
more cutter geometry. Both changes need new geometry round-trip tests. Raising
the XML depth limit requires a bounded parser stack or a checked stack budget.

**Decided after the second review:** Start imported threading tools with zero
pitch. Require a positive recorded `ThreadPitch` for Hurco taps and thread
mills. Refuse PathPilot threading tools by name because CSV has no pitch
field. Refuse Hurco left-hand tap exports by name because the supported
export fields cannot preserve the hand.

**Why:** A guessed pitch can produce an incorrect tapping feed. An export
must not change a left-hand tap into a right-hand tap.

**Considered and dropped:** Guessing pitch from free-form descriptions or
using a default pitch does not establish the cutter's required pitch.
Writing an unverified Hurco hand field cannot establish a safe round trip.

**Decided after the third review:** Refuse Hurco export by tool name for
non-center-cutting bull-nose, chamfer, and ball-nose tools. Keep type 15 for
non-center-cutting flat end mills.

**Why:** The supported Hurco types cannot preserve this restriction for the
three cutter kinds. Changing the flag could permit an unsafe plunge.

**Considered and dropped:** Writing an unverified controller field does not
establish a safe round trip. Support for another field requires verification
and round-trip tests before these exports can be accepted.

**Decided after the fourth review:** Split `tool_table_hurco_export.rs` from
`tool_table_hurco.rs` and verify every exported Hurco tool through the Hurco
reader. Refuse Hurco export by tool name when the holder diameter or length
cannot survive the conversion, and refuse boring bars with a positive nose
radius. Export judged holder dimensions when a tool carries a modelled body.

**Why:** Hurco `.TCM` has no verified holder diameter field and `TipRadius` is
verified only for bull-nose cutters. Shrinking a wide holder or dropping a
boring-bar nose radius changes collision and boring geometry silently.

**Considered and dropped:** Guessing unverified Hurco XML tags or dropping the
holder diameter to zero on import without an incomplete-holder gate. Both
approaches remain reversible if verified controller fields or a general
incomplete-holder gate are added later.
