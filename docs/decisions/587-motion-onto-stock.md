## #587 (M7.3 4/6) - motion reads and cuts stock through `Stock` / `Columns`

- **Chosen:** camcraft-motion names no `Dexel` outside tests.
  - The masks motion takes (`pocket3d_prev`, `pocket3d_with_rest_prev`, the `offset` twins, `multiaxis_rough_prev`) are `Option<&Columns<'_>>`. The app wraps its program-ahead grid with `Columns::of_grid`, which borrows, as #585 did for the prior readers.
  - Motion's own sims (`sim_of`, `run_rest`, `optimize_feed`, `quality::stock_for` / `measure` / `rest_area`) hold a `Stock` built on `Backend::Column`. They read through `Columns` (`above`, `top_over`, `top_at`) and score with `Stock::measure`.
  - The seam gains the edits motion needs, in a new `stock_sim::backend_edit`: `Stock::subtract_flat` (`Dexel::subtract`), `Stock::cut_chord` (`Dexel::cut_chord`) and `Stock::set_tops` (`Dexel::set` per column, writing only the columns whose top changes). `Columns::to_stock` gives a rest pass its owned copy of the mask (the clone `run_rest` already made).
  - `quality::measure` and `quality::rest_area` now return `Result`, because a seam cut can refuse. The column arm never does.
- **Why:** this is behaviour-neutral. On the column arm each new method is the old `Dexel` call unchanged, so the finish3d, rough and quality goldens and `stock_engine_agreement` stay byte-identical. A unit test pins `engagement_on` bit-for-bit to the old direct `Dexel` read. The 6/6 retirement then changes only what is behind the seam.
- **Field arm:**
  - `subtract_flat` and `cut_chord` cut the flat tool with the spanning flank `Stock::simulate` uses.
  - `set_tops` refuses by name (`SET_TOPS_FIELD_REFUSAL`). Rewriting column tops has no field meaning short of a solid union, which belongs to 6/6.
  - A posed cutting move is refused on both arms (`POSED_REFUSAL`), as the refusal law requires.
- **Dropped:**
  - Taking `&Stock` for the masks: the app would have to clone its grid into a `Stock::Column` for every call.
  - Routing the subtracts through `Stock::simulate`: its chord budget is a quarter cell, not motion's `CHORD_TOL`, so the goldens would move.
- **Pinned by:** `stock_seam::motion_names_no_dexel_outside_tests`, a source scan of `crates/motion/src` that skips `tests.rs` files and `tests/` dirs.
