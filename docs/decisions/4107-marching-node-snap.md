## #4107 - A marching crossing within 1e-6 of a sample sits on it

**Decided.** In `iso.rs` `tet`, a crossing is put on the sample when the sample's field value is under `NODE_SNAP` (1e-6) of the value span along the edge (`|da| < NODE_SNAP * |da - db|`, and the same for `db`). The facets that then share that sample with two corners are exactly zero-area, and `push` already drops them. The test reads `|da|`, `|db|` and `|da - db|`, which are the same numbers when the edge is crossed from its other end, so the two tetrahedra that share an edge snap together or not at all. A snapped corner is the sample itself, not `pa + (pb - pa)`, which beside a coordinate's rounding residue is not always `pb`.

**Why.** A cut floor on the extraction lattice (the field's cell centres) leaves field values of about 1e-12 at the samples. Marching tetrahedra put the crossing about 1e-12 mm from the sample, under the 1e-9 weld that `open_edges` uses, so the facets that shared it were kept by `push` and collapsed after a weld or an f32 export: 11,394 collapsed facets and 5,799 edges on three or more facets in the pin's block. At a 0.25 mm step the snap moves a corner at most 0.43 nm (a body diagonal of the cell). Dual contouring is not affected. A crossing between 1e-6 of an edge and the f32 rounding step from a sample can still collapse after an f32 export; that is general marching behaviour, not this floor case.

A first form tested the fraction `t` against `NODE_SNAP`. `t` and `1 - t` from the two ends of an edge differ by rounding, so near the threshold one tetrahedron snapped and its neighbour did not, and their corners stood 1e-6 of an edge apart (the unit test `an_edge_gets_the_same_corner_from_either_end_at_the_snap_threshold` found one such value).

**Considered and dropped.**
- Treating a value within an epsilon of zero as zero before the crossing: the same effect, but it changes which side of the surface a sample is on, so it also changes which tetrahedra are cut.
- Dropping facets whose corners meet after a weld, after extraction: it repairs the output and leaves the cause; the weld tolerance would become a second law.
- Making `open_edges` count every edge in the same PR: it is used as a closed-mesh check by several tests, and a stricter count may be wanted separately. The pins count every edge themselves.

**Cost to change later.** One constant and the snap in `tet`'s crossing. Going back is deleting it; the pins in `crates/sim/tests/stock_dual.rs` and the unit test in `iso.rs` name the behaviour.
