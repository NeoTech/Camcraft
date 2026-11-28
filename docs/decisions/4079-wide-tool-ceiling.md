## #4079 - The field ceiling bounds a chord's travel, not the tool's own footprint

**Decided.** `SdfField::subtract_batch` refuses a chord when the cells of its clipped window, minus the cells of the same solid at rest at the chord's start (`Sweep::rest_window`, the same reach, padding and clipping), pass `MAX_CELLS_PER_MOVE`. The value stays 1 << 22. The lathe sweep has no rest window and keeps the old bound.

**Why.** The refusal asks the caller to segment the move. Segmenting shrinks the travel, never the tool. A wide tool (a 50.8 mm flat tool over a 30 mm block at 0.254 mm cells) passed the old bound on a 1 mm move, and no segmenting could bring it under. The column engine cuts that move, and `backend.rs` says refusals are backend-independent. `cut_batch` keeps only a brick-to-sweep list and dismisses an unreachable brick by one test at its centre, so memory does not grow with the window.

**Considered and dropped.**
- A higher value: a wide enough tool passes any value, and a long chord of a small tool would pass it unrefused.
- A cap scaled by the tool's diameter: it reads one dimension of a window that has three, and a spanning flank's height would still be counted.
- Tiling the window: the walk is already brick by brick; there is nothing to tile.
- Changing the f32 cut twin (`TWIN_MAX_CELLS_PER_CHORD`) in the same PR: it bounds the twin's own work list and has its own tests. It is a follow-up if the two laws are to stay the same.

**Cost to change later.** One function (`Sweep::rest_window`) and one subtraction in `subtract_batch`. Going back to the old bound is deleting the subtraction; the pins in `crates/sim/tests/field_wide_tool.rs` name the behaviour.
