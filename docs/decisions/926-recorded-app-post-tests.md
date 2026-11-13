## #926 - The ten probe and arc post suites move to crates/post/tests (slice 2)

**What was decided.**
- The probe suites (`fanuc_post`, `probe_dialects`, `probe_guard`,
  `probe_report`, `probe_tormach`, `inverted_probe`) read recorded `.cnc`
  streams instead of calling `motion::probing::probe_cycle`. Each stream is
  recorded once per distinct (cycle, dump) pair the old tests built (28 files
  in `crates/post/tests/recorded/inputs/`). They are written and guarded by
  app's `recorded_post_streams`, beside slice 1's probe and drill guards: app
  is still the only crate that sees both motion and post, and the `.cnc`
  stream is the dumped records, so the guard has to call `post::dump`.
- `probe_guard`'s first test also compares every GUARD with the path's own
  moves. `cl::text` carries no probe touches, so it now reads two recordings
  per mode: the stream and the path as `.cl` (`probe_<mode>.cl`). The touch
  moves are found from the stream's motion records (one record per move, in
  order), and positions are compared on the micron lattice (1e-6, not 1e-9).
- `vertical_arc_entry` is split. The generator half (true arcs, tangency, the
  oblique note, verify) goes to `crates/motion/tests` (motion already sees
  sim). Each of its three tests also guards its path's `.cl` recording. The
  posting half goes to `crates/post/tests` and reads those recordings.
- `arc_planes`, `inverted_equivalence` and `inverted_post` need no
  recording: their inputs are built with cl's `PathBuilder` or by hand.
  `common::gen_to_cltext` / `cl_read::parse_cl` became `cl::text::to_text` /
  `from_text`; the golden still matches byte for byte. The one project-file
  test (`ArcDto`) stays in app, in `float_roundtrip`.
- `inverted_support` and `macro_b` move with their suites. App's
  `inverted_axis_display` was the last app user of `inverted_support`. It now
  reads the fixture stream as `recorded/inputs/inverted_probe.cnc`, which
  post's `inverted_post` guards, so app keeps no second copy of the
  300-line helper.
- Every file in `recorded/inputs/` must have a generator.
  `every_recorded_input_has_a_generator` (app) compares the directory with
  the guard tables, so a file that nothing writes fails the build.
- The goldens stay in `crates/app/tests/golden/` (the post suites use
  `recorded::app_golden`). Other app suites, and xtask, read the same
  goldens. One home for the goldens is #919's sub-issue 6.
- xtask's asset and gate-map unit tests named `fanuc_post` / `arc_planes`
  as app suites. They now read every crate's suites step, and that change
  bumps PROCESS_VERSION.

**Why.** This is the issue's Done means: the tests run outside app, read
recorded input, add no crate edge, and each recording has a generator-side
test that fails when the recording goes stale. It follows the pattern slice 1
(PR #1002) already set.

**Considered and dropped.**
- Recording `.cl` paths only, and dumping them in post: `cl::text` has no
  probe touch records.
- Adding serde to `Toolpath` so the whole path could be recorded: this is a
  new dependency in cl.
- A dev-dependency of post on motion: this is a new crate edge (#919 Q4).
- Moving the goldens into post: other app suites read them.

**Cost to change later.** Low. The guard tables are one array in
`recorded_post_streams.rs`, and `WRITE_GOLDEN=1` regenerates every recording.
