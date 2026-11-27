## #4180 - The #4107 snap is decided per sample, not per edge

**Decided.** `marching` reads a sample within `NODE_SNAP` (1e-6) of a step of zero as +0.0 (`on_surface`): on the surface, on the outside of `d < 0`. `tet`'s crossing returns the sample itself when one end reads exactly zero, in place of the per-edge test `|da| < NODE_SNAP * |da - db|`. A value is one per sample, so every tetrahedron that shares the sample reads the same one.

**Why.** A cut floor on the lattice reads about +-1e-12 at its samples, with both signs. Decided per edge, two such samples of opposite sign each took the corners of their long edges, while the edge between them (both values tiny, so a moderate ratio) kept its crossing mid-edge. The facet through the three corners lay on one lattice edge, its normal was exactly zero, and `push` dropped it, so the mesh opened by three edges. On the pin's box: 15 facets dropped, 45 open edges. Our lane's cut stock showed 24, 39 and 48 open edges after #4117, all multiples of three. Per sample, no edge joins two snapped samples of opposite sign. The surface moves by at most `NODE_SNAP * step` (0.25 nm at a 0.25 mm step). The per-edge test is removed rather than kept as a second guard. A sample just above the per-sample threshold can still pass the per-edge ratio on a long edge (spans reach about 1.7 steps), which would leave the same collinear facet in a narrow band.

#4107's decision listed "a value within an epsilon of zero as zero" under Considered and dropped, because it changes which side a sample is on and so which tetrahedra are cut. That is the intent here: a floor of both signs at 1e-12 becomes a floor of one sign, the case that was already closed.

**Considered and dropped.**
- Keeping the per-edge snap and also snapping the edge between two snapped ends: more cases in `tet`, and it is still decided per tetrahedron.
- Keeping a dropped collinear facet (`push` keeping exact-zero normals when the three corners are distinct): the mesh stays closed, but with a zero-area facet that the closed checks and an STL reader reject. That is the state before #4117.

**Cost to change later.** One function and two lines in `cross`. The pins are `a_floor_on_the_lattice_reading_both_signs_stays_closed` in `iso.rs` and #4107's unit tests and `stock_dual` pins, which still hold.
