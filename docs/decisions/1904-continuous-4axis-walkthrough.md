## #1904 - Continuous 4-axis walkthrough for the 770MX + microARC 4

Sources: CamCraft's own UI, post and machine file; the four PathPilot Hub
readings on issue #796; the LinuxCNC G-code reference for G93 / G94. No
Fusion or HSMWorks material was consulted.

- **Decided: the walkthrough ends with an air cut, and tells the reader not
  to cut.** **Why:** Rotary Parallel finishes both end faces of the model
  down to the fitted axis. On a bar held in a chuck the chuck-side end face
  is inside the bar, so that pass is a groove from the skin to the centre
  (the example program feeds to Z-0.03 at X-1.3125 with the stock going on
  to X-2). No setting keeps the pass off an end face: Bottom Z is a floor
  in the laid frame, not a radius, and there is no axial window. A guide
  that told a new user to cut this would be unsafe. Filed as #2343.
  **Dropped:** (a) a model as long as the stock, as the #796 kit golden has
  it: CamCraft then sees air where the real bar goes on, and the guide
  would hide the cut; (b) Bottom Z above the axis: it keeps only a sector
  of the bar and still cuts the end faces; (c) waiting for #2343: the user
  asked for the page now, and the air cut is also the Hub check that
  PR #2342 still needs. **Cost to change:** when #2343 closes, give the
  page a part that can be cut and drop the limit; the tests name the facts
  that must change (`the_pass_cuts_the_chuck_side_end_face_to_the_centreline`).
- **Decided: the demo operation is Rotary Parallel, not Rotary Offset.**
  **Why:** Rotary Parallel provides the finishing pass described here.
  Each Rotary Offset level that cuts develops two laps of the bar. The
  original machine file refused that count at -360..360 (#2344). The
  shipped file now states no A travel limit, and that refusal no longer
  applies. The refreshed example uses the shipped machine without an
  added limit. **Dropped:** retaining a bounded example machine and
  presenting its limit as a shipped restriction. No Rotary Offset machine
  run is claimed. Wrapping a drawing round a cylinder is not implemented.
- **Decided: the example tool has a 5/8 in flute.** **Why:** with a 1/2 in
  flute the shank check blocks the post at the chuck-side end face
  ("shank/holder at move N collides with the uncut stock"). That guard is
  right, and the page says so; the long flute is there only so that a
  program exists to air-cut. Pinned both ways
  (`a_short_flute_is_blocked_at_the_chuck_side_end_face`).
- **Decided: the committed program is what the committed project posts,
  not what a fresh build posts.** **Why:** a saved toolpath keeps fewer
  digits than a fresh one, so the two programs differ in a last digit of
  some A and F words. A reader opens the project file, so the page quotes
  that file's program. The project file is held structurally to the
  document that the test builds. Only cached move coordinates and tool
  axes permit one step in their six-decimal serialization lattice across
  platforms. The version, tool number, machine, parameters, cache structure
  and other values stay exact. Larger coordinate changes are rejected.
  **Dropped:** byte-for-byte comparison of independently generated cache
  coordinates, which can differ at platform rounding boundaries.
- **Decided: the pictures are SVG files drawn by a script from the
  committed project, model and program** (`site/tools/
  first_continuous_4axis_pictures.py`), and a site test holds the files to
  the script's output. **Why:** the UI renderer of #1902 (PR #2078) is not
  merged, and a PR must not add snapshot PNGs or hand screenshots. The
  operation reference already uses drawn SVG (`operation-paths.svg`); these
  use its colours. **Dropped:** stacking on PR #2078 as well as #2342.
  **Cost to change:** when #1902 merges, add UI renders beside them.
- **Decided: the page is in Getting started, beside "Your first 4-axis
  part", and is linked from the Multi-axis section.** Same place and shape
  as the indexed half of this issue (PR #2135).
- **Also corrected:** `continuous-4-axis.md` still said that no shipped post
  writes inverse-time feed, and the Multi-axis index said that
  `tormach.cps` "indexes A only". PR #2342 made both untrue.
- **Not exercised:** the program has not been run in the PathPilot Hub or
  on a machine. The issue's "Done means" asks for a Hub run to M30; the
  page lists what to check and asks for the result on #1904.
- **Review decision: compare the NC examples with tightly bounded numeric
  differences.** **Why:** supported platforms can differ at rounding
  boundaries in A and inverse-time F words. Keep the committed programs
  and the independent rate checks. Reject changes outside the numeric
  bounds, and reject changes to line structure or other words.
  **Dropped:** replacing each golden with one platform's output, which
  does not establish agreement across platforms. **Cost to change:** a
  post-format change needs new bounds and pins.
- **Review decision: this partial delivery uses `Refs #1904`.** **Why:**
  neither program has a recorded PathPilot Hub run to M30, and the wrapped
  engraving or slot is not available. The guide does not complete the
  issue's acceptance work. **Dropped:** keeping a closing keyword without
  that evidence. **Cost to change:** complete and record the missing work
  before closing #1904.
- **Review decision: inherit the feed-time fix and trajectory pin from
  parent PR #2342.** **Why:** the defect originates in that parent, whose
  head `b93fe06131248f91e41e90104e0d050409add852` has a review that clears
  the original feed-time and NC-comparison code findings. **Dropped:** a
  separate safety fix in this child. Refresh this child from the parent
  and validate the merged head before claiming the finding is fixed.
