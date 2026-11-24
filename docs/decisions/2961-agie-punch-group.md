## #2961 - Group two captured punches in one native wire EDM job

Use two complete punch solids on one 20 mm plate. Each operation owns one solid, one threading hole in the waste, and its own capture declaration. Every model in the document must have an owning operation. Both operations share one setup and wire tool. Control-generated separation can run after both profiles. Cross-work order must therefore be immaterial to clearance and capture.

Compare conservative XY bounds that include the entire retained solid, the hole, the approach and return, the 3 mm lead, the largest possible rough miter, and the wire radius. A separating X or Y interval proves that every pass of one work clears the other. This can refuse some safe nested or diagonally spaced layouts. Those layouts require a more precise geometric proof.

Keep both geometry files in common setup coordinates with zero work translations. Attach each resource and start point to its own WORK in one native GROUP. The single-work generation and names retain their previous format. Pair names reserve one of the eight DOS filename characters for the work number. Publish the complete resource map, piece and two geometry files in one directory rename.

### Port onto the zero-step setup writer (#2984)

**Decided.** The group is a fourth public setup type, `agie_setup::PunchGroup20`, beside `PunchSetup20`, `CutoffSetup20` and the inspection writer. It takes the group stem, the stock, the 20 mm height and a list of `GroupPunch` entries (ISO stem and start hole). Each entry is one work `CUT1`, `CUT2` in list order, with the side `c_punch`. The piece writer `agie_setup/piece.rs` now takes one `Piece` (stem, stock, height) and a list of `Work` (ISO stem, side, start, exit), and loops over the works. The SBR has one resource line for each work. The post export `agie::export_group_with_review` sends one job to `export_with_review` unchanged, and two jobs to the group path. The app layer (`posting_agie_group.rs`) chooses the model of each operation and refuses a shared or unowned model.

**One quality for all works.** The SBL creates the quality `CCB030` once, before the group, and each work refers to it (`ID_QUALITY,je_q1`). The old PR created `QN2` and `QN3`, one name for each work. #2984 found that a control can bind different quality names to different resident data. One quality object removes that risk. On the emulated control, both works showed CCB030 with Ra 0.30, Tkm 1.0 and Te 1.5, and TECCUT gave the same technology (`estbca25nnn060b030020`) and the offsets 245, 155, 140 and 134 µm on each work.

**One ISO for each work.** The PR's control-checked job had `PAIR201.ISO` and `PAIR202.ISO`, and the SBR listed both. The port keeps that layout. One shared ISO with two `AGIE.USING_*` names was not checked on a control, so it was dropped.

**Two models, not two loops of one model.** The PR's control-checked job used two separate modeled solids, one for each operation. The port keeps that. Two loops of one model would need a loop selection on a punch source, which no control run covers.

**Setup fields.** All field values of each work are those of the checked zero-step punch (#2984, golden `CCSEP20.SBL`). Only the work name, the `ID_GEOMETRY` resource and the `stp1` position change for each work. The start holes (X-30 Y0 and X1 Y0), the stock (80 × 60 × 20 mm) and the geometry are those of the PR's control-checked pair. The CCSEP20, DIE20, LOWER20 and inspection goldens are unchanged. The new golden is `crates/post/tests/golden/agie-setup/PAIR20.SBL`.

**Checks.** `PunchGroup20::encode` refuses a group with other than two works, a repeated or invalid ISO stem, a height other than 20 mm, stock outside the 350 × 250 mm span, and a hole that is not completely inside the stock. Each work also passes the full single-punch checks of `contour::prepare`. The band between the works uses the longest rough miter of each work, which is at least the rough offset of the selected thickness (0.245 mm at 20 mm), plus the 3 mm lead and the 0.125 mm wire radius. A test pins a layout where the lead and wire alone leave a 0.3 mm band, and the rough corners close it.

**Dropped.** Groups of more than two works, a die or an open cutoff in a group, and other heights: no control run covers them. `PunchSetup20` with a list of holes: it would change the checked single-punch type and its documentation example.

**Cost to change.** A third work needs a control run and a change of the work count in `group.rs`. A different side in a group needs a side field on `GroupPunch` and a control run.
