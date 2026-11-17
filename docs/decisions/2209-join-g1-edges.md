## #2209 - A smooth crease between two curved surfaces is one feature edge

**Decided.** `geometry::feature::chains` gets a G1 seam rule
(`feature/g1_seam.rs`). Two feature edges join at a vertex when all hold:

- both separate the same two welded patches (`Features::facet_patch`), or
  both are the rim of one open sheet patch;
- the vertex has exactly these two real feature edges (a tessellation seam
  inside one patch does not count; any other feature edge does);
- the turn between them is at most `CHAIN_TANGENT_DEG` (30 degrees), the
  same limit the face rule already uses.

The trail walk (`feature/trail.rs`) keeps a joined pair consecutive, so a
component that also holds tessellation seams does not split the crease.
The rule only adds joins. It moves no point and drops no chord.

**Why.** fig8.step has 70 `EDGE_CURVE` entities; the file does not store
short edges. The split was ours: the face rule joins two edges only when
they bound one planar face, and the faces are exact-plane clusters. On a
crease between a conical chamfer and a cylindrical wall, each chord has a
different planar strip on both sides, so no two chords shared a face. On
fig8 the edge below the chamfer was 719 runs (longest 17.22 mm); it is now
10 runs (longest 188.49 mm), the same count as the edge above it.

**Layer.** `chains()` is the one source of chains, runs, pick, highlight,
chain rows and operation chains. A fix there reaches all of them.

**Dropped.**

- A join at STEP import (keep the B-rep edge ids through solidcore into the
  mesh). It is the exact answer, but it needs edge provenance through the
  import and the tessellator, and an STL has no B-rep. It is larger than
  this fix.
- A join in the pick graph only. Operation chains and highlight would still
  see the pieces.
- A separate angle limit. One limit for the face rule and the seam rule
  keeps one meaning for "tangent" in the chain code.

**Cost to change.** A different limit is one constant. Edge provenance from
import can replace the patch-pair test later without a change to callers.
