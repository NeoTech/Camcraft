## #3211 - The wire gouge depth reads how far the wire stands inside the part

**Decided.** The kerf law reads a signed clearance (`dist::entry`). When a
move stands in the product, the reading is minus the distance from the
move's deepest point to the product's boundary. The gouge depth is then the
kerf reach plus that distance. Before, the clearance was 0 for such a move,
so each gouge read the kerf offset alone (0.162 mm on a 2.8 mm entry).

**Why.** The verify message names a depth and "the deepest reading at any
height". An operator uses that number to decide what to change (the verify
tolerance, the model, the path). A depth that is always the kerf offset
hides how far the cut enters the part. The verdict stays the same: each
move that gouged before still gouges, and the post stays blocked.

**How the deepest point is found.** Split the move at each product boundary
crossing, including the stock boundary for a die. Search every inside
interval with endpoint bounds on its 1-Lipschitz boundary distance. Refine
an interval while its bound can exceed the best reading by more than the
move length divided by 1024. The reported depth underestimates the deepest
entry by at most that amount, apart from floating-point roundoff. The
reported point is inside the product. An excluded stock boundary can hold
the supremum; the search approaches it from inside to the same tolerance.
The two move ends are read exactly when they belong to the product.

The former coarse walk and local refinement could miss a whole inside
interval. Its stated error bound was false. The review's 100 mm move
through the square at X=[50.5625, 52.5625] now reads a 1 mm entry. Global
interval bounds replace that local search; another deeper interval cannot
be discarded because its coarse samples are outside.

**Dropped.**
- An exact maximum through the medial axis of the section: much more code
  for a number that only sizes a refused gouge.
- A change to the threading and open-end laws: they need only "in or not
  clear", and they say "at the point itself" for 0. They stay on the
  unsigned `clearance`.
- A change to which move the finding names: it stays the first move that
  gouges at the first plane, with the deepest reading over all planes.

**Cost to change later.** One function in `crates/sim/src/verify/wire/dist.rs`
and its two callers.
