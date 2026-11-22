## #3335 - An armed Boundary row takes viewport clicks in every pick mode

**Decided.** `interact::feature_routed` returns true while a Boundary row
(Include, Exclude, an extra entry, the open boundary or "Do not touch") is
armed. A viewport click then goes through `feature_click::click` to the row,
also when "Click selects" is on Models. Support for "Do not touch" on more
operation kinds is not in this PR: one sub-issue for each kind (#3343 -
#3349).

**Why.** A test that drives a real click into the "Do not touch" group of a
2D Offset showed that in Models mode the click selected the whole model
("selected guard") and the group stayed empty. `feature_click::ask` already
takes faces and edges for an armed row in every mode (#1727); only the route
did not agree. The same route rule already holds for the WCS role draft and
the start/exit pick.

**Dropped.** (1) Force the toolbar mode to Edges each frame while a row is
armed: it changes a user setting behind his back, and the toolbar shows a
mode that he did not choose. (2) Disable the Models radio while a row is
armed: it hides the cause and does not help a click that a script sends.
(3) Add "Do not touch" to more kinds here: that changes toolpaths and posted
G-code (safety floor), so each kind gets its own `effort:high` or `max`
issue.

**Cost to change.** One line in `feature_routed`, pinned by the two tests in
`crates/ui/src/egui_app/tests/do_not_touch_click.rs`.
