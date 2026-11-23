## #3447 - setups keep their creation order

**What was decided.** `SetupDef` has a new field `order: Option<u32>`.
`CreateSetup` writes one past the highest stored order (0 for the first
setup). `setup::setups` sorts by this field. Setups without the field
(every earlier file) sort first, by name, as before. The project file
saves the field as an `order` key on the setup row, and the project
version steps from 76 to 77. `get_job`, `add_setup`, the operation order and the post
all read `setup::setups`, so they all follow the new order.

**Why.** The issue asks that a setup added second never posts first. A
setup made with `datum_from` takes its datum and its stock from the
source; it must come after it. The operation fix of #3022 uses the same
pattern: a stored position, and the name order only for older data.

**Considered and dropped.**
- Sort by the entity id (creation order without a stored field). Dropped:
  a project load creates the entities again in file order, and undo of
  `DeleteSetup` creates a new entity. The order would not survive.
- Give each older setup an order at load time. Dropped: it changes the
  saved bytes of every older file without a user action. Name order for
  these files is the old behaviour, so nothing changes for them.
- Add the reorder command in this PR. Dropped to keep the fix small. It
  is filed as #3484 (command, MCP tool, refusal when a setup would post
  before its datum source).

**Test change.** `setup_delete_order.rs` pinned the old law: its fixture
created `B_Middle` last and expected it in the middle by name. The
fixture now creates the setups in A, B, C order. The test still pins
that undo of `DeleteSetup` gives the setup back its old place.

**Project version 77 (second review).** The `order` key changes the
posted program order. An older build that ignored the key would post a
setup before its datum source with no message. That is the rule of v50:
an old reader must refuse the file rather than post in name order. So
this PR steps the version to 77. The step is pure: a v76 file has no
`order` key, opens unchanged and keeps name order. A v76 build refuses a
v77 file. Dropped: an additive key with no version step, because an old
reader then posts the wrong order with no warning.

**Cost to change later.** Low. The field is optional; a reorder command
writes it.

**Review boundary decision.** Refuse `CreateSetup` when the highest stored
order is `u32::MAX`. The last representable order remains usable. Checked
addition prevents a tied order that could sort a new setup before its datum
source. Renumbering existing setups was dropped because creation should not
rewrite their stored order. The boundary pin also loads the maximum order
from a project and checks that refusal leaves the document unchanged.
