## #3637 - Refuse waterjet posting until the jet records exist

**What was decided.** Take option 1 of the issue. The post gate
(`verification_report_by`) refuses each unsuppressed waterjet operation by
name. The refusal names what is missing: the standoff height, the jet
on/off records, the pierce dwell records and the waterjet laws (#1882,
#1885). A user post that declares `serves = ["waterjet"]` goes through the
same gate and writes no file.

The stock simulation skips waterjet operations, as it skips wire cuts. The
program sim then refuses its verdict by name ("waterjet stock removal is
not judged"), and the post gate shows that reason on its visible
not-judged line. The nozzle no longer enters the dexel model as a cutter.

The waterjet clause reads the program list on its own. When that list
cannot be read, the clause adds nothing: the sim clause already refuses
with the same reason. A second copy made the post gate name one fault two
times (the first attempt of this issue failed its gate on that).

The path notes now say that the jet and dwell records are not
implemented, and they name a pierce on the kerf ring when the lead-in is
zero. The page shows the same zero lead-in consequence. The compensation
choice reads "In control" without G-code words. Each tab row has its own
name ("Tab 1 position").

**Why.** Option 1 is the smaller change and closes the risk: no CL
without a standoff or jet records can reach a machine. A wrong program is
refused, not posted.

**Considered and dropped.**
- Option 2 (a standoff height, jet on/off and dwell CL records). That is
  the work of #1882 and #1885, with a CL format change and post support.
- A per-op "not judged" note while the program sim still runs the other
  operations. The wire clause already refuses the whole verdict by name;
  waterjet follows the same law.
- A note for corner slow-down, taper and kerf lag. These are not
  modelled; this file records it. The refusal blocks posting until the
  waterjet laws exist, so no output depends on them.

**Cost to change later.** When #1882 and #1885 land, remove
`posting_waterjet.rs` and its call, and replace the sim skip with a
waterjet model or a non-blocking note.
