## #33 - C unwind at clear links (continuous C, slice 2)

**Decided.** A C-stamped live-tool path on a machine whose rotary C
states a travel is unwound by whole turns (`cl::unwind_c`, run by
`rotary_gate::stamp_spindle_c` right after `stamp_c`). The path is cut
into stretches at its clear links. A clear link is a point the tool
reached by a rapid that stands at the path's own clearance, meaning the
highest Z any move reaches or the largest radius any move reaches. On a
standing bar's Rotary Contour, that is the point between each ring's
retract and the next ring's plunge. A clearance counts only if it is
strictly beyond every point the tool cuts: every feed and plunge end,
and every feed start. A plunge's start is where it leaves the
clearance. Without this rule, a ring at the path's largest radius would
have read as its own retract cylinder, and the tool would have orbited
in the cut. A stretch whose C span fits the travel is left alone.
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

## #33 - C unwind review round: Parallel/Offset scope, the C feed cap

**Decided (orchestrating session).** Rotary Parallel and Rotary Offset
needed either an end-to-end test like the contour's or a refusal by
name. They took the refusal: neither unwinds. On a machine that states a
C travel, each posts only a path whose stamped C already fits that
travel, which is what the machine check passed before slice 2. Where C
would need a rewind, generation refuses by name ("... unwinding C at
links is proven for Rotary Contour only"). This is pinned by
`rotary_gate::tests::parallel_and_offset_are_refused_by_name_where_c_would_need_a_rewind`.
Why the refusal and not the tests: Rotary Parallel on the spindle's C
needs a bar that leans between 1 and 2.6 degrees off Z. It refuses a
standing bar, and the C gate refuses anything leaning further, so no
honest fixture exists. Rotary Offset's end-to-end run is a heavy-lane
fixture (center_cut_multiaxis is one of the slow suites). Neither fit
the time box. Dropped: refusing these two kinds on every C-travel
machine, even when the path fits. That would remove behaviour that
already worked before this PR.

**Decided (requested by the review).** The rewind stays as coordinated
G01 moves. A G00 with C is not coordinated on most controls, so the tool
would leave the verified chord. fanuc-lathe.cps caps the composite F of
every C block, not only rewinds, so that neither the X/Z part nor the C
part outruns the machine file. Under a composite F, the control drives
X/Z at `F * lin / lcmd` and C at `F * |dC| / lcmd`. F is therefore the
smallest of three values: the composite value, `feedRateMax * lcmd /
lin`, and `C rate * lcmd / |dC|`. The C rate is the C's stated normal
speed, or its rapid speed if only that is stated. The script's rotary
inventory now carries each rotary's own `speeds` (deg/min) for this. The
rates still stay out of every mm/min field.

**Stated limit of the data.** A machine file that states no C rate
leaves C uncapped. The rotary rate is then unbounded by data, and only
the linear part is held to the stated feed. Inventing a rate was
rejected.

**What moved.** On the shipped-rate bench (C 6000 deg/min), the rewind
orbit now posts F6000 where it posted F22954. The retract stays F4000,
which is exactly the stated linear feed. The slice-1 golden
(`c_axis_helix.nc`) does not move: its machine states no C rate, and
the linear cap never binds below the composite value, because a block's
radial and axial change is never longer than its part-frame chord.

## #33 - C unwind review round 2: the radial link is opt-in, off for every shipped kind

**Decided.** `unwind_c` takes a `radial` flag. The retract cylinder (a
rapid at a radius strictly beyond every cut) counts as a clear link only
when the caller asks for it. Rotary Contour, the one kind that unwinds,
does not ask: its links are the axial clearance plane. A contour stretch
whose only clear point is radial is refused by name ("... radial links
are not taken for this kind"). The radial branch keeps its own unit
tests (unwinds at the cylinder radius, not at or inside the cut radius)
for the day Rotary Parallel or Rotary Offset is proven end to end and
admits it.

**Why.** The review found the radial branch live for Rotary Contour and
untested end to end. Which link a contour gets is decided by its
kernel, not by any op parameter. The waterline ladder links every ring
over the clearance plane (`clearance_z`, or `retract_z` when set), and
its rapids stand at the rings' own radii. So no honest radial-link
contour fixture exists: a radial link would only come from a rapid that
happens to sit outside the cut radius, a case no shipped op is built to
produce. Leaving that branch reachable and unproven was rejected.
Removing it outright was also dropped: Rotary Parallel and Rotary
Offset link radially by construction and will need it.
