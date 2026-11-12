## #538 - Wire EDM skim passes (per-pass offset and condition)

#532 cut one pass at one condition's kerf offset; #533 posted it. The umbrella (#290) put skims in the family as further offsets of the same path. These are the points this issue settled.

**Decided:**

- **A skim is a condition id and a direction.** `WireContourOp.skims: Vec<WireSkim { condition, reverse }>`, in cut order after the rough pass. The offset is never a number on the op: it is wire radius + gap of that skim's condition (#531's table, its one source), so editing the table re-marks the cut Stale (the stamp digests each skim's id, wire, gap and direction; with no skims it writes nothing, so no stored stamp moves).
- **File / MCP shape is additive.** `skims` is absent when empty (a rough-only file and every golden read and write as before); each entry's `condition` is required, `reverse` defaults to false. The MCP params door (`params_from_json`) carries it unchanged, and the edit rides `EditOperationParams` (undoable).
- **Refusals stand at the command door** (create, paste, edit), because they need only the machine's numbers: a skim naming no condition ("skim N of the Wire Contour names no EDM condition"), a condition the machine does not state or states without a gap / wire diameter (#531's door, prefixed by the skim's number), a pass nearer the profile than the finishing (last) pass ("would cut past the finished profile" - the rough pass included), a skim farther out than the pass before it ("would cut only air"), and a skim whose condition states another wire than the rough's. The generator (`motion::wire_edm::wire_passes`) runs the same `check_passes` again before any geometry.
- **One wire for every pass.** One threading cuts all passes; a skim condition for another wire diameter would need a wire change the program does not write, and its offset would be computed for a wire that is not there. Refused by name rather than rethreading.
- **Equal offsets are allowed** (a repeated finishing pass at the same condition); only a pass nearer than the finish, or farther than its predecessor, is refused. Tolerance: the generator's `PATH_TOL` (10 nm).
- **Each pass is the single-pass generator at its own offset**, every #532 law judged at that offset (kerf certificate, cover law, start-hole side). The cover (corner-minimum) law is therefore judged at the rough's offset too: a profile whose inside corners only the skims could reach is refused, conservatively, rather than left with rough-pass material in a corner.
- **Every pass starts and ends at the threading point** with the same straight lead; the wire stays threaded between passes. A reversed skim walks the same closed walk backwards (clockwise).
- **No CL format change.** The passes are one path; the post learns where each skim begins from the stream: `posting_wire` inserts a mid-section `PARAM EdmCondition <skim id>` before the first move after each return to the threading point, and refuses the section if the path does not return once per pass.
- **The post (`wire-iso.cps`)**: a condition that arrives while the wire is threaded is the next skim's; at that pass's first move the post writes `(SKIM n)`, the machine's condition select and `comp_off` again (compensation stays cancelled - the offset is in the path). No new `edm.codes` key is needed. The rough-only program is byte-identical to #533's golden.

**Why:** Prime Directive - a skim whose offset cuts past the finish, or whose condition the machine does not state, is wrong output, refused by name; the offset has one source (the condition table).

**Considered and dropped:**

- An offset (or "stock to leave") number per skim on the op: a second source beside the condition's gap that could disagree with it.
- A typed pass record in the CL: a `.cl` format change with every reader and golden touched, for a boundary the threading-point returns already state.
- Rethreading for a different wire per skim: not in the issue, and unknown to the AWT codes #533 requires.
- Refusing only "nearer than the finish" and letting an out-of-order skim cut air: harmless to the part, but almost certainly a mis-ordered list; refusing it by name costs nothing.
- Judging the cover law at the finishing offset only: would let the rough pass silently leave corners uncut that the program claims to cut.

**Left open:**

- **A punch drops after its rough pass** unless held; skims on a punch need tabs (#539) or the operator's own hold. The program does not claim otherwise.
- **The op page** (#535 / PR #1143) has no skim list editor yet; skims are set through the file and the MCP params door. Follow-up: #1148.

**Not exercised on a real EDM control (D10):** the golden posts against the synthetic fixture `crates/app/tests/edm/wire-iso-fixture.mch`, which gained three synthetic conditions (S21, S31, W20) with invented numbers.
