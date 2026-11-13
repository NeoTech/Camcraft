## #882 - Bound the cost of the adaptive growth rings' opening

**Status.** Implementation, independent Linux fixture measurements,
debug/release STEP repeats, full adaptive-unit run and targeted quick
gate are complete.

**Baseline.** The fork is synced at v0.18.0, with process files at
`PROCESS_VERSION 52`. Issue branch `issue/882-adaptive-opening` merges
upstream main for the dependency fixes: its baseline `74c6faea` has the
same tree as upstream `4c229ed365e198bc64b4f3e9daae2f8eb31434f7`.
The named blockers #845 and #982 are closed. Historical held work is
`mkinney/Camcraft` branch `issue/882-adaptive-open-simplify`, commit
`70ac996f`; its earlier timings and fixture verdicts are historical,
not measurements of this candidate.

**What was decided.** Simplify each area's outline at the growth-ring
opening using Ramer/Douglas-Peucker, rather than simplifying only the
original pocket. `SIMPLIFY` is the existing `ARC_TOL`, 0.00025 mm
(0.25 um). Each closed loop is split at the vertex farthest from its
first vertex; an explicit stack avoids recursive call depth. Keep the
original region if simplification drops nothing, would leave an invalid
loop, or normalization changes the loop count or hole count. These
checks catch obvious topology changes; they are not a containment proof.

Erode the candidate by `rho + INSET` and dilate by `rho`, then intersect
with the original area. `INSET` is `SIMPLIFY + ARC_TOL + 0.00001 mm`,
or 0.51 um, to avoid creating tiny clipped wall chords in ordinary
cases. The test-only `OPEN_INSET` perturbation remains additive. The
load, travel and geometry budget constants are unchanged.

**Why.** The held investigation found that repeated offsets and openings
create most of the vertices: simplifying only the source outline did not
remove the expensive geometry produced during growth. The opening's
cost rises roughly quadratically with its input vertex count. Reducing
that count before each opening targets the repeated work.

**Safety argument and limits.** A per-chord simplification tolerance and
an erosion allowance alone are not a general proof for arbitrary polygon
topology. The candidate's exact boolean intersection is intended to
prevent the opening from adding area outside its input, subject to the
geometry engine's integer-lattice semantics. Independent tests check
the entire polygon difference, including holes, close components, narrow
necks, sharp notches, cyclic seams and degenerate outlines. Containment
does not prove that the resulting toolpath respects load, crossing,
travel, rapid-in-stock and kink laws; their fixtures must be measured
separately. Historical held-candidate outcomes are not substituted for
this branch's measurements.

In particular, mathematical opening is monotone, but independently
simplified approximate inputs need not preserve nesting. No nesting or
load guarantee is inferred from the exact-opening theorem. Clipping is
the containment backstop; the existing final path guards and measured
load/crossing/kink checks remain necessary.

**Measured evidence.** Independent verifier,
Linux x86-64, Intel Core i7-1360P (16 logical CPUs), 61 GiB RAM,
Rust/Cargo 1.99.1. Logs are local verification artifacts under
`/tmp/codex-882-verification/`, not tracked repository files.

The release `adaptive_hsm::hsm_report_full_resolution` report passed for
baseline and candidate. Values below compare the generated adaptive
output, not the separate Offset rings comparison run. Loads are the
measured fraction of cutter diameter; breaks are in-cut breaks over
5 degrees. Every reported adaptive output had zero verifier warnings.

| Fixture | Baseline -> candidate | Load, baseline -> candidate | Breaks, baseline -> candidate |
|---|---|---|---|
| Bore R12.5 | HSM, 1070 -> 1070 moves | 15.4 -> 15.4 % D | 0 -> 0 |
| Rectangle 60 x 40 | HSM, 13726 -> 13726 moves | 15.4 -> 15.4 % D | 0 -> 0 |
| L-pocket 60 x 60 minus 30 x 30 | HSM, 35130 -> 35130 moves | 15.7 -> 15.7 % D | 0 -> 0 |
| Narrow slot 60 x 14 | HSM, 14262 -> 14664 moves | 15.4 -> 15.7 % D | 0 -> 0 |
| Island 60 x 40, island 12 x 12 | Named fallback to rings, 162 -> 162 moves | 100 -> 100 % D | 42 -> 42 |
| Channel | Named fallback to rings, 60 -> 60 moves | 100 -> 100 % D | 40 -> 40 |

The island and channel remain fallback plans, not successful HSM-law
outputs. Their existing full-width cuts and sharp breaks are reported
explicitly rather than counted as lawful HSM improvements. The slot's
cut length changes from approximately 3029 to 3041 mm, air from 899 to
956 mm and lifted travel from 1780 to 1719 mm; its out-of-cut/cut ratio
stays 0.88. The report took 107.849 s baseline and 85.509 s candidate
while tests/compilation competed for the machine, so these are not
isolated benchmark timings.
The island's fallback program is unchanged, but its named refusal occurs
later: 122 -> 146 revolutions, approximately 600.3 -> 605.6 k processed
vertices and 9.3 -> 12.4 k tool radii of path. The same 600 k vertex /
5 k radius limits trigger it. The channel's refusal text is unchanged
(2 % rest/trochoid landing loop 1). The unit budget probes charge fewer
vertices: slot 248127 -> 219741 and channel 176482 -> 124506, with no
limit recalibration. The lowered-limit island still refuses after
11 revolutions (23.4 -> 23.2 k processed vertices).

**Actual STEP bore.** The frozen baseline's
`camcraft-app` test
`the_users_bore_roughs_adaptive_level_by_level_and_verifies_clean`
passed in 277.759 s; the initial candidate reached the exact move-count
assertion in 9.267 s, having passed its existing verifier/fallback/budget
safety assertions. The baseline's moves/planar-feeds/arcs are
15561/14837/1561; the candidate's are 16029/15372/1427. The first
candidate run therefore failed the old Linux exact-count pin. After
updating only the measured Linux triple, the repeat passed in 5.866 s
with the same triple. The initial debug run took 31.954 s with the same
triple, again failing only the old count pin; its final repeat passed in
31.308 s with the same triple.
The observed machine-specific end-to-end bounds are 15 s release and
40 s debug. They include STEP import, generation of every growth level,
and verification, so also bound each growth level on this machine; they
are not universal real-time guarantees. The test-artifact
invocation was `<cargo-reported pocket3d_hsm executable>
the_users_bore_roughs_adaptive_level_by_level_and_verifies_clean --exact
--ignored --nocapture`, with repository-root CWD. Build time is excluded
(baseline app release build: 72 s); the runs shared the machine with
other verification/builds. The old Windows and macOS count triples are
left as previously measured, not widened to guess this candidate's
results; those platforms still require measurement. All existing safety
assertions remain in place.

Seven simplification tests passed in release (28.10 s) and debug
(72.83 s). The coarse/fine growth-cost fixture alone took about
14.1/42.3 s in debug and is moved to the named heavy lane rather than
left in the quick gate. All seven tests were run with their assertions
before that classification change; the final six defaults pass (one
ignored) in 15.16 s debug. The three adversarial
tests use whole-region difference, not just boundary samples, for acute
notches, a 0.2 um wall around a hole, components with a 0.1 um gap, a hole
with a nested island, a 0.1 um sliver, both winding directions at five
cyclic seams, and empty/invalid-tolerance inputs.

**Rotary fixtures.** All six `rotary_adaptive` tests, including ignored
tests, passed before (233.470 s) and after (227.714 s), with no new
fallback or refusal verdict. Each row below is a developed band; move
and arc counts describe the generated planar band. No row has an
in-cut break over 5 degrees, before or after.

| Fixture and radius | Moves, baseline -> candidate | Arcs, baseline -> candidate | Load, baseline -> candidate |
|---|---|---|---|
| r7 bar, 7.0 skin | 7405 -> 7405 | 2313 -> 2313 | 0 -> 0 % D |
| r7 bar, 6.5/6.0 (each) | 21916 -> 21916 | 8031 -> 8031 | 19.5 -> 19.5 % D |
| r7 bar, 5.5 | 5732 -> 5733 | 1269 -> 1348 | 17.3 -> 17.3 % D |
| r7 bar, 5.0 | 7095 -> 8383 | 1739 -> 1851 | 18.4 -> 18.0 % D |
| Window, 7.0 | 3749 -> 4044 | 954 -> 916 | 0 -> 0 % D |
| Window, 6.0/5.0 (each) | 6919 -> 8285 | 1614 -> 2027 | 18.4 -> 18.4 % D |
| r6.2 neck, skin | 18261 -> 18261 | 6717 -> 6717 | 0 -> 0 % D |
| r6.2 neck, 5.7 | 5277 -> 5306 | 1329 -> 1582 | 17.3 -> 17.3 % D |
| r6.2 neck, 5.2 | 5867 -> 5490 | 1399 -> 1546 | 17.7 -> 17.7 % D |

The r7 bar's world-space total changes from 180360 to 183860 moves;
neither has an in-cut break. The Offset rings comparison digest stays
5263 moves / 11592310142122113043. Sub-micron geometry can change
margin selection and fitted arcs; identical output is not claimed.
The candidate reader also prints an extra skin row rounded to radius
7.000: one move, zero arcs, zero measured load and zero breaks, no rings
fallback. The reader deduplicates exact Z values but prints three decimal
places, so distinct nearby planes can appear under the same rounded
label; the exact Z of this extra row was not separately measured.

**Other fixtures.** Both `adaptive_band` tests passed before and after:
the 14/20 mm annular bands and user's band still refuse HSM up front
by name and use Offset rings. All four motion `pocket3d_hsm` tests
passed on the frozen baseline (33.226 s) and candidate (11.660 s).
The small-helix rectangle crossing test passed on the candidate
(1.517 s). Motion/app clippy, all targets with warnings denied, passed
(9.19/26.40 s).
The targeted `cargo xtask gate quick --base upstream/main` passes
(51 s, no VOID), with 271 default tests passed and 12 ignored heavy
tests across motion lib, adaptive band/HSM/orbit seam, motion pocket3d
and app pocket3d. The diff-sensitive gate ran the changed tests without
replacing the explicit heavy fixture runs above.

**Reproduction.** Build test executables with:

```sh
cargo test --release -p camcraft-motion --lib --test adaptive_hsm --test adaptive_band --test rotary_adaptive --test pocket3d_hsm --no-run --message-format=json
cargo test --release -p camcraft-app --test pocket3d_hsm --no-run --message-format=json
```

Debug builds omit `--release`. Executables come only from Cargo's JSON
artifact list. Invoke `adaptive_hsm` with
`hsm_report_full_resolution --exact --ignored --nocapture`, and separately
`a_small_helix_rectangle_never_crosses_itself --exact --ignored --nocapture`.
Invoke rotary and motion pocket3d binaries with
`--include-ignored --nocapture --test-threads=1`; band with
`--nocapture --test-threads=1`. The unit binary filters
`machining2d::adaptive::rings::simplify::` with `--nocapture
--test-threads=1`. CWD is the repository root. Logs and exact artifact
paths are in the corresponding `*-results.jsonl` files.

The full adaptive-unit run (`machining2d::adaptive:: --include-ignored
--nocapture --test-threads=1`) passes: frozen baseline 45 tests in
314.244 s, candidate 52 in 244.970 s. All budget twins and twenty slot
perturbations pass. Graphify
is unavailable on this machine (`graphify` is not on PATH); no generated
graph files are edited or committed.

**Dependency fixes retained.** #982 removed the coarse-chord stock
sliver rather than adding a tolerance to microlift acceptance: the
strict comparison still stands. #845 compares whole-revolution readings
when choosing the least overload. #1051 counts each margin attempt's
budget separately, retaining the limits. Their behavior is already in
the baseline; none of these acceptance laws is relaxed by this candidate.

**Considered and dropped.**
- Simplifying only the source area: the held investigation found no
  useful speedup because growth recreates the detailed vertices.
- Using a smaller tolerance without robust planner decisions: the held
  investigation found that even 16 nm changed fixture verdicts.
- Re-pinning a fixture to worse output, weakening a safety threshold or
  raising the geometry budget to hide a regression: disallowed. The load,
  crossing and kink laws remain acceptance requirements.
- Relying on Douglas-Peucker plus inset as a complete containment proof:
  insufficient for arbitrary topology; retain an exact boolean backstop.

**Cost to change later.** The production change is local to
`adaptive/rings.rs` and the new private `adaptive/simplify.rs` module.
Changing the simplification, inset or containment method requires a new
adaptive fixture comparison; these are toolpath-affecting choices under
the safety floor.
