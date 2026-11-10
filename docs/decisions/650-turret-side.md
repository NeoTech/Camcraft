## #650 - lathe tool orientation numbers are read in the machine's turret frame

**Decided.** A machine states its turret side: `.mch` key `general.turret`,
`front` | `rear` (`Machine.turret`, parsed in `post::machine::parse`; any
other word refuses the file naming `turret`, and the Machines pane's validator
names `general.turret`). Absent, null or empty reads as **front** - every
machine file and project written before #650. The built-in Lathe states
`front` explicitly. The orientation chart (LinuxCNC's figure, drawn for a front
tool post) is mirrored in X on a rear turret: 1<->4, 2<->3, 6<->8; 5, 7, 9
unchanged (`camcraft_tools::turret`). The physical offset of the control
point from the nose centre, in the control's own X (radius) and Z, is the same
on both machines; only the number changes. This supersedes the line in
`630-nose-tip.md` that said the number does not depend on the turret side.

**Storage: canonical, displayed per frame.** `Tool.orientation` (and the
project key `orientation`) keeps the chart's own front-frame code - a physical
fact about how the insert is mounted. Every place a number meets a person
renumbers it into the frame of the relevant machine:
- the tool editor's orientation row reads the ACTIVE setup's machine
  (`setup_mount::turret_of_active`): the list runs 1..9 in that machine's
  numbers, the default reads "default: 3 - OD, cuts toward the chuck" on a
  rear turret, and the caption names the frame; picking rear "3" stores 2;
- the post (`posting_nose::posted_path`) reads the op's OWN setup's machine
  (`setup_mount::turret_of_setup`): the geometry uses the canonical code, the
  fit rule is judged on it, and the refusal's fitting list, the tool's own
  number and the `ToolOrientation` PARAM (so the Fanuc Lathe `ORIENTATION q`
  note) are in the machine's numbers, with "(front|rear turret numbering)"
  in the refusal;
- defaults follow automatically: OD insert 2 front / 3 rear, boring bar
  3 front / 2 rear, blade left corner 2 front / 3 rear; blade codes
  [1, 2, 6] front / [3, 4, 8] rear.
Why canonical: one tool can serve setups on different machines, and a
library tool must mean the same steel on each; a number stored "as typed for
some machine" would silently turn an OD tool into a boring bar when the tool
moves to a machine on the other side (a 2r diameter error). Front machines see
the stored code unchanged, so old projects and libraries round-trip byte for
byte and every front golden is unchanged (only the new
`nc/fanuc-lathe/facing_rear.nc` is added). No version step: nothing on `main`
writes `orientation` yet (#640 is unmerged), so no stored code needs
migrating.

**Considered and dropped.** Storing the per-machine number as typed (simpler
editor, no conversion): ambiguous once a tool is shared across setups or
libraries, and it makes the stored value depend on which machine was active
when it was typed. A per-tool "frame" tag: a second key for a fact that
belongs to the machine. Inferring rear from the vendor name: guessing.

**Mill-turn.** The same key and default: a mill-turn states `rear` if its
turning tools stand behind the spindle; the built-in `millturn.mch` states
nothing (front). No Tormach lathe `.mch` exists on this stack (every shipped
`tormach-*.mch` is a mill), so none was marked rear; a Tormach 8L/15L file
(#626's machines) states `"turret": "rear"`.

**The PathPilot data points (contributor's Hub 8L v2.14.4 conversational
samples - behaviour reference only, nothing copied).**
- (a) Facing with "Tool Orientation = 3", a turning tool on the X+ side,
  facing to centre: under the mirror, rear 3 = front 2 (tip (-X,-Z)), which
  fits facing ({-Z, -X}: front [2, 6, 7] = rear [3, 7, 8]). Unmirrored, 3 is
  the boring-bar tip (+X,-Z) and the facing would be refused. **Supports the
  mirror only.**
- (b) External threading asked for 8 where the tool was 6: under the mirror,
  rear 8 = front 6 = (-X, 0), a tip pointing at the axis - the OD thread
  form. Unmirrored, 8 points +X (an internal thread). **Supports the mirror
  only.**
- (c) A chamfer refused orientation 3 with "[1, 5, 6] or [2, 6, 7]". Read in
  our contact rule (corner tip within 45 degrees, axis tip within 90):
  - unmirrored, [1, 5, 6] = {-X, +Z} and [2, 6, 7] = {-X, -Z}: the two OD
    chamfers (a back chamfer on a shoulder facing the chuck, and the free-end
    chamfer) - refusing 3 = an ID tool. Consistent with an OD chamfer.
  - mirrored, rear [1, 5, 6] = front [4, 5, 8] = {+X, +Z} and rear
    [2, 6, 7] = front [3, 7, 8] = {+X, -Z}: the two ID chamfers (a step
    inside a bore facing the chuck, and the bore-mouth chamfer) - refusing
    rear 3 = an OD tool. Consistent with an ID chamfer.
  So (c) is consistent with the mirror **only if PathPilot read that chamfer
  as an internal one** (its X/Z inputs describing the bore side, or an ID
  chamfer selected). If the sample truly was an external chamfer with an OD
  tool, (c) fits only the unmirrored reading and the three points do not fit
  one mapping: (a) and (b) would need the mirror and (c) would contradict it.
  Without the sample's inputs this cannot be settled here. The mirror is
  taken because (a) and (b) are unambiguous about the tool (an OD turning
  tool, an external thread) while (c)'s geometry is not, and because the
  mirror is the only reading under which a PathPilot user's own numbers for
  their OD tool (3) and OD thread tool (8) post correctly. **Owed:** re-run
  the chamfer sample on the Hub as an explicit OD chamfer with tool 3; if
  PathPilot still refuses, the mirror reading must be revisited before #626
  relies on it.
- Pinned: `lathe_turret_side` (rear 3 posts front 2's path; rear facing with 3
  reaches centre and pins `facing_rear.nc`, byte-identical to the front golden
  but for the note's number; rear 8 = (-r, 0) and rear 6 refused in rear
  numbers; refusals in each frame; the chamfer pair as rear [1, 5, 6] /
  [2, 6, 7] for the +X contacts; the `.mch` key; the project round trip),
  `lathe_blade_corner::a_rear_turret_numbers_the_corners_in_its_own_frame`,
  `camcraft_tools::turret` unit tests, and the editor row's
  `a_rear_turret_numbers_the_row_in_its_own_frame`.

**Not changed.** The contact rule and the front chart (#630), the blade
geometry (#641), the cached toolpath and every front output. The Fanuc Lathe
post writes no orientation word to the control (G40), so only its tool note
changes number on a rear machine.

**Cost to change later.** The frame is one module (`crates/tools/src/turret.rs`);
if the chamfer re-test contradicts the mirror, `Turret::shown` is the single
place that renumbers, and stored tools (canonical) need no migration.
