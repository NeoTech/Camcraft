# Decision record - issue #2829

## #2829 - Rotary Flow: the multi-band, single-helix and chain-set laws

**Decided:** the picked chains read as ONE ordered set, and the law
moved into a new `rotary_flow_set` module of the motion crate.
Three readings were taken unattended:

1. **Multi-band.** Three or more ordered chains machine one band
   per adjacent pair, and the SHARED boundary chain is cut once:
   band k > 0 starts at its first interior pass (t = 1 / (count -
   1)), because band k - 1's last pass already walked the shared
   chain at the same stations, the same push-out and the same climb
   setting. This supersedes the #2824 decision that dropped the
   shared-edge skip ("safe and simple" won then; the issue now asks
   for it, and the alignment law turned out to need no second form:
   the skip is one index, not a second alignment).
2. **Single helical chain.** A lone chain machines the full
   revolution: the kernel synthesizes the second boundary as the
   chain's own next lap - `th + TAU` in the unwrap's own reading
   (verified in `rotary_flow_unwrap::unwrap`: positive theta runs
   e1 toward e2 about the stated axis), every other coordinate
   held. The lap is exact by construction, so the pair takes no
   whole-turn alignment (the alignment would otherwise walk it back
   onto the chain). The developed band is exactly one revolution
   wide at the chain's own outermost radius. The lap's two edges
   are the SAME world curve - the band closes on itself - so the
   closing pass is dropped and the picked chain emits once: the
   same dedup law the issue states for shared boundaries, read on
   the closed band's own seam.
3. **Winding sense.** The #2824 refusal of chains wound in opposite
   senses is SUPERSEDED: a chain wound against its pair is walked
   backwards (`Unwrapped::reverse`, the matched-place knots rebuilt
   ascending), and the notes say so. The app wiring (#2826) made
   the reason plain: the feature layer reads a chain's direction
   from the mesh, never from the pick - the probe on the bar
   fixture showed the end-face rings extracting wound -TAU while
   the profile rings extract +TAU, so the old refusal fired on
   legal picks the user had no way to reverse. The refusal was also
   load-bearing geometrically: the raw matched-place pairing of
   two oppositely wound chains twists the band a whole turn, and
   the reversal untwists it - the normalized pair is exactly the
   band the old law demanded the user hand-build.
4. **Set refusals, all by name.** Chains that MEET OR CROSS (the
   exact 3-D segment-pair law, Ericson 5.1 as the guard cites it)
   are refused at the meet's own spot, at the corner-lattice dust
   tolerance. And consecutive bands whose mean developed offsets
   oppose (dot < 0) are refused as a set out of band order - the
   lower band would re-machine ground the upper band already cut. A perpendicular turn is NOT a doubling back:
   only a strictly negative dot refuses, so bands may tile around a
   corner. Zero-length, non-finite and duplicate chains keep the
   #2824 refusals.

**Why:** the issue names all three behaviours and leaves the
witnesses open. The 3-D meet test is exact and cheap, and a meet
within dust is refused, not only a transversal crossing - a band
pinched to a point machines nothing honest there, and the two
witnesses (touch, cross) cannot be separated by distance alone.
The backtrack law is the overlap condition itself: bands tile
unless one mean offset has a negative component along its
neighbour.

**Considered and dropped:** a developed-offset REVERSAL law (any
pairwise opposition of the matched-place offsets refuses the pair
as a band that folds) - the #2824 box band refuted it on the
first run: two straight box edges that pass either side of the
axis-nearest point read a lens-shaped band whose offset pinches
through zero, and that band is established, pinned, legal
behaviour, so the lens stays machinable and the direction is read
as a mean only; refusing non-adjacent chain pairs that cross (the
kernel only ever blends adjacent pairs; the swept-ball guard
still proves every emitted move); a cheaper between-bands hop
at the retract cylinder for bands that share a boundary (the
clearance hop is the existing between-band law and is proved; the
saving is one rapid per band); an app-side door validation of the
chain count (the create door cannot read mesh geometry without
duplicating the kernel's walk - the generate door's named refusals
are the law); a left-handed lap morph (`-TAU`) chosen from the
chain's winding sense (the +TAU lap machines the same full
revolution and keeps one law; the winding sense only sets the pass
travel, which `climb` already answers).

**Cost to change:** the set law lives in
`crates/motion/src/multiaxis/rotary_flow_set.rs`; the band loop's
dedup is the `start` index in `rotary_flow.rs`; the lap morph is
`lap_of` plus the `lap` flag that skips the pair alignment. The app
wiring needed no change: `selection::drive_chains` already passes
every picked chain, so the app arm is untouched and the new
behaviour rides the same `RotaryFlow` params.

## #2829 rework (PR #4018 review, 2026-11-25) - set normalization and the ring lap guard

**Decided:** two laws from mkinney's safety-tier review, both taken
unattended on the same branch.

1. **Normalize the COMPLETE set once.** The first head resolved a
   guide's winding band by band, on a CLONE of the next track, and
   the stored track kept its raw pick winding - so a three-guide
   set with only the middle pick reversed cut band 1 from the
   normalized pair but band 2 from the raw middle chain, and the
   last three passes wound `-TAU` while the first four wound
   `+TAU` at `climb = true`: the second band changed cutting
   direction without a change to climb. The law now runs ONE pass
   over the complete ordered set before any band is cut
   (`normalize_set`): each guide after the first is resolved
   against its (already normalized) pair-neighbour - wound against
   it, walked backwards - and the SAME normalized track bounds both
   adjacent bands. The first guide's pick keeps its winding (the
   #2824 behaviour of a two-guide pair is byte-identical), and a
   reversal cascades: guide k is judged against the normalized
   guide k - 1, never against the raw pick. `climb` is NOT part of
   the normalization: it reverses every pass uniformly at emit, so
   the set's direction consistency is independent of it - the pins
   run both settings.
2. **Refuse the azimuth-only single guide.** The lap morph gave
   every lone chain its own next lap, and a chain at ONE axial
   station and ONE radius - a closed ring or an open arc of one -
   reads a positive developed width (a whole revolution of arc)
   while it sweeps NO band area: every pass of the family is the
   same circle again, and the review's probe posted 56 passes of
   one closed cut. The law now checks the COMPLETE track before a
   pass is generated (`refuse_ring_lap`): both the axial-station
   span and the radius span under the corner-lattice dust `cell`
   refuse by name, closed rings and open ring arcs alike. A guide
   that advances in station OR in radius - a helix, an axial line,
   a radial arc - sweeps a real rotational band and passes; the
   tolerance is the house DUST scale, so a genuine but
   dust-shallow helix is also refused as a band that machines
   nothing honest.

**Why:** both defects are silent wrong output - the worst failure
class this codebase knows. The direction flip needs no user error
at all (the extraction of the app layer winds rings either way),
and the ring lap reads every existing guard green: the width is
positive, the meet law never runs on a single chain, the passes
clear the oracle. Only a check on the complete input closes each
class.

**Considered and dropped:** resolving the winding against `climb`
inside the normalization (climb states the TRAVEL of a pass, not the
winding of the set; folding it in would reverse legal aligned
picks); per-band normalization on the stored track (equivalent for
two guides, but it re-reads `opposite_sense` per pair and the note
ordering changes for no gain); a per-pass swept-area check at emit
(a degenerate family is an INPUT defect - refused at the lap door,
never half-generated); refusing EVERY constant-station guide
(a radial arc at one station sweeps a real annulus - only the
constant-radius and constant-station class is degenerate); refusing
by the angular width alone (the width is positive exactly when the
band is degenerate - the width is the wrong witness).

**Cost to change:** `normalize_set` and `refuse_ring_lap` live in
`rotary_flow_set.rs`; the band loop lost its per-band reversal and
now clones an already-normalized track. The follow-up the review
named non-blocking - crossings between NON-adjacent guides still
pass - is filed as its own issue and stays out of this branch.
