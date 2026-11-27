## #4348 - G49 before the program-end retract when G43.4 is still active

**What was decided.** The Fanuc and Doosan DNM posts get one flag each,
`tcpmLeft`. `endPreamble` sets it to the value of `tcpmOn` just before it
resets `tcpmOn`. `onClose` reads it: when the program end writes its own
retract (no rotary is held off zero, and no trailing Go home), and the last
section left G43.4 active, it writes `G49` first. A section that tilts and
then stands upright again is the case: no rotary is held, so `uprightAtEnd()`
writes nothing, and `G91 G28 Z0` came under G43.4.

The FANUC 30i operator's manual (B-63944EN/02) lists the G codes that are
permitted in G43.4 mode. A reference return is not on the list.

The other exits already write `G49` first: `uprightAtEnd()` (a held
rotary), the Go home after-action, and the next section's preamble line.
Those paths clear `tcpmOn` (or write `G49` themselves) before `endPreamble`
copies it, so `tcpmLeft` is true only when no `G49` follows the last
`G43.4`.

One recorded golden moves: `crates/post/tests/recorded/golden/fanuc/rotary.nc`
(a tilted section that ends at `A0 B0`). It gains one `G49` line after `M5`.
That golden recorded the defect.

**Why.** It is the smallest change. Programs that never arm G43.4, or that
end through an existing `G49` path, are byte-identical.

**Considered and dropped.**
- A `G49` in each `endPreamble` when `tcpmOn` is set. It is also correct,
  but it adds a line to every tilted section that a later section follows,
  where the next preamble already writes `G49`. More goldens move for no
  gain.
- A `G49` at each program end. It moves every golden.
- A fix of the Fanuc setup-change path (`afterOp`, `next` set with a stop
  ask), which writes `G91 G28 Z0` under G43.4 too. The Doosan post already
  writes `G49` there (#1647). The issue names only the program end, so the
  change stays in its scope. The PR body names it as a follow-up.

**Cost to change later.** Small. One variable and two lines in each post.
