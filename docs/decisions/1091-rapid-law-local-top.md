## #1091 - The rapid law reads the stock's top under the dive, not its highest point

**What was decided**

1. **Reproduced with stock only.** A casting set as "from model" stock,
   solid to Z 20 over a 16 mm hole's mouth with a boss to Z 30 18 mm
   away, then removed from the tree (the stock keeps its copy) so the
   program verdict does not judge it as a part. A part-sourced Bore
   (part imported) and a stock-sourced Bore (no part at all) both
   rapid to Z 22 and were convicted `RapidInStock { at: 0 }`.
2. **Re-read the convicted dive, do not change the column law.** A new
   sim reading, `verify::acquit_by_stock_top`, works the way
   `acquit_by_prior` does: it only removes a conviction, and only from
   an upright vertical rapid down in a column mode. The column law
   (`rapids.rs`, `Rest::surface_before`) and its flat cap are
   unchanged, so every other conviction - lateral rapids, the posed and
   radial readings, the solid law - keeps its verdict.
3. **The bound is exact and conservative.** The re-reading takes the
   highest vertex of every stock or part facet whose plan comes within
   the tool's disc. On a closed mesh that is never below the material
   standing anywhere in the disc. It reads each band of the stack
   separately: the flute at its full radius (not `r - gouge_tol`, and
   with no ball lift) from the tip, the shank from the flute's top, and
   the holder from the shank's top. The dive is acquitted only when
   every band ends above its disc's top, less `Z_TOL`. A boss reaching
   under the rim, or under a wide holder over a stubby tool, still
   convicts. It fails closed: a non-finite vertex, a non-finite stack
   number, no judged stock (the op's region stood in for it), or a
   flute disc that meets no facet all leave the conviction standing.
4. **Only a proven closed shell is read (PR #1098 review).** The bound
   holds only for a closed surface. In an open mesh (a boss without its
   top facets) the facets over the dive are missing, and the dive was
   acquitted into stock that is there. So every solid must pass
   `stock_top_shell::closed_shell`, or the conviction stands. The gate
   checks:
   - the solid has facets, and every vertex is finite;
   - no facet is degenerate in 3-D (two equal vertices, or zero area);
   - every undirected edge (endpoints matched exactly, bit for bit) is
     used by **exactly two facets, once in each direction**. That
     refuses a hole, a non-manifold edge, a duplicated facet and a
     flipped neighbour;
   - the facets form one edge-connected shell;
   - the signed volume is positive (outward-wound).

   It is computed once per solid per operation, in one O(F) pass with a
   hash map of edges, never per rapid. It is stricter than the proof
   needs on purpose: an inside-out shell, or several shells handed in
   as one solid, are refused even though the bound would hold for them,
   because a refusal only keeps a conviction.

   **What the acquittal now relies on:**
   - The solids are closed surfaces. The gate checks this exactly.
   - A point enclosed by a closed surface sends its upward ray across
     it, so a facet covering its plan stands above it. This ray argument
     holds even for a self-intersecting surface, and the gate does not
     test self-intersection.
   - The meshes are the stock and part the program really meets. The
     whole verifier rests on this too.
   - Like the column law, the re-reading does not read `model_excess`
     (#864). A coarse dome whose true apex bulges above its highest
     vertex reads the same as before.

   No cheaper independent cross-check proves an empty column.
5. **The solids are `judged_stock` (the stock the column law judged,
   #1066: the setup's or carried stock, or the stock itself for a
   stock-sourced Bore/Thread) and the op's own material.** Both are
   read in the path's frame: world at the upright site, and the
   plane's local facets at the 3+2 site. Thread's descent is a
   vertical rapid, so it is re-read the same way.
   `acquit_by_prior` still runs afterwards, for stock the program ahead
   removed.
6. **Pinned by** `crates/app/tests/stepped_stock_rapid.rs` (the dive
   over the low part passes and posts. With the boss over the hole, or
   with its edge 1.5 mm beside the dive's centre under the 4 mm rim, the
   dive is convicted and the post is blocked by the rapid law. Each
   case runs for both sources). The review's open boss (its top facets
   missing, over the hole) stays convicted and blocks the post, for both
   sources; this test fails with the gate disabled. Also pinned by
   `sim::verify::stock_top`'s unit tests: the holder band; the part
   counting too; an open shell, a non-manifold or duplicated edge, a
   degenerate facet, an inside-out or split solid, an empty solid and a
   non-finite vertex all keep the conviction; and the posed, lateral and
   radial/revolved/rotary exclusions. The `ladder_entry_height` pins and the #96 pins are
   unchanged.

**Why**

The Prime Directive asks for correct output or a visible refusal. A
refusal of a legal program is not a gouge, but it leaves a user with a
casting no way to post. Loosening the law itself (a per-sample local
cap inside `surface_before`) would move every column verdict and lean
on the footprint's sampling bound. A re-reading that can only acquit,
and only on an exact upper bound of the material, cannot let a dive
into material pass.

**Considered and dropped**

- *A local cap per footprint sample inside `Rest::surface_before`.*
  This would also cover lateral rapids, but it inherits the footprint's
  sampling bound (a boss sliver between samples could pass) and it
  changes every column law that reads the cap.
- *The stock dexel (`stock_grid`) as the prior even when nothing is
  ahead.* The grid samples cell centres, so a boss edge between centres
  reads low. That is not an upper bound.
- *A new `VerifyContext` field.* It would touch about 50 construction
  sites for a reading that only needs the dive's convictions.

**Left as it is (follow-ups filed)**

- Lateral rapids under the clearance plane over a stepped stock still
  read the flat cap. They over-report and do not hide anything (#1095).
- A stock-from-model's source model left in the tree is judged as a
  part by the program verdict, so the post reports a gouge. The
  reproduction removes the casting to keep that out of the way (#1094).
- A model that is neither the op's source nor inside the stock (a
  second part poking up through a low step) is not in the solids. The
  flat cap did not read it there either, since `stock_top_z` is the
  stock's and the source's top.

**What it would cost to change later**

- Lateral rapids could take the same re-reading over the swept capsule.
  That is a second reading in `stock_top.rs`, and the rest of the law
  stays unchanged.
- If verify ever gets a true stock model (#928), this re-reading is
  replaced by the column law reading that model directly.
