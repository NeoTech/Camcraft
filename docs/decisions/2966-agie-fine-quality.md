## #2966 — Request a Ra 0.35 µm work quality for one 20 mm punch

Use the existing Wire Contour condition field for both qualities. The condition `AGIE_QN2` requests quality CCB030 (Ra 0.30 µm), as before. The new condition `AGIE_BERCO_RA035` requests the Fine+ values of the quality presets of our own implementation: Ra 0.35, Te 1.50 and Tkm 1.00 µm, three options disabled. Both conditions require the existing preview gap and the 0.25 mm wire. The project file keeps the selected condition.

Qualify the new request for one closed polygon punch on 20 mm cold die steel with Berco Cut 0.25 mm wire. Refuse other thicknesses, a die opening, an open cutoff, a punch group and a DXF arc punch. No control check covers them. Pin each refusal and pin that no directory is written.

### Port onto the zero-step setup (#2984) and the stack of #2978 (2026-11-24)

**The frame.** #2984 removed the old script writer (`jescript.rs`) that this PR changed. The zero-step setup writes one quality object for the piece and arms no technology. The operator assigns TECCUT on the control after a review. The Ra 0.35 request keeps that rule: the job names the requested quality and leaves each work at zero steps.

**Chosen: a quality value on the piece, not a new setup type.** `camcraft_post::agie_setup::Quality` has two values, `Ccb030` (the default) and `Ccb035`. The private `Piece` carries it. `PunchSetup20::encode_quality` is the one way to request CCB035; `encode`, `encode_die`, `PunchGroup20` and `CutoffSetup20` keep CCB030 by construction. The public `PunchSetup20` and `AgieJob` keep their fields, so no caller and no test constructor changed. `camcraft_post::agie::export_punch_with_review` takes the quality; with `Ccb030` it is `export_with_review`. Dropped: a `quality` field on `PunchSetup20` or `AgieJob` (about 30 constructors change, and a die or group could then carry CCB035 by accident), and a separate `PunchSetup20Fine` type (it would copy the stock and hole checks).

**Quality name `CCB035`.** #2984 found that a generic name (`QN2`) can bind other resident data. The name follows `CCB030`: CamCraft, Berco, Ra in hundredths. The old PR wrote `QN2` for both requests.

**Offsets by quality.** `CCB035_OFFSETS_UM` holds one row, 20 mm: 245, 155, 140 and 134 µm. Source: the control readings of this PR (T100 on all four passes, same offsets as the 20 mm CCB030 row) and the port check below. The contour checks (clearance, miter and entry) take the rough offset of the selected quality. For 20 mm it is the same 245 µm, so the 0.4 mm margin does not change.

**Unchanged bytes.** CCSEP20, DIE20, LOWER20, PAIR20, RACE20, CIRCLE20 and the inspection goldens keep their tests. A new test pins that the CCB030 path through the quality export writes the bytes of `export_with_review`. The FINE20 golden differs from CCSEP20 (renamed) in the one `JE_CreateQuality` line only.

**Control check (emulated Classic Cut 2S, 2026-11-24).** FINE20 imported byte for byte with `CCB035[0]` and one formal error, `Technologie : te definiëren`. Work Details showed CCB035 with Ra 0.35, Tkm 1.0 and Te 1.5. The TECCUT header was `Dia:0.25 H:20 Ra:0.35 Tkm:1.0 Te:1.5`. The control preselected row 0204: 4 passes, technology `estbca25nnn060b030020`. That is the Ra 0.30 technology that CCB030 also preselects. The exact Ra 0.35 row, 0289 (`estbca25nsn060b035020`, type S, 5 passes), was not preselected. With row 0204 assigned: `CCB035[4]`, T100 on each pass, offsets 245/155/140/134 µm, FORMALCHECK `geen fouten` (warnings: options 131072 and reference values). The GRAFICHECK trace equals the 15 mm CCB030 punch trace except Z, with a 1.000 mm separation. So on this control the request changes the displayed work quality and not the cut. The guide says so and tells the operator to stop if the preselected row or the offsets differ. Measuring Ra needs a physical cut.

### Request the Ra 0.35 technology itself (2026-11-24, measured on the control "25 Nov")

**Chosen: CCB035 sets the speed flag.** The job's quality object is now `JE_CreateQuality(je_piece,"CCB035","0.35,1.50,1.00,T,F,F",je_q1)`. The first option is the speed flag ("Snelheid" on the control). Decided by the user after the measurement below.

**Measurement** (`Camcraft-notes/agie-imports-20261125/FINE20-0289/summary.txt`, emulated Classic Cut 2S, throwaway clone, image SHA-256 unchanged):

- With `F,F,F`, the TECCUT header was `TYPE:V` and the control preselected the Ra 0.30 technology `estbca25nnn060b030020` (4 passes). The Ra 0.35 technology `estbca25nsn060b035020` was listed but not preselected.
- With `T,F,F` (probe), the quality page showed the speed option set and the header changed to `TYPE:VS`. The control then preselected `estbca25nsn060b035020` by itself. The import still showed `CCB035[0]`: the job arms no technology.
- Assigned with Toewijzen: 5 passes (main cut and four trims) at 234, 189, 154, 139 and 129 µm. FORMALCHECK: `geen fouten` (warnings: options 131584 and reference values). GRAFICHECK: each face offset equals the table offset. The last trim runs in the main direction and ends on the face.
- The first TECCUT column ("Dis") is a match distance, not a row id: the same technology was 0289 under `TYPE:V` and 0199 under `TYPE:VS`. The technology name is the stable key. The review text and the user guide name it, not a number.

**Offsets.** `CCB035_OFFSETS_UM` is now `[(20, [234, 189, 154, 139, 129])]`, five entries. `Quality::offsets_um` returns a slice of all passes; `Quality::rough_um` gives the main cut. CCB030 keeps its 4-entry table and `qn2_offsets_um` keeps its `[u16; 4]` type, so the die, group, cutoff and DXF paths do not change. The review text lists all offsets of the quality. The contour checks (clearance, miter and entry) take 234 µm for CCB035. The stock margin stays 0.4 mm: 0.234 + 0.125 = 0.359, rounded up to 0.4 (pinned by `ccb035_has_five_measured_passes_and_keeps_the_0_4_mm_margin`).

**Bytes.** Only the quality line of the FINE20 SBL changes (`F,F,F` to `T,F,F`). The FINE20 ISO and SBR and all CCB030 goldens stay byte-identical.

**Dropped:** keep `F,F,F` and tell the operator to pick the Ra 0.35 row by hand (an operator choice that the control does not check, and a row number that is not stable); arm the technology in the job (the zero-step setup of #2984 arms nothing).
