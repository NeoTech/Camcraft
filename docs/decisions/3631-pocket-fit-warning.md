## #3631 - 2D Pocket: warn at edit time when the tool cannot fit

**What was decided.**

- A new advice, `operation::pocket_fit_advice`, reads the 2D Pocket's
  region the way Generate reads it: `collect_job`, then `region_for_job`,
  then the area controls (`apply_area_full`). It then applies the
  generator's own predicate, `machining2d::pocket_tool_fits` (the
  `non_sliver_center(region, r + stock_to_leave)` test of
  `pocket_started_on`).
- The MCP replies of `add_operation`, `edit_operation`, `set_tool` and
  `set_operation_area` carry the sentence in `warning`. This is the
  `clearance_warning` pattern. When both warnings apply, they are joined
  with "; ".
- `EditOperationParams` refuses a plain Slot edit that changes `width` or
  `stock_to_leave` and leaves the mounted tool too wide. It uses the #3475
  door (`slot_gate`) and its sentence.

**Why.** The issue requires the same region as Generate, or no check.
The advice does not run when it cannot build that region exactly:

- a refused collect;
- a tilted frame or a rotated WCS (generate moves the region first);
- a sketch contour source;
- rest machining (the rest generator has its own area);
- the retired Adaptive strategy (Generate refuses it by name).

So a legal pocket never gets the warning from an estimate.

The Slot refusal runs only when `width` or `stock_to_leave` changes. An
older document whose slot was already too narrow stays editable for other
fields (feed, heights).

**Dropped.**

- A bounding-box or width estimate. It can refuse legal work (an open
  side, an Outside containment).
- A refusal for the pocket. The issue asks for an accepted edit with a
  warning, because the region can change after the edit (stock, model).
- The same text in the operation page of the UI. The UI's display advice
  is memoized by the frame-cost rule (`height_memo::clearance_advice`).
  A new cached advice needs its own cache key (tool, area, stock) and
  counter tests. This is filed as #3859. Running
  `collect_job` on each frame is not permitted.

**Cost to change later.** Low. The advice is one function. A UI cache can
call it as it is.
