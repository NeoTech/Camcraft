## #1616 - Carvera states the clearance Z before the first X/Y traverse after M6

**Decided.** `carvera.cps` now writes Z first after `M6 T..` too, the same
as at a program start and as on grblHAL and TinyG. The first X/Y traverse
after a tool change comes after `G0 Z<ClearancePlane|RetractPlane>` (never
below the move's own Z). The X/Y-first branch for the Carvera (the
`changedAt` flag) is removed. This is option (b) of the #1632 decision
(`docs/decisions/1616-first-traverse-clearance.md`): a public source now
says where the firmware leaves the spindle after `M6`. That file's
"Carvera: M3's option (c)" and "The #1614 hazard stays" paragraphs are
superseded by this one. That file belongs to merged PR #1632 and is not
edited here.

**Evidence (public GPL source, behaviour only, read 2026-11-14).**
- MakeraInc/CarveraFirmware, commit
  `6e12557cfa5eff446bb5fe014354eaa03abf0caa`,
  `src/modules/tools/atc/ATCHandler.cpp`
  (https://github.com/MakeraInc/CarveraFirmware). When `M6` arrives,
  `on_gcode_received` waits for the planner to empty and reads
  `last_pos` with `THEROBOT->get_axis_position` (machine coordinates).
  When the change script is done, `on_main_loop` does a machine-coordinate
  rapid to `Z = coordinate.clearance_z` ("return to z clearance
  position"). Then it does a rapid to `last_pos` X/Y ("return to saved x
  and y position"). So the spindle is at the machine clearance Z, over
  the X/Y where it was before `M6`. It is not left over the tool rack.
- Carvera-Community/Carvera_Community_Firmware, commit
  `2c97111cb0bbd6d3edfac2c138963c0255248865`, the same file
  (https://github.com/Carvera-Community/Carvera_Community_Firmware). The
  same order (Z to `clearance_z`, then the saved X/Y).
- The defaults: `coordinate.clearance_z` is -3 (C1) or -5 (Air), in
  machine Z. Z homes to its top (`gamma_homing_direction home_to_max`,
  `gamma_max 0`), so machine Z0 is the top of travel.
- `src/modules/robot/Robot.cpp` in the same repository:
  `soft_endstop.enable` and `soft_endstop.halt` are on by default, with
  the Z maximum at -1. A move above it halts the machine with "Soft
  Endstop ... was exceeded". A work-offset `G0 Z` that cannot be reached
  stops the program. It does not rapid through the part.
- Community notes:
  https://github.com/infinitask/carvera-examples/blob/main/docs/carvera-firmware-upgrades.md
  (older stock firmware went to the work origin after a change; the
  community firmware 2.1.0c goes back to the X/Y before the change; both
  are at the clearance Z, not at the rack).
- Codes: https://wiki.makera.com/en/supported-codes and
  https://carvera-community.gitbook.io/docs/firmware/supported-commands/mcodes/tool-change.

**Why Z first is safe after M6.** The `G0 Z` after `M6` is a vertical
move at the X/Y where the program already was. That X/Y is the end of the
previous section (it retracted there) or the operator's position at the
program start (the same case #1632 accepted with no tool change). It is
not over the rack. The `G0 Z` goes to the section's clearance, which is
above the stock. Then the X/Y traverse is made at a stated height. The
old order made the X/Y traverse at the changer's exit height, which a
long tool on a high work offset can put below the part top.

**Considered and dropped.**
- Keep X/Y first after `M6` (the #1632 option c). Dropped: the rack
  concern is answered by the source, and the bare traverse can cross the
  part with a long tool.
- Write `G53 G0 Z..` or `G28` before the traverse. Dropped: the firmware
  already lifts to `clearance_z` at the end of `M6`; `G28` also moves
  X/Y; and the post does not know the machine's `clearance_z`.
- Refuse Carvera programs with a tool change. Dropped: not needed now
  that the order is proven.

**Taken on trust.** The firmware on a given machine is one of the two
read here, with the default soft limits. Not run on a real Carvera.

**Goldens.** 19 Carvera NC goldens change: each `M6` block swaps
`G0 X.. Y..` / `G0 Z..` to `G0 Z..` / `G0 X.. Y..`. They are not
regenerated in this PR; the maintainer decides (the PR body lists them).

**Cost to change.** One function in `carvera.cps` (`xyFirst`), the pins
in `crates/post/tests/first_traverse_clearance.rs`, and the goldens.
