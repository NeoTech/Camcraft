## #1620 - The chord law judges every piece of a chord

**Decided:** `chord_clears` (now in `crates/motion/src/machining2d/chord_clear.rs`)
splits the chord at every loop vertex it touches without a crossing (a
vertex where both edges stay on one side of the line, or one edge lies
on it). The midpoint of each piece
must be inside the region or on its boundary. Before, one midpoint decided
the whole chord.

**Why:** A chord can run along an edge of one piece of the area, leave the
area at the end of that edge, and come back on the edge of a second piece
on the same line. No edge crosses the chord, so the old test found no
crossing and read one midpoint. On `Milling-test.step` that midpoint was in
a piece, and the 2D Adaptive path fed along Y -32 from (-8.05, -32) to
(32, -32) at every level, 0.94 mm into the part. The law is the cause; each
generator that links with it (2D pockets, Adaptive, Offset, the 3D ladders)
gets the fix at once.

The first draft also split the chord at the ends of each edge that lies
on the chord line. A mutation test showed that this adds nothing: those
ends are vertices, and the edges next to them already split the chord.
It was removed.

The issue's "10 mm at (-1.44, -29.94)" came from a stock with its top at
Z 10, under the part's top at Z 20. The sim reads every part column above
the stock as a 10 mm gouge, and that spot is the first part column. The
pin uses a stock that covers the part (70 x 70 x 27 on Z -7) and an end
mill with 30 mm flutes, because the catalog's 15 mm flutes cannot reach
the 25 mm deep part without a shank hit (a separate, correct refusal).

**Considered and dropped:** A check in the Offset ring linker only. It
fixes this job but leaves the same hole for every other caller of the law.
A full segment-region boolean (clip the chord against the region). It is
correct but much slower on a law that runs for every link.

**Cost to change later:** Low. The law has one function and unit tests in
the same file. A tighter law only refuses more links; a refused link
becomes a retract and a plunge, which is safe.
