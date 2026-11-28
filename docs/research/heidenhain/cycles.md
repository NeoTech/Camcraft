# HEIDENHAIN cycle numbers: names, purposes and versions

This table lists every HEIDENHAIN cycle number that occurs in these notes (`differences.md`, `differences-by-version.md`, `dialects.md` and the script output, which is not published), plus the touch probe cycles of the same families. Use it to read a bare number such as "205" as "205 UNIVERSAL PECKING".

## How this table was made

- **Names** come from the cycle headings and cycle overview tables of the HEIDENHAIN English user manuals. A script searched the extracted manual text for the form `Cycle <n> <NAME> (ISO: G<n>)` and for the section headings of each cycle. No manual was read in full.
- The **name** column gives the short name of the newest manual (TNC7 81762x-20 where the cycle exists there), then older or long forms in parentheses. The control shows the short form in the `CYCL DEF` block.
- The **purpose** column is written in our own words. It is not HEIDENHAIN text.
- **DIN/ISO** is the G code of the cycle in DIN/ISO programs. "(none found)" means that no G code was found in the extracted text.
- **Option** is the software option number that the TNC 620 (`620: #19`), TNC 640 or TNC7 manuals name for the cycle. On the iTNC 530, TNC 320 and TNC 640, option 19 functions are standard. Turning cycles need the turning option (#50) on the TNC 640 and TNC7.
- **In example blocks** comes from `CYCL DEF <n>` lines in the manuals of each version (script output, the cycle list per version). "320 340 551-01 on" means: seen from 340 551-01 to the newest manual of that control. "A to B" means: seen from A to B, not in the newer manuals. "only" means: seen in one version only. The list shows where an example block exists. It does not prove that a version without an example block lacks the cycle. Gaps inside a range are not shown.
- **Introduced / changed / removed** gives the facts of sections B2, B3 and C of `differences.md`. "C names it: ..." lists the versions whose new-function or changed-function list names the cycle (generated; read the bullet in section C for the detail).
- **Reference** is the HEIDENHAIN manual ID and the PDF page (not the printed page) of the cycle heading or of the contents line that was used. `inventory.md` maps each ID to the control, the NC software, the title and the download URL.

Control abbreviations: 128 = TNC 128, 310 = TNC 310, 320 = TNC 320, 410 = TNC 410, 426/430 = TNC 426 / TNC 430, iTNC = iTNC 530, 620 = TNC 620, 640 = TNC 640, TNC7 = TNC7.

Cycle families in short:

- **1-5, 17, 18**: old fixed cycles with `CYCL DEF n.0` ... `n.x` blocks and no Q parameters.
- **6, 14, 15, 16**: SL I contour cycles (TNC 410 era). **14, 20-25, 270, 271-277**: SL II and OCM contour cycles.
- **7, 8, 10, 11, 26, 19, 247**: coordinate transformations and presets. **9, 12, 13, 32**: special cycles.
- **200-299**: Q-parameter machining cycles. **800-899**: turning cycles. **1000-1099**: grinding cycles. **1271-1282**: OCM standard figures.
- **TCH PROBE 0-4, 400-499, 1400-1499**: touch probe cycles. **600/601**: camera monitoring cycles. These run as `TCH PROBE n` blocks, not `CYCL DEF n`.

## Numbers that are not cycles

The script list of cycle numbers per version also contains numbers that are not HEIDENHAIN cycles. They are extraction artefacts:

- **140**: `CYCL DEF 14.0 CONTOUR GEOMETRY` read without the point (cycle 14).
- **42, 56, 60, 66, 68, 70, 72, 84, 94, 242**: block numbers or page numbers next to a `CYCL DEF` line. No cycle with these numbers was found in the manuals.
- **G122, G124, G125** and similar in section C are the DIN/ISO codes of cycles 22, 24 and 25.
- The TNC 406 / TNC 416 numbers (1, 3, 7, 8, 10-12, 14, 16, 17, 19 in the script list) are EDM cycles. They are out of scope and are not explained here.

## Names marked unsure

- **202**: the TNC7 81762x-20 manual heading reads "Cycle 202 REAMING" (1358775-24 p207). All other manuals, the G code and the cycle text give BORING. We use BORING.
- **1051 STEP. CYLIND. GRIND** and **1040 END CYLIND. GRINDING**: only the short form was found; the long name and the exact purpose are unsure.
- **18 THREAD CUTTING**: one TNC 640 manual prints the ISO code as G18 (1303406-20). G18 is the plane code; we use G86.
- Options marked "(unsure)" were not found next to the cycle heading.

## Old fixed cycles, SL I, SL II, coordinate transformations and special cycles (1-39)

| No. | HEIDENHAIN name | Purpose (own words) | DIN/ISO | Option | In example blocks: first → last per control | Introduced / changed / removed | Reference (manual ID, PDF page) |
|---|---|---|---|---|---|---|---|
| 1 | PECKING | Old peck-drilling cycle (CYCL DEF 1.0 to 1.x blocks, no Q parameters). | G83 | - | 310 286 140 on; 410; 426/430 280 462 on; iTNC 340 420 only | iTNC 340 422: removed from the soft keys, old programs still run (375 738-24 p10). 620 81760x-05 / 640 34059x-08: OLD CYCLS soft key removed, cycle can still be edited and run (1322094_02 p63; 1322095_03 p76). | 309 740-25 p10; 1322094_02 p63 |
| 2 | TAPPING | Old tapping cycle for a floating tap holder. | G84 | - | 410; 426/430 280 472 on; iTNC 340 420 only | iTNC 340 422: removed from the soft keys, old programs still run (375 738-24 p10). 620 81760x-05 / 640 34059x-08: OLD CYCLS soft key removed, cycle can still be edited and run (1322094_02 p63; 1322095_03 p76). | 309 740-25 p10; 1322094_02 p63 |
| 3 | SLOT MILLING | Old slot-milling cycle. | G74 | - | 410; 426/430 280 472 on; iTNC 340 420 only | iTNC 340 422: removed from the soft keys, old programs still run (375 738-24 p10). 620 81760x-05 / 640 34059x-08: OLD CYCLS soft key removed, cycle can still be edited and run (1322094_02 p63; 1322095_03 p76). | 309 740-25 p10; 1322094_02 p63 |
| 4 | POCKET MILLING | Old rectangular pocket roughing cycle. | G75/G76 | - | 320 340 551-01 to 340 55x-03; 410; 426/430 280 472 on; iTNC 340 420 only; 620 340 56x-01 only | iTNC 340 422: removed from the soft keys, old programs still run (375 738-24 p10). 620 81760x-05 / 640 34059x-08: OLD CYCLS soft key removed, cycle can still be edited and run (1322094_02 p63; 1322095_03 p76). | 309 740-25 p10; 1322094_02 p63 |
| 5 | CIRCULAR POCKET | Old circular pocket roughing cycle. On the TNC 310 one of the two old cycles. | G77/G78 | - | 310 286 040 on; 320 340 551-01 to 340 55x-03; 410; 426/430 280 462 on; iTNC 340 420 only; 620 340 56x-01 only | iTNC 340 422: removed from the soft keys, old programs still run (375 738-24 p10). 620 81760x-05 / 640 34059x-08: OLD CYCLS soft key removed, cycle can still be edited and run (1322094_02 p63; 1322095_03 p76). | 309 740-25 p10; 1322094_02 p63 |
| 6 | ROUGH-OUT | SL I contour pocket roughing (old contour-cycle family, replaced by SL II cycles 20-24). | G57 | - | 410 | TNC 410 (309 740-25 contents). Later controls: not found in the cycle lists. | 309 740-25 p11; ISO 345 940-22 |
| 7 | DATUM SHIFT | Shifts the workpiece datum by values or by a row of a datum table. | G54 | - | 128 77184x-01 on; 310 286 040 on; 320 340 551-01 on; 410; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-16 to 81762x-18 | All contouring controls. TNC7 from 81762x-17: changed into TRANS DATUM when edited ("The TRANS DATUM function replaces Cycle 7", 1358773-24). TRANS DATUM on 620 340 56x-03, 320 340 55x-05. C names it: iTNC 340422-xx. | 1303406-23 p241 |
| 8 | MIRRORING (older: MIRROR IMAGE) | Mirrors the contour about one or more axes. | G28 | - | 128 77184x-01 on; 320 340 551-01 on; 410; 426/430 280 472 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | All contouring controls. TRANS MIRROR as alternative: 640 34059x-16, 620 81760x-16. C names it: 620 81760x-03, 81760x-04; 640 34059x-06, 34059x-07. | 1358775-24 p777 |
| 9 | DWELL TIME | Stops the program run for a set time (seconds). | G04 | - | 128 77184x-01 on; 320 340 551-01 on; 410; 426/430 280 472 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | All contouring controls. `FUNCTION DWELL` as alternative: 640 34059x-07, 620 81760x-04, 320 77185x-04, TNC7; never on the iTNC 530. | 1358775-24 p807 |
| 10 | ROTATION | Rotates the coordinate system in the working plane. | G73 | - | 310 286 040 on; 320 340 551-01 on; 410; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | All contouring controls. TRANS ROTATION as alternative: 640 34059x-16, 620 81760x-16. C names it: 620 81760x-04, 81760x-16; 640 34059x-07, 34059x-16. | 1358775-24 p779 |
| 11 | SCALING FACTOR (also SCALING) | Scales the contour by one factor in all axes of the plane. | G72 | - | 128 77184x-01 on; 320 340 551-01 on; 410; 426/430 280 472 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p781 |
| 12 | PGM CALL (also PROGRAM CALL) | Makes another NC program callable like a cycle (CYCL CALL / M99). | G39 | - | 128 77184x-01 on; 310 286 040 on; 320 340 551-01 on; 410; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-16 on | C names it: 620 81760x-16; 640 34059x-16. | 1358775-24 p110 |
| 13 | ORIENTATION (older: ORIENTED SPINDLE STOP) | Stops the spindle at a defined angle (for example for back boring or a tool change). | G36 | - | 128 77184x-01 on; 320 340 551-01 on; 410; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-03 on; 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p808 |
| 14 | CONTOUR (older: CONTOUR GEOMETRY) | Lists the subprograms (labels) that make up an SL contour. | G37 | - | 320 340 551-01 on; 410; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-16 on | C names it: 620 81760x-05; 640 34059x-08. | 1358775-24 p118 |
| 15 | PILOT DRILLING | SL I pre-drilling of the plunge points for cycle 6. | G56 | - | 410 | TNC 410 (309 740-25 contents). Later controls: not found. | 309 740-25 p11; ISO 345 940-22 |
| 16 | CONTOUR MILLING | SL I finishing along the contour of cycle 14. | G58/G59 | - | 410 | TNC 410 (309 740-25 contents). Later controls: not found. | 309 740-25 p11; ISO 345 940-22 |
| 17 | RIGID TAPPING GS | Old tapping cycle without a floating holder (spindle-synchronised). Replaced by 207. | G85 | - | 410; 426/430 280 472 on; iTNC 340 420 only | iTNC 340 422: removed from the soft keys, old programs still run (375 738-24 p10). 620 81760x-05 / 640 34059x-08: OLD CYCLS soft key removed, cycle can still be edited and run (1322094_02 p63; 1322095_03 p76). | 309 740-25 p10; 1322094_02 p63 |
| 18 | THREAD CUTTING | Spindle-synchronised thread cutting along the tool axis with a turning-style tool. | G86 | - | 128 77184x-06 only; 320 77185x-05 on; 426/430 280 462 on; iTNC 340 420 only; 620 81760x-05 on; 640 34059x-08 on; TNC7 81762x-17 on | iTNC 340 422 removed it with the old cycles (375 738-24 p10); 620 from 81760x-05 and 640 from 34059x-08 list it again as a current cycle (example blocks). One TNC 640 manual prints "G18" (1303406-20): typo, G18 is the plane code. C names it: iTNC 340422-xx. | 1358775-24 p243 |
| 19 | WORKING PLANE | Tilts the working plane by axis angles (the pre-PLANE tilting method). | G80 | 620/640: #8 | 320 340 55x-03 on; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on | TNC 426 280 462 on, iTNC all, 320 from 340 55x-03, 620, 640. Not TNC 410 (345 940-22 p19: "Cycle G80, not TNC 410"). TNC7 from 81762x-17: can be edited and run, but not inserted as a new element (1373081_03 p23). C names it: 320 34055x-03; TNC7 81762x-17. | 1303406-23 p249 |
| 20 | CONTOUR DATA | Sets the common machining data (depth, allowances, overlap) of the SL II cycles. | G120 | 620: #19 | 320 340 551-01 on; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-16 on | - | 1358775-24 p335 |
| 21 | PILOT DRILLING | SL II pre-drilling of the plunge points for roughing. | G121 | 620: #19 | 320 340 551-01 on; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p337 |
| 22 | ROUGH-OUT (also ROUGHING) | SL II roughing of the contour pocket, with optional rest machining. | G122 | 620: #19 | 320 340 551-01 on; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-16 on | C names it: 320 34055x-04; iTNC 34049x-03, 34049x-04; 620 34056x-02, 81760x-01; 640 34059x-04. | 1358775-24 p339 |
| 23 | FLOOR FINISHING | SL II finishing of the pocket floor. | G123 | 620: #19 | 320 340 551-01 on; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p343 |
| 24 | SIDE FINISHING | SL II finishing of the contour walls. | G124 | 620: #19 | 320 340 551-01 on; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: iTNC 34049x-06, 60642x-01; 620 81760x-06; 640 34059x-09; TNC7 81762x-19; TNC7 basic 81762x-19. | 1358775-24 p346 |
| 25 | CONTOUR TRAIN | Mills along an open or closed contour defined with cycle 14. | G125 | 620: #19 | 320 340 551-02 on; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: 320 34055x-04; iTNC 60642x-03, 60642x-04; 620 34056x-02, 81760x-02; 640 34059x-05. | 1358775-24 p352 |
| 26 | AXIS-SPECIFIC SCALING | Scales each axis by its own factor. | (none found) | - | 128 77184x-01 on; 320 340 551-01 on; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p783 |
| 27 | CYLINDER SURFACE | Machines a contour unrolled on a cylinder surface with a rotary axis. | G127 | 620/640/TNC7: #8 | 320 340 551-02 on; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: iTNC 34049x-05. | 1358775-24 p823 |
| 28 | CYLINDRICAL SURFACE SLOT | Mills a guide slot on a cylinder surface, with wall correction. | G128 | 620/640/TNC7: #8 | 320 340 551-02 on; 426/430 280 474 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: iTNC 34049x-05. | 1358775-24 p826 |
| 29 | CYL SURFACE RIDGE | Mills a ridge on a cylinder surface. | G129 | 620/640/TNC7: #8 | 320 340 551-02 on; iTNC 340 422 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: iTNC 34049x-05. | 1358775-24 p830 |
| 30 | RUN 3-D DATA | Runs a point-to-point 3-D program from an external CAM system in several passes. | G60 | - | 426/430 280 472 on; iTNC 340 420 on | Not found in the 320/620/640/TNC7 cycle lists. | 670388-24 p27 |
| 32 | TOLERANCE | Sets the path tolerance (and optional rotary tolerance) for smoothing at high feed. | G62 | - | 320 340 55x-03 on; 426/430 280 472 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | B2: TNC 426/430 280 472 on, iTNC all, 320 from 340 55x-03, 620, 640, TNC7. Not TNC 410 (345 940-22 p19). C names it: 320 34055x-04; iTNC 34049x-03; 620 34056x-02; TNC7 81762x-19; TNC7 basic 81762x-19. | 1358775-24 p810 |
| 39 | CYL. SURFACE CONTOUR | Mills an outside contour on a cylinder surface. | G139 | 620/640/TNC7: #8 | 320 77185x-02 on; iTNC 340 422 on; 620 81760x-02 on; 640 34059x-05 on; TNC7 81762x-17 on | C names it: iTNC 34049x-05, 34049x-06, 60642x-01; 620 81760x-02; 640 34059x-05. | 1358775-24 p834 |

## Drilling, tapping, thread, pocket, pattern, face, OCM and interpolation-turning cycles (200-299)

| No. | HEIDENHAIN name | Purpose (own words) | DIN/ISO | Option | In example blocks: first → last per control | Introduced / changed / removed | Reference (manual ID, PDF page) |
|---|---|---|---|---|---|---|---|
| 200 | DRILLING | Single drilling cycle with optional pecking and dwell. | G200 | - | 128 77184x-01 on; 310 286 040 on; 320 340 551-01 on; 410; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-16 on | Q211 from TNC 426/430 280 474; Q395 (depth reference) from iTNC 60642x-04, 640 34059x-04, 620 81760x-01, 320 77185x-01, TNC 128 77184x-02. C names it: 128 77184x-05; 320 77185x-01, 77185x-04; iTNC 60642x-03, 60642x-04; 620 81760x-03. | 1358775-24 p201 |
| 201 | REAMING | Reaming with feed-in and feed-out. | G201 | 620: #19 | 128 77184x-01 on; 310 286 040 on; 320 340 551-01 on; 410; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p205 |
| 202 | BORING | Fine boring with oriented spindle stop and retract from the wall. | G202 | 620: #19 | 128 77184x-01 on; 320 340 551-01 on; 410; 426/430 280 472 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | TNC 410/426 on. The TNC7 81762x-20 manual heading prints "Cycle 202 REAMING" (1358775-24 p207); the text and G202 show BORING. C names it: iTNC 60642x-01; 620 81760x-03, 81760x-08; 640 34059x-06, 34059x-11. | 1358775-24 p207 |
| 203 | UNIVERSAL DRILLING | Drilling with decreasing pecking depth, chip breaking and dwell. | G203 | 620: #19 | 128 77184x-01 on; 320 340 551-01 on; 410; 426/430 280 472 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | Q395 from the same releases as cycle 200 (B2). C names it: 128 77184x-05; 320 77185x-01, 77185x-04; iTNC 60642x-03, 60642x-04; 620 81760x-03. | 1358775-24 p211 |
| 204 | BACK BORING | Counterbores from the underside of the workpiece (oriented spindle, eccentric approach). | G204 | 620: #19 | 128 77184x-01 on; 320 340 551-01 on; 410; 426/430 280 472 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: 620 81760x-03, 81760x-08; 640 34059x-06, 34059x-11; TNC7 81762x-20; TNC7 basic 81762x-20. | 1358775-24 p236 |
| 205 | UNIVERSAL PECKING | Deep-hole pecking with chip breaking, decreasing infeed and a deepened start point. | G205 | 620: #19 | 128 77184x-01 on; 320 340 551-01 on; 426/430 280 474 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | Q379/Q253 from iTNC 340 422; Q208/Q395 from iTNC 60642x-04, 640 34059x-04, 620 81760x-01, 320 77185x-01 (1096959-20 p9); Q373 from 640 34059x-16, 620 81760x-16, 320 771851-18. TNC 426/430 from 280 474 (318 179-24 p11). Not TNC 410 (345 940-22 p18) (B2, B3). C names it: 128 77184x-05; 320 77185x-01, 77185x-04; iTNC 340422-xx, 60642x-03, 60642x-04. | 1358775-24 p216 |
| 206 | TAPPING (older: TAPPING NEW) | Tapping with a floating tap holder (Q-parameter successor of cycle 2). | G206 | - | 128 77184x-01 on; 320 340 551-01 on; 426/430 280 474 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | TNC 426/430 from 280 474. Not TNC 410 (345 940-22 p18). C names it: iTNC 60642x-01. | 1358775-24 p246 |
| 207 | RIGID TAPPING (older: RIGID TAPPING NEW) | Tapping without a floating holder, spindle-synchronised (successor of cycle 17). | G207 | - | 128 77184x-01 on; 320 340 551-01 on; 426/430 280 474 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | TNC 426/430 from 280 474 (318 179-24 p11). Not TNC 410 (345 940-22 p18). Same Q list in every B3 row. C names it: iTNC 60642x-01. | 1358775-24 p249 |
| 208 | BORE MILLING | Mills a hole on a helix. | G208 | 620: #19 | 320 340 551-01 on; 426/430 280 474 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | TNC 426/430 from 280 474. Not TNC 410 (345 940-22 p18). C names it: 320 34055x-03; iTNC 34049x-03; TNC7 81762x-18. | 1358775-24 p223 |
| 209 | TAPPING W/ CHIP BRKG (TAPPING WITH CHIP BREAKING) | Rigid tapping with chip breaking steps. | G209 | 620: #19 | 320 340 551-01 on; 426/430 280 476 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | TNC 426/430 280 476 on. Not TNC 410 (345 940-22 p6, p18). Q403 from iTNC 34049x-04 (B3). C names it: 320 34055x-03; 426/430 280476-xx; iTNC 34049x-04, 60642x-01. | 1358775-24 p253 |
| 210 | SLOT RECIP. PLNG | Slot milling with reciprocating plunge. | G210 | - | 320 340 551-01 to 340 55x-03; 410; 426/430 280 472 on; iTNC 340 420 to 340 49x-03; 620 340 56x-01 only | TNC 410, 426/430 280 472 on. Removed from the soft keys: iTNC 34049x-04, 320 340 55x-04, 620 340 56x-02 (still run); replaced by 253. 620 81760x-05 / 640 34059x-08: OLD CYCLS removed. C names it: 320 34055x-04; iTNC 340420-xx, 34049x-04; 620 34056x-02, 81760x-05; 640 34059x-08. | 309 740-25 p10; 1322094_02 p63 |
| 211 | CIRCULAR SLOT | Circular slot with reciprocating plunge. | G211 | - | 310 286 040 on; 320 340 551-01 to 340 55x-03; 410; 426/430 280 462 on; iTNC 340 420 to 340 49x-03; 620 340 56x-01 only | As 210; replaced by 254. C names it: iTNC 340420-xx. | 309 740-25 p10; 1322094_02 p63 |
| 212 | POCKET FINISHING | Finishes a rectangular pocket. | G212 | - | 320 340 551-01 to 340 551-02; 410; 426/430 280 472 on; iTNC 340 420 to 340 49x-03; 620 340 56x-01 only | As 210; replaced by 251. | 309 740-25 p10; 1322094_02 p63 |
| 213 | STUD FINISHING | Finishes a rectangular stud. | G213 | - | 310 286 040 on; 320 340 551-01 to 77185x-06; 410; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 to 81760x-07; 640 34059x-01 to 34059x-10 | As 210; replaced by 256. | 309 740-25 p10; 1322094_02 p63 |
| 214 | C. POCKET FINISHING (CIRCULAR POCKET FINISHING) | Finishes a circular pocket. | G214 | - | 410; 426/430 280 472 on; iTNC 340 420 only | As 210; replaced by 252. C names it: 320 34055x-04; iTNC 34049x-04; 620 34056x-02. | 309 740-25 p10; 1322094_02 p63 |
| 215 | C. STUD FINISHING (CIRCULAR STUD FINISHING) | Finishes a circular stud. | G215 | - | iTNC 340 420 only | As 210; replaced by 257. | 309 740-25 p10; 1322094_02 p63 |
| 220 | POLAR PATTERN (older: CIRCULAR PATTERN) | Calls the active machining cycle on points of a full or partial circle. | G220 | 620: #19 | 128 77184x-02 on; 310 286 040 on; 320 340 551-01 on; 410; 426/430 280 462 on; iTNC 340 420 on; 620 34056x-04 on; 640 34059x-02 on; TNC7 81762x-17 on | C names it: iTNC 340422-xx; 620 81760x-18; 640 34059x-18; TNC7 81762x-18. | 1358775-24 p148 |
| 221 | CARTESIAN PATTERN (older: LINEAR PATTERN) | Calls the active machining cycle on a grid of points. | G221 | 620: #19 | 128 77184x-01 on; 320 340 551-01 on; 410; 426/430 280 472 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: 620 81760x-18; 640 34059x-18. | 1358775-24 p152 |
| 224 | DATAMATRIX CODE PATTERN | Places the active cycle on the dots of a DataMatrix code built from a text. | G224 | 620: #19 | 320 771851-18 on; 620 81760x-07 on; 640 34059x-10 on; TNC7 81762x-17 on | C names it: 320 77185x-18; 620 81760x-07; 640 34059x-10. | 1358775-24 p155 |
| 225 | ENGRAVING | Engraves text along a line or an arc. | G225 | - | 320 34055x-06 on; iTNC 340 49x-07 on; 620 34056x-04 on; 640 34059x-02 on; TNC7 81762x-17 on | C names it: 320 77185x-01, 77185x-04; iTNC 34049x-07, 60642x-02, 60642x-03, 60642x-04; 620 73498x-02, 81760x-01, 81760x-02, 81760x-04, 81760x-05, 81760x-07, 81760x-08, 81760x-16; 640 34059x-02, 34059x-04, 34059x-05, 34059x-07, 34059x-08, 34059x-10, 34059x-11, 34059x-16; TNC7 81762x-18. | 1358775-24 p480 |
| 230 | MULTIPASS MILLING | Faces a rectangular area in several passes. | G230 | - | 310 286 040 on; 320 340 551-01 to 77185x-01; 410; 426/430 280 462 on; iTNC 340 420 on; 620 340 56x-01 to 81760x-01; 640 34059x-01 to 34059x-04 | TNC 310/410/426 on. 620 81760x-05 / 640 34059x-08: OLD CYCLS removed (still run); replaced by 233. | 309 740-25 p11; 1322094_02 p63 |
| 231 | RULED SURFACE | Mills a ruled surface between four corner points. | G231 | - | 320 340 551-01 to 77185x-01; 410; 426/430 280 472 on; iTNC 340 420 on; 620 340 56x-01 to 81760x-01; 640 34059x-01 to 34059x-04 | As 230. | 309 740-25 p11; 1322094_02 p63 |
| 232 | FACE MILLING | Faces a rectangular area in several strategies (older face cycle). | G232 | 620: #19 | 320 340 551-01 on; iTNC 340 422 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p443 |
| 233 | FACE MILLING | Faces an area with boundaries and strategies (successor of 230/232). | G233 | 620: #19 | 128 77184x-02 on; 320 77185x-01 on; 620 81760x-01 on; 640 34059x-04 on; TNC7 81762x-17 on | C names it: 320 77185x-01; 620 81760x-01, 81760x-06, 81760x-07; 640 34059x-04, 34059x-09, 34059x-10. | 1358775-24 p450 |
| 238 | MEASURE MACHINE STATUS | Records machine-component condition for monitoring. | G238 | #155 | 620 81760x-07 on; 640 34059x-10 on; TNC7 81762x-17 on | C names it: 620 81760x-07, 81760x-16; 640 34059x-10, 34059x-16. | 1358775-24 p818 |
| 239 | ASCERTAIN THE LOAD | Weighs the table load to adapt the axis control. | G239 | #143 | 620 81760x-02 on; 640 34059x-05 on; TNC7 81762x-17 on | C names it: 640 34059x-08. | 1358775-24 p815 |
| 240 | CENTERING | Spot drilling (centering) to a diameter or a depth. | G240 | 620: #19 | 128 77184x-01 on; 320 340 55x-03 on; iTNC 340 49x-01 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: 320 34055x-03. | 1358775-24 p240 |
| 241 | SINGLE-LIP D.H.DRLNG (SINGLE-LIP DEEP-HOLE DRILLING) | Deep-hole drilling with a single-lip (gun) drill. | G241 | 620: #19 | 128 77184x-01 on; 320 340 55x-04 on; iTNC 340 49x-05 on; 620 340 56x-02 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: 320 34055x-04; iTNC 34049x-05, 34049x-06, 60642x-01; 620 34056x-02, 81760x-16. | 1358775-24 p228 |
| 247 | PRESETTING (older: DATUM SETTING) | Activates a preset from the preset table (on old software: set a datum value). | G247 | - | 128 77184x-01 on; 320 340 55x-03 on; 426/430 280 476 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | Meaning changed in iTNC 340 422: activates a preset-table row (before: datum value) (375 738-24 p10). TNC 426/430 280 476 on, 320 from 340 55x-03. Not TNC 410 (345 940-22 p6, p19). PRESET SELECT as alternative: 640 34059x-11, 620 81760x-08, TNC7. C names it: 128 77184x-05; 320 77185x-04; 426/430 280476-xx; iTNC 340422-xx; 620 81760x-03; 640 34059x-06. | 1358775-24 p784 |
| 251 | RECTANGULAR POCKET | Complete rectangular pocket (roughing and finishing). | G251 | 620: #19 | 128 77184x-01 on; 320 340 55x-04 on; iTNC 340 422 on; 620 340 56x-02 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: 320 34055x-04; iTNC 340422-xx; 620 34056x-02, 81760x-03, 81760x-04; 640 34059x-06. | 1358775-24 p286 |
| 252 | CIRCULAR POCKET | Complete circular pocket. | G252 | 620: #19 | 128 77184x-02 on; 320 340 55x-04 on; iTNC 340 422 on; 620 340 56x-02 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: 620 81760x-02; 640 34059x-05. | 1358775-24 p293 |
| 253 | SLOT MILLING | Complete straight slot. | G253 | 620: #19 | 128 77184x-03 on; 320 340 55x-04 on; iTNC 340 422 on; 620 340 56x-02 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: 128 77184x-03; iTNC 60642x-03, 60642x-04, 60642x-04SP8. | 1358775-24 p299 |
| 254 | CIRCULAR SLOT | Complete circular slot. | G254 | 620: #19 | 320 340 55x-04 on; iTNC 340 422 on; 620 340 56x-02 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: iTNC 60642x-03, 60642x-04, 60642x-04SP8. | 1358775-24 p305 |
| 256 | RECTANGULAR STUD | Complete rectangular stud. | G256 | 620: #19 | 128 77184x-01 on; 320 340 55x-04 on; iTNC 340 49x-04 on; 620 340 56x-02 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: 128 77184x-06; 320 77185x-05; iTNC 34049x-04, 34049x-08, 60642x-03; 620 73498x-02, 81760x-05; 640 34059x-02, 34059x-08. | 1358775-24 p312 |
| 257 | CIRCULAR STUD | Complete circular stud. | G257 | 620: #19 | 320 340 55x-04 on; iTNC 340 49x-04 on; 620 340 56x-02 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: 320 34055x-04; iTNC 34049x-04, 34049x-08; 620 34056x-02, 81760x-05; 640 34059x-08. | 1358775-24 p318 |
| 258 | POLYGON STUD | Complete regular polygon stud. | G258 | 620: #19 | 320 77185x-04 on; 620 81760x-03 on; 640 34059x-06 on; TNC7 81762x-17 on | C names it: 320 77185x-04; 620 81760x-03; 640 34059x-06. | 1358775-24 p323 |
| 262 | THREAD MILLING | Mills an internal thread in a pre-drilled hole. | G262 | 620: #19 | 320 340 551-01 on; 426/430 280 476 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: 426/430 280476-xx. | 1358775-24 p259 |
| 263 | THREAD MLLNG/CNTSNKG (THREAD MILLING/COUNTERSINKING) | Mills a thread and a countersink. | G263 | 620: #19 | 320 340 551-01 on; 426/430 280 476 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p263 |
| 264 | THREAD DRILLNG/MLLNG (THREAD DRILLING/MILLING) | Drills from solid and mills the thread. | G264 | 620: #19 | 320 340 551-01 on; 426/430 280 476 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p268 |
| 265 | HEL. THREAD DRLG/MLG (HELICAL THREAD DRILLING/MILLING) | Mills a thread and a countersink on a helix without pre-drilling. | G265 | 620: #19 | 320 340 551-01 on; 426/430 280 476 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p273 |
| 267 | OUTSIDE THREAD MLLNG (OUTSIDE THREAD MILLING) | Mills an external thread on a stud. | G267 | 620: #19 | 320 340 551-01 on; 426/430 280 476 on; iTNC 340 420 on; 620 340 56x-01 on; 640 34059x-01 on; TNC7 81762x-17 on | C names it: 426/430 280476-xx. | 1358775-24 p277 |
| 270 | CONTOUR TRAIN DATA | Sets the approach and departure data for cycle 25. | G270 | 620: #19 | 320 77185x-02 on; iTNC 340 49x-05 on; 620 81760x-02 on; 640 34059x-05 on; TNC7 81762x-17 on | C names it: iTNC 34049x-04; 620 81760x-02, 81760x-05; 640 34059x-05, 34059x-08. | 1358775-24 p350 |
| 271 | OCM CONTOUR DATA | Sets the common data for the OCM (optimised contour milling) cycles. | G271 | #167 | 320 771851-18 on; 620 81760x-07 on; 640 34059x-10 on; TNC7 81762x-17 on | C names it: 620 81760x-07; 640 34059x-10. | 1358775-24 p382 |
| 272 | OCM ROUGHING | Trochoidal (constant-engagement) roughing of a contour. | G272 | #167 | 320 771851-18 on; 620 81760x-07 on; 640 34059x-10 on; TNC7 81762x-17 on | C names it: 620 81760x-07; 640 34059x-10; TNC7 81762x-20. | 1358775-24 p385 |
| 273 | OCM FINISHING FLOOR | OCM floor finishing. | G273 | #167 | 320 771851-18 on; 620 81760x-07 on; 640 34059x-10 on; TNC7 81762x-17 on | C names it: 620 81760x-08; 640 34059x-11. | 1358775-24 p391 |
| 274 | OCM FINISHING SIDE | OCM wall finishing. | G274 | #167 | 320 771851-18 on; 620 81760x-07 on; 640 34059x-10 on; TNC7 81762x-17 on | - | 1358775-24 p394 |
| 275 | TROCHOIDAL SLOT | Mills a slot along a contour with trochoidal motion. | G275 | 620: #19 | 320 77185x-01 on; iTNC 340 49x-06 on; 620 81760x-01 on; 640 34059x-04 on; TNC7 81762x-17 on | C names it: 320 77185x-01; iTNC 34049x-06, 60642x-01; 620 81760x-01; 640 34059x-04. | 1358775-24 p357 |
| 276 | THREE-D CONT. TRAIN | Contour train that also follows Z values of the contour. | G276 | 620: #19 | 320 77185x-05 on; iTNC 340 49x-07 on; 620 81760x-05 on; 640 34059x-08 on; TNC7 81762x-17 on | C names it: iTNC 34049x-07, 60642x-02, 60642x-03, 60642x-04; 620 81760x-05; 640 34059x-08. | 1358775-24 p363 |
| 277 | OCM CHAMFERING | Deburrs (chamfers) the edges of an OCM contour. | G277 | #167 | 320 771851-18 on; 620 81760x-08 on; 640 34059x-11 on; TNC7 81762x-17 on | C names it: 620 81760x-08, 81760x-17; 640 34059x-11, 34059x-17; TNC7 81762x-17. | 1358775-24 p397 |
| 285 | DEFINE GEAR | Defines the gear geometry for 286/287. | G285 | #157 | 640 34059x-09 on; TNC7 81762x-17 on | C names it: 640 34059x-09. | 1358775-24 p416 |
| 286 | GEAR HOBBING | Gear hobbing on a mill-turn machine. | G286 | #157 | 640 34059x-09 on; TNC7 81762x-17 on | C names it: 640 34059x-09. | 1358775-24 p419 |
| 287 | GEAR SKIVING | Gear skiving on a mill-turn machine. | G287 | #157 | 640 34059x-09 on; TNC7 81762x-17 on | - | 1358775-24 p427 |
| 290 | INTERPOLATION TURNING | Turns a rotationally symmetric shoulder by interpolating the spindle with the axes. | G290 | software option (iTNC; #96 unsure) | iTNC 34049x-08 on | Successors 291/292 on 640/620/TNC7. C names it: iTNC 34049x-07, 60642x-02, 60642x-03, 60642x-04. | 670388-24 p12 |
| 291 | COUPLG.TURNG.INTERP. (COUPLING TURNING INTERPOLATION) | Couples the spindle to the axes for interpolation turning. | G291 | #96 | 640 34059x-05 on; TNC7 81762x-17 on | C names it: 640 34059x-06. | 1358775-24 p460 |
| 292 | CONTOUR.TURNG.INTRP. (INTERPOLATION TURNING, CONTOUR FINISHING) | Finishes a rotationally symmetric contour by interpolation turning. | G292 | #96 | 640 34059x-05 on; TNC7 81762x-17 on | C names it: 640 34059x-17. | 1358775-24 p466 |

## Turning cycles (800-899, mill-turn controls)

| No. | HEIDENHAIN name | Purpose (own words) | DIN/ISO | Option | In example blocks: first → last per control | Introduced / changed / removed | Reference (manual ID, PDF page) |
|---|---|---|---|---|---|---|---|
| 800 | ADJUST XZ SYSTEM (older: ADAPT ROTARY COORDINATE SYSTEM) | Aligns the turning coordinate system and the tool to a tilted position. | G800 | #50 | 640 34059x-01 on; TNC7 81762x-16 on | C names it: TNC7 81762x-18. | 1358775-24 p788 |
| 801 | RESET ROTARY COORDINATE SYSTEM | Resets the settings of cycle 800. | G801 | #50 | 640 34059x-05 on; TNC7 81762x-17 on | - | 1358775-24 p796 |
| 810 | TURN CONTOUR LONG. | Longitudinal roughing/finishing of any turning contour. | G810 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p517 |
| 811 | SHOULDER, LONGITDNL. | Longitudinal turning of a simple shoulder. | G811 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 to 34059x-11 | C names it: 640 34059x-16. | 1358775-24 p499 |
| 812 | SHOULDER, LONG. EXT. | Longitudinal turning of a shoulder with extra elements (chamfers, radii). | G812 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | C names it: 640 34059x-16. | 1358775-24 p503 |
| 813 | TURN PLUNGE CONTOUR LONGITUDINAL | Longitudinal turning with plunging (falling contour elements). | G813 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | C names it: 640 34059x-16. | 1358775-24 p508 |
| 814 | TURN PLUNGE LONGITUDINAL EXT. | Extended longitudinal plunge turning. | G814 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | C names it: 640 34059x-16. | 1358775-24 p512 |
| 815 | CONTOUR-PAR. TURNING | Turning parallel to the finished contour. | G815 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p522 |
| 820 | TURN CONTOUR TRANSV. | Facing (transverse) of any turning contour. | G820 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p543 |
| 821 | SHOULDER, FACE | Facing of a simple shoulder. | G821 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p526 |
| 822 | SHOULDER, FACE. EXT. | Facing of a shoulder with extra elements. | G822 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p529 |
| 823 | TURN TRANSVERSE PLUNGE | Facing with plunging. | G823 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p534 |
| 824 | TURN PLUNGE TRANSVERSE EXT. | Extended transverse plunge turning. | G824 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p538 |
| 830 | THREAD CONTOUR-PARALLEL | Turns a thread that follows a contour. | G830 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p622 |
| 831 | THREAD LONGITUDINAL | Turns a longitudinal thread. | G831 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | C names it: 640 34059x-08. | 1358775-24 p612 |
| 832 | THREAD EXTENDED | Turns a thread with run-in and run-out (taper allowed). | G832 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | C names it: 640 34059x-08. | 1358775-24 p616 |
| 840 | RECESS TURNG, RADIAL | Recess turning (plunge and turn) of a radial contour. | G840 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-02 on; TNC7 81762x-17 on | - | 1358775-24 p566 |
| 841 | SIMPLE REC. TURNG., RADIAL DIR. | Simple radial recess turning. | G841 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-02 on; TNC7 81762x-17 on | C names it: 640 34059x-02. | 1358775-24 p548 |
| 842 | ENH.REC.TURNNG, RAD. | Extended radial recess turning. | G842 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-02 on; TNC7 81762x-17 on | - | 1358775-24 p552 |
| 850 | RECESS TURNG, AXIAL | Recess turning of an axial contour. | G850 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-02 on; TNC7 81762x-17 on | - | 1358775-24 p571 |
| 851 | SIMPLE REC TURNG, AX | Simple axial recess turning. | G851 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-02 on; TNC7 81762x-17 on | C names it: 640 34059x-02. | 1358775-24 p557 |
| 852 | ENH.REC.TURNING, AX. | Extended axial recess turning. | G852 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-02 on; TNC7 81762x-17 on | - | 1358775-24 p561 |
| 860 | CONT. RECESS, RADIAL | Radial recessing (grooving) of any contour. | G860 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | C names it: 640 34059x-07, 34059x-16. | 1358775-24 p598 |
| 861 | SIMPLE RECESS, RADL. | Simple radial recess (groove). | G861 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p576 |
| 862 | EXPND. RECESS, RADL. | Extended radial recess. | G862 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | C names it: 640 34059x-16. | 1358775-24 p581 |
| 870 | CONT. RECESS, AXIAL | Axial recessing of any contour. | G870 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | C names it: 640 34059x-16. | 1358775-24 p604 |
| 871 | SIMPLE RECESS, AXIAL | Simple axial recess. | G871 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-01 on; TNC7 81762x-17 on | - | 1358775-24 p587 |
| 872 | EXPND. RECESS, AXIAL | Extended axial recess. | G872 | #50 (turning; from the chapter, unsure per cycle) | 640 34059x-16 on; TNC7 81762x-17 on | C names it: 640 34059x-16. | 1358775-24 p592 |
| 880 | GEAR HOBBING | Gear hobbing in turning mode (older gear cycle). | G880 | #50 and #131 | 640 34059x-05 on; TNC7 81762x-17 on | - | 1358775-24 p647 |
| 882 | SIMULTANEOUS ROUGHING FOR TURNING | Simultaneous multi-axis turning roughing. | G882 | #158 | 640 34059x-11 on; TNC7 81762x-17 on | - | 1358775-24 p627 |
| 883 | TURNING SIMULTANEOUS FINISHING | Simultaneous multi-axis turning finishing. | G883 | #158 | 640 34059x-09 on; TNC7 81762x-17 on | - | 1358775-24 p633 |
| 892 | CHECK UNBALANCE | Checks the unbalance of the turning spindle set-up. | G892 | #50 | 640 34059x-05 on; TNC7 81762x-17 on | - | 1358775-24 p658 |

## Grinding cycles (1000-1099, TNC 640 / TNC7 with grinding option)

| No. | HEIDENHAIN name | Purpose (own words) | DIN/ISO | Option | In example blocks: first → last per control | Introduced / changed / removed | Reference (manual ID, PDF page) |
|---|---|---|---|---|---|---|---|
| 1000 | DEFINE RECIP. STROKE | Defines a reciprocating stroke for jig grinding. | G1000 | #156 | 640 34059x-10 on; TNC7 81762x-17 on | C names it: 640 34059x-10; TNC7 81762x-19. | 1303406-23 p722 |
| 1001 | START RECIP. STROKE | Starts the reciprocating stroke. | G1001 | #156 | 640 34059x-10 on; TNC7 81762x-17 on | - | 1303406-23 p725 |
| 1002 | STOP RECIP. STROKE | Stops the reciprocating stroke. | G1002 | #156 | 640 34059x-10 on; TNC7 81762x-17 on | - | 1303406-23 p726 |
| 1010 | DRESSING DIAMETER | Dresses the grinding wheel diameter. | G1010 | #156 | 640 34059x-10 on; TNC7 81762x-17 on | C names it: 640 34059x-10, 34059x-16. | 1358775-24 p668 |
| 1011 | DRESSING SIDE A/I | Dresses the outer or inner side of the wheel. | G1011 | #156 | TNC7 81762x-19 on | C names it: TNC7 81762x-19. | 1358775-24 p672 |
| 1012 | DRESSING D AND A/I | Dresses diameter and side in one cycle. | G1012 | #156 | TNC7 81762x-19 on | - | 1358775-24 p676 |
| 1015 | PROFILE DRESSING | Dresses a defined wheel profile. | G1015 | #156 | 640 34059x-10 on; TNC7 81762x-17 on | C names it: 640 34059x-10. | 1358775-24 p680 |
| 1016 | DRESSING OF CUP WHEEL | Dresses a cup wheel. | G1016 | #156 | 640 34059x-11 on; TNC7 81762x-17 on | - | 1358775-24 p687 |
| 1017 | DRESSING WITH DRESSING ROLL | Dresses with a dressing roll. | G1017 | #156 | 640 34059x-16 on; TNC7 81762x-17 on | C names it: 640 34059x-16. | 1358775-24 p691 |
| 1018 | RECESSING WITH DRESSING ROLL | Plunge dressing with a dressing roll. | G1018 | #156 | 640 34059x-16 on; TNC7 81762x-17 on | - | 1358775-24 p698 |
| 1021 | CYLINDER, SLOW-STROKE GRINDING | Grinds a cylinder with slow strokes. | G1021 | #156 | 640 34059x-16 on; TNC7 81762x-17 on | - | 1303406-23 p760 |
| 1022 | CYLINDER, FAST-STROKE GRINDING | Grinds a cylinder with fast strokes. | G1022 | #156 | 640 34059x-16 on; TNC7 81762x-17 on | - | 1303406-23 p768 |
| 1025 | GRINDING CONTOUR | Grinds along an open or closed contour. | G1025 | #156 | 640 34059x-11 on; TNC7 81762x-17 on | C names it: 640 34059x-11; TNC7 81762x-20. | 1303406-23 p774 |
| 1030 | ACTIVATE WHEEL EDGE | Activates the wheel edge used for dressing. | G1030 | #156 | 640 34059x-10 on; TNC7 81762x-17 on | - | 1358775-24 p703 |
| 1032 | GRINDING WHL LENGTH COMPENSATION | Corrects the wheel length. | G1032 | #156 | 640 34059x-10 on; TNC7 81762x-17 on | C names it: 640 34059x-10; TNC7 81762x-20. | 1358775-24 p800 |
| 1033 | GRINDING WHL RADIUS COMPENSATION | Corrects the wheel radius. | G1033 | #156 | 640 34059x-10 on; TNC7 81762x-17 on | C names it: 640 34059x-10; TNC7 81762x-20. | 1358775-24 p802 |
| 1040 | END CYLIND. GRINDING | Grinding cycle on a cylindrical grinding machine (end face; name from the overview, unsure). | G1040 | #156 | TNC7 81762x-19 on | - | 1358775-24 p735 |
| 1041 | LONG STROKE DEF. | Defines a long stroke (cylindrical grinding). | G1041 | #156 | TNC7 81762x-19 on | - | 1358775-24 p735 |
| 1042 | SHORT STROKE DEF. | Defines a short stroke (cylindrical grinding). | G1042 | #156 | TNC7 81762x-19 on | - | 1358775-24 p735 |
| 1051 | STEP. CYLIND. GRIND | Stepped cylindrical grinding (name unsure: short form only). | G1051 | #156 | TNC7 81762x-19 on | - | 1358775-24 p30 (contents) |
| 1053 | CONTINOUS CYLIND. GRIND. | Continuous cylindrical grinding (HEIDENHAIN spelling). | G1053 | #156 | TNC7 81762x-19 on | - | 1358775-24 p735 |

## OCM standard figures (1271-1282)

| No. | HEIDENHAIN name | Purpose (own words) | DIN/ISO | Option | In example blocks: first → last per control | Introduced / changed / removed | Reference (manual ID, PDF page) |
|---|---|---|---|---|---|---|---|
| 1271 | OCM RECTANGLE | OCM rectangle figure (pocket, island or boundary). | G1271 | #167 | 320 771851-18 on; 620 81760x-08 on; 640 34059x-11 on; TNC7 81762x-17 on | - | 1358775-24 p166 |
| 1272 | OCM CIRCLE | OCM circle figure. | G1272 | #167 | 320 771851-18 on; 620 81760x-08 on; 640 34059x-11 on; TNC7 81762x-17 on | - | 1358775-24 p170 |
| 1273 | OCM SLOT / RIDGE | OCM slot or ridge figure. | G1273 | #167 | 320 771851-18 on; 620 81760x-08 on; 640 34059x-11 on; TNC7 81762x-17 on | - | 1358775-24 p173 |
| 1274 | OCM CIRCULAR SLOT | OCM circular slot figure. | G1274 | #167 | 320 771851-18 on; 620 81760x-18 on; 640 34059x-18 on; TNC7 81762x-18 on | C names it: 620 81760x-18; 640 34059x-18; TNC7 81762x-18. | 1358775-24 p176 |
| 1278 | OCM POLYGON | OCM polygon figure. | G1278 | #167 | 320 771851-18 on; 620 81760x-08 on; 640 34059x-11 on; TNC7 81762x-17 on | - | 1358775-24 p179 |
| 1281 | OCM RECTANGLE BOUNDARY | Rectangular boundary for OCM figures. | G1281 | #167 | 320 771851-18 on; 620 81760x-08 on; 640 34059x-11 on; TNC7 81762x-17 on | - | 1358775-24 p183 |
| 1282 | OCM CIRCLE BOUNDARY | Circular boundary for OCM figures. | G1282 | #167 | 320 771851-18 on; 620 81760x-08 on; 640 34059x-11 on; TNC7 81762x-17 on | - | 1358775-24 p185 |

## Touch probe cycles (TCH PROBE 0-4, 400-499, 600-601, 1400-1499)

| No. | HEIDENHAIN name | Purpose (own words) | DIN/ISO | Option | Introduced / changed / removed | Reference (manual ID, PDF page) |
|---|---|---|---|---|---|---|
| TCH PROBE 0 | REF. PLANE (older: REFERENCE PLANE / DATUM PLANE) | Probes one coordinate in a chosen axis direction. | G55 | #17 | - | 1358777-24 p326 |
| TCH PROBE 1 | POLAR PRESET (older: POLAR DATUM PLANE) | Probes one point in a polar direction. | (none found) | #17 | - | 1358777-24 p328 |
| TCH PROBE 3 | MEASURING | Freely programmable probing, result to Q parameters. | (none found) | #17 | - | 1358777-24 p379 |
| TCH PROBE 4 | MEASURING IN 3-D | Probes along a 3-D vector. | (none found) | #17 | New on 620 81760x-01 (C). | 1358777-24 p382 |
| TCH PROBE 400 | BASIC ROTATION | Measures misalignment from two points on a line and compensates it. | G400 | #17 | - | 1358777-24 p148 |
| TCH PROBE 401 | ROT OF 2 HOLES | Measures misalignment from two holes. | G401 | #17 | C names it: iTNC 34049x-03. | 1358777-24 p152 |
| TCH PROBE 402 | ROT OF 2 STUDS | Measures misalignment from two studs. | G402 | #17 | C names it: iTNC 34049x-03. | 1358777-24 p157 |
| TCH PROBE 403 | ROT IN ROTARY AXIS | Compensates misalignment by turning a rotary axis. | G403 | #17 | C names it: iTNC 34049x-05. | 1358777-24 p162 |
| TCH PROBE 404 | SET BASIC ROTATION | Sets a basic rotation value. | G404 | #17 | C names it: iTNC 60642x-03, 60642x-04. | 1358777-24 p167 |
| TCH PROBE 405 | ROT IN C AXIS | Aligns a hole centre by turning the C axis. | G405 | #17 | C names it: iTNC 60642x-01. | 1358777-24 p169 |
| TCH PROBE 408 | SLOT CENTER PRESET (older: DATUM SLOT CENTER / SLOT CENTER REF PT) | Sets the preset at the centre of a slot. | G408 | #17 | C names it: 620 81760x-06; 640 34059x-09. | 1358777-24 p218 |
| TCH PROBE 409 | RIDGE CENTER PRESET (older: DATUM RIDGE CENTER) | Sets the preset at the centre of a ridge. | G409 | #17 | - | 1358777-24 p223 |
| TCH PROBE 410 | PRESET INSIDE RECTAN (older: DATUM FROM INSIDE OF RECTANGLE) | Sets the preset at the centre of a rectangular pocket. | G410 | #17 | - | 1358777-24 p228 |
| TCH PROBE 411 | PRESET OUTS. RECTAN (older: DATUM FROM OUTSIDE OF RECTANGLE) | Sets the preset at the centre of a rectangular stud. | G411 | #17 | - | 1358777-24 p233 |
| TCH PROBE 412 | PRESET INSIDE CIRCLE (older: DATUM FROM INSIDE OF CIRCLE) | Sets the preset at the centre of a hole. | G412 | #17 | C names it: iTNC 34049x-04. | 1358777-24 p238 |
| TCH PROBE 413 | PRESET OUTS. CIRCLE (older: DATUM FROM OUTSIDE OF CIRCLE) | Sets the preset at the centre of a circular stud. | G413 | #17 | C names it: iTNC 34049x-04. | 1358777-24 p244 |
| TCH PROBE 414 | PRESET OUTSIDE CORNER (older: DATUM FROM OUTSIDE OF CORNER) | Sets the preset at an outside corner. | G414 | #17 | - | 1358777-24 p250 |
| TCH PROBE 415 | PRESET INSIDE CORNER (older: DATUM FROM INSIDE OF CORNER) | Sets the preset at an inside corner. | G415 | #17 | - | 1358777-24 p257 |
| TCH PROBE 416 | PRESET CIRCLE CENTER (older: DATUM CIRCLE CENTER) | Sets the preset at the centre of a bolt-hole circle. | G416 | #17 | - | 1358777-24 p263 |
| TCH PROBE 417 | PRESET IN TS AXIS (older: DATUM IN TOUCH PROBE AXIS) | Sets the preset in the touch-probe axis. | G417 | #17 | - | 1358777-24 p269 |
| TCH PROBE 418 | PRESET FROM 4 HOLES (older: DATUM AT CENTER OF 4 HOLES) | Sets the preset at the crossing of the lines between four holes. | G418 | #17 | - | 1358777-24 p273 |
| TCH PROBE 419 | PRESET IN ONE AXIS (older: DATUM IN ONE AXIS) | Sets the preset in one selectable axis. | G419 | #17 | C names it: 620 81760x-06; 640 34059x-09. | 1358777-24 p278 |
| TCH PROBE 420 | MEASURE ANGLE | Measures an angle in the plane. | G420 | #17 | - | 1358777-24 p330 |
| TCH PROBE 421 | MEASURE HOLE | Measures a hole (centre and diameter). | G421 | #17 | C names it: 320 77185x-04; iTNC 34049x-04; 620 81760x-03; 640 34059x-06. | 1358777-24 p333 |
| TCH PROBE 422 | MEAS. CIRCLE OUTSIDE (older: MEASURE HOLE OUTSIDE) | Measures a circular stud. | G422 | #17 | C names it: 320 77185x-04; iTNC 34049x-04; 620 81760x-03; 640 34059x-06. | 1358777-24 p339 |
| TCH PROBE 423 | MEAS. RECTAN. INSIDE | Measures a rectangular pocket. | G423 | #17 | - | 1358777-24 p345 |
| TCH PROBE 424 | MEAS. RECTAN. OUTSIDE | Measures a rectangular stud. | G424 | #17 | - | 1358777-24 p350 |
| TCH PROBE 425 | MEASURE INSIDE WIDTH | Measures a slot width. | G425 | #17 | - | 1358777-24 p355 |
| TCH PROBE 426 | MEASURE RIDGE WIDTH | Measures a ridge width. | G426 | #17 | - | 1358777-24 p359 |
| TCH PROBE 427 | MEASURE COORDINATE | Measures one coordinate. | G427 | #17 | C names it: 320 77185x-04; 620 81760x-03; 640 34059x-06. | 1358777-24 p363 |
| TCH PROBE 430 | MEAS. BOLT HOLE CIRC. | Measures a bolt-hole circle. | G430 | #17 | - | 1358777-24 p368 |
| TCH PROBE 431 | MEASURE PLANE | Measures the angles of a plane. | G431 | #17 | - | 1358777-24 p372 |
| TCH PROBE 440 | MEASURE AXIS SHIFT | Measures thermal axis shift with a tool-setting probe. | G440 | #17 (unsure) | C names it: 426/430 280476-xx. | 670388-24 p37 |
| TCH PROBE 441 | FAST PROBING | Sets global probing parameters (feed, angle tracking) for later probe cycles. | G441 | #17 | C names it: 320 77185x-05; 620 81760x-05, 81760x-18; 640 34059x-08, 34059x-18. | 1358777-24 p390 |
| TCH PROBE 444 | PROBING IN 3-D | Probes one point along a 3-D vector and checks a tolerance. | G444 | #17 | C names it: 640 34059x-07, 34059x-10. | 1358777-24 p384 |
| TCH PROBE 450 | SAVE KINEMATICS | Saves and restores the active machine kinematics. | G450 | #48 | KinematicsOpt. C names it: iTNC 34049x-03, 34049x-04. | 1358777-24 p429 |
| TCH PROBE 451 | MEASURE KINEMATICS | Measures the rotary axes on a calibration sphere and optimises the kinematics. | G451 | #48 | KinematicsOpt. C names it: 620 73498x-02, 81760x-04, 81760x-17; 640 34059x-02, 34059x-17; TNC7 81762x-17, 81762x-18. | 1358777-24 p433 |
| TCH PROBE 452 | PRESET COMPENSATION | Compensates preset errors after a head change. | G452 | #48 | KinematicsOpt. C names it: 620 81760x-04, 81760x-17. | 1358777-24 p449 |
| TCH PROBE 453 | KINEMATICS GRID | Measures the kinematics on a grid of positions. | G453 | #48 (and #52 unsure) | KinematicsOpt. C names it: 620 81760x-17. | 1358777-24 p461 |
| TCH PROBE 460 | CALIBRATION OF TS ON A SPHERE (older: CALIBRATE TS) | Calibrates a 3-D touch probe on a sphere. | G460 | #17 | C names it: 620 73498x-02, 81760x-04; 640 34059x-02, 34059x-07. | 1358777-24 p104 |
| TCH PROBE 461 | TS CALIBRATION OF TOOL LENGTH (also CALIBRATE TS LENGTH) | Calibrates the probe length. | G461 | #17 | C names it: 620 73498x-02; 640 34059x-02. | 1358777-24 p112 |
| TCH PROBE 462 | CALIBRATION OF A TS IN A RING (also CALIBRATE TS RADIUS INSIDE) | Calibrates the probe radius in a ring gauge. | G462 | #17 | C names it: 620 81760x-04; 640 34059x-07. | 1358777-24 p114 |
| TCH PROBE 463 | TS CALIBRATION ON STUD (also CALIBRATE TS RADIUS OUTSIDE) | Calibrates the probe radius on a stud. | G463 | #17 | C names it: 620 81760x-04; 640 34059x-07. | 1358777-24 p117 |
| TCH PROBE 480 | CALIBRATE TT | Calibrates the tool touch probe (TT). | G480 | #17 (unsure) | C names it: 620 81760x-08; 640 34059x-11; TNC7 basic 81762x-20. | 1358777-24 p121 |
| TCH PROBE 481 | CAL. TOOL LENGTH | Measures the tool length with the TT. | G481 | #17 (unsure) | C names it: 128 77184x-05. | 1358777-24 p407 |
| TCH PROBE 482 | CAL. TOOL RADIUS | Measures the tool radius with the TT. | G482 | #17 (unsure) | - | 1358777-24 p410 |
| TCH PROBE 483 | MEASURE TOOL | Measures tool length and radius with the TT. | G483 | #17 (unsure) | C names it: 128 77184x-05; 620 81760x-08; 640 34059x-11. | 1358777-24 p414 |
| TCH PROBE 484 | CALIBRATE IR TT | Calibrates a wireless (infrared) TT. | G484 | #17 | C names it: 620 81760x-02, 81760x-08; 640 34059x-05, 34059x-11. | 1358777-24 p123 |
| TCH PROBE 485 | MEASURE LATHE TOOL | Measures a turning tool with the TT. | G485 | #17 and #50 (unsure) | C names it: 640 34059x-11; TNC7 basic 81762x-19. | 1358777-24 p418 |
| TCH PROBE 600 | GLOBAL WORKING SPACE | Takes a camera image of the working space for visual monitoring. | G600 | #136 | C names it: 640 34059x-06. | 1261174-20 (TNC 640 Setup) |
| TCH PROBE 601 | LOCAL WORKING SPACE | Takes a camera image of a local area for visual monitoring. | G601 | #136 | As 600. C names it: 640 34059x-06. | 1261174-20 (TNC 640 Setup) |
| TCH PROBE 1400 | POSITION PROBING | Probes one position (newer probe-cycle family). | G1400 | #17 | C names it: 620 81760x-16; 640 34059x-16. | 1358777-24 p282 |
| TCH PROBE 1401 | CIRCLE PROBING | Probes a circle centre. | G1401 | #17 | - | 1358777-24 p286 |
| TCH PROBE 1402 | SPHERE PROBING | Probes a sphere centre. | G1402 | #17 | - | 1358777-24 p291 |
| TCH PROBE 1403 | RECTANGLE PROBING | Probes a rectangle. | G1403 | #17 | C names it: TNC7 81762x-19; TNC7 basic 81762x-19. | 1358777-24 p296 |
| TCH PROBE 1404 | PROBE SLOT/RIDGE | Probes a slot or ridge. | G1404 | #17 | - | 1358777-24 p302 |
| TCH PROBE 1410 | PROBING ON EDGE | Probes two points on an edge (misalignment). | G1410 | #17 | C names it: 620 81760x-06, 81760x-07, 81760x-16; 640 34059x-09, 34059x-10, 34059x-11, 34059x-16. | 1358777-24 p174 |
| TCH PROBE 1411 | PROBING TWO CIRCLES | Probes two holes or studs (misalignment). | G1411 | #17 | C names it: 620 81760x-07, 81760x-16; 640 34059x-16. | 1358777-24 p181 |
| TCH PROBE 1412 | INCLINED EDGE PROBING | Probes an inclined edge. | G1412 | #17 | - | 1358777-24 p189 |
| TCH PROBE 1416 | INTERSECTION PROBING | Probes the intersection of two edges. | G1416 | #17 | C names it: 620 81760x-17; 640 34059x-17; TNC7 81762x-17. | 1358777-24 p196 |
| TCH PROBE 1420 | PROBING IN PLANE | Probes three points on a plane (tilt angles). | G1420 | #17 | C names it: 620 81760x-07, 81760x-16; 640 34059x-16. | 1358777-24 p204 |
| TCH PROBE 1430 | PROBE POSITION OF UNDERCUT | Probes a position with an L-shaped stylus under an undercut. | G1430 | #17 | - | 1358777-24 p308 |
| TCH PROBE 1434 | PROBE SLOT/RIDGE UNDERCUT | Probes a slot or ridge under an undercut. | G1434 | #17 | - | 1358777-24 p313 |
| TCH PROBE 1493 | EXTRUSION PROBING | Repeats a probe point along a line (extrusion). | G1493 | #17 | C names it: TNC7 basic 81762x-20. | 1358777-24 p393 |

