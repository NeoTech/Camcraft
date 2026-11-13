## #1470 - Lathe chuck bores drawn inscribed, envelope pinned to the tessellation allowance

**Decided:** a chuck's bore (the jaw chuck's through bore, the collet
chuck's collet bore, through nose and body) is drawn as a 64-gon whose
vertices lie on the true bore circle, so its flats sit inside the bore by
the sag r(1 - cos(pi/64)), about 0.12 % of r. Outer faces stay as they
were: flats on the true circle, vertices outside it. The drawn solid then
holds every point of the true chuck - a tool clear of the drawing is clear
of the real steel.

**Why this direction is the safe one:** the collision solid is the steel;
the bore is the absence of steel. A bore polygon with its flats on the
circle (circumscribed, what #1442 shipped) has its vertices outside the
bore, so near every vertex the true steel between r and r/cos(pi/64) was
missing from the drawing - a tool could pass there unflagged. Inscribed,
the drawing's bore is smaller than the true one: it can only claim extra
steel, never drop real steel.

**Cost accepted:** the drawn steel now stands up to that sag into the
bore, which is the work's own room. A part or stock exactly at the collet
bore can read as overlapping the chuck by up to 0.12 % of its radius. The
part/stock-versus-fixture clash check (`fixture_clash`) is a warning, never
a refusal, and counts only past 0.05 mm, so it stays quiet up to a bore of
about 83 mm; above that a move of a part held exactly in the bore may warn.
Toolpath judgement is unaffected except near the bore mouth, where it is
now conservative by the same hair.

**Considered and dropped:** applying the allowance inward by keeping
flats-on and shrinking the radius by 1/cos (same geometry, more arithmetic);
raising `SIDES` to shrink the gap (smaller, still the wrong side).

**Test:** the sim envelope test now probes a micrometre (1e-3 mm) inside
every true outer face and outside every true bore wall at every polygon
vertex and flat middle, plus the jaw tip, gripping face and sides, and air
a micrometre past each face's allowance. The old probes stood 0.5-1 mm off
the rim and jaw tip and never probed the bore wall.

**Cost to change later:** one line in `camcraft_sim::chuck::ring`.
