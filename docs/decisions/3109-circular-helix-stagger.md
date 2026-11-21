## #3109 - Circular: keep the helix entries of the rings apart

**Decided.** A flat Circular with a helix entry (or a helix level
transition) turns the start of each later ring to a vertex of that ring's
own polygon. At that vertex, the predicted helix orbit keeps 0.01 mm from
every helix orbit that the earlier rings flew. The prediction uses the
same first rule as the descent: the first of the eight compass offsets
whose disc the cutter-center disk holds. After each ring, the kernel reads
the descents (feeds that change Z) that the ring really flew. It reads
them with the crossing law itself (`verify::crossed_pairs`, at 0.01 mm and
0.001 mm chords) against the earlier rings' descents whose box and band
meet them. If a pair crosses, the kernel refuses the operation by name
("helix entry: the descent at ... crosses the descent of an earlier
ring ...").

**Why.**
- All rings of a flat Circular run the same level ladder. Thus the helix
  of each ring at a level shares its depth band with the helix of each
  other ring at that level, not only the ring before it.
- A helix makes a full turn at least, so a clear orbit circle is a clear
  helix. The circle test is a safe, cheap prediction.
- The reading uses the law that verification uses. The refusal therefore
  fires only where verification would block the post. A first version
  read orbit circles only. It refused a table row that verifies clean
  (`circular_arcs` repro): two half-turn arcs can share a band and still
  miss the points where their circles meet.
- The first ring, and any ring with no conflict, keeps vertex 0. Plunge
  entry, ramp entry and the vault mode do not change. No golden file
  moves.
- A helix that has no clear start is refused by name. The issue permits
  this outcome. Examples: a helix as wide as the cutter in a small circle,
  and three rows of the `circular_arcs` table (helix radius 0.5, radius 4
  and radius 6). Before this change, Crossing warnings held each of these
  rows: 30, 154 and 350 warnings.

**Dropped.**
- The pocket's z-major order (tried on #2898): it removed the crossings,
  but the inner rings then started with an axial plunge that no note
  named. Four ramp-entry tests went red.
- A refusal before generation for `stepover <= 2 * helix_radius`: it is
  simple, but it refuses the common job of the issue (stepover 2, helix
  radius 1). The stagger generates that job.
- A trial generation of each candidate start on a copy of the path
  builder: exact, but the copy grows with the path. The cost is quadratic
  in the ring count.
- A change to the crossing law: the law is a safety check under the
  safety floor, and the issue does not ask for it.
- The stagger in the vault mode: vault rings descend in separate bands,
  so their helices do not share a depth.

**Test change.** `crates/app/tests/circular_arcs.rs`: the three rows
above now expect the named refusal. The radius 4 and 6 rows were also the
table's gouge check. They are judged again with a plunge entry, and they
must still gouge.

**Cost to change later.** Low. The logic is in
`crates/motion/src/machining2d/circular_helix.rs`. `circular.rs` calls
it at two points: the start vertex before each ring, and the reading
after it.

**Also.** The test module of `circular.rs` moved to `circular_tests.rs`
(the `#[path]` pattern of `multiaxis/`). This keeps `circular.rs` under
the 400-line law.
