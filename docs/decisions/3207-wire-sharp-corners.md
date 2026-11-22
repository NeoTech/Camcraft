## #3207 - Wire Contour: sharp inside corners at the wire radius

**Decided.** A Wire Contour has a new per-operation option,
`sharp_corners` ("Sharp inside corners at the wire radius" on the page,
`sharp_corners` in the project file and through MCP `edit_operation`).
The default is off, and off behaves as before: the cover law refuses
any rest past `COVER_TOL`. When the option is on, the cover law accepts
an uncut rest up to the kerf offset (wire radius + spark gap) at each
point of the profile. The worst accepted rest is said:

- as a note on the generated path (`wire EDM: sharp inside corners
  stated - ...`), and
- on the post's wire verify line, after the op's slug words.

It is a note, not a refusal. A rest past the kerf offset is still
refused, with a message that says the option was on.

**Why.** The report's known-good program cuts a sharp 10 mm square die;
the corner simply takes the wire's radius. That is normal wire EDM
practice. The bound "rest up to the kerf offset" accepts every sharp
inside corner of 60 degrees or more (a corner of angle a keeps
`k / sin(a/2) - k`, which is `k` at 60 degrees; a 90-degree corner keeps
0.414 k). A sharper corner, or a slot deeper than `k` that the kerf
cannot enter, leaves more than the wire's own radius and stays refused:
that rest is not "the corner takes the wire radius", it is material the
drawing calls cut.

**Where it applies.** The vertical closed cut (die and punch), its skims
and taper (they go through the same `cut_pass` / `WireParams`), the
burnout's outer wall, and the cover law of an open run. The open run's
own mitre-fold check (`open::offset`) is unchanged.

**Dropped.**
- A free tolerance number (like the 4-axis `heal_tolerance`). The issue
  asks for "inside corners at the wire radius"; a yes/no option says
  that, and the bound follows from the wire, not from a typed number.
- Applying the option to the 4-axis cut. `ruled` reads its own
  `heal_tolerance` through `cover_healed`, which ignores the new flag.
  A 4-axis cut with the option on is judged as before.
- A new field on the sim crate's `WireVerdict`. The rest is a generator
  reading, so the verify line reads it from the cached path note
  (`operation::wire_verify::sharp_corner_said`), keyed on the shared
  prefix `wire_edm::SHARP_NOTE`.

**Cost to change.** The bound is one function (`laws::sharp_reach`). A
typed tolerance later would add one DTO key and keep this key's meaning.
