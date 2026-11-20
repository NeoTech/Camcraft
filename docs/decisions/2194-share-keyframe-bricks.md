## #2194 - Playback keyframes share the cut bricks that no move changed

**What was decided.** The three cut layers of `SdfField` (`cuts`, `cut_ups`,
`cut_downs`) hold each brick in an `Arc` (`CutLayer` in
`crates/sim/src/stock_sim/field.rs`). A clone of the field copies the keys
and the pointers only. A write goes through `layer_brick_mut`, which uses
`Arc::make_mut`: it copies a brick first when a clone still holds it. The
keyframe store counts a keyframe by `KeyCost::key_bytes_after`: the bricks
that the keyframe before it does not share, and the map entries of the three
layers.

**A visit is not a write.** `claim_record` and `up_record` read the cell
first and take the writable brick only when the value changes. Before, they
took the writable slot first. With shared bricks that copied each brick that
a move reached, also when no cell changed. On the staged 24 mm face the
read-first order holds 9.9 MB for 28 keyframes; without it 16.4 MB; whole
copies cost 45.3 MB. The values that the cut writes are the same, bit for
bit: an absent brick reads NaN, and NaN always took the write.

**Why.** A keyframe was a whole copy of the cut layers (#2181). Two keyframes
one stride apart differ only in the bricks that the moves between them
touched. Sharing the other bricks lets the same 512 MiB budget hold more
keyframes, so the stride stays short on a large program. Copy-on-write keeps
each keyframe an exact prefix of the program: no write can reach a brick that
a keyframe holds. The map entries are counted because a store of fully shared
keyframes would otherwise cost "0 bytes" and escape the budget.

**The count can read high, never low.** The test for "shared" is pointer
identity with the keyframe before. Each brick allocation is counted at the
first keyframe that holds it. A live seek that restores an old keyframe and
cuts again makes equal bricks in new allocations; those count again, which is
what the memory holds. After a thinning the store sums again from the keys
that stay.

**Considered and dropped.**
- A delta chain (each keyframe stores only its changed bricks, a restore
  walks back to key 0). Dropped: a restore would cost a walk over all
  keyframes, and a thinning would need a merge of deltas. Shared pointers give
  the same memory with a restore that is one map clone.
- An exact count with a set of all brick pointers in the store. Dropped: it
  needs the store to know the brick type, and the pairwise count is already
  never low.
- `Arc` on the whole layer map (as the mesh layer has). Dropped: the first
  write after a keyframe would copy every brick again.
- Share the mesh-layer bricks one by one too. Dropped: that map is already
  shared whole and is written only while the field is built.

**Cost.** Each cell write does the `Arc::make_mut` uniqueness check (two
atomic operations, no lock). The swept-distance evaluation of the cell
dominates it. `CostLedger` (the cut region, the full-brick ledger, the dirty
set) is still copied whole by each keyframe and is not counted, as before.

**Cost to change later.** `CutLayer` and `layer_brick_mut` are the only
places that know the brick is shared. A different store (a pool, an arena)
replaces those two and `cut_bytes_after`.
