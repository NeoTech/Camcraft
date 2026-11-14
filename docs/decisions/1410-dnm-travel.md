## #1410 - Doosan DNM 5700 travel stated per the travel rule

**Decided:** `assets/machines/doosan-dnm.mch` states the catalog strokes (1020/570/510 mm) full length either way: X `-1020..1020`, Y `-570..570`, Z `-510..510`. These replace the machine-frame Z `-510..0` and the half-stroke X/Y.

**Why:** the limit check reads bands in program coordinates through the work offset (`assets/machines/AGENTS.md`, travel bullet; #457). `-510..0` refused every move above G54's Z0, and `±510`/`±285` refused a corner-origin program past half the table. This is the same fix `vmc.mch` and `tilthead.mch` took.

**Considered and dropped:**
- Keep X/Y as stated and fix only Z. Dropped: half strokes still refuse legal corner-origin programs, and the rule names the full stroke.
- State an invented `nose_to_axis` for the optional A so the 3-axis face job posts end to end. Dropped: never invent a safety figure. The remaining refusal is filed as #1433.

**Cost to change later:** a data edit plus `vmc_stroke.rs`'s DNM pins.
