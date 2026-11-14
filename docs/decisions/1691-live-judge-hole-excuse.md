## #1691 - the post's live re-judge of kept parts uses the generation's law split

**What was decided.** `operation::kept_rejudge::moved_part_problem` (the
post-time re-judge of #1628) now picks the part law the same way
`generate::run_job_arms` does for a stock source:

- an Engrave or Trace: the mark law (as before);
- a Drill, Bore or Thread: the containment law, with the declared holes'
  modelled walls excused (#1631), with the same thread grant;
- a cross drill: the radial law `cross_part::part_clear` (#1635);
- every other kind: the plain solid law (as before).

On a part source the neighbours keep the plain solid law, and a cross drill
also answers to `cross_part::neighbours_clear` (#1641), as at generation.

**Why.** Before, every kept part took the plain solid law. That law does not
excuse a modelled hole's faceted wall, so a legal stock-source drill in the
part's own hole was refused at the post while generation passed it. The fix
gives the post the generation's verdict for an unchanged document. A cut into
solid part material is still refused: the containment law convicts it, and
`a_part_added_under_a_stock_source_drill_is_judged_at_the_post_by_the_hole_law`
pins it.

**The thread grant.** Generation grants the thread allowance only over round
walls (`thread_op::walls_round` on the source material, in the op's frame).
The re-judge reads it the same way. If it cannot read it, it grants nothing,
which is the stricter law.

**Considered and dropped.** Re-using one shared function between
generation and the re-judge: a larger change to `run_job_arms` while the
v0.20.0 cut waits. The law split now lives in two places; a follow-up can
move it into one function.

**Cost to change later.** Low: one function in `kept_rejudge.rs`.
