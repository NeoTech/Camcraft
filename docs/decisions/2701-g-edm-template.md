## #2701 - G-EDM Wire EDM (template) machine file

**Decided.** `assets/machines/g-edm.mch` ships as "G-EDM Wire EDM
(template)": kind `wire_edm`, a table with X and Y, a `guides` head,
`edm.awt: false`, and `controller.post: "G-EDM Wire EDM"`. It states no
travel, no rapid, no guide height, no taper, no wire diameters, no
`edm.codes` and no conditions. The description names gedm.org and tells
the user to enter the travels, the guide heights and the measured
conditions in the Machine editor.

**Why.** Each G-EDM build is different, and the public G-EDM pages give
no fixed travel or spark gap. The unstated-limits law
(`assets/machines/AGENTS.md`, "absent is not zero") forbids a guessed
limit. With no condition, the Wire Contour door (#531) refuses the
operation by the condition's name, so the template cannot cut before the
user states a measured gap.

**The template names its post.** The Setup page then offers the G-EDM post
when the template is mounted (#2374). The mount does not change the post by
itself. Dropped: no `controller.post`. Then a G-EDM user must find the post
in the picker, and the default wire post writes ISO words that a Grbl-family
controller does not read.

**The tests join `g_edm_post.rs`** (the G-EDM suite), not a new top-level
test file: `punch_doc` now has a `punch_on(machine_text, ...)` form.

**Cost to change later.** A stated limit from a future public G-EDM
specification is one edit to the `.mch` file and to the test's expected
axes.
