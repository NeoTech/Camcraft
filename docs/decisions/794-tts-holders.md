## #794 - Built-in Tormach TTS holders (ER20, ER16, set-screw)

**What was decided.**

- **Nine TTS entries** in the built-in holder library
  (`crates/tools/src/holder_library_tts.rs`), drawn with #793/#792's
  outline function and listed in the same `holder_library::library()`
  the tool editor's picker and #799's MCP tool read:
  - "TTS ER20" (default gauge 52 mm) and "TTS ER16" (49 mm);
  - "TTS Weldon" 1/4, 5/16, 3/8, 1/2 in and 6, 10, 12 mm (60 mm).
- **The TTS mount is a `Taper` with no standoff.** A new `TTS` constant
  (collar Ø38.1 × 12.7 mm over a Ø19.05 shank, `ring: "collar"`). With
  no standoff, the outline goes from the collar's top face straight to
  the axis, so the collar's top face is the gauge line. The 3/4 in
  shank above it sits in the spindle's R8 collet, out of reach of the
  work, and is not drawn (the same as a steep taper). `Taper` gains a
  `ring` field ("flange" / "collar") so the picker says which it is.
- **Gauge length semantics.** Tormach's TTS manual says the ground
  shoulder seats on the spindle face and that every tool is measured
  from that shoulder. So the TTS gauge line is the collar face, and
  projection (tip to nose) plus gauge length (nose to collar) is the
  collar-to-tip length the tool setter measures and the H offset holds.
  This is pinned by `a_tts_holder_tops_out_at_the_collar_the_setter_measures_from`
  and `every_tts_profile_is_closed_and_tops_out_at_the_collar_face`.
- **ER nuts come from the one ER table** (#792's `ER_SIZES`):
  - ER20 is drawn at Ø35 × 19.5. Tormach publishes Ø34, so ours is 1 mm
    larger.
  - ER16 is drawn at Ø32 × 17.5. Tormach publishes Ø27 (a slim nut), so
    ours is 5 mm larger.

  Both are never smaller than the steel, and there is still one table.
  A user who wants Tormach's slimmer ER16 nut edits the profile (it
  lands as Custom).
- **The 45-degree flare envelope** from #792 applies unchanged (ER20:
  1.55 mm from nut radius to collar radius).
- **Set-screw bodies** are drawn at Ø38.1 plus the 1 mm `WELDON_SCREW`
  allowance (r 20.05), which is wider than the collar. The outline steps
  in to the collar at the collar's front face.

### Sources and numbers

| Item | Value | Source |
|---|---|---|
| TTS shank | 3/4 in straight, ground shoulder seats on the spindle face; measure tools from the shoulder | Tormach TTS Operation Manual rev C (https://littlemachineshop.com/cdn/shop/files/TTSOperationManualRevC.pdf), "Description", "Principles of Operation", "Tool Measurement" |
| TTS ER20 holder (PN 31829) | OD 1.34 in (34 mm), installed height 2.05 in (52 mm), M25×1.5 | https://tormach.com/tts-toolholder-er20-31829.html |
| TTS ER16 holder (PN 31831) | OD 1.06 in (27 mm), installed height 1.93 in (49 mm), M22×1.5 | https://tormach.com/tts-toolholder-er16-31831.html |
| Set-screw holder bores | 1/4 (31818), 5/16 (31819), 3/8 (31820), 1/2 (31821) in; 6 (31822), 10 (31823), 12 (31824) mm | https://tormach.com/tts-set-screw-holder-31820-html.html, Tormach TD10408 / TD10416 set lists (https://tormach.com/media/asset/t/d/td10408_tts_cnc_operators_set_0820a.pdf, https://tormach.com/media/asset/t/d/td10416_tts_basic_tooling_set_0120a.pdf), part list in https://forums.tormach.com/t/tts-tool-holder-solid-models/3089 (file names only, models not used) |
| TTS blank holder | Ø1.5 in machinable end, 2.36 in long | Little Machine Shop listing, https://littlemachineshop.com/products/blank-tool-holder-tormach-tts (read through a search summary; the page itself rate-limited) |
| MX mills' spindle | BT30, not TTS; PCNC and M mills are R8 + TTS | https://www.tormach.com/tormach_mills/ |

**Not published, so enveloped (larger, never smaller):**
- *Collar diameter.* Drawn as Ø1.5 in (38.1 mm), the TTS blank's
  machinable diameter. A third-party TTS ER20 listing gives a 1.5 in
  "flange", which agrees.
- *Collar thickness.* Drawn as 0.5 in (12.7 mm). A thicker collar draws
  more Ø38.1 steel below the gauge line.
- *Set-screw body diameter.* Drawn as Ø1.5 in (the blank's diameter)
  plus the screw allowance.
- *Set-screw default gauge length.* Drawn as 60 mm (2.36 in, the
  blank's machinable length). The field is the user's to set from
  their tool setter.

**Not exercised: the user's own holders.** The maintainer's machine
(770MX) is BT30, so these TTS outlines have not been checked against a
real holder. A user with a TTS mill can measure the collar OD and
thickness, the nut OD and the installed height, and tighten them.

### The machine default: filed, not built

#794 asked for the 770MX (and 1100MX/M) machine files to default new
tools to TTS "where the holder model supports a machine default". It
does not: a `.mch` has no spindle-mount or holder field. Building one is
a machine-schema plus `AddTool` change, so #828 was filed. Tormach's
public mill comparison also shows that **the 770MX and 1100MX have a
BT30 spindle**, not TTS: TTS belongs to the 770M/1100M/PCNC/440. #828
proposes BT30 ER20 as the MX mills' default and TTS ER20 for the R8
mills.

**Considered and dropped.**
- *5/8 and 3/4 in set-screw holders.* No Tormach TTS part was found for
  either. A 3/4 in shank tool takes the TTS collar (PN 32432) on its own
  shank instead. That is a collar-only "holder", which would need a new
  clamp kind; left for a follow-up if wanted.
- *TTS ER32, ER11, shell-mill and face-mill arbors.* No public Tormach
  dimensions were found. A guessed flange or arbor would be an unchecked
  collision body.
- *A separate slim ER16 nut row.* That would make a second ER table.
  The larger common nut is the safe side.
- *A new `Mount` enum in place of `Taper`.* That would touch every
  entry and the picker for the same outcome: a zero-standoff `Taper`
  draws a collar exactly.

**Cost to change later.** Low. These are table constants (`TTS`,
`TTS_BODY_D`, the bores, the defaults). Applied holders are saved as
drawn points, so a measured correction changes only the holders picked
after it.
