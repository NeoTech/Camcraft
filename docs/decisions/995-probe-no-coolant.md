## #995 - a touch probe section starts no coolant

**Decided.** Every shipped mill post (fanuc, haas, sinumerik, heidenhain,
mazak, doosan-dnm, carvera; tormach already did) gates its preamble's
coolant on the section being a touch probe - the same test #523 / #574
use for the spindle, a section that states a speed of 0
(`spindleWait && !(sRpm > 0)`). Such a section resolves no coolant: no
M7 / M8 / M88 at its start, so no M9 / M89 at its end. The gate wins over
the op's ask: an explicit Mist / Air / Through-tool on a probe op is
neither posted nor refused.

**Why in each post, not in the stream.** The obvious alternative was to
have the probe op's dump write `PARAM Coolant off`. Dropped because:
- the stream does not know a section is a probe (`post::dump` sees a
  toolpath and an rpm), and the probe's TOUCH / FOUND records come
  after the preamble has already been written, so the rpm-0 test is the
  only signal available where the coolant is decided;
- `tormach.cps` declares no `coolantModes`, so the engine would refuse
  every probe op on Tormach by name for asking "off";
- a recorded or hand-written probe stream with no `PARAM Coolant` (the
  #926 `probe` stream - flood by silence) would still post M8.
The post-side gate is what Tormach already did (#108 / #362), so the
seven other posts now match it.

**Why an explicit ask is dropped, not refused.** Fluid or air on a stylus
and on the faces it measures spoils the reading on every dialect; there
is no coolant a probe wants. Refusing would block a probe op whose
coolant row was merely left on a non-Flood value (a machine whose
default is Air starts every op, probes included, on Air - #869), for no
safety gain. Recorded so a reviewer can overrule it: the cost of turning
it into a named refusal later is one `msg` per post.

**Goldens.** None move: every committed probe golden is posted
machine-less, and the non-Doosan posts state no coolant without a
machine. The #926 `probe` golden of `doosan-dnm` (PR #1002, unmerged
when this was written) is listed there as a known bug; whichever of the
two PRs merges second drops that known-bug entry and re-records the
golden without the M8 / M9.

**Left as it was.** doosan-dnm still writes no spindle-stop M5 ahead of a
probe's first move (#574 covered the other mill posts) - a separate gap,
not coolant.
