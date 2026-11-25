## #3678 - A contour dive below the levels above pecks one stepdown at a time

**Decided.** A 2D Contour dive on a level below its ring's first level
reads the floor that the posted cuts left under the full tool disc. The
cuts are the feeds at one height and the straight plunge columns. If the
dive goes more than one stepdown below that floor, it pecks: a feed down
one stepdown, a rapid up to the plunge height over the material, a rapid
down to the plunge height over the last depth, the next feed. One note
for each path says it. The guard is on the wall-side graze dive (and the
tabbed bore dive, the same function) and on the dive past a tab.

The floor with no cut is the material top, or one stepdown over the
ring's first level where that is lower (a shelf band). A ring's first
level never pecks.

**Why.** The issue's proposed fix. It is local to the two dive sites. It
changes no XY path: the tab layout, the leads and the level order stay.
The posted path is the only input. The guard uses a conservative proof;
the independent test oracle checks the actual rim of the tool disc.

**Dropped.** Cut the window lead-ins on the levels above. That changes the
untabbed levels of every tabbed contour (more XY travel at each level) and
the stay-down descents. It also leaves the 13.5 mm dive of the stay-down
preset, whose untabbed levels do not leave the ring. The peck covers that
case too.

**Not changed.** The lead-in arc after a tab window still feeds at the
level through the 2.5 mm crescent of the disc that the levels above did
not cut. The issue's "Expected" is about the plunge only.

**Review decisions.** Replace the sampled floor with a conservative
coverage proof. An earlier cut must contain the entire target disc. An
exact matching endpoint or plunge column proves coverage. An axis-aligned
straight feed also proves coverage when the target centre lies on its
centreline. Treat all uncertain coverage as uncut material. This can add
pecks where several earlier sweeps jointly cleared the disc.

The shortened sample radius and additional rim samples were dropped.
Finite samples cannot prove coverage between the samples. Arc chords
were also dropped as proof: they approximate the actual sweep.

Preserve every stepdown past the old 4,096-peck cap. The existing contour
depth limit bounds the operation. Appending the final depth after that
cap was dropped because it could produce a cut deeper than one stepdown.

**Cost to change.** Small: the guard is `contour_peck::dive`, called at two
sites.
