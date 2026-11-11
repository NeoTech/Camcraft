## #850 - The growth planner's geometry budget: a vertex count, not a clock

**What was decided.** The growth-ring planner (`adaptive/grow.rs`) gets a
third budget law beside #13's two (`adaptive/budget.rs`): every step it
tries counts the polygon vertices it runs through - the cleared area it
offsets and the ring it opens - plus each revolution's margin area. Past
`GEOM_MAX` = 500,000 vertices (summed over every margin attempt, like the
length budget) the plan is refused by name ("... too detailed to plan in
seconds (an island?)") and `pocket_hsm` falls back to the trochoid, then the
Offset rings, named as before. The length budgets (`LEAST_WORK_MAX`,
`WORK_MAX`) are unchanged: the numbers did not ask for it.

**Why a vertex count.** The issue's measurement holds: the #13 counter is
path length, but the planner's time is whole-ring polygon work per trial,
and its cost per tool radius differs by shape (island ~2 ms/r, channel
~1 ms/r). Measured here (release, this machine): the island's first margin
attempt spent 865 k vertices in 25 s, the 60 x 14 slot 252 k in 3.7 s -
time follows vertices to within a factor of two, path length not at all
(the island had used 13.7 k r, the slot 6.1 k r). A vertex count is
deterministic: the same project plans, or is refused, on every machine and
under any load - the goldens and the user's output never depend on how busy
the machine was.

**Calibration** (counts are deterministic; measured with a local probe,
not committed):

| fixture (suite) | ends | vertices | share of budget |
|---|---|---|---|
| 60 x 14 slot (`adaptive_hsm`, pocket) | Done | 252,328 | 0.50 |
| 60 x 14 slot (`adaptive::tests`, plan) | Done | 246,290 | 0.49 |
| 60 x 22 channel (`adaptive_hsm`) | Done (trochoid/rings later, unchanged) | 175,217 | 0.35 |
| whole-model pocket, largest corner (`whole_model_pocket`) | Done | 73,496 | 0.15 |
| level links (`motion level_links`) | Done | 37,545 | 0.08 |
| pocket overhead / machining2d | Done | 34,508 | 0.07 |
| 14 / 20 mm band, shoulder | refused up front (#13) | 0 | - |
| island 60 x 40, 12 x 12 | was refused by length after 865 k + 1.95 M | > 500 k | refused |

Every fixture that grows today stays at most half the budget (pinned at a
1.9x margin). The island is refused about halfway into its first margin
attempt: 88 revolutions, ~502 k vertices.

**Bound in seconds** (this machine, the whole island pocket - growth,
trochoid refusal and Offset rings): release 52.6 s before (the issue
measured ~68 s on its tree) -> 19.0 s after; debug 38.7 s after (before
not measured - by the same ratio well over a minute). The 14 / 20 mm bands
are refused up front by #13's estimate, before and after (0.14 / 0.21 s
release). A tighter budget (400 k, ~15 s on the island) was dropped: it
would leave the slot only a 1.6x margin.

**Considered and dropped.**
- *A wall-clock limit*: makes the toolpath depend on machine load (one
  machine plans, another falls back) - unacceptable for reproducible output
  and goldens. No wall-clock backstop was added either: the vertex bound
  already caps the time, and a clock refusal could never be pinned.
- *Counting trials*: the island and the slot both spend ~550 trials; trial
  count does not tell them apart.
- *Extending the up-front estimate to islanded regions*: the island's
  channel estimate is 339 r (the slot's 113 r), both far under the 2,000 r
  bound; an island-specific estimate would need a model of the C-ring's
  travel that does not exist yet, and would be a new geometric law (the
  island-cell decomposition #7 proposes is the real fix).
- *A cheaper per-trial geometry*: a different change (the issue says so);
  #44 measured that the walk is only 15-25 % of the time.
- *Weighting vertices super-linearly* (n^1.5 fits the island/slot time
  ratio better): harder to explain and calibrate; linear is enough.

**Cost to change later.** One constant in `budget.rs` and its pin in
`budget_fixtures.rs`. If the planner gets faster per vertex, the budget can
rise; if a real part that should plan is refused by it, the refusal names
the count, so the gap is visible and the constant is the lever.
