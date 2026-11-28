## #4342 - Unique Mazak probe labels in numbered programs

**What was decided.** The Mazak post records each used N number in one
program-scoped set. Both automatic block numbers and probe labels use this
set. Automatic numbers follow the selected start and increment, and skip a
number already used. Probe labels count from 9001, and skip a number already
used. The post reserves a probe label before it writes the conditional
branch and alarm. The target uses the existing `rawBlock` function, so it
has one N word and no automatic prefix.

The post engine creates a fresh script context for each program. The set
therefore starts empty for each output file. The direct engine API does
not validate declared option bounds. The allocator refuses a nonpositive
rounded increment or arithmetic that cannot advance to a finite number.
This protects its skip loop without adding a controller-specific limit.

**Why.** A target such as `N110 N9001` does not give `GOTO9001` the intended
target. A standalone target also fails when an ordinary block has the same
number. One set prevents collisions with numbers written before a label
and with automatic numbers written later. The labels remain unique across
sections. The existing output with automatic numbering off stays the same.

**Considered and dropped.** Removing only the prefix does not prevent
duplicate numbers. Reserving a fixed range imposes a new limit on either
kind of number. Predicting every future automatic number is unnecessary.
The shared set follows the numbers the post actually writes.

**Cost to change later.** The rule is local to the Mazak number allocator
and probe alarm helper. It stores one set entry for each emitted N number.
The tests use the existing recorded probe stream and post engine. One new
golden records the Every block option. No dependency is added.

**Independent work.** PR #4413 changes the Mazak feed-mode preamble.
PR #4371 changes the retract before a tool change. This fix changes neither
function. Its special-option golden is separate from their existing
goldens. If #4413 merges first, this new golden must receive its G94 word.
If this fix merges first, #4413 must update this golden too.
