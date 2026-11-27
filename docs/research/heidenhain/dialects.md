# Proposal: how CamCraft models HEIDENHAIN dialects

Source facts: `differences.md` (sections B2 and B3 hold the version thresholds used below). Status: proposal only. It does not change code.

Cycle numbers: the first bare number in each section has its HEIDENHAIN short name in parentheses. `cycles.md` has the full table.

## 1. What the machine file states

CamCraft already has the right field (#3181, `assets/machines/AGENTS.md` lines 299-305 on `origin/main`):

```json
"controller": {
  "post": "Heidenhain Klartext",
  "identity": { "model": "iTNC 530", "nc_software": "340490-03", "installed_options": ["1", "19"] }
}
```

Recommendation: **add no new machine-file field.** The file keeps stating facts only (control model, NC software, options). The post derives the dialect from these facts. This follows the existing rule "The file states facts only. Each post decides which documented releases and operations it serves."

Make the values canonical, so that one resolver can read them:

| Field | Canonical form | Examples |
|---|---|---|
| `model` | the model name as HEIDENHAIN writes it | `TNC 124`, `TNC 128`, `TNC 310`, `TNC 320`, `TNC 410`, `TNC 426`, `TNC 430`, `iTNC 530`, `TNC 620`, `TNC 640`, `TNC7`, `TNC7 basic` |
| `nc_software` | 6-digit ID, `-`, 2-digit release, optional ` SP<n>` | `340490-03`, `606425-04`, `606425-04 SP8`, `817600-18`, `340595-11`, `771851-18`, `817625-20` |
| `installed_options` | HEIDENHAIN option numbers as strings | `"8"` (Advanced Function Set 1: tilting/PLANE, rotary table), `"9"` (Advanced Function Set 2: FUNCTION TCPM / M128 5-axis), `"19"` (Extended programming functions: FK, cycles 201-205/208/209 on TNC 620), `"23"` (0.01 µm display step). The iTNC 530 uses the older numbers `"1"` (tilting) and `"2"` (5-axis) for the same functions. |

The resolver reads the family from the first five digits and the last digit as the variant: `34049` + `0..4` = iTNC 530 single/dual processor; `60642` + `0..5` = iTNC 530 HSCI line; `34056`/`73498`/`81760` = TNC 620; `34059` = TNC 640 (all releases -01 to -18); `34055`/`77185` = TNC 320; `77184` = TNC 128; `81762` = TNC7 (TNC7, TNC7 basic and TNC7 go share it, so key the resolver on `(model, nc_software)`). The release number (`-03`) orders versions inside one family. The iTNC 530 order is: 340420 < 340422 < 34049x-01 ... -05 < 60642x-01 (= 34049x-06 functions) < -02 (= -07) < -03 (= -08) < -04 < -04 SP8.

An unknown model or a version outside the table: the post refuses by name (as the Klartext post does now), or falls back to the lowest profile of that model with a warning. Recommendation: refuse; a silent fallback can post a block that the control rejects at load time, or worse, that it reads differently (cycle 247 (PRESETTING) before 340 422).

## 2. Capability flags

Grouped by the branch that a post takes. "ISO" means the DIN/ISO post, "KT" the Klartext post.

| Flag | Values | Meaning for the post |
|---|---|---|
| `language.klartext` | bool | Klartext programs (all controls) |
| `language.iso` | bool | DIN/ISO programs exist on this control/version |
| `blockForm` | `"contour"` / `"paraxial"` | `paraxial` = TNC 124/128: one axis per block, no `L`, no arcs, `R+`/`R-`. CamCraft should refuse these controls (or add a separate paraxial post later). |
| `decimals` | 3 / 4 | coordinate decimals (1 µm or 0.1 µm input resolution) |
| `programName` | `"number"` / `"name8"` / `"name"` | TNC 124 number only; TNC 310 up to 8 characters; later controls longer names |
| `toolCallByName` | bool | `TOOL CALL "NAME" Z` permitted |
| `toolNameMax` | 16 / 32 | name length in TOOL CALL / TOOL DEF |
| `toolDefLR` | bool | `TOOL DEF n L.. R..` permitted (false on TNC7 from 81762x-19) |
| `feedFzFu` | bool | FZ/FU feed words |
| `m140` | bool | `L M140 MB MAX` retract |
| `m136m137` | bool | M136/M137 feed-unit switch |
| `arcs.spatial` | bool | C/CC arcs outside the working plane run as spatial arcs |
| `arcs.helixLin` | bool | `C ... LIN_Z..` (KT) / third axis on G02/G03 (ISO); else helix only as polar `CP` / polar ISO |
| `drill.cycles` | set | available drilling cycle numbers, after the option check |
| `drill.c200` | `"base"` / `"q211"` / `"q395"` | cycle 200 (DRILLING) parameter level |
| `drill.c205` | `"base"` / `"q257"` / `"q379"` / `"q208"` / `"q373"` | cycle 205 (UNIVERSAL PECKING) parameter level (each level includes the earlier ones) |
| `oldCycles` | `"define"` / `"runOnly"` / `"none"` | cycles 1 (PECKING)/2 (TAPPING)/17 (RIGID TAPPING GS): definable, only executed from old programs, or absent |
| `preset.cycle247` | `"datum"` / `"presetTable"` | 247 sets a datum value (old) or activates a preset-table row |
| `preset.select` | bool | `PRESET SELECT` exists |
| `dwell` | `"cycle9"` / `"functionDwell"` | KT dwell form. ISO always uses `G04 F..` (cycle 9 (DWELL TIME)); `FUNCTION DWELL` in an ISO program is not confirmed for any version. |
| `tolerance` | bool | cycle 32 (TOLERANCE) / ISO G62 |
| `tilt` | `"none"` / `"cycle19"` / `"plane"` | working plane function; `plane` includes cycle 19 (WORKING PLANE) except on TNC7 (cycle 19 cannot be inserted from 81762x-17) |
| `planeAxial`, `planeSym` | bool | PLANE AXIAL; SYM instead of SEQ |
| `tcpm.kt` | `"none"` / `"M128"` / `"FUNCTION TCPM"` | 5-axis form in Klartext |
| `tcpm.iso` | `"none"` / `"M128"` / `"FUNCTION TCPM"` | 5-axis form in ISO (FUNCTION TCPM in ISO only from TNC 640 34059x-10 / TNC 620 81760x-07) |
| `q.qs`, `q.qlqr` | bool | string and local/nonvolatile parameters |
| `q.formulaPriority2020` | bool | TNC 640 34059x-11 changed operator priority: always write parentheses in formulas |
| `trans.datum` | bool | TRANS DATUM (TNC7: cycle 7 (DATUM SHIFT) is converted to TRANS DATUM on edit) |

## 3. Profiles (control + NC software → flags)

Only the controls with full data. TNC 124 and TNC 128 → `blockForm: paraxial` (refuse). TNC 406/416 (EDM) → out of scope.

Each row names its sources in the last column. "B3" means a row of section B3 of `differences.md` (manual ID and PDF page). "B2" means the row of that name in section B2. "cycle list" means the script list of `CYCL DEF` numbers per version (not published). A cell is **unknown** when the sources do not settle it; a post must refuse the function, or ask, for an unknown cell. All cells were checked again against their sources after the review of PR #4099.

| Profile | Covers | iso | dec | byName | m140 | c200 | c205 | oldCycles | 247 | dwell (KT) | tilt | tcpm.kt / tcpm.iso | other | Sources |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `tnc310` | TNC 310 286 040 / 286 140 | no | 3 | no (not found) | no (not found) | base | - (no 205) | define (1, 5) | - (no 247) | unknown (cycle 9 not in the cycle list) | none | none | name ≤ 8 chars; ~6000 blocks | iso, dec: B2 (331 645-22 printed p228); c200: B3 31350922 p106, 33164522 p119; cycles: cycle list |
| `tnc410` | TNC 410 286 060/080 | yes | 3 | no | no (not found) | base | - (not TNC 410) | define | - (not TNC 410) | cycle9 | none (cycle 19 not TNC 410) | none (M128 not TNC 410) | no cycle 32 (not TNC 410); cycles 200-204, 210-214 | iso: 345 940-22; dec: 340 135-22 spec; byName, 205-209, 247, 19, 32, M128: 345 940-22 p6, p12, p17-19; c200: B3 30974025 p139; cycle 9: cycle list |
| `tnc426a` | TNC 426 280 460-470 | no (not found) | 4 | yes (280 462 on) | no (not found) | base | - (no 205) | define | - (not found) | unknown (cycle 9 not in the cycle list) | cycle19 | none | | byName: 311 062-21; c200: B3 31106221 p152, 31320620 p161; cycle 19: cycle list |
| `tnc426-472` | TNC 426/430 280 472 | yes | 4 | yes | no (not found) | base | - (no 205) | define | - (not found) | cycle9 | cycle19 | M128 / M128 | | iso: 318 179-24; c200: B3 32293824 p171; 205 "only with ... 280 474-xx": 318 179-24 p11; M128: B2 (first mention 280 472) |
| `tnc426-474` | TNC 426/430 280 474 (/475) | yes | 4 | yes | no (not found) | q211 | q257 | define | - (not found) | cycle9 | cycle19 | M128 / M128 | 205-208 new | c200, c205: B3 33164422 p182, p190; 205-208: 318 179-24 p11 |
| `tnc426-476` | TNC 426/430 280 476/477 | yes | 4 | yes | yes | q211 | q257 | define | datum | cycle9 | cycle19 | M128 / M128 | 209, 262-267 new | m140, 247, 209: 340 135-22 p6; c200, c205: B3 34013522 p240, p250 |
| `itnc-340420` | iTNC 530 340 420 | yes | 4 | yes | yes | q211 | q257 | define | datum | cycle9 | cycle19 | M128 / M128 | | byName: 36099423 p139; m140: 36099423 p20; c200: B3 36947521 p59; c205: B3 36099423 p257 |
| `itnc-340422` | iTNC 530 340 422 | yes | 4 | yes | yes | q211 | q379 | runOnly | presetTable | cycle9 | plane | FUNCTION TCPM (unsure) / M128 | cycles 251-254 | old cycles, 247, PLANE: 375 738-24 p7, p10; c200, c205: B3 38951220 p65, p245 |
| `itnc-49x` | iTNC 530 34049x-01 … -08, 60642x-01 … -03 | yes | 4 | yes | yes | q211 | q379 | runOnly | presetTable | cycle9 | plane (+AXIAL from 49x-03) | FUNCTION TCPM / M128 | FZ/FU from 49x-01; QS from -03; QL/QR from -05; spatial arcs from -03 | c200, c205: B3 53318820 p237, p262 to 670388-23 SP7 p80, p98; others: B2 rows of the same names |
| `itnc-60642x-04` | iTNC 530 60642x-04 (+SP8) | yes | 4 | yes (+QS) | yes | q395 | q208 | runOnly | presetTable | cycle9 | plane | FUNCTION TCPM / M128 | **the only iTNC with Q395/Q208** | c200, c205: B3 670388-24 p74, p92; "Cycle 205: Input parameter Q208 ... added": 670388-25 p13 |
| `tnc320-early` | TNC 320 340 551-01/-02 | no (not found) | 4 (unsure) | yes | yes | q211 | q379 | none (1/2/17 not found) | - (not found) | cycle9 | none | none | | byName: 55067120 p106; m140: 550 671-20 p17; c200, c205: B3 55067120 p183, p194 |
| `tnc320-34055x` | TNC 320 340 55x-03 … 34055x-06 | from -04 (-03: unknown) | 4 | yes | yes | q211 | q379 | none | presetTable (-03 on) | cycle9 | cycle19 (-03), plane (-04 on) | none | QL/QR from -05 | iso: B2 (679 226-20); c200: B3 550671-22 p67, 679226-22 p49; c205: B3 550671-22 p230, 679220-22 p77; 247, 19: cycle list |
| `tnc320-77185x` | TNC 320 77185x-01 … 771851-18 | yes | 4 | yes | yes | q395 | q208 (-01 on), q373 (-18) | none | presetTable | cycle9 (≤-02), functionDwell (-04 on) | plane | none | | c200: B3 1096959-20 p89; c205: 1096959-20 p9 and B3 p82; q373: B3 1303429-20 p104; dwell: B2 `FUNCTION DWELL` |
| `tnc620-early` | TNC 620 340 56x-01 | unknown | 4 | yes | yes | q211 | q379 | none | presetTable | cycle9 | cycle19 | M128 / M128 | | iso: B2 says not found, the keyword search found ISO words (636 026-20 p536); byName: 636026-20 p133; c200, c205: B3 636026-20 p69, p239 |
| `tnc620-mid` | TNC 620 340 56x-02 … 73498x-02 | yes | 4 | yes | yes | q211 | q379 | none | presetTable | cycle9 | plane | FUNCTION TCPM (-03 on) / M128 | 201-205, 208, 209 need option 19 | c200: B3 679295-20 p62, 819499-21 p55; c205: B3 679295-20 p80, 679295-22 p82; TCPM: B2 |
| `tnc620-81760x` | TNC 620 81760x-01 … -18 | yes | 4 | yes | yes | q395 | q208 (q373 from -16) | none | presetTable | cycle9 (-01..-03), functionDwell (-04 on) | plane (SYM -06 on) | FUNCTION TCPM / M128 (-01..-06), FUNCTION TCPM (-07 on) | helixLin -16; PRESET SELECT -08; option 19 as above | c200: B3 1096886-20 p92; c205: B3 1096886-20 p84, 1303427-21 p95; others: B2 |
| `tnc640-a` | TNC 640 34059x-01 / -02 | yes | 4 | yes (32 chars from -02) | yes | q211 | q379 | none | presetTable | cycle9 | plane | FUNCTION TCPM / M128 | | c200: B3 892905-20 p64, 892903-21 p57; c205: B3 892905-20 p82, 892905-21 p88 |
| `tnc640-b` | TNC 640 34059x-04 … -06 | yes | 4 | yes | yes | q395 | q208 | none | presetTable | cycle9 | plane | FUNCTION TCPM / M128 | | c200, c205: B3 892905-22 p98, p90 |
| `tnc640-c` | TNC 640 34059x-07 … -09 | yes | 4 | yes | yes | q395 | q208 | none | presetTable | functionDwell | plane (SYM -09) | FUNCTION TCPM / M128 | spatial arcs | c200: B3 892903-25 p73; c205: B3 892905-25 p97; dwell, SYM: B2 |
| `tnc640-d` | TNC 640 34059x-10 … -18 | yes | 4 | yes | yes | q395 | q208 (q373 from -16) | none | presetTable | functionDwell | plane | FUNCTION TCPM / FUNCTION TCPM | formula priority -11; PRESET SELECT -11; helixLin -16 | c200: B3 892905-28 p123; c205: B3 892905-28 p104, 1303406-21 p106; TCPM ISO: B2 (1322095-02 p109) |
| `tnc7` | TNC7 81762x-16 … -20 (+basic) | -16: no; -17 on: yes | 4 | yes | yes | q395 | -16: unknown (no 205 example); -17 on: q373 | none | presetTable | functionDwell | plane (cycle 19 (WORKING PLANE) edit-only from -17) | FUNCTION TCPM / FUNCTION TCPM | `toolDefLR` false from -19; cycle 7 (DATUM SHIFT) → TRANS DATUM | iso: B2 (1373081_03 p17, 1358773-20); c200: B3 1358773-20 p386; c205: B3 1358775-21 p116 |

Changes after the review of PR #4099: TNC 320 77185x-01 gets Q208/Q395 (was -05); the TNC 410 profile drops 205, 247, cycle 19 and M128 (the shared ISO manual excludes them for the TNC 410) and gets `c200: base`; the TNC 426/430 280 472-477 profile is split into three (205 from 280 474, M140 and 247 from 280 476); TNC 320 340 551-01 to 34055x-06 get `c205: q379` (B3 shows Q379/Q253 from 340 551-01); TNC 620 340 56x-01 `iso` is unknown; TNC7 81762x-16 `c205` is unknown.

Option gating on top of the profile (from `installed_options`): `tilt` needs option 8 (iTNC: 1); `tcpm.*` needs option 9 (iTNC: 2); on the TNC 620 cycles 201-205/208/209 need option 19. An absent `installed_options` list means "unknown": the post should then refuse functions that need an option, or warn. An empty list means none installed.

## 4. What the two posts branch on

**Klartext post** (`assets/posts/heidenhain-klartext.cps`):

- `onOpen`: resolve the profile; refuse `paraxial` and unknown profiles (replace the fixed `iTNC 530` / `340490-03` check at line 74).
- Program name rule (`programName`), coordinate decimals (`decimals`).
- Tool change: `TOOL CALL n` vs `TOOL CALL "NAME"` (`toolCallByName`, `toolNameMax`).
- Retract: `L M140 MB MAX` only when `m140`; else `L Z.. R0 FMAX M91/M92` from the machine file.
- Preset: `CYCL DEF 247` only when `preset.cycle247 == "presetTable"`.
- Feed unit reset `M137` only when `m136m137`.
- Drilling (not written today): CYCL DEF 200 (DRILLING)/203 (UNIVERSAL DRILLING)/205 (UNIVERSAL PECKING)/206 (TAPPING)/207 (RIGID TAPPING) with the Q list of `drill.c200` / `drill.c205`; refuse a cycle that is not in `drill.cycles` after the option check.
- Dwell: `CYCL DEF 9.0 DWELL TIME` / `9.1 DWELL ..` or `FUNCTION DWELL TIME ..` (`dwell`).
- Helix: `CC` + `CP IPA.. IZ.. DR±` always; `LIN_` only with `arcs.helixLin`.
- Tilt/5-axis (not written today): `tilt`, `tcpm.kt`, `planeAxial`, `planeSym`.

**DIN/ISO post** (`assets/posts/heidenhain.cps`):

- `onOpen`: refuse when `language.iso` is false (TNC 310, TNC 128, TNC7 81762x-16, TNC 320 before 340 55x-04, TNC 426 before 280 472).
- G200: write `Q395` only when `drill.c200 == "q395"`; write `Q211` only from `q211`.
- G205: write `Q379`/`Q253` only from `q379`; `Q208`/`Q395` only from `q208`; refuse a dwell or deeper start that the level cannot express.
- G205/G203 on TNC 620 without option 19: refuse.
- Dwell: always `G04 F<s>` (cycle 9 (DWELL TIME)). Remove `FUNCTION DWELL TIME..` from ISO output.
- `G247 Q339=` only with `preset.cycle247 == "presetTable"`.
- `G62` only with `tolerance`.
- Coordinate format `decimals`.
- Tilt/5-axis later: `tcpm.iso` (`M128` before TNC 640 34059x-10 / TNC 620 81760x-07; always `M128` on the iTNC 530 unless the ISO FUNCTION TCPM form is confirmed).

## 5. What today's posts already assume (origin/main, read through git)

**Klartext post** - hard-wired to iTNC 530 340490-03 and the Fehlmann machine (`heidenhain-klartext.cps:74-79`). It assumes:

| Flag | Value assumed | Valid for 340490-03? |
|---|---|---|
| `language.klartext` | true | yes |
| `blockForm` | contour (L, CC/C) | yes |
| `decimals` | 4 (`local()` rounds to 4) | yes (0.1 µm) |
| `toolCallByName` | false (number only, 1-32767) | yes (name not used); note the 30 000 tool limit from 340 422 (`375 738-24 p10`), so 32767 is above the table maximum |
| `m140` | true (`L M140 MB MAX`) | yes (M140 from 340 420) |
| `preset.cycle247` | presetTable (`CYCL DEF 247 ... Q339=+n`) | yes (from 340 422) |
| `m136m137` | true (`M137`) | yes |
| drilling, dwell, tilt, TCPM, helix | refused | - |

**DIN/ISO post** - written from the TNC 640 ISO manual 892909-25 (34059x-07/-08 generation). It assumes:

| Flag | Value assumed | Lowest version where this is valid |
|---|---|---|
| `language.iso` | true | see profiles |
| `decimals` | 3 (`fu()` writes 3 decimals) | all (4 would also be valid on 0.1 µm controls) |
| tool call | `T<n> G17 S..` (number) | all ISO controls |
| `drill.c200` | `q395` (`Q395=+0` on G200) | iTNC 60642x-04; TNC 640 34059x-04; TNC 620 81760x-01; TNC 320 77185x-01 |
| `drill.c205` | `q208` (writes Q379, Q253, Q208, Q395) | same as above (Q379/Q253 alone: iTNC 340 422) |
| G206 / G207 | Q206 Q211 / Q239 | all controls with these cycles |
| `preset.cycle247` | presetTable (`G247 Q339=n`) | iTNC 340 422; TNC 320 340 55x-03; TNC 620/640 all |
| `tolerance` | true (`G62 T..`) | iTNC all; TNC 320 from 340 55x-03 |
| `dwell` | `FUNCTION DWELL TIME<s>` inside an ISO program, with no space before the value | **not valid on any iTNC 530**; on TNC 640 from 34059x-07 / TNC 620 81760x-04 / TNC 320 77185x-04 only if the ISO syntax accepts it (not confirmed). Use `G04 F<s>`. |
| helix | refused | - |
| 620 option 19 | not checked | G205 needs option 19 on a TNC 620 |

Result: the DIN/ISO output as it is today is fully valid only on TNC 640 34059x-04 or later, TNC 620 81760x-01 or later (with option 19), TNC 320 77185x-01 or later, and iTNC 530 60642x-04 (the programming station used in #3095, 606425-04). On the Fehlmann iTNC 530 340490-03 the words `Q395` (G200, G205) and `Q208` (G205) do not exist in that software, and `FUNCTION DWELL` never exists on an iTNC 530. This confirms defects 2 and 3 of [`picomax-55-review.md`](picomax-55-review.md), now with a version source: the iTNC 530 60642x-04 cycle manual lists "Cycle 205: Input parameter Q208 for retraction feed rate added" (670388-25 p13), and the cycle 200 (DRILLING)/205 (UNIVERSAL PECKING) example blocks of all 340 49x manuals end without Q395/Q208 (`differences.md` section B3, for example 53318820 p237 and p262).

## 6. Shipped machine files that use a HEIDENHAIN post

`git grep -l -i heidenhain origin/main -- assets/machines` returns one machine file:

| Machine file | `controller.post` | Stated identity | Probable profile |
|---|---|---|---|
| `assets/machines/fehlmann-picomax55.mch` | `Heidenhain Klartext` | iTNC 530, NC 340490-03 | `itnc-49x` (340 49x-03): PLANE (option 1 needed), FUNCTION TCPM (option 2), Q379/Q253 in 205, no Q395/Q208, no FUNCTION DWELL, QS strings yes, QL/QR no. The machine is 3-axis with a fixed W head, so tilt/TCPM do not apply. `installed_options` is not stated. |

No other shipped machine file names a HEIDENHAIN control. (The programming station used for the tests in #3095 is iTNC 530 606425-04 = profile `itnc-60642x-04`; it has no machine file.)

## 7. Open points (not checked)

- The exact ISO form of FUNCTION TCPM on the iTNC 530, and whether `FUNCTION DWELL` is accepted in ISO programs on TNC 620/640/320.
- TNC 320 340 551-01/-02 and TNC 620 340 56x-01: DIN/ISO availability (no ISO manual on the portal).
- TNC 426 280 460 (scan only) and 280 470: DIN/ISO.
- Option numbering per control (iTNC 1/2 vs 8/9): confirm from the options brochure (`_brochures/827222-27_TNC_Optionen_Zubehoer_en.pdf`) before the resolver uses it.
- Block-number limits and maximum program length per control: only TNC 124/310 found.
