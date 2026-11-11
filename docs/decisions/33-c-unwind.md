## #33 - C unwind at clear links (continuous C, slice 2)

**Decided.** A C-stamped live-tool path on a machine whose rotary C
states a travel is unwound by whole turns (`cl::unwind_c`, run by
`rotary_gate::stamp_spindle_c` right after `stamp_c`). The path is cut
into stretches at its clear links. A clear link is a point the tool
reached by a rapid that stands at the path's own clearance, meaning the
highest Z any move reaches or the largest radius any move reaches. On a
standing bar's Rotary Contour, that is the point between each ring's
retract and the next ring's plunge. A stretch whose C span fits the travel is left alone.
A stretch that does not fit is moved by the fewest whole turns that make
it fit. A stretch no shift can fit is refused by name at generation
("... no clear link stands inside it to unwind at"). The first stretch
is shifted without an orbit, because the section opens with a rapid to
its first C anyway. Every later rewind is written into the path as the
part-frame orbit it physically is: rapid chords round the circle through
the link point, each stating its C and the radial station pose, split so
the machine's C-linear arc stays within the op tolerance of the chord.
A machine that states no C travel keeps the stamped path exactly
(unstated-limit law).

**Why.** The rewind turns the part under a waiting tool. If it were only
a renumbering of C (or a single C-only block), the verifier, the solid
bar audit and the steel laws would never see the tool sweep round the
part, which is exactly the "wrap/unwind the verifier has not seen" this
issue must refuse. As real moves, every downstream law judges it with no
special case. Taking the fewest turns means the least spindle travel at
each link.

**Considered and dropped.**
- *Control roll-over (modulo-360 C, shortest path).* This depends on a
  per-control parameter the machine file does not state. Posting
  wrapped C words to a control without roll-over would rewind in the cut.
  It could be a future machine word, alongside the C-sense word.
- *Centring each stretch in the travel to rewind less often.* This needs
  half-turn shifts, which are not whole turns and would move the part.
  Whole turns centred gave no gain on the ±360 machine.
- *Unwinding anywhere a rapid occurs.* A rapid leaving the cut is not
  clear of the work. Only the path's own clearance (top Z or outer
  radius) is used.
- *Only rapids that start at the clearance.* The contour kernel retracts
  each ring straight up and then plunges, with no rapid between. That
  rule found no link on a real Rotary Contour, so the point between the
  rapid and the plunge counts as the link.

**Cost to change.** Local. `unwind_c` is one function in
`crates/cl/src/cl/rotary_unwind.rs`, and the policy (which links, how
many turns) is two small blocks in it.
