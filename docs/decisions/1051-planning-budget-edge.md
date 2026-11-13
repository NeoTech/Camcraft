## #1051 - The planning budget is held per margin attempt

**What was decided.** The growth planner's two running budgets
(`adaptive/budget.rs`: law 2, 30 k tool radii of revolutions; law 3,
600 k vertices past 5 k r) are counted for each margin attempt on its
own (`Work::attempt`, called at the top of `Planner::grow`), no longer
summed over the attempts. The limits themselves, the up-front law 1 and
every refusal text are unchanged.

**What the knife edge was.** Not the limits. The planner grows an area
from a wall margin that closes 2.5 um a revolution; where the margin is
spent before the fronts are done (`Grown::Short`, about 140 revolutions
at the first rung) the whole plan is made again from a wider margin
(`MARGIN_SCALES` 1.0, 1.6, 2.4). Whether a plan that needs about 140
revolutions finishes inside the first margin or a revolution after it
is a matter of nanometres of ring geometry. That alone would only choose
between two HSM plans - but the budget was summed over the attempts, so
"ran out of margin" also meant "the retry starts with a whole attempt
already spent". The r7 bar's developed level at radius 5.0 (a band of
440 mm2, D6):

| openings deeper by | first attempt | second attempt | summed (before) | per attempt (now) |
|---|---|---|---|---|
| 0, 8, 16, 32, 510 nm | Short at 22.5 k r, 377-380 k vertices | Done in 24.5-24.7 k r, 396-400 k vertices | refused at 30 k r, 85-86 revolutions into the retry | HSM-planned |
| 62, 125, 250 nm | Done in 22.3 k r, 375-377 k vertices | - | HSM-planned | HSM-planned |

The window's levels 6.0 and 5.0 (the same band) are the same case, with
the rows at other nanometres (first attempt Done at 8, 125 and 250 nm).
The summed budget therefore jumped by 22.5 k r on a change of 8 nm.
Counted per attempt, the sums of one attempt moved by at most 0.7 %
(length) and 1.6 % (vertices) over every perturbation tried, so only a
plan within about that of a limit can still turn on such a change; the
nearest attempt on our fixtures is 24.7 k of 30 k r (1.21x under) and
400 k of 600 k vertices (1.50x under), both pinned as margins.

**Why this and not a refusal on both sides.** The plan in question fits
the budget from either margin, holds the load (18.4 % D on the suite's
dexel against the 19.8 % law) and names no overload; what the refusal
sent the level to is Offset rings at a full-width load with 13 breaks.
Nothing honest refuses the 22.3 k r plan that finishes in one attempt -
the levels above it on the same bar finish in 17.6 k r - so the only
stable verdict is "planned".

**Verdicts that change, and why.** Release, Linux x86-64, on PR #1080's
tree (this PR is stacked on it):

- `rotary_adaptive`, the r7 bar: level 5.0 was "refused past the budget,
  rings"; it is HSM-planned (8545 planar moves, 1935 arcs, 18.4 % D, no
  in-cut break past 5 degrees). The pin now names the four levels 6.5,
  6.0, 5.5, 5.0 as planned and forbids a budget refusal.
- `rotary_adaptive`, the window: levels 6.0 and 5.0 were refused past
  the budget; they are HSM-planned (18.8 % D, no break). Pinned the same
  way.
- Nothing else: in every motion suite and every app suite that plans
  Adaptive (`--include-ignored`, release) no other plan reaches a second
  margin attempt, and every other refusal (the island, the stepped
  block, the bands refused up front) falls inside its first attempt,
  with the same counts as before (island: 109 revolutions, 600 037
  vertices, 9.4 k r).

**The twins** (`adaptive::budget::twins`, heavy lane; openings 0, 8, 16,
62 and 250 nm deeper through #982's `rings::OPEN_INSET`): every level
under the skin of the bar and of the window is HSM-planned in every
twin, no budget refusal, no planner overload, the load within the
suite's law. Without the change both tests are red at 0 nm.

**What still differs between twins, and is not settled here.** Which
margin the plan is made from still turns on the nanometres, so the twins
post different (all lawful-load) plans. On the r7 bar's level 5.0 the
plan made inside the first margin (62, 125, 250 nm) posts 5 in-cut
breaks of 5.0-6.2 degrees at the band's end; the plan from the wider
margin (0, 8, 16, 32, 510 nm, the unperturbed bar among them) posts
none. Those breaks are #1050's (the same end kinks as the r6.2 neck's
`R62_BREAKS`), they are on main already whenever the first margin
suffices, and this change accepts no new plan that has them: the plan
it newly accepts is the wider-margin one. The twins hold level 5.0 to
at most 5 (`BAR_50_BREAKS`, to go with #1050) and the unperturbed bar to
none; the window's levels post none in any twin.

**Cost.** The "never hang" bound is now three attempts of 30 k r
instead of 30 k r in all. Only a plan that runs out of margin can use
more than one, and law 3 still stops a heavy plan inside its first
attempt. Measured: the bar's level 5.0 took ~11 s to be refused (30.1 k
r) and takes ~14 s to be planned (22.5 k + 24.6 k r).

**Considered and dropped.**
- *Lowering `WORK_MAX` to refuse level 5.0 on both sides* (20 k r sits
  between the bar's 17.6 k and 22.3 k levels): keeps the old pin green,
  but refuses a plan that holds every law, and leaves the jump in place
  for any other plan whose first margin runs out.
- *Refusing at once when a first attempt runs out past half the budget*
  (the retry "cannot fit"): saves the retry's seconds, same knife edge.
- *Hysteresis on the summed budget*: the sum jumps by a whole attempt,
  far more than any margin could absorb.
- *Removing the margin edge itself* (finish a nearly-done plan on the
  tail schedule, or pick the rung up front from the channel's length):
  it would make the twins' plans alike, not only their verdicts, but it
  changes every grown plan's wall runs; that is #882's work.
- *A budget over the three attempts of 3x, counted once*: the same
  bound, but a first attempt's spend would again decide the last one's.

**Stacked on PR #1080** (#982, itself on #1007): the twins need its
`OPEN_INSET` hook, its level reader, and its microlift fix - on main the
same levels flip to "refused for travel" on the lift edge once the
budget lets them through.

**What it would cost to change later.** One call (`Work::attempt` in
`grow`) and its pins: `budget::tests::each_margin_attempt_is_held_to_the_budget_on_its_own`,
the two twin tests, and the two `rotary_adaptive` pins' level lists.
