## #2898 - Circular rings link through clearance

**Decided.** The Circular generator lifts the tool straight up to the
clearance plane at the end of each ring, before the next ring starts.
The next ring then enters from clearance, as the first ring does. This
applies to the flat mode and to the vault.

**Why.** Each ring runs its own level ladder. The first descent of a
ladder expects the tool at clearance. After the first ring, the tool was
on that ring's floor. The descent then made one diagonal rapid from the
floor up to clearance over the next ring's start. That rapid crossed the
core that the next ring had still to cut. Verification convicted it
(`rapid move N travels through stock`) and held the post. A 5 mm hole
with a Ø6 tool has only one ring, so it did not show the fault. The
vault had the same diagonal rapid.

**Dropped.**

- Cut the flat mode level by level (the pocket's z-major order, all
  rings at one plane). It was tried on this branch. It removes the
  helix crossings, but the inner rings then start with an axial plunge
  that is not named, also when the user asked for a ramp entry. Four
  ramp-entry tests went red (`circular_ramp_angle`, `entry_ramp_angle`,
  `circular_arcs`) and `flat_rings_step_inward_ring_major` pins the
  ring-major order on purpose. That is a design change, not this fix.
- Link the rings at the retract plane. The ladder's first descent
  traverses at clearance, so a lift to the retract plane would still
  leave a diagonal rapid.
- Refuse more than one ring. That refuses a legal job that the lift
  makes safe.

**Not fixed here.** With the helix entry, each ring's helix passes
through that ring's start, with its centre one helix radius inward.
When the stepover is two helix radii, the helix of ring k-1 and the
helix of ring k touch at the start of ring k, in the same depth band.
The crossing law convicts that touch and holds the post. The program
is refused, not unsafe. A new helix placement rule is a design choice,
so it has its own follow-up issue.

**Cost to change.** Low. The lift is one guarded move in
`crates/motion/src/machining2d/circular.rs`. A later link strategy for
rings replaces that move. Two goldens moved by that one move each
(`circular_ring_arcs.cl`, `circular_ramp_stay_down.cl`).
