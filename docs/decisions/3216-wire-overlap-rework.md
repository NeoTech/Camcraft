# 3216 - wire overlap rework (PR #3617 review items)

Date: 2026-11-25. Recorded by the rework worker for PR #3617 (issue
#3216), per the review of head f019072d8 (2026-11-25, eight items).

## Item 1 - cache invalidation (Centre run-out clearance)

Implemented. The Centre slit's run-out clearance places its threading
point in the air, so the generation stamp now digests it for the
Centre side as it always did for left / right. Chosen because the
value already shaped the generated moves; only the stamp missed it.
Nothing was dropped.

## Item 2 - overlap bounds (the seam segment)

Implemented. `ring::past_seam` now rides the seam edge's own remainder
back to the seam point, so a stated overlap covers exactly its arc;
a ride longer than one full lap is refused by name at generation
("rides past one full lap ... state a shorter overlap"). Chosen over
a cap because a capped ride would post a shorter burn than the note
states - the review's own finding. Refusing past one lap drops the
"extension stops after one full lap" behavior that silently cut less
than stated.

## Item 3 - pass boundaries and verification (open skims)

Implemented. The representation is the plasma precedent: a Rapid
travel move joins one pass's run-out to the next pass's run-in (no
wire through it), keeping the path one chained program. The sim's
polyline records each travel as a pass boundary; `passes::split_open`
splits an open run at the boundaries, each pass judged at its own kerf
offset and its own ends. The post (`announce_skims`) announces each
skim's condition after the travel; wire-iso.cps cuts the wire at the
run-out, travels, and threads anew at the run-in (AWT codes or M00
notes). The Sodick post refuses an open run's skims by name at its
rapid guard instead of writing a structure its manual does not state
- a named refusal, never a silent wrong post.

Correction (the follow-up rework): item 3 said "the post announces each
skim after the travel" as if it covered every open run. It covered
left / right only: wire-iso.cps's `openRun()` did not match "centre",
so a Centre slit with a skim reached the rapid guard unthreaded and the
whole post was refused. Fixed: `openRun()` matches "centre" too - a
Centre travel boundary cuts the wire at the run-out, travels, and
threads again at the run-in like left / right - and the note sites say
the open run's words for Centre: the SIDE note names the centre-line
slit through the air with the kerf law said skipped (the "KERF IN THE
PATH" note is closed runs only), and the thread and cut notes say the
run-in / run-out variants. A Centre post test with one skim pins the
path against its own golden
(`crates/app/tests/golden/nc/wire-iso/wire_contour_centre_skims.nc`).
The verify wording stands as it was: the ends' clearance is judged and
the kerf law is said skipped by statement.

## Item 4 - unsupported combinations (burnout, 4-axis)

Refused by name at the command door (`wire_op::overlap_refusal`,
called from `condition_of`, so the create, edit and retarget doors,
generation and the verify line all say it). The review allowed
implementation or refusal; refusal chosen because the burnout cuts
offset rings with no seam lap, and a 4-axis cut leans rulings between
two profiles - implementing a seam-riding overlap there is new
geometry, not a fix to what the feature claims. The acceptance lost:
an overlapped burnout or 4-axis cut stays unsupported until asked for
by name.

## Item 5 - taper overlap

Refused by name at the same door, for the same reason: the tapered
builder leans one clean ring (`taper.rs` drops two endpoints and
assumes one ring), and an overlapped lap is not one. The review's
alternative - apply the overlap after taper generation - was dropped:
it would need the overlap re-derived on the leaned lower plane, new
geometry rather than a door check, and no user asked for a tapered
overlap burn.

## Item 6 - the three-file merge

Merged keeping both sides: the MCP SKILL.md line is main's text plus
the PR's `overlap` field; `wire_verify.rs`'s helper is main's picks /
restraint fields plus `overlap: 0.0`; `wire_page.rs` keeps the PR's
closing-overlap row and centre-slit hints with main's `picks` start
group.
