## #4122 - The closed check counts every edge not on two facets

**Decided.** `iso::unpaired_edges` counts the edges, after the same 1e-9 weld as `open_edges`, that are not on exactly two facets; an edge from a corner to itself (a facet collapsed at the weld) always counts. `iso::extract_with` serves the marching mesh when the dual mesh has any, and `app::stock_export::machined_mesh` refuses to write a mesh that has any. `open_edges` (odd edges) stays for its other callers.

**Why.** `open_edges` passes an edge on four or six facets. The dual mesh of a cube smaller than a cell was three flat fins on one edge, volume 0, with no odd edge, so `extract_with` returned it and the export would have written it as a solid. The same count hid the marching slivers of #4107. Measured on main a1c57b475 across all `camcraft-sim` tests and the `camcraft-app` suites `stock_export`, `contour_stock_run` and `stepped_stock_rapid` (937 passed): of 20 meshes at the two sites, that one dual mesh was the only one the new count refuses. Its marching mesh is closed, every edge on two facets.

**Considered and dropped.**
- Making `open_edges` itself count every edge: its name says open, and its callers in tests read it as "no hole"; a second name for the stronger law keeps both readable.
- Also requiring each edge to be used once in each direction: fins of back-to-back triangles use their edge in both directions, so the count is the law that catches them; orientation is a separate check if wanted.
- Repairing the dual mesh (dropping fins): the marching mesh of the same samples is already the stand-in for a dual mesh that fails the check.

**Cost to change later.** One function and two call sites. Going back is calling `open_edges` at those sites; the pins in `crates/sim/tests/stock_dual.rs` and the unit tests in `crates/sim/src/stock_sim/iso_edges.rs` (split from `iso.rs` at the 400-line rule; `iso` re-exports both counts) name the behaviour.
