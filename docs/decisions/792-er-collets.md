## #792 - Built-in ER collet chucks (ER11-ER40), and #800's review envelopes

**What was decided.**

- **One ER table.** `holder_library::ER_SIZES` (ER11, ER16, ER20, ER25,
  ER32, ER40) is the only ER nut table. The taper-mounted ER entries of
  #793 and the new standalone chucks both draw from it (#800 review:
  consolidate to one source).
- **Standalone chucks.** Six new entries, "ER11 chuck" to "ER40 chuck":
  a straight-shank ER collet chuck drawn as its nut and a body at the
  nut's diameter, up to the body's back face. That back face is the
  gauge line, where the straight shank goes into the spindle or adapter.
  The shank above it is not drawn, the same way the steep taper is not:
  the stack laws put the spindle housing on the profile's top. Default
  gauge lengths (nose face to back face): ER11 30, ER16 40, ER20 45,
  ER25 50, ER32 55, ER40 65 mm. These are typical chuck-head lengths,
  rounded up: a longer body draws more steel, never less, and the field
  is the user's to set.
- **BT40 ER40** is added (default gauge 80 mm), which #793 left for this
  issue. Its Ø63 nut is as wide as the BT40 flange, so the outline runs
  straight into the flange with no step.
- **The flare envelope (#800 review).** From the body's radius out to
  the flange's, a 45-degree cone ending at the flange's front face is
  drawn. It starts `(flange r - body r)` below the front face. A real
  holder's fillet or cone between the nut or body and the flange is
  under that envelope as long as it rises no steeper than 45 degrees.
  The minimum gauge length grows to hold it:
  `front face + nut length + max(1 mm, flare length)`.
  The judging laws read a sloped segment as a band at its larger radius
  (`SpunProfile::bands`), so in practice the flange is judged as a
  cylinder reaching down to the cone's foot. That is more conservative
  than the cone itself, and it costs no more reach than the band
  representation already imposes.
- **The Weldon clamp screw (#800 review).** A Weldon body is drawn
  `WELDON_SCREW` = 1.0 mm larger in radius than its catalog diameter
  (`HolderEntry::body_r`), all round. The DIN 1835-B side-lock screw sits
  in a counterbore and is nominally flush. A worn, substitute or
  longer screw, or a shallow counterbore, leaves the head standing a
  fraction of a millimetre to about a millimetre proud. 1 mm is our
  engineering allowance, not a standard figure; no public standard gives
  the protrusion. It supersedes #793's "about flush, not drawn" line.
- **The picker shows the projection (#800 review).** Beside the gauge
  length is a "projection" field: the nose face above the tool tip. It
  starts at the stored tool's stickout (flute plus shank length), the
  command's own default, and lands through the same
  `CommandKind::SetToolHolder` / `SetToolBody::library` (one undo step).
- **The live shank check reads the stored tool (#800 review note).**
  The picker's live "does not fit" line judges the document's tool, the
  same tool Apply holder judges, instead of the editor's unapplied
  draft. The two can no longer disagree.

### ER table (mm)

| Size | Collet range Ø | Nut thread | Nut Ø | Nut length |
|---|---|---|---|---|
| ER11 | 0.5-7 | M14×0.75 | 22 | 11.5 |
| ER16 | 0.5-10 | M22×1.5 | 32 | 17.5 |
| ER20 | 1-13 | M25×1.5 | 35 | 19.5 |
| ER25 | 1-16 | M32×1.5 | 42 | 20.0 |
| ER32 | 2-20 | M40×1.5 | 50 | 22.5 |
| ER40 | 3-26 | M50×1.5 | 63 | 29.0 |

Sources:
- **Collet ranges and threads:** ISO 15488 (DIN 6499) collet sizes. The
  ER11 M14×0.75 thread is cross-checked against the CGTK ER collet
  dimension table (https://www.cgtk.co.uk/metalwork/data/er).
- **Nut diameters and lengths:** ISO 15488 does not fix these. The values
  are typical standard round-nut (DIN 6499 form) catalog values.
  - ER11: makers range from Ø19 to Ø22, and the top of that range is
    taken.
  - ER40: Ø63 is the common standard round nut. Hoffmann's ER40
    clamping-nut sheet lists lengths of 20 to 29 mm across its styles
    (https://hoffmann-group.com/GB/en/houk/p/309621-40), and the longest
    is taken.
  - ER16 to ER32 are #793's values, unchanged.
- **Wording.** #800's review notes that "rounded up where makers differ"
  overstates ER16 to ER32: those are the common round-nut diameters, not
  maxima. Bearing ("power") and some slotted nuts run a few mm wider. A
  user with such a nut edits the drawn profile (it lands Custom).

**Why.** The user's spindle is BT30 with ER20 collets. Both "BT30 ER20"
and "ER20 chuck" are pinned point by point
(`the_bt30_er20_outline_is_the_table`, `the_er20_chuck_outline_is_the_table`).
The user will measure the real nut and holder later. The flare and
screw envelopes close the two places where #800's outline could be
smaller than the steel, because under-reported material can hide a
collision.

**What changed in #800's numbers.**
- The BT30 ER20 outline's fourth point moves from (90, 17.5) to
  (84.5, 17.5): the flare foot, 5.5 mm (23 - 17.5) below the flange
  front.
- `a_longer_gauge_raises_the_flange_and_keeps_the_nose` now finds the
  first flange-width band 9.5 mm lower: BT40 Weldon 12 mm, with a body r
  of 21 + 1 against a flange r of 31.5.
- The long BT30 Weldon 6 mm body is judged at r 13.5 (Ø27), not 12.5.
  The 15 mm wall in `a_long_gauge_weldon_clears_where_a_short_one_collides`
  stays clear.

**Considered and dropped.**
- A cone at the real flare angle per holder: makers differ, and no
  standard fixes it.
- A fillet arc: under the band reading it judges the same as the cone.
- Saying only "the flare is not drawn": the review offered this, but a
  missing envelope is exactly the silent under-report the Prime
  Directive forbids.
- Standalone Weldon holders: not asked.
- ER11 on BT30/BT40: not asked, and it can be added from the same table.
- A wider standalone body than the nut: straight-shank chuck bodies sit
  at or under the nut's diameter.

**Cost to change later.** Low. These are table constants (`ER_SIZES`,
`WELDON_SCREW`, `FLARE`, the standalone gauges) plus one outline
function. Applied holders are saved as drawn points, so a project never
shifts under a library edit.
