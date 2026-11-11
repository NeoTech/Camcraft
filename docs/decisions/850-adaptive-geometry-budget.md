## #850 - The growth planner's geometry budget: heavy geometry over a long walk

**What was decided.** The growth-ring planner (`adaptive/grow.rs`) gets a
third budget law beside #13's two (`adaptive/budget.rs`). Every step it
tries counts the polygon vertices it runs through (the cleared area it
offsets, the ring it opens, each revolution's margin area), beside the
revolution length #13 already sums. A plan is refused by name once it has
worked past `GEOM_MAX` = 600,000 vertices **and** walked past `GEOM_LEN` =
5,000 tool radii of revolutions (both summed over every margin attempt);
`pocket_hsm` then falls back to the trochoid and the Offset rings, named as
before. The #13 budgets (`LEAST_WORK_MAX`, `WORK_MAX`) are unchanged.

**Why two conditions** (rework after the heavy-lane check on PR #865). The
first cut refused on vertices alone (500 k). It cut a level of the user's
STEP bore (app `pocket3d_hsm`) that plans on main: 33 revolutions over a
finely tessellated outline, 1.14 M vertices, but only 2.6 k r of walk. A
vertex count measures outline resolution as much as planning difficulty.
Measured (release, local probe, not committed), the planner's time is in
the morphological opening (`rings::open`, two fine offsets per trial) and
grows about with the square of the ring's vertex count: ~60-100 ns per
(vertices per trial)^2 on the slot, the island and the bore alike. On that
measure the bore (200 s of growth for one level, on main) is the slowest
plan of all, slower than the island's whole refusal (~48 s). So **no
time-faithful deterministic unit can keep the bore and refuse the island
sooner**: any budget the bore fits under, the island fits under too. The
rule therefore bounds what the bore does not do: walk far. Every plan that
finishes on main is either light (at most 358 k vertices) or short (the
bore, 2.6 k r); the plans that never finish (the island, the stepped block,
the rotary developed levels) are both heavy and long well before the length
budget catches them.

**Measured** (release, this machine, main semantics = the budget lifted;
the counts are deterministic):

| fixture (lane) | on main | revs | vertices | walk (r) | growth time |
|---|---|---|---|---|---|
| user's STEP bore, one level (app `pocket3d_hsm`, heavy) | Done | 33 | 1,138,443 | 2,595 | 200 s |
| rotary r6.2 neck, level 1 (`rotary_adaptive`, heavy) | Done | 99 | 357,937 | 14,026 | 7.4 s |
| rotary developed level (`rotary_adaptive`, heavy) | Done | 112 | 333,172 | 17,593 | 7.2 s |
| rotary r6.2 neck, level 2 | Done | 113 | 305,905 | 15,556 | 6.4 s |
| boss island (app `pick_agnostic`, heavy) | Done | 67 | 289,301 | 3,094 | 4.5 s |
| 60 x 14 slot (`adaptive_hsm`) | Done | 109 | 252,528 | 6,072 | 3.6 s |
| 60 x 22 channel (`adaptive_hsm`) | Done | 37 | 175,217 | 2,251 | 3.8 s |
| rotary developed window (`rotary_adaptive`, heavy) | Done | 72 | 171,398 | 5,701 | 5.0 s |
| whole-model pocket, level links, pocket overhead | Done | <= 37 | <= 73,496 | - | < 2.1 s debug |
| island 60 x 40, 12 x 12 (`adaptive_hsm`) | refused (#13 length) | 146 + 163 | 864,899 + 1,086,955 | 30,222 | 48 s |
| stepped block Z -10 (motion `pocket3d_hsm`, heavy) | refused (#13 length) | 146 + 180 | 834,155 + 1,053,615 | 30,032 | 48 s |
| rotary developed level / window, heavy | refused (#13 length) | 144 + 86 | 380 k + 206 k | 30,100 | 11 s |
| 14 / 20 mm band, shoulder | refused up front (#13) | 0 | 0 | 0 | 0 |

**Margins.** Plans past `GEOM_LEN` that finish: the heaviest is 358 k, so
`GEOM_MAX` is 1.68x over it. The heavy plan that finishes, the bore, walks
2.6 k r, so `GEOM_LEN` is 1.93x over it. The slot and the channel are pinned
at a 1.9x vertex margin; the bore is pinned to plan every level as on main.

**The island's bound.** Release, the whole pocket (growth, trochoid
refusal, Offset rings): 52.6 s on main, about 22 s with this rule (refused
at ~600 k vertices and ~8 k r, inside its first margin attempt). The stepped
block and the rotary developed levels are refused sooner too, with the same
outcome as on main.

**Considered and dropped.**
- *Vertices alone* (the first cut, 500 k): cuts the bore. Raising it until
  the bore fits (1.7 M at a 1.5x margin) leaves the island at ~43 s: no
  useful bound.
- *A time-faithful unit* (sum of squared vertices per trial): it tracks the
  time well, but the bore is the most expensive plan. Any budget that keeps
  the bore lets the island run its full ~50 s.
- *A wall clock*: makes output depend on machine load. Not added, not even
  as a backstop.
- *Trial counts, or the up-front estimate*: neither tells the island from
  the plans that finish (both ~550 trials; estimates 339 r vs 113 r).
- *Islanded regions only* (a vertex budget where `slit` opened an island):
  it keeps the bore and the boss island (289 k), but it leaves the stepped
  block (48 s) unbounded and rests on a shape guess rather than on work done.
- *Simplifying the outline to the planner's tolerance before the opening*:
  it would cut the bore's 200 s and the island's time at the root, but it
  changes toolpaths. That is the "cheaper per-trial geometry" change, filed
  separately.

**Cost to change later.** Two constants in `budget.rs` and their pins:
`budget_fixtures.rs` and `adaptive::tests` (margins), `adaptive_hsm` (the
island) and app `pocket3d_hsm` (the bore plans every level as on main). The
refusal names both counts, so a real part refused by it shows where it
stood.
