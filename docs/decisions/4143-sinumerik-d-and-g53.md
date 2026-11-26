## #4143 - Siemens 840D post: the tool call selects D1, a machine file can state D = T

**Decided.** `sinumerik.cps` writes `T<n> D1 M6`. A new optional machine-file
fact, `tooling.d_number`, selects the rule: `first-edge` (the default, D1) or
`tool-number` (D equal to T). Any other value is refused by name when the file
is read (`machine file: tooling.d_number: ...`). The post reads the fact as
`machine.dNumber`; the machine sheet shows it in the General group.

**Why.** SINUMERIK 840D sl Programming Manual Fundamentals, 12/2018
(A5E40869423B AB), section 6.6 "Tool offset call (D)", pp. 82-83: D activates
an offset data block of the active tool, and D0 deactivates it. "Type of D
programming" is set by machine data: with "D number = cutting edge number"
each tool has D1 to D12 (max.), assigned to its cutting edges; with "free
selection of D numbers" the shop assigns them. The manual's own examples
load a tool with D1 (`N10 T1 D1`, `N110 T="WZ2" M6 D1`). So D<T> selects an
edge that tool 12 does not have under the first rule: an alarm, or an
empty edge with no length. D = T is a valid shop rule only on a control set
to free D numbers, and only the machine owner knows that.

**Dropped.** A per-post option (`declaredOptions`): the D rule is a fact
of the control's machine data, so it goes with the machine, as the issue
asks. Writing the D on a separate line after `M6` (the 810M post's form):
the 840D examples write `T.. D1` / `M6 D1` in one block, and the line
order of the existing goldens stays.

**Cost to change.** One function (`dWord`) in the post, one enum in
`camcraft-post` (`DNumberRule`). A third rule (for example a fixed D
other than 1) is one more enum value.

## #4144 - Siemens 840D post: each G53 retract is `G0 G53 Z.. D0`

**Decided.** Every G53 Z retract of `sinumerik.cps` (the setup-change
lift, the home ask, the program end) is written as `G0 G53 Z<safe> D0`.
A later section that keeps the same tool (no tool change) writes `D1`
(or the stated D rule) before its first move, and its first traverse
states its height again.

**Why.**
- G53 is not a motion word. 840D sl Fundamentals 12/2018, section 9.1,
  p. 151 and section 13.10, p. 358: G53 suppresses the settable and
  programmable frames non-modally; G153 and SUPA suppress more frames
  and offsets (basic frame, DRF, external zero offset, PRESET). None of
  them is a motion command, so without `G0` the block moves with the
  modal G1 (at the feed) or alarms after G2/G3. The old program end
  wrote `G53 Z..` with no `G0`.
- None of G53, G153 or SUPA deselects the tool length. Section 6.6, p. 82:
  "When D0 is programmed, offsets for the tool have no effect". With the
  length active, `G53 Z<safe>` sends the tool TIP to the safe height, so
  the spindle goes the tool length higher and can reach the soft limit.
- The manual's program example retracts in this form: `N5 G0 G53 X280
  Z380 D0` and `N130 G0 G53 X280 Z380 D0 M5` (section 4.4, pp. 54-55).
  The 810M post (`sinumerik-810m.cps`) writes the same `G0 G53 Z.. D0`.
- Section 6.6 also says the length applies with the first traverse of the
  length axis after a D is programmed; the re-selected D is written before
  any motion and the first traverse states its Z first (#1633's law).

**Dropped.** `SUPA` in place of G53: it also suppresses the PRESET and
external offsets and the DRF handwheel offset, which a machine builder
can use for the tool change position; G53 is what the existing
`safe_retract` fact was specified against (#503). A separate `D0` line
before the retract: the manual's example writes it in the retract block.

**Not exercised.** No run on a real 840D control. The behaviour is from
the cited manual pages only.

**Cost to change.** One function (`retractZ`) in the post.
