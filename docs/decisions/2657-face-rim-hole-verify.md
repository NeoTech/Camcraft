## #2657 - Filter open region rims by wall classification in Boundary tangency with a model

**Decided.** In `sim::verify::section::tangency`, when `cx.mode == Mode::Boundary` and `cx.model.is_some()`, each edge of `cx.region` is checked against the move's `chords` only when `edge_is_wall(p, q, section)` holds (part material in `section` stands `WALL_PROBE` off the edge's midpoint on either side). All edges of `section` itself remain unconditionally checked, and `Mode::Area` as well as no-model `Mode::Boundary` keep checking every edge of `cx.region`. The midpoint probe previously inline in `wall_gouge` was extracted to `edge_is_wall` so both laws share one predicate.

**Why.** A `Face` operation with `holes: lift` clips row pieces at a radius off each hole. When a hole lies within two tool radii of the open outer rim, a plunge or feed endpoint lands less than one tool radius inside that rim. With no material standing above `z + lift` at the rim, that rim is open and not a wall; checking all `cx.region` loops unconditionally raised false `Warning::Gouge` findings that blocked posting. Walls standing above the cut plane — including walls whose top sits below the highest point of the model — remain present in `section` and satisfy `edge_is_wall`, so moves within `r - tol` of those walls still report `Warning::Gouge`.

**Dropped.**
- Dropping `cx.region` altogether whenever `Mode::Boundary` has `cx.model.is_some()`: dropped because keeping `edge_is_wall`-filtered `cx.region` edges preserves per-loop `floor_of` tolerances and aligns `tangency`'s edge law with `wall_gouge`.
- Adding a Face-specific exemption flag on `VerifyContext`: dropped to keep the verifier independent of operation names and consistent with the existing `wall_gouge` geometry rule.

**Cost to change later.** One helper (`edge_is_wall`) and one guard (`boundary_with_model`) in `crates/sim/src/verify/section.rs`.
