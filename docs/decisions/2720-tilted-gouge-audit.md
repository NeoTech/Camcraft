## #2720 - Audit: each tilted operation kind refuses a gouge at generation

**Decided.** One new test binary, `crates/app/tests/tilted_gouge_kinds.rs`,
holds one probe for each kind that accepts a tilt, except Slot (its pins are
in `tilted_gouge.rs`). Each probe cuts into a correct box part (12 facets)
in a box stock on the 4th-axis mill, upright, at A 90 and at A -90, each
case in its own world. The test asserts a blocking verdict at generation
(`Warning::Gouge`, or `Warning::StockChamferGouge` for the 2D Chamfer), a
refused post ("post blocked", and "cuts past the finish surface" when
tilted) and no file written. All kinds are refused; no P1 was filed.

**Why.** Under a tilt the stock sim does not judge posed moves, so the
operation's own verdict is the only guard (#2600). A synthetic box part has
no mesh defect, so a miss would be a law gap and not an import fault.

**Probes chosen, and the ones dropped.**
- Contour: a contour of its own part cannot cut that part. Side Inside on a
  whole section, and a negative stock to leave, are refused at create. The
  probe is a second part 2 mm past the contoured part's wall, along X (the
  tilt axis), so the neighbour stands beside the wall in every pose.
- 2D Chamfer on a part source has no depth law by design (it breaks the
  part's edge). The probe is the stock-source chamfer where the part fills
  the stock (#1631).
- Face and Circular: stock source, cut below the part's skin (#1607).
- Pocket, 2D Adaptive and 2D Offset: whole part, area offset 3 mm into the
  part (#1603).
- Engrave and Trace: whole part, 7 mm deep on a 3 mm mark budget (#1595).
- 2D Adaptive is the slow kind in debug: one test for each tilted pose
  keeps each test under the 60 s budget. Its upright pose took more than
  60 s and was dropped; the keep law's own suite (#1603) pins it. A higher optimal load (0.5) was tried to
  shorten it and dropped: it ran for more than 9 minutes in debug.
- A known-fail layout (one assert in each test) was not needed, because no
  kind missed.

**Cost of a later change.** A new tilted kind adds one `Case` builder, one
arm in `set_tilt` and one test. A change of the refusal text changes the
two `contains` asserts in `convicted`.
