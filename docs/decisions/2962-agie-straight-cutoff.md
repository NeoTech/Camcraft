## #2962 - Post a straight Agie cutoff with distinct air endpoints

Use one selected wall of the full-height retained prism. The wall must cross between opposite stock faces, and the whole workpiece must lie on the retained side. Use its nominal direction and the requested left or right native side. The open work receives two air start points. The control can then reverse the next trim on its intended offset. A common start point produced an extra cutting diagonal through the stock in the reference controller experiment.

Require two independent setup declarations on the operation. Both pieces must remain clamped through all passes. The stock must be centered on both installed travel midpoints. The first enables trims after full separation. The second locates the stock in the 350 × 250 mm stroke for the air-tail and wire-envelope checks. A slug catcher permits movement. Stroke length by itself cannot locate the work. Old operations load with both declarations clear.

Qualify a straight X- or Y-parallel cut with 4 mm air tails and 3 mm perpendicular approaches on the waste side. Use the measured maximum 20 mm QN2 offset plus wire radius in both axes. Refuse custom travel limits, drilled entry, larger clearance and open polygons until the full envelope has controller evidence.

### Port onto the zero-step setup writer (#2984)

**Decided.** The open cut is a third public setup type, `agie_setup::CutoffSetup20`, beside `PunchSetup20`. It has a `CutoffSide` (`Left` or `Right`), an entry point and an exit point in micrometres, and the 20 mm height. The SBL writer moved into `agie_setup/piece.rs`, and the closed and open works share it. `export_with_review` sends a `Left` or `Right` job to `cutoff::prepare`, which returns the line and the setup. The separation rule is now: 0.0 for an open cut, 1.0 for a closed cycle. A closed cycle with 0.0 is still refused.

**Why.** #2984 removed the old script writer (`jescript.rs`) and made the control import each work with zero erosion steps. The cutoff must use the same writer, so that the open cut also imports with zero steps (ASDELETE only). The old open-cut writer also wrote ASAGIE; the port drops it. The open-cut fields are the fields of this PR's control-checked job: `ID_PUNCH` `c_left`/`c_right`, `ID_SEPCUT` "0.0", `ID_EXITMODE` `c_entper` instead of `ID_ENTRY`, `ID_STPENTRY`/`ID_STPOPEN` instead of `ID_STP`/`ID_STPMODE` for `stp1`, and `stp2` with `ID_STPEXIT`/`ID_STPOPEX`. All other fields are those of the checked zero-step punch (#2984). The CCSEP20 and DIE20 goldens are unchanged.

**Added check.** `CutoffSetup20::encode` refuses endpoints that are not in air beyond opposite stock faces, or that put the stock, endpoints and work origin outside the 350 × 250 mm span. It refuses any thickness other than 20 mm. The geometry checks of `cutoff::prepare` stay as they were.

**Dropped.** A `PunchSetup20` with an optional exit point: it would make the start-hole check of a closed cycle optional. A separate open-cut writer: two writers can drift from the checked zero-step text.

**Cost to change.** A new open-cut field is one line in `piece.rs` and a new golden (`LOWER20.SBL`).
