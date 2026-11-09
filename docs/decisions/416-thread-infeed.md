## #416 - threading infeed method and decreasing-depth pass schedule

- **Chosen:** a Threading op carries an infeed plan
  (`turning::ThreadInfeed`, op field `infeed_plan`): method (radial,
  flank, alternating flank) with a flank angle, depth schedule (constant
  depth, constant area), minimum depth per pass, finish allowance and
  spring passes. The kernel (`turning::infeed`) lays the pass list from
  it; `thread_planned_in` cuts it; `thread_in` is the default plan.
- **The existing `infeed` field is the first-pass depth.** In a
  constant-depth ladder the first pass and every pass are the same
  number (LinuxCNC G76 J with R1.0 works the same way), so a separate
  first-depth field would only add a cross-field rule. In a constant-area
  ladder `infeed` is Δd and pass n reaches Δd·√n. Dropped: a second
  `first_depth` field.
- **Defaults reproduce today exactly.** The default plan (radial,
  constant depth, allowance 0, 0 spring passes) calls the same
  `blade::rungs(root, ref_top, infeed, 0.0)` the kernel called before,
  every station is `c_first + 0.0`, and the record is filled the same way.
  The existing golden suites (motion `turning_thread`,
  `turning_thread_cycle`; app `turning_thread`, `lathe_post_g76` with its
  G76 golden, `lathe_field_tools`, `project_roundtrip`) pass with no golden
  touched. The stamp digests the plan only when it is not the default, so
  every cached thread stays fresh. The project key is written only when
  not the default, so no version step (the approach / holes precedent).
  A file written with a non-default plan and opened in an older build
  loses the plan and cuts the radial ladder. That is not a gouge, and it
  is the price the additive law always pays.
- **Constant-area schedule:** cumulative depth `max(Δd·√n, previous +
  Δdmin)`, the last roughing pass clamped to `root + allowance`. The
  clamp is on each pass's own travel. The public G76 references define
  Δdmin as the minimum cutting travel of a pass (GSK988TA manual G76,
  Fanuc-compatible). The manual's OCR'd formula fragment can also be read
  as clamping against the unclamped previous depth, but that reading would
  let a pass travel less than Δdmin, against the parameter's own
  definition. Recorded for #417.
- **Finish allowance:** roughing stops at `root + allowance`, then one
  finishing pass at the root. 0 means the last roughing pass is the root
  itself (today's ladder). **Spring passes:** repeats of the root pass,
  capped at 98 (the two-digit finishing count of the lathe cycles, one
  of them the finish).
- **Flank infeed geometry and the safety law:** each pass's start moves
  along its travel by `-(tip - root)·tan(θ)`. Shallower passes stand
  behind the lattice centre, deeper ones move toward the direction of
  travel, so the leading flank cuts and the trailing flank rides the cut
  flank. The finishing pass and the spring passes stand on the lattice
  itself. A pass h above the root shifted by h·tan(θ) keeps both flanks
  inside the finished V exactly when θ ≤ A/2 (A = the insert's included
  angle). So θ > A/2 is refused by name at Create, Paste, Edit and retype
  (`thread_gate`, which reads the named insert's tip angle), and again by
  the kernel at Generate (a later tool change meets it there). The shift
  moves the pass start only. The lead is unchanged, and every pass starts
  on the meridian, so the helix phase relative to the spindle is the
  lattice's.
- **Shift sign:** LinuxCNC's G76 reference says a positive compound
  angle "causes the leading edge of the tool to cut more heavily". The
  plan follows that sign. Containment holds either way, so the sign is a
  chip-control question, not a gouge question.
- **Alternating flank:** each roughing pass alternates the sign of the
  same shift (tip zig-zags between the two lines through the root at ±θ).
  Containment is the same proof. There is no single compound angle, so
  the path carries no thread-cycle record (a note says so). The passes
  post as G32. With G76 chosen, the post refuses on the missing record;
  #417 gives the record a flag and the refusal its own name.
- **A flank plan's record starts at the crest's flank station:** a rapid
  at the clearance radius to `c_first - height·tan(θ)` opens the span, so
  the record's start is where a pass at the crest would start, and pass n
  is at `start + depth_n·tan(θ)`. Only flank plans get this extra rapid,
  so the radial path is unchanged.
- **The verdict reads the laid geometry:** `thread_warnings` takes each
  pass's station from its own plunge and arc endpoints. So a shifted pass
  is judged where it stands (pinned by `the_verdict_reads_the_shifted_pass`,
  which moves a pass past the half-angle and is convicted). The lathe
  stock simulation does not simulate threads (it refuses them by name, as
  before).
- **Defaults of the new numbers:** flank angle 29.5° (the modified-flank
  figure the tooling guides give for 60° forms; read only by flank and
  alternating) and minimum depth 0.05 mm (the guides' smallest
  recommended infeed; read only by constant area).
- **Considered and dropped:** a per-depth repeat pass (#418) and in-pass
  chip-break dips (#419) are their own issues. Haas-style one-block G76
  P1-P4 has no post here.
- **Cost to change later:** the plan is one struct with a DTO mirror. A
  new method or schedule is an enum arm in both, plus a `passes` arm.
