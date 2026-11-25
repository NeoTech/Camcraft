## #2828 - Rotary Flow field-eye parity and the radial link C unwind

**Decided:** this PR admits `OpParams::RotaryFlow` on the field eye
(`rotary_kind_lift` answers `Ok(())`) on a measured parity, and wires
the C unwind for the kind's own link shape - the retract cylinder.

The parity was measured, not assumed (release, the verdict class 1e-4
then the eye's class cell/4 on the 0.5 mm lattice, the laid bar's
neck band, its passes picked between the rings at x 2 and x 16):

- `Stock::simulate_rotary` chords the band's program into 35,491
  sub-chords (2,063 moves, 1,559 turning, half-angle 1.990e-3, centre
  sag 5.940e-6, within the 1e-4 pose law), wall 46.6 s at the verdict
  class, 1.312 ms per sub-chord.
- Height parity: 168 sphere-cut tops within 1.269e-5 of the
  independently written swept-ball/flute law (bound 2.011e-5 = the
  centre sag 5.94e-6 + `FIELD_VERDICT_EPS` 1e-5 + the law's sampling
  sag 4.2e-6), 1,015 standing tops exact, 0 flute-cut, 0 grazing,
  0 sliver columns.
- Sign parity: 7,053 probed cell centres, 0 misses, worst 0.000e0,
  off the swept skin by more than the band 1.832e-2.
- The eye's own class: 2,131 sub-chords in 3.0 s (1.414 ms each).

No epsilon was widened: `FIELD_VERDICT_EPS` and the sag bounds stand
as the other three kinds left them.

**The fixture's band is the neck, not the shoulder.** The flow
fixture in `common/rotary_bar.rs` first tried the shoulder's two edge
rings (x -2 and x 2); the feature chains on the laid bar store those
two rings advancing about the axis in OPPOSITE senses, and the kernel
refuses opposite-sense picks by name (`rotary_flow_unwrap`). The neck
ring (x 2) and the top cap rim (x 16) advance the same way, so the
parity band is the neck cylinder - 8 passes at a 2 mm stepover, each a
full ring. The flow kernel's swept-ball guard also needs the finer
revolve the band suite proves on: the fixture uses 72 facets
(`laid_at(72)`), because a 16-gon ring's 22.5 deg knots swing the pose
past the guard's interpolated-axis drift margin (measured refusal:
"sweeps the ball 3.0100 mm off the part, inside its 3.000 mm radius").
The other kinds keep the 16-facet bar - their engines are chord-fed,
not knot-fed.

**The unwind law reads a feed retract's arrival as rest.** The flow
kernel leaves each pass the rotary twins' way: a FEED retract along
the radial to the link cylinder (`rotary_links`' cut-feed exit), then
rapid index chords, then a plunge. Under `unwind_c`'s old cut-reach
reading ("every non-rapid end plus every feed start") the retract's
end poisoned its own rest cylinder - the strict clearance test
`rmax > r_cut` could never hold, and no radial rewind point existed
for ANY path that retracts by feed. The refinement is the mirror of
the law's own plunge-start rule ("a plunge's start is where it leaves
the clearance, not where it cuts"): a feed a RAPID follows ended at
rest, not in a cut, so its end leaves the cut reach. Safety holds two
ways: the point was fed to, so the tool's envelope there is already
swept (an orbit through it cuts nothing new), and every inserted orbit
chord is still judged by the verifier, the solid bar and the post gate
downstream - the module's own law, unchanged. The axial contour case
is untouched (its retracts are rapids; its fed ends sit below the
plane), and the cl-level pins that refuse a radial link at or inside
the cut radius still refuse.

**The gate names the cylinder.** `stamp_spindle_c`'s boolean `unwinds`
became the enum `Unwind { NoProof, Axial, Radial }`: Rotary Contour
proven over the axial plane, Rotary Flow proven on the retract
cylinder, everything else refused where it would need a rewind - the
refusal's words now name both proven kinds. No new `Option` context
field was added anywhere: the enum carries the radial admission
itself, so there is no `None` case to discipline.

**Dropped:** posting the unwound flow band through the lathe post was
considered and held out - the radial field law reads the ball resting
on the band's convex crease ring as a gouge past its allowance, the
finding #2826 already reports, and that warning blocks the post of a
clean unwind. The #906 rate law is pinned instead: the multi-pass band
rewinds on a stated C rate and is refused by name without one.

**Cost to change later:** a new rotary kind still lands on
`ROTARY_KIND_UNMEASURED` until its own parity lane is green (the const
stays, no kind carries it today). A kind whose links rest on a new
clearance shape (neither the axial plane nor a retract cylinder)
extends the enum with its own arm and its own end-to-end proof, the
way this PR did for the cylinder.
