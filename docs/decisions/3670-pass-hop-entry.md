## #3670 - A 3D Pocket pass restart in a level takes the entry type of the operation

**Decided.** A 3D Pocket pass that starts again inside a level after a hop
(the zigzag row on the far side of an island) requires a full-disc
clearance proof before it keeps a straight restart. The restart axis must
lie on an earlier straight cut by the same tool. The whole cut must be at
or below the restart depth. The proof accepts exact endpoints and points
on the segment, with no XY slack. It does not use arc chords as proof.
Where clearance is not proved, a centre-cutting tool takes, in this order:

1. The entry type of the operation (helix, ramp, arc), through the same
   `descend` that the level entry uses.
2. The ramp (zig-zag) along the pass.
3. The straight plunge, as before, with a note that names it.

Steps 1 and 2 run on a copy of the path. A copy is kept only if it is
clean: it posts no plunge into uncut stock, and it crosses no move that is
already posted (the verifier's Crossing law; on a lifted offset shell a
descent can cross the shell's legs).

A tool that does not cut at its centre keeps its M11.3 fallback. An
operation whose entry is Plunge keeps the straight plunge. A plunge over a
column whose full disc is proved clear stays a plunge. The separate
non-centre-cutting core law stays unchanged.

**Why.** The user sets a helix entry to avoid a full-width plunge into
stock. The old code honoured it only on the first descent of each level.
The review found that the core law still allowed a restart with stock
under the outer part of the cutter. The radius-3-mm catalog tool had a
clear core at `(-0.138840, -20.062037, 10)`, but uncut stock remained at
`(-0.138840, -17.8, 10)`. A nearby cut cannot prove the full disc clear.
The new proof can miss clearance made by several cuts. This costs an
entry move. It cannot grant clearance from an offset centre line.
The trial copy is necessary: where the helix does not fit at the pass start
(a 2.2 mm stub between the dome and the stock edge in the catalog case),
`descend` puts the helix far away at the widest room and then dives again
at the pass start. That gives a helix and also the plunge, which is worse
than the plunge alone.

**Dropped.**
- Use `plunge_law::into_solid` as the restart policy. It proves clearance
  only for the non-centre-cutting core. Keep that law for its own purpose.
- Use flattened arc chords as full-disc proof. A chord can lie outside
  the cutter's actual swept area.
- Use the level transition (`links.arm(entry)`) for the restart. A restart
  is a new entry into stock at the same plane, not a move between levels,
  so the entry type is the correct choice.
- Change the pass order so that each row starts where a helix fits. That
  changes the zigzag walk and every 3D Pocket program.
- Refuse the operation when no helix or ramp fits for a centre-cutting
  tool. A centre-cutting tool can plunge. The note tells the user.
- The two "Also" items of the issue (a plunge over a cleared column still
  feeds from the hop plane; 16 moves shorter than the post resolution post
  as zero-length lines). They are a different site and go to the
  follow-up issue #3852.

**Cost to change.** Small. The rule is in one function,
`machining3d::reentry::reenter`, and one call site in `ladder.rs`. Each 3D
Pocket program with a helix, ramp or arc entry and pass hops changes when
this rule changes.
