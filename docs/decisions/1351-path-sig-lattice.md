## #1351 - the path signature hashes the lattice form the file stores

**Decided:** `path_sig::of` hashes every float of a move (endpoints, feed,
arc center) snapped to the 1 um lattice (`cl::lattice`), the same snap
`MoveDto::from_move` applies when the project file writes a cached path.
A generated path and the same path saved and loaded now sign alike, so a
dependent op's rest digest (which reads its predecessor's signature)
survives a reopen and the chain stays Ok.

**Why:** the bug was a disagreement between two forms of one path - the
in-memory one (e.g. y = 6.050000000000001) and the stored one (6.05).
Making the signature read the stored form removes the disagreement at its
root without loosening any comparison: the rest digest is still an exact
equality of two words. Values already on the lattice (what clipper and
typed inputs give) hash bit-for-bit as before, so every stamp that matched
before still matches; only the stamps that were already broken change.

**Safety:** a change of half a micron or more still moves the signature
(pinned: a 1 um move of TH0's first move changes it). A change smaller
than that is one the project file itself cannot tell apart, and the posts
print no finer than the lattice either, so it cannot be a change of the
cut. No project-file version bump, no posted-output change, no golden
change: nothing that is written or posted reads the signature.

**Considered and dropped:**
- Snap the generated path itself when the cache is built
  (`Toolpath::new`): the in-memory moves would then equal the loaded ones,
  but every generator's cached output would move by up to half a micron
  and feed the sim, verify and post - a posted-output risk and golden
  churn for a staleness bug.
- Compare digests with a tolerance: loosens a safety comparison and needs
  the moves, not one word, per frame.
- Re-stamp dependents on load: hides a real predecessor change made
  between save and load by another build.

**Cost to change later:** one function; the hash prefix
(`camcraft-rest-path-v1`) is kept because on-lattice paths hash unchanged.
