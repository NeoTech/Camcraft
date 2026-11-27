## #2443 - Drill Auto reads the depth in material; the Deep cycle choice

**Decided.**

- Auto measures a hole's depth from its mouth (`Hole::entry_z`), or from
  the R plane when the R plane stands lower. It never counts the air
  between a high R plane and the mouth. The same reading is in the axial
  cycle (`drilling::cycle`), the radial cycle (`drilling::cross`, depth
  from d = 0, not from the R standoff) and the setup sheet. The drill page
  note already read `entry_z - floor_z`, so generation now agrees with it.
- A new cycle choice `Deep` (`"deep"` in the project file and MCP,
  "Deep peck (full retract)" in the combo) resolves to the existing
  `CycleKind::Deep`, which every canned-cycle post writes as `G83`.
- `list_operation_types` already lists the cycle words (#3449). The test
  now also asks for `deep`.

**Why.** A 3.5 mm spot under a 5 mm air gap read as 8.5 mm deep and posted
a `G73` with a peck step longer than the hole. The cycle choice must follow
the material that the drill cuts. A user with a deep or gummy hole had no
way to ask for full retracts.

**The plunge still starts at the R plane.** Only the choice of cycle reads
the mouth. The feed moves, the peck steps and the posted R word do not
change.

**Dropped.**

- Measure from the stock top over the hole. The mouth is what the hole
  list stores, and the R-plane law (#91) already keeps the rapid out of
  uncut stock.
- Put `Deep` at the next free index of the enum in the middle of the combo.
  It is appended (index 10), so the combo indices and the digest bytes of
  older operations do not change.

**Review fixes (PR #4316).**

- The radial peck budget reads the full travel from the R standoff
  (`r_d - bottom`), not the depth in material. Auto still picks the cycle
  by the depth in material. The pecks run from the standoff, so a budget
  that reads only the material let a far standoff (30000 mm) make 10,000
  pecks, and a step that cannot move `t` (R at 1e20) loop for ever. Both
  are refused as `peck plan unusable`. The loop also refuses a peck that
  makes no progress.
- `GENERATOR_OUTPUT_VERSION` goes from 1 to 2 (the #3704 rule). Auto's
  output changes and its parameter digest does not, so without the bump a
  cache saved on main reads current and posts the old cycle. Every cache
  saved under version 1 reads Stale once and clears on regenerate. The
  committed catalog caches are rebuilt and the pinned stamp constants of
  `deburr_cache_stamp` and `wire_stamp` are captured again, as #3704 did.
- Dropped: a salt only on Drill stamps (the way #4062 salts the milling
  envelopes). The rule of #3704 names one knob for generator output, and a
  second, kind-specific knob would leave two ways to do the same thing.

- Auto counts uncut stock over a mouth as depth. The quick gate showed
  that "from the mouth" alone is too short where stock still stands over
  the hole: a raw bar r20 over a part r10 hid 10 mm of steel, and Auto
  chose chip breaks where full retracts are needed. Now:
  - an axial hole reads the highest material over the drill disk (the
    part, the raw stock, and, only where the raw stock stands above the
    mouth, the stock the program ahead leaves). The app passes it in as
    `CycleParams::material_top`; the depth is from the higher of it and
    the mouth, clamped to the R plane;
  - a radial hole reads the raw stock's surface over its mouth
    (`start_d(h, stock_r)`), clamped to `0..=r_d`.
  The issue's case (stock already faced to the plate top) still reads
  3.5 mm and drills `G81`. Pinned by `drill_prior::auto_material` and the motion
  tests `auto_counts_the_material_over_the_mouth` and
  `a_cross_hole_counts_the_raw_stock_over_its_mouth`.
- The setup sheet names the cycle the generator chose (the op's current
  cache), and says `auto` when there is no current cache. It cannot read
  the stock, so a second, stock-blind reading of Auto there would
  disagree with the program.
- Dropped: motion reads the stock itself. That needs a new dependency of
  motion on the stock model; the caller passes the value in instead
  ("No new dependency between our crates without a decision").
- Dropped: the program ahead always read. Its simulation costs time, so
  it is read only where the raw stock stands above a mouth.

**Second review fixes (PR #4316, review on 4cbc56ee9).**

- The Auto limits are inclusive within `AUTO_TOL` (1 micron). Before, a
  depth exactly on a limit (16 mm on a 4 mm drill: 4 x d) chose the cycle
  by float roundoff: a stock radius read as 25.000000000000004 gave full
  retracts on one system and chip breaks on another. The tolerance is in
  `auto_kind`, so the axial cycle, the radial cycle and the drill page
  agree. Pinned by `an_auto_limit_reads_the_same_under_roundoff`.
- The cylinder reference chain of `drilling_cross_box` (three stations,
  16 mm in material on a 4 mm drill) is recorded again. It is now the
  chip-break peck (19 moves). The chain of efd1f26 (43 moves, full
  retract) read the depth from the R standoff.
- Dropped: a strict `<` at the limits (a depth on 4 x d goes Deep). The
  law says "up to four diameters" and the drill page says the same; a
  tolerance keeps the words and removes the roundoff.
- Dropped: move the fixture off the limit. That fixes one case and leaves
  the class open.
- The two app tests of the material reading moved from a new top-level
  binary into the `drill_prior` suite (`drill_prior/auto_material.rs`),
  by the rule "New tests join a directory suite" (#3187).

**Effect on programs.** A hole whose R plane is more than one diameter
above its mouth can change from `G73` to `G81`, or from `G83` to `G73`,
under Auto. A program that does not use Auto, and a hole whose R plane is
at its mouth, posts byte-identical.

**Cost to change later.** One expression in three readers, and one enum
word.
