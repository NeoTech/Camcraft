## #536 - Wire EDM: taper contour (constant angle)

The umbrella (#290) decided the taper is the move's pose (D2: `to` on the lower programmed plane, the pose the wire's direction toward the upper guide, both ends interpolating - the ruled surface), the taper words are machine data (D5), and the verify judges the ruled surface at every height and the angle and U/V against the machine (D7). These are the points it left open.

**Decided:**

- **One signed angle, `taper_deg`, on the Wire Contour.** 0 is the vertical cut; the DTO omits it at 0, so every file written before reads unchanged and a vertical cut's stamp, CL and NC are byte-identical (pinned). Positive widens the profile downward from the top plane - the die's relief and the drafted punch's base, both the common case; negative narrows it.
- **The profile holds on the top plane.** #532 reads the profile on the top plane; the taper keeps it there and leans the wire below it. The kerf is measured across the leaning wire, so the horizontal offset on the top plane is `(wire radius + gap) / cos(angle)`, and every law of the vertical cut (cover, start hole, kerf) runs on that top-plane path unchanged. A corner refusal on a tapered cut therefore quotes the lean-scaled (horizontal) wire radius and gap.
- **Sides lean square to themselves; corners along the mitre.** Each lower-plane vertex is its top-plane vertex moved `H tan(angle)` along the textbook mitre `(n1 + n2) / (1 + n1.n2)`, so each side's face is a plane at exactly the angle and the corners lean a little more (`tan a / cos(half turn)`). Every pose is checked against the machine's maximum, not only the stated angle. A side the lean folds over, or a turn sharper than about 140 degrees, is refused by name.
- **Upright at the start hole.** The wire is threaded vertical; the lead-in leans it to the first pose (a ruled twist the verify reads) and the lead-out stands it upright again with an explicit vertical pose, so the stream commands the return.
- **Refusals and where they stand.** The angle against `edm.max_taper_deg` (an unstated maximum refuses any taper - absent is not zero): at generation, naming the op and the angle. The U/V offset against the U and V strokes, the programmed plane outside the guides, no U/V axis, no guide planes: in the wire EDM machine check, which blocks the post. The guide planes are read in machine Z, moved by the work offset's Z, as the X/Y strokes are judged.
- **The post writes UV, with codes from `edm.codes`.** The app turns each leaning move into the lower guide's X Y and the upper guide's offset (`PARAM WireU`/`WireV`), from the pose and the guide planes; the script writes `taper_on` before the first leaning move, `uv` (a template with `{u}` and `{v}`) on each, and `taper_off` once the wire is upright, before the cut. Each code missing refuses by name. The synthetic fixture states `FX-TAPER-ON`, `FX-TAPER-OFF` and `U{u} V{v}` (D10: invented, not any control's).
- **Verify reads the lean only from a stated taper.** `WireJob.taper`: a cut stating a taper has its poses read as the wire's lean; any other posed move stays unreadable (the #534 refusal stands). At each judged height the wire's section is `to + t (z - z0)`, and the kerf's reach toward the product in direction `u` is `k sqrt(1 + (t.u)^2)` - the ellipse the leaning kerf cuts from the plane. The slug is judged per height from the wire's own section.

**Considered and dropped:**

- **Angle words (a control-side taper with left/right and an angle).** The control then decides the corner and the lead-in lean itself, which the wire laws cannot see - motion the verify never judged. Refused by not being offered: a machine that states only angle codes has no `uv` and the post refuses by name. A follow-up can add it once a corner mode is stated as machine data.
- **The profile on the lower programmed plane.** It would make `to` the profile's own parallel, but #532 reads the profile on the top plane and the product's working edge (die land, punch face) is usually at the top.
- **One kerf radius `k / cos(max lean)` everywhere in the verify.** It over-reads every straight side and refuses a correct cut; the directional reach is exact on sides and safe at corners.
- **Separate swing moves at corners.** Not needed: mitred vertices keep each face planar with one pose per vertex.

**Not exercised on a real EDM control (D10).** The goldens are posted against the synthetic fixture; a user with a real machine fills `edm.codes`, the guide planes, the maximum taper and the U/V strokes from its manual and dry-runs before any cut.

**Cost to change later:** low for angle words (another branch in the script and a code list); variable taper and 4-axis (#537) are a pose per vertex already, so they reuse the verify and post seams unchanged.
