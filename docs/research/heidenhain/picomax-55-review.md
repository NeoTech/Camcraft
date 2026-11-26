# Fehlmann Picomax 55 / HEIDENHAIN: review of CamCraft's two HEIDENHAIN posts and the machine file

> Published copy of a review made for CamCraft issue work (#3095, #4085-#4088). The machine data come from the machine's parameter backup on its delivery CD. Access codes, passwords and file names that identify the single machine are removed. The MP numbers and values are kept. Line numbers such as `heidenhain.cps:381` refer to `origin/main` at commit `034e91b8b`.

- CamCraft checked at `origin/main` `034e91b8b` (read through git, nothing checked out or changed).
- Manual: Fehlmann "PICOMAX 55 CNC 2/3 Programming manual", P07-35-06-E (2006), 340 pages.
  Page references look like "p. 2-19 (PDF 47)". The first number is the page number printed in the manual. The PDF number is the page in the PDF file.
- **Machine data (added later):** the machine's parameter backup from its delivery CD (the MP file). Header: "Maschinenparameter für P55CNC mit iTNC530 (MC422(B)/UV106B) · ID 340490 03 · W. FEHLMANN AG · Stand 1.11.07". The review also used `OEM.SYS`, `PRESET.PR` and the PLC file list (LST file) from the same backup, and the machine owner's notes. Machine parameters are cited as "MP nnnn". The access codes and passwords in these files are left out. **See section 4a.**
- Files reviewed: `assets/posts/heidenhain.cps` (DIN/ISO), `assets/posts/heidenhain-klartext.cps` (Klartext), `assets/machines/fehlmann-picomax55.mch`, the Heidenhain goldens and recorded goldens, `crates/app/tests/klartext_builtin.rs`, issue #3095 and the follow-up notes on #3095.

**About the manual.** This manual does not name the control model or the NC software version. It also does not give travel, spindle or feed limits. It says those are in the operating and maintenance manual, chapter 7 (p. 1-5, PDF 13). The MP file fills these gaps. **Every example in the manual is Klartext. There is no DIN/ISO example anywhere in it.** DIN/ISO files are still allowed on this control (MP 7224, section 4a).

---

## 1. Summary

- **Klartext post:** safe and careful, but narrow. It covers milling only: flat, ball, bull-nose and chamfer cutters, straight moves and flat XY arcs, one setup per program. It refuses all drilling and tapping, helices, smoothing and stops between operations. Its arc check is 160 times stricter than the control (MP 7431 = 0.016 mm). It works for single-setup 2.5D/3D milling jobs. It cannot post a complete part that has holes.
- **DIN/ISO post:** much wider. It has drilling, peck and tap cycles (G200/G205/G206/G207), presets, XZ/YZ arcs as short lines, setup-change stops and smoothing. The control allows `.I` files (MP 7224). It was tested on a programming station (606425-04, #3095), **not yet on the real Fehlmann (340490-03)**. It knows nothing about this machine's **manual tool change** (see defect 1).
- **Machine file:** consistent with the machine parameters. Spindle 9000 rpm in one gear range, rapids 11000/11000/6000 mm/min, iTNC 530 340490-03, manual change, flood M8. Missing: travel limits (X 0..500.2, Y 0..255.2, Z -147.2..0), max feed, rigid tapping (yes), the M92 change position and the tool-table size (254).
- **Top 5 to fix:**
  1. DIN/ISO on the Picomax: the tool call has no retract and no stop. The next tool's length becomes active while the old tool is still in the spindle. Refuse the post on a manual-change machine, or add the Fehlmann sequence. M6 halts the program on this machine (MP 7440 bit 0 = 0), so `M6` is a correct stop.
  2. Check the DIN/ISO output on the real 340490-03 control: Q395 (G200/G205), Q379/Q253 (G205) and `FUNCTION DWELL TIME`. The 2006 software probably does not know them. For the dwell, use G04.
  3. Klartext: add drilling and tapping cycles 200/205/207, and drill/tap tool kinds. This is the biggest gap for everyday work. Rigid tapping works on this machine (MP 7160 bit 2).
  4. Klartext arc check: use the control's tolerance, MP 7431 = 0.016 mm, not 0.0001 mm. Limit tool numbers to 1..254 (MP 7260) in both posts.
  5. Tool change and program end in the Fehlmann way: `L Z+0 R0 FMAX M92` (MP 950: the change height), an XY park move, then M6 or M0. Put the MP values into the machine file (section 4a).

---

## 2. What might be wrong now (defects)

Severity: **P1** wrong output (motion or crash risk) · **P3** a legal program is refused, the control gives an alarm, or a message is misleading · **P7** polish. "Re-rated" means that the MP data changed the rating or removed the doubt.

| # | Sev | Where (origin/main) | What CamCraft does | What the manual / MP says | Risk |
|---|---|---|---|---|---|
| 1 | **P1** (confirmed by MP) | `heidenhain.cps:381-392` (`T<n> G17 S..` then `M3`). The engine does not refuse: `crates/post/src/post/machine_installation.rs:67-68` only passes `toolChangeMode` to the script, and the DIN/ISO post never reads it. | On a tool change the post writes only the T block. It writes no retract, no M6 and no M0. The next block is `G00 G40 G90 Z+<clearance>` with the **new** tool's length. | The manual's change is `L Z+0 R0 FMAX M92`, then `L X-50 Y-50 R0 FMAX M6` (p. 2-11/2-19, PDF 39/47). TOOL CALL comes *before* the M6 move. MP 7440 bit 0 = 0: the program **halts at M06**. MP 7480.0 = 1: TOOL CALL only passes the T number to the PLC. Nothing in the MP file makes TOOL CALL itself stop. | A multi-tool DIN/ISO program runs on with the old tool, and the new tool's length and rpm are active. This gives a wrong Z and a possible crash. It happens if someone selects the DIN/ISO post for the Picomax setup (the machine file names Klartext, but the user can pick another post). |
| 2 | P3 (unsure) | `heidenhain.cps:759` | It writes `FUNCTION DWELL TIME<s>` with no space, for example `FUNCTION DWELL TIME0.5`. This block appears for ream/back-bore dwells and for peck drills with an air gap and a dwell. | The manual has no `FUNCTION DWELL`. The iTNC 530 dwell is cycle 9 (DIN/ISO `G04 F..`). The MP file confirms software 340490 03 (2006/07), before TNC 640-style functions existed. | The control probably refuses the block on load, as an alarm with no motion. Test it on the station, then on the Fehlmann. |
| 3 | P3 (unsure) | `heidenhain.cps:744-747` | G200 writes `Q395=+0`. G205 writes `Q379`, `Q253`, `Q208=+99999.999` and `Q395`. | The manual's cycle 200 has only 8 parameters, with no Q395 (p. 2-28, PDF 56). The MP file confirms the old software (340490 03) and has no MP about cycle parameter lists. | The 606425-04 programming station accepted the blocks (#3095). The 340490-03 software may refuse Q395/Q379/Q253 on import. Only a run on the machine settles this. |
| 4 | **P7** (re-rated from P3) | `heidenhain-klartext.cps:226-235` | At each tool change it writes `M0` with operator checks, never `M6`. | MP 7440 bit 0 = 0: M06 halts the program, as M00 does. M00 also switches the drives off (p. 1-17). The owner's notes on the operating manual (p. 4-76) call "Drive OFF" with the quill at the top the safe state for a manual change. | Both functions stop the program, and M0 also gives the safe drive-off state. Remaining open point: the PLC state that only M6 gives (lamps, interlock). This is polish, not a blocker. |
| 5 | **P3** (re-rated: confirmed) | `heidenhain-klartext.cps:297-301` | It refuses an arc when the start radius and the end radius differ by more than 0.0001 mm. The check uses values that are already rounded to 4 decimals. | **MP 7431 = 0.016 mm** (tolerance for the circle end point). | The post is 160 times stricter than the control. Rounding alone can give up to about 0.00028 mm of radius difference, so the post can refuse correct CAM arcs that the control accepts. Fix: allow up to MP 7431 (0.016), and keep a margin. |
| 6 | P7 | `heidenhain.cps` (no `G30`/`G31`; `onOpen` 260-269) | The DIN/ISO program has no blank definition. | The control needs the blank for the graphic simulation (p. 2-15, PDF 43). MP 7310 = %00001100: 3D graphics are switched on. | This is a likely reason (unsure) why the graphics pane showed `ERROR` in the #3095 station runs. The operator loses the test graphics. |
| 7 | P7 | `heidenhain.cps:793-797`, `heidenhain-klartext.cps:319-322` | Both posts end with M2 at the last clearance height. | The examples end with `L Z+0 R0 FMAX M92`, then an XY park with M30 (p. 2-5). MP 950 = X+120, Y+210, Z-0.05: M92 Z+0 is at the top of the range-1 Z window. | No wrong motion, but the tool stays just above the part for unloading. |
| 8 | P7 | `heidenhain-klartext.cps:227, 238` | It retracts with `L M140 MB MAX`. | MP 7432 = 0 and the range-1 Z limit is MP 910.2 = 0. So M140 MB MAX stops at machine Z 0. M92 Z+0 is Z -0.05 (MP 950.2 = MP 4210.0 "Z-Achse Werkzeugwechselpunkt"). Owner's notes: the physical full retraction is about 12.8 mm *above* the range-1 window (the ranges 2/3 open Z to +20, MP 911/912). | The two moves reach practically the same height. Neither reaches the full quill retraction. The post's own comment ("M140 can stop short of the physical change position") is correct. Writing M92 Z+0 instead is the documented Fehlmann form. Polish. |
| 9 | P7 | `heidenhain-klartext.cps:74-79` | It refuses any control that is not exactly `iTNC 530` / `340490-03`, and any machine that is not exactly `Fehlmann Picomax 55 (iTNC 530)`. | The MP header confirms "ID 340490 03" for this machine. | Correct for the machine, but Klartext output cannot load on the 606425-04 programming station without a second machine file. |
| 10 | **P3** (re-rated from P7) | `heidenhain-klartext.cps:129`; `heidenhain.cps:334-338` (no limit) | Klartext accepts tool numbers up to 32767. DIN/ISO has no limit. | **MP 7260 = 254** (tool-table lines). The manual also says 0..254 (p. 2-16). (MP 7340.6: the shop can switch the table off, which sets the value to 0.) | A tool number above 254 gives an alarm at the control, but CamCraft releases the program without a message. |
| 11 | P7 | `heidenhain-klartext.cps:178` | It accepts a BLK FORM side up to 99999.999 mm. | The maximum is 30,000 mm per side (p. 2-15). | Cosmetic only. |
| 12 | P7 (values now known) | `heidenhain.cps:329-331`; `fehlmann-picomax55.mch` | DIN/ISO checks only the maximum rpm, not the minimum. The machine file has no `feedRateMax`, so neither post checks feeds. | MP 1010: max 11000 mm/min X/Y and 6000 Z. MP 7470 = 0 (no other feed cap). Spindle min about 43 rpm (MP 3240.1 = 0.043 x 1000 motor rpm, ratio 1:1, unsure). MP 3120 = 1: `S0` is not allowed. | The control clamps an overspeed feed per axis, so this is no safety issue. Set `feedRateMax` 11000. The Z limit of 6000 cannot be stated per axis today. Both posts already avoid `S0` (verified). |
| 13 | P7 | `heidenhain.cps:1-11` | The DIN/ISO post was written from TNC 640 manuals. | The machine has an iTNC 530 with software 340490 03 (MP header). | This explains defects 2 and 3. Each word needs an iTNC 530 source or a run on the machine. |
| 14 | **P3 (new, unsure)** | `heidenhain-klartext.cps:18, 68-70`; `heidenhain.cps:27, 299-301` | Both posts accept preset numbers 1..6 (`workOffsetMax = 6`). | The preset table on the CD (`PRESET.PR`) has rows 0..5 only. Row 0 is "Manual coord." and rows 2-5 are Fehlmann demo rows. (`OEM.SYS`: PRESETTABLE = ON.) | `Q339=+6` can name a row that does not exist, and the control then gives an alarm. The shop may have added rows since 2008. Check the row count at the control. |

**Count now:** P1 = 1 · P3 = 5 (defects 2, 3, 5, 10, 14; of these, 5 and 10 are confirmed by the MP file) · P7 = 8 (defects 4, 6, 7, 8, 9, 11, 12, 13).

**Rating changes from the MP data:**
- Defect 1 stays P1, now confirmed: MP 7440 bit 0 = 0 means M06 is the stop, and TOOL CALL alone does not stop.
- Defect 4: P3 to P7. M6 and M0 both halt the program.
- Defect 5: unsure P3 to confirmed P3 (MP 7431 = 0.016).
- Defect 10: P7 to P3 (MP 7260 = 254).
- Defect 14: new P3, unsure.

---

## 3. What is missing for full support

| Feature | Manual / MP | Size | Everyday? | Notes |
|---|---|---|---|---|
| **Klartext drilling/tapping cycles** (CYCL DEF 200 drill, 205 peck, 207 rigid tap, 206 floating tap) and drill/tap/spot tool kinds | p. 2-28/2-29, 3-3..3-10; MP 7160 bit 2 (rigid tap) | Medium-large | **Yes** | Klartext refuses all cycles (`:327`) and all drill tool kinds (`:139`). Call style: CYCL DEF once, then `L X.. Y.. R0 FMAX M99` for each hole. MP 7440 bit 1 = 1: M89 is a modal cycle call at the block end on this machine. MP 7441 bit 2 = 1: a positive depth gives an error (both posts write negative depths, verified). |
| **Helix** (helical ramps, thread milling): Klartext `CC` + `CP IPA.. IZ.. DR±` | p. 3-52 (PDF 140) | Medium | **Yes** (pocket entry) | Both posts refuse a changing-Z arc (Klartext `:287`, DIN/ISO `:603-605`). DIN/ISO could split it into short lines, as it does for XZ/YZ arcs. |
| **Fehlmann tool change sequence**: `L Z+0 R0 FMAX M92`, XY park, M6 (or M0) | p. 2-11, 2-19; MP 950, MP 4210.0, MP 7440 | Small-medium | Yes | The machine file needs the change position. Z = M92 Z+0, which is MP 950.2 = -0.05 from the machine datum. XY park in workpiece coordinates (the manual uses `X-50 Y-50`). MP 7680 bit 3 = 1: a mid-program start ignores the M06 stops. Warn the operator about this. |
| **Program end park**: `L Z+0 R0 FMAX M92`, XY park, M30 | p. 2-5 | Small | Yes | See defect 7. |
| **Coolant 2 (M7, option), compressed air (M37/M38)** | p. 1-17/1-18; MP 4310.0 bit 8 "Kühlmittel 2 einzeln" = 0 | Small | Sometimes | Klartext knows flood only. Whether coolant 2 is fitted is unsure. MP 7340.1 / MP 4310.4 = 0: a user setting couples the coolant to M03 instead of keeping it modal (unsure). Writing M8 after M3, as both posts do, suits this. |
| **Klartext stops between operations** (M0/M1, `afterModes`) and several setups | M00/M01 p. 1-17 | Small-medium | Yes | Klartext refuses both today. |
| **Klartext smoothing** (CYCL DEF 32 tolerance, or M112) | M112 p. 1-19; MP 7680 bits 7-9 (M112 inserts rounding) | Small | Sometimes | Klartext refuses the `Smoothing` parameter (`:126`). |
| **Blank form in DIN/ISO** (`G30`/`G31`) | p. 2-15; MP 7310 | Small | Yes (test graphics) | See defect 6. |
| **Radius compensation in the control** (RL/RR, APPR/DEP, DR wear) | p. 2-4, 3-26, 4-4 | Large | Nice to have | Both posts refuse it on purpose. |
| **Combined spindle + coolant** (M13/M14), oriented stop M19 | p. 1-17; MP 7442 = 19 | Small | No | |
| **AT 100 / AT 125 CNC dividing attachment** (A axis) | ch. 12; MP 100, MP 10, MP 7340.0 | Large | **No: not fitted** | MP 10 = 23 (axes X, Y, Z, W). A is prepared (MP 410.3 = A, 7200°/min) but inactive. There is no `ACHSE_4` drive file. Keep both posts refusing rotary moves. |
| **Quill / 2-axis mode** (M42/M43, M52, M22/M23/M24) | p. 1-17/1-18, 2-41..2-59; MP 4210.14 = 4, MP 7340.9 | Small (a guard) | No | CAM programs need 3-axis mode. MP 4210.14 sets the power-on 2-axis preselection; the value 4 is not documented (owner's notes). The MP file does not say if an M43 at the program start is safe. |
| **Pocket/slot/SL cycles, transforms, FK, Q-parameter programs** | ch. 3-7 | Large | No | For hand programming only. |
| **Fehlmann OEM cycles** 300-305, 310, 313-316, 320, 321, 323, 324, 328, 329 | ch. 11; LST file `PLC:\OEMCY1\DES\CYCDES3xx.DES` | Medium | No | **Installed** on this machine. If anyone uses them: the "old cycles" add the clearance *incrementally* and start where they are called (p. 11-19). |
| **Touch probe / tool setter** | MP 6010 = 0, MP 6500, MP 6580 (TT centre X+250 Y+120 Z-20), a Blum table reference | Large | Depends | The MP file has a tool-setter set-up and a reference to a Blum table, but this does not prove that a probe is fitted (unsure). Both posts refuse probing. |
| **Tool table export** (TOOL.T: NAME, L, R, R2, DL, DR, LCUTS, ANGLE, TYP, NMAX…) | p. 2-16; MP 7260 = 254, MP 7266 (columns), MP 7483 = 0 | Medium | Helpful | Klartext relies on the table. MP 7483 = 0: TOOL CALL takes names or numbers. |
| **Long programs** | MP 7229.0 = 9999 | — | 3D jobs | Unsure: the control may not check programs longer than 9999 blocks before the run. Note for long 3D programs. |

---

## 4a. Answers from the machine parameters (MP file)

| Question | MP | Value | Meaning | What it changes in CamCraft |
|---|---|---|---|---|
| Arc end-point tolerance | MP 7431 | 0.016 mm | Allowed difference between start and end radius | Klartext `:300`: use 0.016 (with a margin), not 0.0001. Defect 5 confirmed. |
| Max machining feed | MP 1010.0/.1/.2; MP 7470; MP 1090/1092 | 11000 / 11000 / 6000 mm/min; MP 7470 = 0; MP 1090.0 = 50, .1 = 1000, MP 1092 = 30000 | MP 1010 is the axis speed limit, and a programmed F is clamped to it. MP 7470 = 0 means no extra cap at 100 % override. MP 1090/1092 are jerk limits and a feed threshold (unsure), not feed caps. | Machine file `feedRateMax` = 11000. Z 6000 cannot be stated per axis today; the control clamps it. |
| Rapid speeds | MP 1010 | X 11000, Y 11000, Z 6000, A 7200°/min, W 3600 mm/min | Max speed per axis | The machine file rapids are correct. |
| Software travel limits (range 1) | MP 910.x / MP 920.x | X 0..+500.2, Y 0..+255.2, Z -147.2..0, W -500.2..0 (A ±90000°) | Soft limits from the machine datum (MP 960: X+502.341, Y-2.617, Z-10, W-508.725) | Add per-axis limits to the machine file. Z gives a 147 mm CNC window at a fixed W. |
| Traverse ranges 2/3 | MP 911/912, MP 921/922; MP 7490 bit 0 = 1 | X, Y the same; Z -200..+20 | Three ranges are on (MP 7490). Only Z is wider: for full quill retraction and 2-axis work (owner's notes) | CAM works in range 1. M140 MB MAX stops at the top of the active range (MP 7432 = 0 tolerance). |
| M92 / M91 positions | MP 950.x; MP 4210.0 | M92 datum X+120, Y+210, Z-0.05; PLC "Z-Achse Werkzeugwechselpunkt" = -0.05 | `L Z+0 R0 FMAX M92` goes to the PLC's tool-change Z | Write `L Z+0 R0 FMAX M92` for the change and the program end (defects 7, 8). MP 7340.3/.4 let the shop change the X/Y of MP 950 (user parameters). |
| M6 behaviour | MP 7440 bit 0 | 0 | The program **halts at M06** | M6 is a valid stop. It confirms defect 1. M0 vs M6: both stop (defect 4 re-rated to P7). |
| M89 behaviour | MP 7440 bit 1 | 1 | M89 is a modal cycle call at the block end | Only matters if a post uses M89. Both posts use M99 (OK). |
| M103 | MP 7440 bit 4 | 1 | M103 (reduced plunge feed) is active | Optional use. |
| Wait for M acknowledgement | MP 7440 bit 2 | 0 | The program waits for the PLC to confirm each M function | No change. |
| Mid-program start (block scan) | MP 7680 bits 2-5 | bit 3 = 1, bit 5 = 1 | A block scan ignores M00/M06 stops and computes from the program start | Operator note in the posts' tool-change comments: after a mid-program start, the tool in the spindle must already be the right one. |
| Tool table | MP 7260; MP 7262; MP 7483; MP 7266 | 254 lines; index up to 2; names and numbers allowed; columns include LCUTS, ANGLE, TYP, NMAX, PITCH | The table is on (MP 7340.6: on = 254, off = 0) | Tool numbers 1..254 in both posts (defect 10). A TOOL.T export can use these columns. |
| Tool number at TOOL CALL | MP 7480.0 | 1 | The T number goes to the PLC only when it changes | TOOL CALL is not a stop (defect 1). |
| Spindle range | MP 3510.0, MP 3515.0, MP 3210.0, MP 3240.1 | 9000 nominal and max, gear 1 only (2-8 = 0); min about 43 rpm (0.043 x 1000, unsure) | One gear range, 0..9000 rpm | The machine file's 50..9000 is correct and a little conservative. No gear-range words needed. |
| S0 | MP 3120 | 1 | `S0` is not allowed | Both posts never write S0 (verified). |
| Spindle override | MP 3310 | 70..130 % | | Operator only. |
| Spindle orientation | MP 7442 | 19 | Cycles orient with M19 | No change. |
| Rigid tapping | MP 7160 | %00100 (bit 2 = 1) | The spindle runs position-controlled in cycles 17/207/18 | Set `rigidTapping: true`. G207/CYCL DEF 207 is correct for this machine. |
| Tapping override | MP 7110 | 90..110 % | Feed override range during tapping | No change. |
| Depth sign check | MP 7441 bit 2 | 1 | A positive depth in Heidenhain cycles gives an error | Both posts write negative depths (OK). |
| DIN/ISO availability | MP 7224.0/.1/.2; MP 7220 | all 0; 10 | No file type is locked, so `.I` files are allowed. DIN block numbers step by 10. | DIN/ISO can load on this control. The open point is only the word set of 340490-03 (defects 2, 3). |
| Dialog language | MP 7230 | 1 (German) | Cycle comments appear in German | No effect on import (the control re-adds comments). |
| Active axes | MP 10; MP 100; MP 410 | 23 = binary 10111 = X, Y, Z, W; A (axis 4) prepared but off | MP 7340.0 user parameter: "A-Achse EIN = 31 / AUS = 23" | No A axis in the machine file (correct). Keep refusing rotary moves. |
| Dividing attachment switch | MP header variants AT100Vs (standard), AT100uA, AT125Vs, AT125uA; MP 331.3 | encoder variants only | The parameter set is ready for AT100/AT125 | Only if fitted (it is not: no `ACHSE_4` drive file). |
| Coolant / aux | MP 4310.0 bit 8; MP 4310.4 (user parameter MP 7340.1) | 0; 0 | "Coolant 2 separate" is off; coolant is coupled to M03, not modal (unsure) | Keep M8 after M3 (both posts do this). The MP file does not show whether M7 or M37 hardware is fitted. |
| OEM cycles | LST file | `PLC:\OEMCY1\DES\CYCDES300..329.DES` | The Fehlmann cycles 300-329 are installed | Not needed by CamCraft. See the warning in section 3. |
| Preset table | `OEM.SYS` PRESETTABLE = ON; `PRESET.PR` | rows 0..5 (2008 snapshot) | Cycle 247 selects a row | `workOffsetMax = 6` may exceed the table (defect 14). |
| Look-ahead | MP 7400 | 1 (512 blocks, needs software option 2) | | No change. |
| Graphics | MP 7310 | %00001100 (3D allowed, blank shift allowed) | | DIN/ISO should write G30/G31 (defect 6). |
| Program-check length | MP 7229.0 | 9999 blocks | (unsure) the control's program check covers up to 9999 blocks | Note for long 3D programs. |

---

## 4. Machine file check (`assets/machines/fehlmann-picomax55.mch`), revised with MP values

| Value in file | Line | Real value (MP) | Verdict |
|---|---|---|---|
| model `Fehlmann Picomax 55 (iTNC 530)`, kind `mill` | 3, 5 | P55CNC with iTNC 530 (MP header) | OK |
| X, Y, Z linear; head "Quill" on Z | 8-22 | MP 10 = X, Y, Z, W active. Z = quill. W = head, never programmed. | OK. W is not modelled, which is correct, because it is set by hand per job. |
| spindle 50-9000 rpm | 18 | MP 3515.0 = 9000, single gear; min about 43 rpm (unsure) | **OK** (50 is conservative) |
| coolant: Flood M8 only | 19 | M8 = coolant 1. MP 4310.0 bit 8 (coolant 2 separate) = 0. M7/M37 hardware unknown. | OK. Add M7/M37 only if fitted. |
| controller identity iTNC 530 / 340490-03 | 26 | "ID 340490 03" (MP header) | **OK** |
| controller post `Heidenhain Klartext` | 25 | All manual examples are Klartext. DIN/ISO is also allowed (MP 7224). | OK |
| rapid X/Y 11000, Z 6000 mm/min | 28-30 | MP 1010: 11000 / 11000 / 6000 | **OK** |
| tool change `manual-retract-top` | 33 | M06 halts (MP 7440). Change Z = M92 Z+0 = MP 950.2 / MP 4210.0 = -0.05. The physical full retraction is above range 1 (MP 911/912 Z +20; owner's notes). | Consistent. **Add** the M92 change position (Z) and an XY park. |
| travel limits: "unstated" | 6 | MP 910/920: X 0..500.2, Y 0..255.2, Z -147.2..0 (range 1), from the datum MP 960 | **Missing:** add per-axis limits |
| — `feedRateMax` | — | 11000 (MP 1010 X/Y; Z 6000) | **Missing** |
| — `rigidTapping` | — | true (MP 7160 bit 2) | **Missing** |
| — tool-number limit | — | 254 (MP 7260) | **Missing** (no field today; enforce in the posts) |
| — preset rows | — | 1..5 in the 2008 `PRESET.PR` | Check at the control (defect 14) |
| — A axis | — | prepared, inactive (MP 10 = 23) | OK to leave out |

---

## 5. DIN/ISO vs Klartext

- **What the manual and the control expect:** Klartext. The whole manual is conversational, and the machine file names Klartext. **DIN/ISO is allowed** on this control: MP 7224 locks no file type, and MP 7220 sets the DIN block-number step (10). The open point is only which words the 340490-03 software accepts (defects 2, 3).

| Area | DIN/ISO (`heidenhain.cps`) | Klartext (`heidenhain-klartext.cps`) |
|---|---|---|
| Program frame | `%name G71 *` … `N99999999 %name G71 *` | `BEGIN PGM … MM` … `END PGM` |
| Blank form | none | BLK FORM 0.1/0.2 |
| Machine/control guard | none (any 3-axis machine) | strict: Picomax, iTNC 530, 340490-03, XYZ only, manual change |
| Tool change | T block only, no stop (defect 1) | M0 with operator checks, M140 MB MAX, checks |
| Tool numbers | no limit | 1..32767 (control: 254) |
| Preset | `G247 Q339=n` (1-6) | `CYCL DEF 247` Q339 (1-6) |
| Drilling/tapping | G200, G205, G206, G207, and expanded ream/back-bore | refused |
| Arcs | XY with absolute I/J; XZ/YZ as short lines; helix refused | XY `CC`/`C` only, with a 0.0001 mm check (control: 0.016) |
| Cutter compensation | refused | refused |
| Coolant | flood/mist/air/through, from the machine list | flood only |
| Stops between ops, setup change | yes | refused |
| Smoothing | `G62 T..` | refused |
| Probing, inch, rotary | refused | refused |
| Tested on a control | station 606425-04 (#3095) | not yet on a control |

---

## 6. Verified OK (matches the manual or the MP file)

- Klartext program frame, `;` comments, BLK FORM order (MIN absolute, then MAX) — p. 2-4, 2-15.
- `TOOL CALL n Z S..` with M3 in its own block after the call — p. 2-28.
- `L X.. Y.. R0 FMAX`, then the Z move down, as in the examples — p. 2-4.
- Klartext arcs: `CC X.. Y..` (absolute centre), then `C X.. Y.. DR±`, DR- = clockwise — p. 3-22.
- M3/M5/M8/M9, M0, M2 (M30 means the same as M02) — p. 1-17. M137 and M140 are in the M list — p. 1-20.
- Cycle calls with M99 one hole at a time — p. 2-31. Not M89, whose meaning this machine changes (MP 7440 bit 1).
- Rigid tapping cycle 207 is right for this machine — p. 2-29, MP 7160 bit 2.
- Negative cycle depths (MP 7441 bit 2) and no `S0` (MP 3120) — both posts.
- M8 written after M3 suits the coolant coupled to M03 (MP 4310.4 = 0, unsure).
- Machine file: spindle 9000, rapids 11000/11000/6000, iTNC 530 340490-03, manual change, flood M8, no A axis — all match the MP file.

---

## 7. Suggested issues (at review time; see the README for the issues filed since)

**Safety (do first)**
1. `DIN/ISO post: no stop or M6 at a tool change on a manual-change machine (Picomax)`. **P1**, effort:high. Refuse when `toolChangeMode` is manual or manual-retract-top, or write `L Z+0 R0 FMAX M92` (`G00 G90 Z+0 M92`) and M6 (a halt by MP 7440). Also add an engine law: a post that ignores `toolChangeMode` is refused on a manual-change machine.

**Control compatibility**
2. `DIN/ISO post: FUNCTION DWELL TIME is not iTNC 530 syntax; write G04 F`. P3, effort:medium.
3. `DIN/ISO post: Q395/Q379/Q253 in G200/G205 on NC 340490-03`. Comment on #3095 first. If the control refuses them, P3, effort:medium.
4. `Klartext post: arc check is 0.0001 mm, control MP7431 is 0.016 mm - correct arcs refused`. **P3**, effort:medium (confirmed).
5. `Heidenhain posts: tool numbers above 254 pass, Picomax tool table has 254 lines (MP7260)`. **P3**, effort:low. Needs a machine-file field.
6. `Heidenhain posts: preset 6 may not exist on the Picomax (PRESET.PR rows 1..5)`. P3, effort:low, after a check at the control.

**Fehlmann workflow**
7. `Klartext/DIN-ISO: Fehlmann tool change and program end (M92 Z+0 = MP950, XY park, M6 or M0)`. P6, effort:medium.
8. `Machine file: Picomax travel limits, feedRateMax 11000, rigidTapping, M92 change position, tool-table size (from the MP file)`. P7, effort:low.

**Features**
9. `Klartext post: drilling and tapping cycles 200/205/206/207 and drill tool kinds`. P6, effort:high.
10. `Heidenhain posts: helical arcs (Klartext CP IPA IZ; DIN/ISO short lines)`. P6, effort:high.
11. `DIN/ISO post: write the G30/G31 blank form`. P6, effort:low.
12. `Klartext post: stops between operations, several setups, smoothing (CYCL DEF 32)`. P6, effort:medium.
13. `Klartext post: M7 / M37 from the machine file` (only if fitted). P7, effort:low.
14. `Klartext post: let the 606425 programming station load test output`. P7, effort:low.
15. `Heidenhain posts: control radius compensation RL/RR with APPR/DEP`. P8, effort:max.
16. Polish: BLK FORM 30000 mm limit, DIN/ISO minimum rpm, operator note about block scan ignoring M06 (MP 7680 bit 3). P7, effort:low.
- Dropped: the AT 100/125 dividing attachment. It is not fitted (MP 10 = 23, no axis-4 drive file).

---

## 8. Open questions for the machine owner (only what the MP data cannot answer)

1. **M6 vs M0 at the tool change.** Both halt the program (MP 7440). Does the tool-release button or a lamp work only after M6? Or is M0, with its drive-off, the preferred state? Which XY park position is used (the manual uses `X-50 Y-50`)?
2. **Fitted options the MP file cannot show:** coolant 2 (M7), compressed air (M37), a touch probe, a tool setter (the MP file has a TT/Blum set-up). Also: which software options does MOD → SIK list?
3. **3-axis mode:** should a CamCraft program write **M43** at its start, or keep the "CONFIRM XYZ MODE" comment? What does MP 4210.14 = 4 do at power-on?
4. **Preset table:** how many rows does the preset table on the control have now (the 2008 snapshot has 0..5)?
5. **Real-machine run:** load the DIN/ISO drill files with Q395/Q379/Q253 and one `G04` dwell on the Fehlmann (defects 2, 3). Does the control accept them?
