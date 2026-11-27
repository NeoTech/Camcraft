## #4031 - a cutter hand on the tool, refused or turned M4, never run M3

**What was decided.** `Tool` gains a `cutter_hand` field (`crates/tools`
`CutterHand`, right by default), separate from the tap's `thread_hand`:
a left-hand tap cuts a left thread while its spindle may still turn
right, so the two hands are stated apart. The tool gate refuses a
left-hand cutter by name for every operation whose setup posts through
a built-in post that starts a spindle M3 and never reverses it (Haas,
Tormach PathPilot, Doosan DNM, Grbl 1.1, grblHAL, Makera Carvera,
Heidenhain, Heidenhain Klartext) - at creation (`tool_refusal`), and as
an operation's after-state (`tool_problem`), so generation, the post,
the tree and the page all refuse with the same sentence. A left-hand
cutter rides the generation digest, so a tool edited to left-hand marks
its operations stale. The project file steps to v83 and a `.camtools`
library with a left-hand cutter steps to version 7; both older readers
must refuse, because they would drop the hand and post M3.

**Why.** A posted M3 under a left-hand cutter is wrong G-code: the
cutter turns backwards and the work is scrapped or the tool breaks. The
maintainer scoped steps 1, 2 and 5 (field, gate, versions); steps 3 and
4 - the operations reading the hand and asking the spindle reverse via
`ask_reverse_spindle` - are the follow-up.

**Considered and dropped.** Reading the `millSpindleReverse` manifest
flag out of each post script at gate time: heavier, and the maintainer's
step 2 names the M3-only posts by registry name. A user's own
`PostRef::File` post is left open - nothing here knows its words.

**Cost to change later.** The M3-only list lives in
`tool_gate.rs::M3_ONLY_POSTS`; when a post gains a reverse path, remove
its name there and the gate opens.
