## #417 - G76 output states the passes that were verified

- **Chosen:** the thread-cycle record states the laid passes in the
  cycle's own terms. With `G76Cycle` the two blocks
  `G76 P(m)(r)(a) Q(dmin) R(d)` / `G76 X Z P(k) Q(dd) F` describe the
  same passes the kernel laid and the verdict judged:
  - dd = the first pass's depth;
  - dmin = the rung (constant depth) or the plan's minimum (constant area);
  - d = the finishing pass's own depth;
  - m = 1 + spring passes;
  - a = twice the flank angle.

  Anything the cycle cannot state is refused by name in the post:
  - alternating flank infeed (a new `alternating` flag on the record);
  - a flank angle whose double is not 0/29/30/55/60/80 (the existing
    check; a 29.5° modified flank is a=59 and refused);
  - a one-pass thread (d would equal k).
- **The control's schedule, as modelled** (GSK988TA manual G76,
  Fanuc-compatible; LinuxCNC G76 for the flank sign):
  - roughing pass n reaches `max(dd·√n, previous + dmin)`, up to `k - d`;
  - the first finishing pass cuts d to k, and m - 1 more repeat k;
  - with a ≠ 0, each pass starts `depth·tan(a/2)` along Z from the
    start point.

  The clamp reading follows the manual's definition of Q(Δdmin) as the
  *minimum cutting travel* of a pass. Its OCR'd formula can be read as
  clamping against the unclamped previous depth, but then a pass could
  travel less than dmin, which contradicts that definition.
  `lathe_post_g76_schedule` computes this schedule from the posted words
  and checks it against the laid passes' depths and stations to within
  1 µm, for a constant-area flank thread (new golden `thread_g76_area.nc`)
  and for the default thread.
- **The existing mismatch, and the honest fix (golden changed):** the
  research comment on #363 said the control cut a decreasing ladder
  because we posted `Q = infeed` as the first depth. That is not what
  happens: dmin was also posted as the infeed, so the clamp makes every
  step the rung, and the control's roughing matched ours. The real
  mismatch was `R0`. With d = 0 the control roughs all the way to k
  (0.4, 0.8, 1.2) and then cuts its finishing pass at k again, which is a
  fourth pass at the root that the verdict never judged. It is not a
  gouge, because it is the same root, but it is not what was verified.
  The fix: R states the finishing pass's own depth, the last rung's 0.4.
  The control then roughs 0.4 and 0.8 and finishes at 1.2, the three
  passes verified. `thread_g76.nc` changes from `G76 P010000 Q400 R0` to
  `... R0.4`, and nothing else in it changes. The G32 output and the
  laid passes are unchanged (the #416 default stays byte-identical).
  The motion `turning_thread_cycle` test now pins `finish_allowance`
  0.4 instead of 0.
- **Why not the other way round** (add a root pass so R0 is honest):
  that would change the default G32 output and every pass the verdict
  judges, which #416 promised not to do, and it would add a spring pass
  nobody asked for.
- **A flank record starts at the crest's flank station** (from #416):
  the posted `G00 X Z` start is `c_first - k·tan(θ)`, so the control's
  pass n (at `start + depth_n·tan(a/2)`) lands where the kernel laid it,
  and the finishing pass lands on the lattice.
- **Not verifiable from the sources read, flagged for Hub / machine:**
  - The sign of the per-pass Z shift. We follow LinuxCNC's "positive
    angle, leading edge cuts more heavily": deeper passes move in the
    direction of travel.
  - Whether the control also shifts each pass's END Z, or ends every
    pass at the programmed Z. If the end is fixed, early passes run up
    to k·tan(θ) further (or shorter) at the runout, shallower than the
    root. That is not a gouge of the thread form.
  - Micron rounding: words are whole microns (R at 0.001 mm). With
    authored numbers off the micron lattice, the control's ladder can
    differ by under 1 µm. In an edge case where k - d rounds above the
    last roughing depth, it could add one sub-micron pass.

  None of these moves the finished form, so none blocks the merge. They
  cannot be checked in the PathPilot Hub simulation: its lathe is
  LinuxCNC's one-block `G76 P.. J.. K..`, not Fanuc's two-block G76.
  They stay open for a run on a Fanuc lathe control (or its vendor's
  simulator), recorded here and in the PR's "Not exercised".
- **Record plumbing:** `ThreadCycle.alternating` and `ThreadRec.alternating`.
  The stream line writes `ALT 1` only when set (absent reads 0), so
  every earlier stream keeps its bytes. The post property is
  `Alternating`. The project cache key is written only when set, so no
  version step. With G76 chosen, an alternating thread now carries a
  record and is refused by name, instead of meeting "no thread cycle
  record".
- **Cost to change later:** a dialect with an alternating cycle (Haas
  one-block G76 P2/P4, a LinuxCNC post) reads the same flag. Changing
  the clamp reading is a one-line change in `infeed::area_rungs` and in
  the test's model.
- **The Tormach (LinuxCNC) G76 reads the new finish allowance too**
  (merge with main's #612, 2026-11-13). Main's PathPilot lathe post
  refused any non-zero `FinishAllowance`, which was right while the
  field meant an extra allowance (0 on every radial ladder). Under this
  PR the field is the finishing pass's own step, never 0, so every G76
  would have been refused. Chosen: LinuxCNC's `R1` ladder (J, 2J, ...
  clamped at K) has no finish word, but its finishing pass takes the
  ladder's own last step, `K - (ceil(K/J) - 1)·J`; a record whose
  finish allowance is that step (within a micron) posts, any other is
  refused by name, and an alternating record is refused by name.
  Dropped: dropping the check (a stated allowance the control cannot cut
  would pass silently), or posting R with another degression (changes
  the passes the verdict judged). Pinned by
  `lathe_post_tormach_thread::g76_takes_the_record_whose_finishing_step_is_the_ladders_own`
  and the alternating case in `a_g76_record_the_cycle_cannot_state_exactly_is_refused_by_name`.
- **TH1 Stale after a save/load is a main defect** (not this PR's data):
  the save snaps every move to the 1 um lattice, the in-memory path
  signature hashes the unsnapped moves, so an off-lattice predecessor's
  signature moves across a round trip and every op behind it reads
  Stale. Reproduced on upstream/main with a 12 TPI radial thread;
  reported to be filed on its own, not patched here. The infeed pin now
  also asserts `rest::prior_digest` is equal before the save and after
  the load.
- **R comes from the rounded words' own ladder** (M3's review of head
  49d0a3c2, 2026-11-13). Each G76 word rounds to the least increment on
  its own, and the control works on the words: under G20 the default
  thread's `Q157 P472 R0.0157` put k - d = 0.0315 one increment past
  the second rung 0.0314, so the control cut a fourth, unverified pass.
  Chosen: count the record's rough passes N from its own (mm) ladder,
  rebuild rungs N-1 and N from the rounded Q/Q/P, and write the R whose
  last rough depth P - R lies in (rung N-1, rung N] nearest the record's
  allowance - exactly N rough passes by construction (inch golden:
  R0.0158). More than one least increment from the record's allowance is
  refused by name, naming the Q/Q/P words and the window. Dropped:
  re-rounding R alone with ceil/floor (fixes this golden, not a ladder
  whose sqrt rungs drift); a tolerance wider than one increment (would
  let a visibly different finishing depth post); checking every rung's
  depth against the laid one (long sqrt ladders drift by a few
  increments with no extra pass, and would refuse legitimate threads).
  The Tormach (LinuxCNC) G76 has the same class of gap in J/K (inch
  default `J0.0314 K0.0944` gives ceil(K/J) = 4 passes for a 3-pass
  record); left out of this PR because its goldens must stay
  byte-identical to the Hub-run files, and filed on its own.
