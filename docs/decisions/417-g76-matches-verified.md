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

  None of these moves the finished form; all three should be checked in
  the PathPilot Hub simulation or on a lathe before the PR is flagged
  ready.
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
