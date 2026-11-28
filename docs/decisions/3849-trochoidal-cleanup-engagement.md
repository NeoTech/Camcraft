## #3849 - 2D Adaptive (Trochoidal): the clean-up passes are checked against the engagement limit

**Decided:** The clean-up chains of each level are measured against a
post-lane stock model at every limit, the 90.0 deg default included. When
they exceed the limit, the peel step steps down (1.0, 0.75, 0.5, 0.35,
0.25 of the step). The result is one of three:

- a step holds the limit: the path posts with the note "clean-up peel step
  was reduced";
- no step holds a CUSTOM limit: the whole-path measurement that `main`
  already had refuses the path (`GenError::Invalid`, "cleanup and lanes
  exceed the ... deg engagement limit"). This refusal stays as it was;
- no step holds the 90.0 deg default: the path posts with an advisory
  `Warning::EngagementExceeded` on the clean-up feed through the measured
  peak. The warning shows on the operation and in the status line. It
  does not hold the post.

**Why:** `main` measured nothing at the default limit, so a clean-up
overload at 90.0 deg posted silently (the 20 x 14 notch pocket with a
6 mm tool: 118.5 deg, no note). A custom limit was already a refusal, and
the Prime Directive prefers a refusal over a weaker diagnostic, so the
refusal is kept. The default limit gets a warning and not a refusal: a
refusal there would refuse paths that `main` posts today, and the risk is
tool load, not a gouge.

**Dropped:**

- Advisory warning for a custom limit, in place of the refusal (the first
  version of this PR). It removed a refusal that also covered lanes and
  links, and no test showed that lanes and links hold the limit.
- A whole-path refusal at the default limit. It also measures the lanes at
  90.0 deg, which `main` never did, and it can refuse programs that post
  today. That is a separate decision.

**Cost to change later:** Small. The default-limit outcome is one branch
in `plan_cleanup_checked`'s caller and one warning kind; a refusal there
would change only `adaptive_troch.rs` and its pins.
