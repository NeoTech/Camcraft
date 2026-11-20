## #1481 - The ream on the lathe posts is pinned by tests and goldens; no post changed

**What was decided.** A new test binary, `crates/post/tests/ream_lathe.rs`,
posts an axial ream on the spindle centreline through each shipped lathe
post, on the shipped machine file of that post (`lathe.mch`,
`tormach-15l-slant-pro.mch`). It adds one golden for each lathe post in
`crates/post/tests/golden/ream/`. No post script and no source file
changed.

- `fanuc-lathe` writes one `G85` (never a different cycle), seated on R on
  the centreline, with the expanded chain suppressed and `G80` before the
  exit rapid.
- `tormach-lathe` has no canned cycle. It posts the expanded chain: feed
  in, feed out at the same F, no rapid in the reamed hole.
- A dwell stays with the ream: `G85 P` on `fanuc-lathe`, `G4` at the floor
  on `tormach-lathe`.
- A ream off the spindle axis is refused by name on the two posts.
- `every_lathe_post_is_covered` fails when a `*lathe*.cps` post ships
  without a row in the test.

**Why.** The two posts already post the ream correctly on `upstream/main`
`e048b103c`. The issue asks for the pins, and "Done means" asks for a
golden for each lathe post. On unchanged code only the golden test was
red (the golden files did not exist); the behaviour tests were green.
Three mutations of the post scripts prove that the tests catch a wrong
mapping (see the PR body).

**Considered and dropped.**

- Rows for the lathe posts in `ream_cycle.rs`. Dropped: that test posts on
  `3axis-vmc.mch` with holes off the spindle axis, which a 2-axis lathe
  refuses. The lathe needs its own stream and its own machine files.
- A change to `fanuc-lathe.cps` that suppresses the chain of a refused
  off-axis cycle (it posts today as plain `G01` moves, as `tormach-lathe`
  does not). Dropped: the program has a warning and is never released, and
  the change is not part of this issue.
- A reading of the retract feed of `G85` on a Fanuc T-series control. The
  post states "feed in, feed out at F". This PR pins our mapping only. It
  was not compared with a control manual or a machine.

**Cost to change later.** Low. A change to a lathe post's ream output
moves one golden; regenerate it with
`WRITE_GOLDEN=1 cargo test -p camcraft-post --test ream_lathe` and read
the diff.
