# HEIDENHAIN TNC: functions by NC software version (generated)

This file is section C of `differences.md`. A script made it from the "New functions" / "Changed functions" lists in the HEIDENHAIN manuals and from the "Overview of New and Modified Software Functions" documents. `inventory.md` lists each document (ID, title, date, URL).

**Copyright.** The manuals are © HEIDENHAIN. Each bullet below keeps only the first words of one list entry, as a pointer, with the document ID and the PDF page. Read the manual for the full text. Bullets cut in the middle are marked "…".

## C. Per control: NC software versions in release order (added / changed / removed)

**Cycle numbers in this section are not annotated.** Look up each number in `cycles.md` (HEIDENHAIN name, purpose, DIN/ISO code, option and versions).

Generated from the new/changed-function lists (see the method note at the top). "[new]" and "[changed]" follow the heading of the list. The source after each bullet is the manual ID (file name in the inventory) and, for overview documents, the PDF page. "(see ... page N)" inside a bullet is the PRINTED page of that manual.

Versions with no list: TNC 124, TNC 310, TNC 410 (the shared ISO manual 345 940-22 lists 280 476 functions for the TNC 426/430 only), TNC 426 280 460/280 462/280 470-474, TNC 320 340 551-01/-02, TNC 620 340 56x-01 and 73498x-01, TNC 640 34059x-01, TNC 128 77184x-01, TNC7 81762x-16 (first versions or old manuals without such a section). For these, use section B and `feature-mentions.md`.


### C.TNC 128


**77184x-03**

- [new] Programs with .HU and .HC extensions can be selected and processed in all operating modes The functions SELECT PROGRAM and CALL SELECTED PROGRAM have been … (819494-23)
- other: Cycle 253 SLOT MILLING has been added (option 19) · Cycle 253 SLOT MILLING has been added (option 19), see "SLOT MILLING (

**77184x-05**

- [changed] For cycle 247: SET DATUM, the datum number from the preset table can be selected with the corresponding parameter With cycle 200 and 203 the behavior of the … (819494-23)
- [changed] In Cycle 247 PRESETTING, the preset number from the preset table can be selected with the corresponding parameter, see "PRESETTING (Cycle 247)", page 601 … (819494-24)

**77184x-06**

- other: Reading system data", page 268 · Cycles 256 RECTANGULAR STUD were extended by the parameters Q215, Q385

**77184x-18**

- [changed] Overview of new and modified software functions Further information about the previous software versions is presented in the Overview of New and Modified … (819494-26)
- [changed] Basic Information | Control model, software and features 1 New functions FUNCTION CORRDATA allows you to activate a line of the compensation table. The … (819494-26)
- [changed] Basic Information | Control model, software and features 1 New functions ISO Programming FUNCTION CORRDATA allows you to activate a line of the compensation … (1263174-21)

### C.TNC 320


**34055x-03**

- [new] Cycle 208 helical finish milling: milling method (climb milling / conventional up-cut milling) can now be selected (661_873-20)
- [new] Cycle 209 tapping with chip breaking: now with rapid retraction (661_873-20)
- [new] The TNC now also supports the tilting functions on machine tools with swivel heads and/or tilting tables (see “Tilting the Working Plane (Software Option 1),” … (550_671-22)
- [new] Cycle 240 Centering added (See “CENTERING (Cycle 240)” on page 215) (550_671-22)
- [new] Cycle 208 Helical Finish Milling: The type of milling (climb or up-cut) can now be selected (See “BORE MILLING (Cycle 208)” on page 231) (550_671-22)
- [new] Cycle 209 Tapping with Chip Breaking: Fast retraction added (See (550_671-22)
- [new] TAPPING WITH CHIP BREAKING (Cycle 209)” on page 237) (550_671-22)
- [new] Cycle 19 Working Plane and 3-D ROT (See “Tilting the Working Plane (Software Option 1)” on page 60) (550_671-22)
- other: The TNC now also supports datum administration via the preset table · The TNC now also supports the tilting of working planes on machine too · The TNC now also …

**34055x-04**

- [new] The PLANE function for flexible definition of a tilted working plane was introduced (see “The PLANE Function: Tilting the Working Plane (Software Option 1)” on … (679_222-20)
- [new] The FUNCTION PARAX function for defining the behavior of the parallel axes U, V and W was introduced (see “Working with the Parallel Axes U, V and W” on page 317) (679_222-20)
- [new] The PATTERN DEF function for defining point patterns was introduced (679_222-20)
- [new] With the CYCL CALL PAT function, cycles can now be run in (679_222-20)
- [new] New Cycle 241 for Single-Fluted Deep-Hole Drilling was introduced (679_222-20)
- [new] The definition of feed rates in fixed cycles can now also include FU (679_222-20)
- [changed] In Cycle 22 you can now define a tool name also for the coarse (679_222-20)
- [changed] The tolerance values set in Cycle 32 are displayed (679_222-20)
- [changed] The pocket-, stud- and slot-milling cycles 210 to 214 were removed from the standard soft-key row (CYCL DEF > POCKETS/STUDS/SLOTS). For reasons of … (679_222-20)
- [new] The PATTERN DEF function for defining patterns was introduced (see (679_220-20)
- [new] Pattern Definition PATTERN DEF” on page 44) (679_220-20)
- [new] With the CYCL CALL PAT function, cycles can now be run in connection with point tables (see “Calling a cycle in connection with point tables” on page 55) (679_220-20)
- [new] SINGLE-FLUTED DEEP-HOLE DRILLING (Cycle 241, DIN/ISO: G241)” on page 84) (679_220-20)
- [new] The definition of feed rates in fixed cycles can now also include FU and FZ values (679_220-20)
- [new] The PLANE function for flexible definition of a tilted working place (679_220-20)
- [new] The FUNCTION PARAX function for defining the behavior of the parallel Conversational Programming) (679_220-20)
- [changed] In Cycle 22 you can now define a tool name also for the coarse roughing tool (see “ROUGH-OUT (Cycle 22, DIN/ISO: G122)” on page 180). (679_220-20)
- [new] New machining cycle for single-lip deep-hole drilling (see "SINGLE- LIP DEEP-HOLE DRILLING (Cycle 241, DIN/ISO: G241)" on page 86) (679_220-21)
- [new] New Cycle 241 for Single-Lip Deep-Hole Drilling was introduced (679_226-21)
- other: Individual characters can now be deleted by using the backspace key · The SEL PATTERN function makes it possible to select point tables · The DECLARE CONTOUR …

**34055x-05**

- [new] The function M101 was introduced (see User’s Manual for Conversational Programming). (679_220-21)
- [new] Tool tables of the iTNC 530 can now be imported in the TNC 320 and Programming). (679_220-21)
- [new] The CYCL CALL POS function was introduced (see "Calling a cycle with CYCL CALL POS" on page 45). (679_220-21)
- [new] Local and nonvolatile QL and QR Q parameters were introduced (see (679_220-21)
- [new] The function M138 for selecting tilting axes was introduced (see (679_220-21)
- [changed] Touch probe cycles can now also be used in the tilted working plane (679_220-21)
- [new] The M101 function was introduced (see "Automatic tool change if the tool life expires: M101" on page 146). (679_226-21)
- [new] Tool tables of the iTNC 530 can now be imported in the TNC 320 and converted to a valid format (see "Importing tool tables" on page 140). (679_226-21)
- [new] The M138 "Selection of swivel axes" function was introduced (see "Selecting tilting axes: M138" on page 323). (679_226-21)
- other: File functions were introduced · The "Define coordinate transformations" function was introduced · The tool table was expanded by the LAST_USE column · The …

**34055x-06**

- other: The maximum number of characters for the NAME and DOC fields in the to

**77185x-01**

- [new] Cycle 205: Q208 (retraction feed rate). Cycles 200, 203, 205: Q395 (depth reference). Checked by hand (1096959-20 p9, p82)
- [new] The character set of the fixed cycle 225 Engraving was expanded by more characters and the diameter sign see "ENGRAVING (Cycle 225, DIN/ISO: G225)", page 276 New … (1096959-20)

**77185x-02**

- [changed] overview of the optional Q parameters that have been added in this software version. You can decide for yourself whether you would like to define optional Q … (1096959-22)
- [new] Programs with .HU and .HC extensions can be selected and processed in all operating modes The functions SELECT PROGRAM and CALL SELECTED PROGRAM have been … (1096950-22)

**77185x-04**

- [changed] New and changed cycle functions of software 77185x-04 New Cycle 258 POLYGON STUD, see "POLYGON STUD (Cycle 258, DIN/ISO: G258)", page 159 Cycle 225 has been … (1096959-22)
- [changed] New cycle 258 POLYGON STUD Cycles 421, 422 and 427 were expanded by the parameters Q498 and Q531 For cycle 247: SET DATUM, the datum number from the preset … (1096983-22)

**77185x-05**

- [changed] New Cycle 441 FAST PROBING. With this cycle you can set various touch probe parameters (e.g. positioning feed rate) that are globally effective for all … (1096983-23)
- [new] New FUNCTION COUNT function for controlling a counter, see "Defining a counter", Page 349 New FUNCTION LIFTOFF function for retracting the tool from the contour … (1096950-24)

**77185x-18**

- [changed] Basic Information | Control model, software and features 1 New functions The BLK FORM FILE function allows you to use STL files to define the workpiece … (1096983-25)
- [changed] Basic Information | Control model, software and features 1 New functions ISO Programming The BLK FORM FILE function allows you to use STL files to define the … (1263173-21)
- other: Overview of new and modified software functions Further information ab · Cycle 224 DATAMATRIX CODE PATTERN (ISO: G224) This cycle allows the co

### C.TNC 410

No new-function list for the TNC 410. The shared DIN/ISO manual 345 940-22 (TNC 410, TNC 426, TNC 430) has one list, "New features of the NC software 280 476-xx" (PDF p6). Its own model table (PDF p5) maps 280 476/280 477 to the TNC 426/430 and 286 060/286 080 to the TNC 410, so the list is not a TNC 410 version step. Several entries say "not TNC 410" (cycles 209 and 247, M142, M143, M144). The same manual marks these as "not TNC 410" in its contents (PDF p17-19): cycles 205-209, 262-267, SL II (20-28), 19 (G80), 32 (G62), 247, 60, M116, M118, M128, M136, M138, tilting. The 280 476 entries are under C.TNC 426 TNC 430.

### C.TNC 426 TNC 430


**280476-xx**

- [new] Tapping Cycle 209 with chip breaking (see “TAPPING WITH CHIP BREAKING (Cycle 209)” on page 233) (340_135-22)
- [new] Cycle 247(see “DATUM SETTING (Cycle 247)” on page 324) (340_135-22)
- [new] M140 (see “Retraction from the contour in the tool-axis direction: M140” on page 188) (340_135-22)
- [new] M141 (see “Suppressing touch probe monitoring: M141” on page 189) (340_135-22)
- [new] M142 (see “Delete modal program information: M142” on page 190) (340_135-22)
- [new] M143 (see “Delete basic rotation: M143” on page 190) (340_135-22)
- [new] M144 (see “Compensating the machine’s kinematic configuration for ACTUAL/NOMINAL positions at end of block: M144” on page 197) II (340_135-22)
- other: Cycles for automatic tool measurement with the TT 130 according to ISO · All cycles for the automatic datum setting can now also be run during  · Thread milling …

### C.iTNC 530


**340420-xx**

- [new] Automatic cutting data calculation in ISO programs (see (369_475-21)
- [new] Structuring ISO programs (see “Structuring Programs” on page 104) (369_475-21)
- other: Find/Replace any text · New Q parameter functions: Check sign and Calculate modulo value when  · Plunging feed rate for infeed for finishing new in Cycles 210 …

**340422-xx (added by hand; the script did not pick up this list)** - from 375 738-24 p7 and p10:

- [new] Datum management via the preset table; new cycles 251 RECTANGULAR POCKET, 252 CIRCULAR POCKET, 253 SLOT MILLING, 254 CIRCULAR SLOT; CYCL CALL POS; cycle 205 gets …
- [changed] Cycle 7 with datum tables: REF-based datums no longer available; preset table instead. "Cycle 247 now activates a preset from the preset table". "The old …

**34049x-01**

- [new] The tooth feed fz and feed per revolution fu can now be defined as alternate feed entries (670391-23)
- [new] New M function M150 for suppressing limit switch messages (see "Suppressing the limit switch message: M150" on page 342) (670391-23)
- [new] M128 is now also permitted for mid-program startup (see "Mid- program startup (block scan)" on page 549) (670391-23)
- [new] The number of available label numbers was expanded to 1000. Now label names can be assigned as well (see "Labeling subprograms 340422-xx/340423-xx and program … (670391-23)
- [new] In the Q parameter functions FN9 to FN12 you can now also assign label names as jump targets (see "If-then decisions with Q parameters" on page 283) (670391-23)
- [changed] The entry range for the EULPR precession angle in the PLANE EULER function was expanded (see "Defining the machining plane with Euler angles: EULER PLANE" on … (670391-23)
- [changed] The plane vector in the VECTOR PLANE function no longer has to be entered in standardized form (see "Defining the working plane with two vectors: VECTOR … (670391-23)
- [changed] Positioning behavior of the CYCL CALL PAT function has been (670391-23)
- [changed] The plane vector in the PLANE EULER function no longer has to be entered in standardized form (see “Defining the Machining Plane with Two Vectors: VECTOR … (bhb)
- [changed] Positioning behavior of the CYCL CALL PAT function modified (see (bhb)
- [new] Suppress limit switch message: M150” on page 358) (670_387-20)
- [new] Suppress limit switch message: M150” on page 318) (670_391-20)
- [new] Suppress limit switch message: M150” on page 376). (670_387-21)
- [new] Suppress limit switch message: M150” on page 330). (670_391-21)
- [new] Suppress limit switch message: M150” on page 386). (670_387-22)
- [new] Suppress limit switch message: M150” on page 340). (670_391-22)
- other: Several columns were added to the tool table · The tool types available for selection in the tool table were increase · predecessor versions 340 422-xx and 340 …

**34049x-02**

- [changed] In inch-programs, the function M136 (feed rate in 0.1 inch/rev) can no longer be combined with the FU function (670391-23)
- other: A new cycle that enables you to set global touch probe parameters was  · The active tool-axis direction can now be set as the active machining  · Instead of the …

**34049x-03**

- [new] In the PLANE function the working plane can now also be defined directly by its axis angle (see "Tilting the working plane through axis angle: PLANE AXIAL (FCL 3 … (670391-23)
- [new] In Cycle 22 ROUGH-OUT, you can define a feed-rate reduction if the tool is cutting on its entire circumference (FCL3 function, see User's Manual for Cycles) (670391-23)
- [new] In Cycle 208 BORE MILLING, you can now choose between climb or (670391-23)
- [changed] In Cycle 22 you can now define a tool name also for the coarse (670391-23)
- [changed] In the PLANE function, an FMAX can now be programmed for the automatic rotary positioning (see "Automatic positioning: MOVE/TURN/STAY (entry is mandatory)" … (670391-23)
- [changed] Arcs that are not programmed in the active working plane can now also be run as spatial arcs (see "Circular path C around circle center CC" on page 221) (670391-23)
- [changed] The tolerance values set in Cycle 32 are displayed (670391-23)
- [new] Now, in Cycle 22 ROUGH-OUT, you can define a feed-rate reduction if the tool is cutting on its entire circumference (FCL3 function, see (bhb)
- [new] In Cycle 208 BORE MILLING, you can now choose between climb or up-cut milling (see “BORE MILLING (Cycle 208)” on page 349). (bhb)
- [changed] In Cycle 22 you can now define a tool name also for the coarse roughing tool (see “ROUGH-OUT (Cycle 22)” on page 435). (bhb)
- [new] Now, in Cycle 22 ROUGH-OUT, you can define a feed-rate reduction if the tool is cutting with its entire circumference (FCL3 function, see (533_190-23)
- [changed] In Cycle 22 you can now define a tool name for the coarse roughing tool as well (see “ROUGH-OUT (Cycle 22)” on page 456). (533_190-23)
- other: New cycle for setting a datum in the center of a slot · New cycle for setting a datum in the center of a ridge · New 3-D probing cycle · Cycle 401 now also …

**34049x-04**

- [new] Cycle 3: Error message can be suppressed if the stylus is already deflected when a cycle starts (see "MEASURING (Cycle 3)" on page 463) (670388-23_SP7)
- [new] Machining patterns can now easily be defined with PATTERN DEF (670391-23)
- [new] Now, in Cycle 209 TAPPING WITH CHIP BREAKING, you can define a factor for the retraction shaft speed, so that you can depart the hole (670391-23)
- [new] (see "Automatic retraction of tool from the contour at an NC stop: M148" on page 341) (670391-23)
- [changed] The input range for polar angles was increased (see "Circular path G12/G13/G15 around pole I, J" on page 231) (670391-23)
- [changed] The pocket-, stud- and slot-milling cycles 210 to 214 were removed from the standard soft-key row (CYCL DEF > POCKETS/STUDS/SLOTS). For reasons of … (670391-23)
- [new] The maximum path permitted for LIFTOFF was increased to 30 mm (see “Automatically retract tool from the contour at an NC stop: M148” on page 321) (533_190-23)
- other: New cycle for saving a machine's kinematic configuration · New cycle for testing and optimizing a machine's kinematic configurati · Cycle 412: Number of …

**34049x-05**

- [new] New machining cycle for single-lip deep-hole drilling (see "SINGLE- LIP DEEP-HOLE DRILLING (Cycle 241, DIN/ISO: G241)" on page 102) (670388-23_SP7)
- [new] Global program settings: Function also partially effective with M91/M92 blocks (see "Global Program Settings (software option)" on page 368) (670391-23)
- [new] M116 allowed in combination with M128 (see "Feed rate in mm/min on rotary axes A, B, C: M116 (software option 1)" on page 432) (670391-23)
- [new] Introduction of local and nonvolatile Q parameters QL and QR (see "Principle and overview" on page 274) (670391-23)
- [new] New Cycle 241 for Single-Fluted Deep-Hole Drilling (see User’s Manual for Cycles) (670391-23)
- [new] SINGLE-FLUTED DEEP-HOLE DRILLING (Cycle 241, DIN/ISO: G241)” on page 94) (670_388-20)
- [new] SINGLE-FLUTED DEEP-HOLE DRILLING (Cycle 241, DIN/ISO: G241)” on page 98) (670_388-21)
- other: The cylindrical surface cycles 27, 28, 29 and 39 can now also be used  · Cycle 403 does not check whether touch points and compensation axis ma · Pallet preset …

**34049x-06**

- [new] In Cycle 241 "Single-Lip Deep-Hole Drilling" it is now possible to define a dwell depth (see "SINGLE-LIP DEEP-HOLE DRILLING (Cycle 241, DIN/ISO: G241)" on page … (670388-23_SP7)
- [changed] New software option 3-D ToolComp: 3-D tool radius compensation depending on the tool’s contact angle on blocks with surface normal vectors (LN blocks) (670391-23)
- [changed] In the Test Run mode, the working plane can now by defined manually (see "Setting a tilted working plane for the test run" on page 542) (670391-23)
- [changed] New DR2TABLE column for definition of a compensation table for tool radius compensation depending on the tool’s contact angle (670391-23)
- [changed] New LAST_USE column, into which the TNC enters the date and time of the last tool call (670391-23)
- [changed] Q parameter programming: QS string parameters can now also be used for jump addresses of conditional jumps, subprograms or program section repeats (see … (670391-23)
- [changed] In the positioning mode TURN of the PLANE function you can now (670391-23)
- [changed] In Cycle 241 "Single-Fluted Deep-Hole Drilling" it is now possible to (670391-23)
- [changed] During tilting in the Distance-To-Go mode, the PLANE function now shows the angle actually left to be traversed until the target position (see "Position … (670391-23)
- [changed] In the Test Run mode, the working plane can now by defined (670387-23)
- [changed] manually (see "Setting a tilted working plane for the test run" on page 630) (670387-23)
- [changed] In the positioning mode TURN of the PLANE function you can now define a clearance height to which the tool is to be retracted before tilting to tool axis … (670387-23)
- [new] On machines without encoders in the rotary axes, the rotary axis coordinates to define the virtual axis direction VT can now be specified via M114 (see “Virtual … (670_387-21)
- [new] In Cycle 241 "Single-Fluted Deep-Hole Drilling" it is now possible to define a dwell depth (see “SINGLE-FLUTED DEEP-HOLE DRILLING (Cycle 241, DIN/ISO: G241)” on … (670_388-21)
- [changed] The approach behavior during side finishing with Cycle 24 (DIN/ISO: G124) was changed (see “Please note while programming:” on page 199) (670_388-21)
- other: New Cycle 275 "Trochoidal Contour Slot" · The approach and departure behavior of Cycle 39 "Cylinder Surface Cont · KinematicsOpt: An additional parameter for …

**34049x-07**

- [new] After angle optimization, a separate measurement series is no longer required for position optimization (see "Various modes (Q406)" on page 492) (670388-23_SP7)
- [new] Up to eight plane measuring points for the measurement of spheres (see "Cycle parameters" on page 489) (670388-23_SP7)
- [new] Rotary axes that are not configured are ignored by TNC when executing the cycle(see "Please note while programming:" on page 488) (670388-23_SP7)
- [new] In the Manual Operation mode you can now move the axes even if TCPM and Tilt Working Plane are active at the same time (670391-23)
- [new] You can now also change tools when M128/FUNCTION TCPM is active (670391-23)
- [new] If the Swapping Axes function is active, it is now permitted to position to machine-based positions on the axes that have not been swapped (670391-23)
- [new] The input range of the delta values DL, DR and DR2 have been increased to 999.9999 mm (see "Tool table: Standard tool data" page 171 ff) (670391-23)
- [new] Importing of tool data in CSV format (see "Importing tool data" on page 198) (670391-23)
- [new] It is no longer necessary to run a separate measurement series for position optimization after angle optimization (670391-23)
- [new] More plane measuring points for ball measurement (670391-23)
- [changed] M124 can now be deactivated by entering M124 without T (see "Do not include points when executing non-compensated line blocks: M124" on page 328) (670391-23)
- [new] Using the new SEL PGM function you can assign variable program names via QS string parameters call them with CALL SELECTED (see "Defining a program call" on page … (670387-23)
- [new] In manual mode, you can now also travel the axes again when TCPM and Tilt Machining Plane are active at the same time (670_387-22)
- [new] Rotary axes that are not configured are ignored by the TNC during the execution of the cycle (see “Please note while programming:” on page 486) (670_388-22)
- other: New Cycle 225 Engraving · New Cycle 276 3-D Contour Train · New Cycle 290 Interpolation Turning · For the thread milling cycles (Cycles 26x), a separate feed …

**34049x-08**

- [new] The SEQ switch of the PLANE function can now also be assigned a Q parameter (see "Selection of alternate tilting possibilities: SEQ +/– (entry optional)" on page … (670391-23)
- [new] The limit plane function is now available (see "Limit plane" on page 379) (670391-23)
- [new] New function REFRESH VIEW for reinitializing a view if the data stock is inconsistent (see "Operating the tool management" on page 195) (670391-23)
- [new] New function "Fill in the table" during the import of tool data (see "Importing tool data" on page 198) (670391-23)
- [changed] The values in the datum table are now displayed in inches if the position display is set to INCH (see "Datum management with the preset table" on page 483) (670391-23)
- other: With Cycle 256, rectangular stud, a parameter is now available with wh · With Cycle 257, circular stud milling, a parameter is now available wi · Machine …

**60642x-01**

- [changed] New software option 3-D ToolComp: 3-D tool radius compensation depending on the tool’s contact angle on blocks with surface normal vectors (LN blocks) (737760-22)
- [changed] Cycle 206: The TNC now monitors the thread pitch if entered in the tool table (670388-25)
- [changed] Cycle 207: The TNC now monitors the thread pitch if entered in the tool table (670388-25)
- [changed] Cycle 209: The TNC now monitors the thread pitch if entered in the tool table (670388-25)
- [changed] Cycle 209: The TNC now completely retracts the tool from the hole during chip breaking if parameter Q256 is defined to be 0 (retraction distance for chip … (670388-25)
- [changed] Cycle 202: The TNC now does not retract the tool from the hole bottom if parameter Q214 is defined to be 0 (disengaging direction) (670388-25)
- [changed] Cycle 405: The TNC now also writes the datum in line 0 of the datum table if the parameter Q337=0 is defined (670388-25)
- [new] In the Test Run mode, the working plane can now be defined manually (see "Setting a tilted working plane for the test run" on page 564) (737760-24)
- [new] In Cycle 241 "Single-Lip Deep-Hole Drilling" it is now possible to define a dwell depth (siehe „SINGLE-LIP DEEP-HOLE DRILLING (Cycle 241, DIN/ISO: G241)” auf … (670388-24)
- [changed] Cycle 209: The TNC now retracts the tool completely from the hole for chip breaking if the parameter Q256=0 is defined (retraction rate for chip breaking) (670388-24)
- [changed] Cycle 202: The TNC does not retract the tool for disengaging at the bottom of the hole if the parameter Q214=0 is defined (disengaging direction) (670388-24)
- other: Improvements regarding functional safety FS (option): · General information on functional safety (FS) · Checking the axis positions · Activating feed-rate …

**60642x-02**

- [new] Up to eight plane measuring points for ball measurement (see "Cycle parameters" on page 493) (670388-25)
- [new] Rotary axes that are not configured are ignored by the TNC when executing the cycle (see "Please note while programming:" on page 492) (670388-25)
- [new] The input range of the delta values DL, DR and DR2 was increased to 999.9999 mm (see "Tool table: Standard tool data" page 176 ff) (737760-24)
- [new] Up to eight plane measuring points for the measurement of spheres (siehe „Cycle parameters” auf Seite 491) (670388-24)
- [new] Rotary axes that are not configured are ignored by the TNC when executing the cycle (siehe „Please note while programming:” auf Seite 490) (670388-24)
- [new] The input range of the delta values DL, DR and DR2 has been increased to 999.9999 mm (see "Tool table: Standard tool data" page 175 ff) (737760-23)
- other: New function for opening 3-D data (software option) directly on the TN · In the thread milling cycles 26x a separate feed rate is now available · New fixed …

**60642x-03**

- [new] Cycle 200: Input parameter Q359 added to allow definition of the depth reference (see "DRILLING (Cycle 200)" on page 75) (670388-25)
- [new] Cycle 203: Input parameter Q359 added to allow definition of the depth reference (see "UNIVERSAL DRILLING (Cycle 203, DIN/ISO: G203)" on page 83) (670388-25)
- [new] Cycle 205: Input parameter Q208 for retraction feed rate added (see "UNIVERSAL PECKING (Cycle 205, DIN/ISO: G205)" on page 91) (670388-25)
- [new] Cycle 205: Input parameter Q359 added to allow definition of the depth reference (see "UNIVERSAL PECKING (Cycle 205, DIN/ISO: G205)" on page 91) (670388-25)
- [new] Cycle 200: Input parameter Q359 added to allow definition of the depth reference (siehe „DRILLING (Cycle 200)” auf Seite 73) (670388-24)
- [new] Cycle 203: Input parameter Q359 added to allow definition of the depth reference (siehe „UNIVERSAL DRILLING (Cycle 203, DIN/ISO: G203)” auf Seite 81) (670388-24)
- [new] Cycle 205: Input parameter Q208 for retraction feed rate added (siehe „UNIVERSAL PECKING (Cycle 205, DIN/ISO: G205)” auf Seite 89) (670388-24)
- [new] Cycle 205: Input parameter Q359 added to allow definition of the depth reference (siehe „UNIVERSAL PECKING (Cycle 205, DIN/ISO: G205)” auf Seite 89) (670388-24)
- other: New software option Active Chatter Control (ACC) · The new column ACC was introduced · Cycle 25: Automatic identification of residual material added · Cycle …

**60642x-04**

- [new] Tool names in the TOOL CALL block can now also be called via QS string parameters (see "Calling tool data" on page 191) (737760-24)
- [new] Cycle 200: Input parameter Q359 added to allow definition of the (737760-24)
- [new] Cycle 203: Input parameter Q359 added to allow definition of the (737760-24)
- [new] Cycle 205: Input parameter Q359 added to allow definition of the (737760-24)
- [new] In the Test Run mode, the working plane can now be defined (737760-23)
- [new] Tool names in a TOOL CALL can now also be called via QS string parameters (see "Calling tool data" on page 190) (737760-23)
- [new] Cycle 200: Input parameter Q359 for specifying the depth reference (737760-23)
- [new] Cycle 203: Input parameter Q359 for specifying the depth reference (737760-23)
- [new] Cycle 205: Input parameter Q208 for the retraction feed rate added (737760-23)
- [new] Cycle 205: Input parameter Q359 for specifying the depth reference (737760-23)
- other: The following new columns are available in the tool table: · Column OVRTIME: Definition of the maximum possible violation of the to · Column P4: Possibility of …

**60642x-04SP8**

- other: With Cycle 253, Slot Milling, a parameter is now available with which  · With Cycle 254, Circular Slot, a parameter is now available with which

### C.TNC 620


**34056x-02**

- [new] The PLANE function for flexible definition of a tilted working plane was introduced (see “The PLANE Function: Tilting the Working Plane (Software Option 1)” on … (679_351-20)
- [new] The FUNCTION PARAX function for defining the behavior of the parallel axes U, V and W was introduced (see “Working with the Parallel Axes U, V and W” on page317) (679_351-20)
- [new] The PATTERN DEF function for defining point patterns was introduced (679_351-20)
- [new] With the CYCL CALL PAT function, cycles can now be run in (679_351-20)
- [new] New Cycle 241 for Single-Fluted Deep-Hole Drilling was introduced (679_351-20)
- [new] The definition of feed rates in fixed cycles can now also include FU (679_351-20)
- [changed] In Cycle 22 you can now define a tool name also for the coarse (679_351-20)
- [changed] The tolerance values set in Cycle 32 are displayed (679_351-20)
- [changed] The pocket-, stud- and slot-milling cycles 210 to 214 were removed from the standard soft-key row (CYCL DEF > POCKETS/STUDS/SLOTS). For reasons of … (679_351-20)
- [new] The PATTERN DEF function for defining patterns was introduced (see (679_295-20)
- [new] Pattern Definition PATTERN DEF” on page 44) (679_295-20)
- [new] With the CYCL CALL PAT function, cycles can now be run in connection with point tables (see “Calling a cycle in connection with point tables” on page 55) (679_295-20)
- [new] SINGLE-LIP D.H.DRLNG (Cycle 241, DIN/ISO: G241, Advanced Programming Features Software Option)” on page 84) (679_295-20)
- [new] The definition of feed rates in fixed cycles can now also include FU and FZ values (679_295-20)
- [new] The PLANE function for flexible definition of a tilted working place (679_295-20)
- [new] The FUNCTION PARAX function for defining the behavior of the parallel Conversational Programming) (679_295-20)
- [changed] In Cycle 22 you can now define a tool name also for the coarse roughing tool (see “ROUGH-OUT (Cycle 22, DIN/ISO: G122, Advanced Programming Features Software … (679_295-20)
- [new] New Cycle 241 for Single-Lip Deep-Hole Drilling was introduced (679_351-21)
- [new] New machining cycle for single-lip deep-hole drilling (see "SINGLE- LIP D.H.DRLNG (Cycle 241, DIN/ISO: G241, Advanced Programming Features Software Option)" on … (679_295-21)
- other: Individual characters can now be deleted by using the backspace key · The SEL PATTERN function makes it possible to select point tables · The DECLARE CONTOUR …

**34056x-03**

- [new] The M101 function was introduced (see "Automatic tool change if the tool life expires: M101" on page 152). (679_351-21)
- [new] Tool tables of the iTNC 530 can now be imported in the TNC 620 and converted to a valid format (see "Importing tool tables" on page 146). (679_351-21)
- [new] The CYCL CALL POS function was introduced (see User's Manual for Cycles). (679_351-21)
- [new] Local and nonvolatile QL and QR Q parameters were introduced (see "Principle and Overview" on page 230). (679_351-21)
- [new] The M138 "Selection of swivel axes" function was introduced (see "Selecting tilting axes: M138" on page 371). (679_351-21)
- [new] The TCPM function was introduced (see "File Functions" on page 333). Changed Functions of Software 340 56x-03 (679_351-21)
- [new] Touch probe cycles can now also be used in the tilted working plane (679_351-21)
- [new] The function M101 was introduced (see User’s Manual for Conversational Programming). (679_295-21)
- [new] Tool tables of the iTNC 530 can now be imported in the TNC 620 and Programming). (679_295-21)
- [new] The function M138 for selecting tilting axes was introduced (see (679_295-21)
- other: File functions were introduced · The "Define coordinate transformations" function was introduced · The LAST_USE column was added to the tool table · The tool …

**73498x-02**

- [new] CAD Import (option 42) The DXF Converter allows you to open DXF files on the third desktop. You can set a preset and extract contours or machining positions. The … (New_and_Changed_Functions__1322094_02_A_ p18)
- [new] Machining cycles Cycle 225 ENGRAVING This cycle allows you to engrave a text in one plane. You can arrange the texts in a straight line or along an arc. (New_and_Changed_Functions__1322094_02_A_ p18)
- [new] Touch probe cycles ( option 17) Cycle 460 CALIBRATION OF TS ON A SPHERE (ISO: G460) This cycle enables the automatic calibration of a 3-D touch probe using a … (New_and_Changed_Functions__1322094_02_A_ p19)
- [new] Miscellaneous functions M197 allows you to avoid the rounding of outside contour corners at high contour speeds. (New_and_Changed_Functions__1322094_02_A_ p19)
- [new] Program functions Many cycles use the same cycle parameters (e.g., Q208 RETRACTION FEED RATE). In the NC program, you can define default values for some of these … (New_and_Changed_Functions__1322094_02_A_ p19)
- [new] Programming Q parameters Freely definable tables (*.tab) allow you to create your own tables for defining measuring positions, for example. You can access the … (New_and_Changed_Functions__1322094_02_A_ p20)
- [new] NC fundamentals The maximum number of characters for the tool name in TOOL DEF blocks (ISO: T) or TOOL CALL blocks (ISO: G51) has been increased from 16 to 32. … (New_and_Changed_Functions__1322094_02_A_ p21)
- [new] Machining cycles The parameter Q437 APPROACH POSITION has been added to Cycle 256 RECTANGULAR STUD (ISO: G256, option 19). This parameter allows you to define … (New_and_Changed_Functions__1322094_02_A_ p21)
- [new] Touch probe cycles ( option 17) After optimizing the angular orientation with Cycle 451 MEASURE KINEMATICS (ISO: G451, option 48), a separate measurement series … (New_and_Changed_Functions__1322094_02_A_ p21)
- [changed] Program functions When programming the PLANE functions (option 8), you can also define SEQ by means of a Q parameter. If a value of 0 is entered in the Q … (New_and_Changed_Functions__1322094_02_A_ p22)
- [changed] Positioning w/ Manual Data Input operating mode If an NC block contains a tool radius compensation in Positioning w/ Manual Data Input operating mode, then … (New_and_Changed_Functions__1322094_02_A_ p22)
- [changed] Status displays If you use a spindle as a C axis, then the control displays the axis letter C instead of S2 in the distance-to-go display of this spindle. In … (New_and_Changed_Functions__1322094_02_A_ p22)
- [changed] Moving the machine axes You can use handwheel superimpositioning with M118 (option 21) when FUNCTION TCPM (option 9) is active. (New_and_Changed_Functions__1322094_02_A_ p22)
- [changed] Manual touch probe cycles (option 17) You can also use manual touch probe cycles in conjunction with mechanical probes or milling cutters. The ACTUAL … (New_and_Changed_Functions__1322094_02_A_ p23)
- [changed] Tilting the working plane (option 8) You can define up to three tilt angles in the 3-D ROT menu. You define the rotation using the spatial angles SPA, SPB … (New_and_Changed_Functions__1322094_02_A_ p23)
- other: The maximum number of characters for the NAME and DOC fields in the to · 1 Software 73498x-02 | New software options · Remote Desktop Manager (option 133) …

**81760x-01**

- [new] The character set of the fixed cycle 225 Engraving was expanded by more characters and the diameter sign see "ENGRAVING (Cycle 225, DIN/ISO: G225)", page 284 New … (1096886-20)
- [new] Conversational languages The Additional Language software option (option 41) is no longer available. The following conversational languages are also available … (New_and_Changed_Functions__1322094_02_A_ p26)
- [new] NC fundamentals The BLK FORM CYLINDER function allows you to define a cylindrical workpiece blank by entering the radius, length, and an optional oversize. If … (New_and_Changed_Functions__1322094_02_A_ p26)
- [new] Touch probe cycles ( option 17) Cycle 4 MEASURING IN 3-D This cycle allows you to determine any position on the workpiece. You define the direction of approach … (New_and_Changed_Functions__1322094_02_A_ p27)
- [new] Programming aids The PROGRAMMING AIDS soft key has been added to the soft-key row for special functions. This soft key opens a menu where you can add a comment … (New_and_Changed_Functions__1322094_02_A_ p27)
- [new] Tables If a table column is assigned the UPTEXT column type, then the control will convert the entered text to uppercase. (New_and_Changed_Functions__1322094_02_A_ p28)
- [new] MOD functions The Graphic settings group that allows you to modify the 3-D motion simulation has been added to the MOD menu. You can choose between the available … (New_and_Changed_Functions__1322094_02_A_ p28)
- [new] NC fundamentals If you use the ACTUAL POSITON CAPTURE key to create a linear block containing the current axis positions, then the linear block contains only the … (New_and_Changed_Functions__1322094_02_A_ p28)
- [changed] Machining cycles The German umlauts and the diameter symbol have been added to the character set of Cycle 225 ENGRAVING (ISO: G225). In Cycle 22 ROUGH-OUT … (New_and_Changed_Functions__1322094_02_A_ p29)
- [changed] DXF Converter (option 42) You can add folders to bookmarks within the file manager of the DXF Converter. If option 42 is active, then, by default, the … (New_and_Changed_Functions__1322094_02_A_ p30)
- [changed] Manual touch probe cycles (option 17) You can use the manual touch probe cycle Probe centerline to determine the centerline between two holes or studs. At … (New_and_Changed_Functions__1322094_02_A_ p30)
- [changed] Tables When you import a tool table from the iTNC 530, the control loads all supported tool types and assigns the tool type Milling cutter,MILL to all … (New_and_Changed_Functions__1322094_02_A_ p30)
- [changed] MOD functions In the Display settings MOD group, you select which information the control is to show in the position display of the general and additional … (New_and_Changed_Functions__1322094_02_A_ p30)
- [changed] Machine-specific user parameters The control supports searching for language-specific names. Certain machine parameters are available only after you have … (New_and_Changed_Functions__1322094_02_A_ p31)
- other: Fundamentals New and changed cycle functions of software · 2 Software 81760x-01 | New software options · Active Vibration Damping (AVD) (option 146) AVD dampens …

**81760x-02**

- [changed] overview of the optional Q parameters that have been added in this software version. You can decide for yourself whether you would like to define optional Q … (1096886-23)
- [new] Programs with .HU and .HC extensions can be selected and processed in all operating modes The functions SELECT PROGRAM and CALL SELECTED PROGRAM have been … (1096883-23)
- [new] NC fundamentals When you program in ISO programming mode using the alphabetic keyboard, the control automatically uppercases the first letter of an NC block. (New_and_Changed_Functions__1322094_02_A_ p34)
- [new] Machining cycles Cycle 270 CONTOUR TRAIN DATA (ISO: G270, option 19) This cycle allows you to add a type of approach, type of departure, and radius compensation … (New_and_Changed_Functions__1322094_02_A_ p34)
- [new] Program functions FUNCTION FEED DWELL allows you to program a cyclic dwell time. Use the parameters D-TIME and F-TIME to define the dwell time and the machining … (New_and_Changed_Functions__1322094_02_A_ p34)
- [new] Programming Q parameters The FN 18: SYSREAD functions (ISO: D18) have been enhanced: FN 18: SYSREAD (D18) ID52 NR1: pocket number of a tool NR2: magazine number … (New_and_Changed_Functions__1322094_02_A_ p35)
- [new] Manual touch probe cycles (option 17) The manual touch probe cycle Probe in plane allows you to measure the misalignment of a plane. After you have probed three … (New_and_Changed_Functions__1322094_02_A_ p35)
- [new] HEROS functions The option Allow USB storage devices has been added to the SELinux settings in the HEROS menu. If you deactivate this option, the control will … (New_and_Changed_Functions__1322094_02_A_ p36)
- [new] Machine-specific user parameters The machine parameter posAfterContPocket (no. 201007) allows you to define how the control moves to clearance height at the end … (New_and_Changed_Functions__1322094_02_A_ p36)
- [changed] NC fundamentals The dialog for shutting down the control has been modified. You define whether the control shuts down or restarts. In the tool call, you can … (New_and_Changed_Functions__1322094_02_A_ p37)
- [changed] Machining cycles The character set provided by Cycle 225 ENGRAVING (ISO: G225) has been expanded. You can engrave the CE mark, the letter ß, and the symbol … (New_and_Changed_Functions__1322094_02_A_ p37)
- [changed] Touch probe cycles ( option 17) The input range of parameter Q305 NUMBER IN TABLE has been extended from +2999 to +99 999 in touch probe cycles. The … (New_and_Changed_Functions__1322094_02_A_ p38)
- [changed] Miscellaneous functions The function M116 (option 8) causes the control to interpret the feed rate of a rotary axis as mm/min or 1/10 inches/min instead of … (New_and_Changed_Functions__1322094_02_A_ p38)
- [changed] File management The COPY FIELD soft key has been added to the Last files pop-up window. This soft key allows you to save the path of the marked file to the … (New_and_Changed_Functions__1322094_02_A_ p38)
- [changed] Program run The behavior of the control in Program run, single block operating mode has been modified. When executing point pattern cycles and the function … (New_and_Changed_Functions__1322094_02_A_ p38)
- [changed] Status displays The basic rotation icon has been modified to match the 3-D basic rotation icon. The control-in-operation icon is shown on the screen as long … (New_and_Changed_Functions__1322094_02_A_ p39)
- [changed] Tilting the working plane (option 8) The PLANE RESET function does not affect Manual operation mode. To deactivate an active tilting function in Manual … (New_and_Changed_Functions__1322094_02_A_ p39)
- [changed] Manual touch probe cycles (option 17) The number of positioning movements has been reduced in the manual calibration cycle Calibration of a TS in a ring. The … (New_and_Changed_Functions__1322094_02_A_ p39)
- other: 3 Software 81760x-02 | New software options · Extended Tool Management (option 93) The extended tool management prov · Renamed software options The following …

**81760x-03**

- [changed] New Cycle 258 POLYGON STUD (option number 19) Cycles 421, 422 and 427 have been expanded to include parameters Q498 and Q531 In Cycle 247: SET DATUM, the … (1096887-22)
- [changed] New and changed cycle functions of software 81760x-03 New Cycle 258 POLYGON STUD, (software option 19)see "POLYGON STUD (Cycle 258, DIN/ISO: G258, software … (1096886-23)
- [changed] New cycle 258 POLYGON STUD (option 19) Cycles 421, 422 and 427 have been expanded to include parameters Q498 and Q531 For cycle 247: SET DATUM, the datum … (1096887-23)
- [new] Program functions FUNCTION S-PULSE allows you to program a pulsing spindle speed (e.g., to avoid self-oscillations of the machine). The SCALE parameter defines … (New_and_Changed_Functions__1322094_02_A_ p42)
- [new] Programming Q parameters The FN 18: SYSREAD (ISO: D18) functions have been enhanced: FN 18: SYSREAD (D18) ID56 NR1: number of lines in the active tool table NR2: … (New_and_Changed_Functions__1322094_02_A_ p42)
- [new] Program run The control differentiates between an interrupted and a stopped NC program. When the control is in an interrupted state, it provides the following … (New_and_Changed_Functions__1322094_02_A_ p43)
- [new] Manual touch probe cycles (option 17) You can transfer the values measured in a manual touch probe cycle into the preset table by defining a line number in the … (New_and_Changed_Functions__1322094_02_A_ p43)
- [new] Tables The PATHNAME column type for tables has been added. For this type of column, you can use the SELECT soft key to select a file path from a pop-up window. … (New_and_Changed_Functions__1322094_02_A_ p43)
- [new] Machine-specific user parameters The optional machine parameter restoreCoolant (no. 201008) allows you to define whether the control restores the status of the … (New_and_Changed_Functions__1322094_02_A_ p43)
- [changed] NC fundamentals The functions APPR LT, APPR LCT, DEP LT, and DEP LCT position all of the required axes simultaneously to an auxiliary point. The functions … (New_and_Changed_Functions__1322094_02_A_ p44)
- [changed] Machining cycles A mirror image that was programmed using Cycle 8 MIRRORING (ISO: G28) is also effective for a subsequent tilting of the working plane … (New_and_Changed_Functions__1322094_02_A_ p44)
- [changed] Miscellaneous functions When tilted machining (option 8), M128, or FUNCTION TCPM (option 9) is active, the control saves the current positions of the rotary … (New_and_Changed_Functions__1322094_02_A_ p45)
- [changed] DXF Converter (option 42) DXF Converter outputs the FUNCTION MODE MILL machining mode as a comment. (New_and_Changed_Functions__1322094_02_A_ p45)
- [changed] Program run A block scan takes into account a feed rate defined with M116 (option 8) in mm/ min for rotary axes. The setting of the feed-rate potentiometer … (New_and_Changed_Functions__1322094_02_A_ p45)
- [changed] Positioning w/ Manual Data Input operating mode The function PLANE RESET (option 8) always positions the rotary axes to their home position, regardless of … (New_and_Changed_Functions__1322094_02_A_ p45)
- [changed] Status displays When M136 is active, the control displays the feed rate in mm/1 or inches/1 in the general status display. In Manual Operation mode, the … (New_and_Changed_Functions__1322094_02_A_ p45)
- [changed] Tables The preset table shows the columns SPA, SPB, and SPC, regardless of the active tool axis. The maximum input range of the columns SPA, SPB, and SPC of … (New_and_Changed_Functions__1322094_02_A_ p46)
- [changed] Tools When you edit a tool in the tool table or tool management, the only table line that is locked for other access is the line containing this tool. If … (New_and_Changed_Functions__1322094_02_A_ p46)
- [changed] MOD functions The control checks the values defined in the Traverse limits MOD function for their validity. If a value is not valid, the control issues an … (New_and_Changed_Functions__1322094_02_A_ p46)
- other: Machining cycles Cycle 258 POLYGON STUD (ISO: G258, option 19) This cy · Programming aids In Programming operating mode, the screen layout PROG · File …

**81760x-04**

- [changed] With Cycle 251 Rectangular pocket, M110 is now taken into account with circles compensated inwards if it is active during machining In the protocol of the … (1096887-23)
- [new] NC fundamentals The control supports programming of spatial arcs. Spatial arcs are defined based on three axes and are perpendicular to the working plane. (New_and_Changed_Functions__1322094_02_A_ p48)
- [new] Program functions FUNCTION DWELL allows you to program a dwell time in seconds or define the number of spindle revolutions for dwelling. (New_and_Changed_Functions__1322094_02_A_ p48)
- [new] Programming Q parameters The FN 0 function allows you to assign the Undefined status to Q parameters. The FN 9 function allows you to compare QS parameters and … (New_and_Changed_Functions__1322094_02_A_ p48)
- [new] Test run (option 20) Press the MEASURING soft key to have the control display the workpiece coordinates at the position of the mouse pointer in 3-D view and plan … (New_and_Changed_Functions__1322094_02_A_ p49)
- [new] Program run If program run has been canceled, the control saves the point of interruption (i.e., the last active NC block). The SELECT LAST soft key allows you … (New_and_Changed_Functions__1322094_02_A_ p49)
- [new] Positioning w/ Manual Data Input operating mode The Positioning w/ Manual Data Input operating mode has been enhanced. The following functions are supported: … (New_and_Changed_Functions__1322094_02_A_ p49)
- [new] Status displays When a pulsing spindle speed is active, the control shows a corresponding icon in the general status display. When an NC block has been executed … (New_and_Changed_Functions__1322094_02_A_ p50)
- [new] Moving the machine axes You can connect up to five portable handwheels to the control. You can use the axis keys of the control to select the axis to be … (New_and_Changed_Functions__1322094_02_A_ p50)
- [new] Tilting the working plane (option 8) In the 3-D ROT menu, the control shows the active machine kinematics. (New_and_Changed_Functions__1322094_02_A_ p50)
- [new] Machine-specific user parameters The optional machine parameter displayFont (no. 100812) allows you to reduce the font size of the machine operating modes to the … (New_and_Changed_Functions__1322094_02_A_ p50)
- [changed] NC fundamentals Cylindrical workpiece blanks can also be defined by means of a diameter. You can use the transition elements RND (ISO: G24) and CHF (ISO: … (New_and_Changed_Functions__1322094_02_A_ p51)
- [changed] Machining cycles Cycle 225 ENGRAVING (ISO: G225) has been enhanced. The parameter Q367 TEXT POSITION allows you to define the starting position based on the … (New_and_Changed_Functions__1322094_02_A_ p51)
- [changed] Touch probe cycles ( option 17) Cycles 460 CALIBRATION OF TS ON A SPHERE (ISO: G460), 462 CALIBRATION OF A TS IN A RING (ISO: G462), and 463 TS CALIBRATION … (New_and_Changed_Functions__1322094_02_A_ p52)
- [changed] Program functions FUNCTION TCPM (option 9) with selected AXIS SPAT takes into account an active mirror function that has been programmed with Cycle 8 … (New_and_Changed_Functions__1322094_02_A_ p52)
- [changed] Programming Q parameters If you determine the length of an undefined Q parameter by means of the function STRLEN, the control displays a value of –1. (New_and_Changed_Functions__1322094_02_A_ p52)
- [changed] Calling an external NC program If an NC program called with CALL PGM (ISO: %) includes an M2 or M30 function, then the control displays a warning. (New_and_Changed_Functions__1322094_02_A_ p52)
- [changed] Test run (option 20) If you select a new NC program or start it with the RESET + START soft key, the control resets the tool data. In BLANK IN WORK SPACE … (New_and_Changed_Functions__1322094_02_A_ p52)
- [changed] Program run If you modify the preset during a program run interruption, you can resume the NC program run only with a block scan or GOTO. Entering into an FK … (New_and_Changed_Functions__1322094_02_A_ p53)
- [changed] Pallets (option 22) While a pallet table is being run, you can edit it only by using the EDIT PALLET soft key. (New_and_Changed_Functions__1322094_02_A_ p53)
- [changed] Status displays On the Overview tab of the additional status display, the control displays no more than 32 characters in the LBL field. The control displays … (New_and_Changed_Functions__1322094_02_A_ p53)
- [changed] Manual touch probe cycles (option 17) The manual touch probe cycle Corner as preset allows you to probe both a circular stud and a hole on a straight line. … (New_and_Changed_Functions__1322094_02_A_ p53)
- [changed] Tilting the working plane (option 8) If you activate the Tilt working plane function in the 3-D ROT menu, then the control uses the COORD ROT transformation … (New_and_Changed_Functions__1322094_02_A_ p54)
- [changed] Tables When you open the preset table, the cursor is on the line of the active preset. When you import a tool table from the iTNC 530, the control correctly … (New_and_Changed_Functions__1322094_02_A_ p54)
- [changed] Tools You are allowed to use the special characters % and , in tool names. When you create a new tool, the columns L and R of the tool table are empty and … (New_and_Changed_Functions__1322094_02_A_ p54)
- [changed] Pallets While a pallet table is being run, you can edit it only by using the EDIT PALLET soft key. (New_and_Changed_Functions__1322094_03_A_ p53)
- other: New and changed cycle functions of software 81760x-04 The positions of · Programming aids The screen layout PROGRAM + SECTS is available in the · File …

**81760x-05**

- [new] New FUNCTION PROG PATH function for taking the entire tool radius into account in 3-D radius compensation, see "Interpretation of the programmed path", Page 433 … (1096883-26)
- [new] Renamed software options Option 42 was renamed as CAD Import. (New_and_Changed_Functions__1322094_02_A_ p56)
- [new] Machining cycles Cycle 276 THREE-D CONT. TRAIN (ISO: G276, option 19) This cycle allows you to machine a contour train of different depths. You define the … (New_and_Changed_Functions__1322094_02_A_ p56)
- [new] Program functions FUNCTION COUNT allows you to define a counter (e.g., for counting the number of produced workpieces) in the NC program. The control considers … (New_and_Changed_Functions__1322094_02_A_ p57)
- [new] Programming Q parameters In the function FN 16: F-PRINT (ISO: D16), you can use a Q or QS parameter to define the path of the source file and the output file. … (New_and_Changed_Functions__1322094_02_A_ p58)
- [new] Programming aids You can turn an NC block into a comment by pressing the INSERT COMMENT soft key or the < key of the alphabetic keyboard. You can turn a comment … (New_and_Changed_Functions__1322094_02_A_ p58)
- [new] CAD Viewer The CAD-Viewer can open STEP (*.stp and *.step) and IGES (*.igs and *.iges) files.CAD Import (option 42) additionally allows you to extract contours … (New_and_Changed_Functions__1322094_02_A_ p59)
- [new] Manual touch probe cycles (option 17) In the manual touch probe cycle Corner as preset, you can compensate for angular misalignment by rotating the rotary table. … (New_and_Changed_Functions__1322094_02_A_ p59)
- [new] Tables You can manually edit the line 0 of the preset table. The machine manufacturer can disable the setting of presets for individual axes. The SERIAL column … (New_and_Changed_Functions__1322094_02_A_ p60)
- [new] Tools You can search for a tool name in the tool table by entering the name using the alphabetic keyboard. After entering the first character, the control opens … (New_and_Changed_Functions__1322094_02_A_ p60)
- [new] Machine-specific user parameters The machine parameter decimalCharakter (no. 100805) allows you to select whether the control uses a period or a comma as the … (New_and_Changed_Functions__1322094_02_A_ p61)
- [new] NC fundamentals You can change the sorting sequence within the pop-up window for tool selection by clicking the header of a column. The sorting sequence and the … (New_and_Changed_Functions__1322094_02_A_ p61)
- [changed] Machining cycles Cycle 225 ENGRAVING (ISO: G225) allows you to engrave the current count. Cycles 256 RECTANGULAR STUD (ISO: G256) and 257 CIRCULAR STUD (ISO: … (New_and_Changed_Functions__1322094_02_A_ p62)
- [changed] The OLD CYCLS soft key and the associated cycles have been removed: Cycle 1 PECKING (ISO: G83) Cycle 2 TAPPING (ISO: G84) Cycle 3 SLOT MILLING (ISO: G74) … (New_and_Changed_Functions__1322094_02_A_ p63)
- [changed] Touch probe cycles ( option 17) The machine manufacturer can define whether the offset R-OFFS of the tool table is taken into account for a tool without … (New_and_Changed_Functions__1322094_02_A_ p63)
- [changed] Program functions In FUNCTION TCPM (option 9), you can define the tool reference point and the center of rotation. The control positions the tool to the tool … (New_and_Changed_Functions__1322094_02_A_ p63)
- [changed] Miscellaneous functions With M94, the control reduces the display value of all rotary axes to a value of less than 360°, even if the axes are not modulo … (New_and_Changed_Functions__1322094_02_A_ p63)
- [changed] Test run (option 20) The machine manufacturer can change the tool change position for Test Run operating mode. The control displays the tool in blue during … (New_and_Changed_Functions__1322094_02_A_ p64)
- [changed] File management The control automatically detects a connected USB device. The soft key for connecting a USB device has been removed. (New_and_Changed_Functions__1322094_02_A_ p64)
- [changed] Program run When returning to the contour, the last axis positioned by the control is, by default, the tool axis. If the tool tip is positioned below the … (New_and_Changed_Functions__1322094_02_A_ p64)
- [changed] Status displays The control displays the spindle speed with at least four digits in the general status display. This means if the spindle speed is less than … (New_and_Changed_Functions__1322094_02_A_ p64)
- [changed] Moving the machine axes In a tilted working plane (option 8), you can also home axes (e.g., a tool magazine) that are not activated in the active kinematic … (New_and_Changed_Functions__1322094_02_A_ p65)
- [changed] Tools When you place the cursor into an input field in the form view of the tool management, the control selects the complete field contents. If you enter a … (New_and_Changed_Functions__1322094_02_A_ p65)
- [changed] Error messages and help system When displaying error messages, the control sorts them first by priority and, second (within the same priority), by time. … (New_and_Changed_Functions__1322094_02_A_ p65)
- [changed] Remote Desktop Manager (option 133) In the Remote Desktop Manager, all of the functions that you previously had to call by clicking the right mouse button … (New_and_Changed_Functions__1322094_02_A_ p65)
- [changed] axes are programmed using one common axis and move synchronously. The OLD CYCLS soft key and the associated cycles have been removed: Cycle 1 PECKING (ISO: … (New_and_Changed_Functions__1322094-02.tx p68)
- other: New Cycle 441 FAST PROBING. With this cycle you can set various touch  · 6 Software 81760x-05 | New software options · Batch Process Manager (option 154) This …

**81760x-06**

- [new] NC fundamentals You can use the unlabeled key between PGM MGT and ERR to toggle between the operating modes. In the TOOL DEF (ISO: G51) function, you can define … (New_and_Changed_Functions__1322094_02_A_ p68)
- [new] Touch probe cycles ( option 17) The following cycles allow you to ascertain the exact workpiece position and compensate it as needed during program run. The … (New_and_Changed_Functions__1322094_02_A_ p69)
- [new] Program functions The PLANE functions (option 8) provide the alternative selection option SYM in addition to SEQ. If SYM is selected, the control calculates the … (New_and_Changed_Functions__1322094_02_A_ p69)
- [new] Programming Q parameters The following options have been added to the text file that defines the output format for FN 16: F-PRINT (ISO: D16): The character * … (New_and_Changed_Functions__1322094_02_A_ p69)
- [new] Positioning w/ Manual Data Input operating mode In Positioning w/ Manual Data Input operating mode, the HIDE soft key that enables you to skip NC blocks has been … (New_and_Changed_Functions__1322094_02_A_ p70)
- [new] Status displays In the general status display, the control displays a feed rate value of less than 10 with one decimal place and a feed rate value of less than 1 … (New_and_Changed_Functions__1322094_02_A_ p70)
- [new] Manual touch probe cycles (option 17) The manual probing functions take an active M128 or FUNCTION TCPM (option 9) into account. When using the manual touch … (New_and_Changed_Functions__1322094_02_A_ p71)
- [new] Tilting the working plane (option 8) The appearance of the 3-D ROT menu has been revised. You can individually activate the settings for Manual operation mode … (New_and_Changed_Functions__1322094_02_A_ p71)
- [new] Tools You define the tool material in the TMAT column of the tool table. In the CUTDATA column, you select the cutting data table for the cutting data … (New_and_Changed_Functions__1322094_02_A_ p71)
- [new] Machine-specific user parameters The machine parameter CfgProgramCheck (no. 129800) allows you to define for NC programs and pallet machining whether and when … (New_and_Changed_Functions__1322094_02_A_ p72)
- [new] NC fundamentals If, in a TOOL CALL block (ISO: T), you do not program a tool, but rather the same tool axis of the active tool, then the control no longer runs a … (New_and_Changed_Functions__1322094_02_A_ p72)
- [new] Machining cycles The parameter Q367 SURFACE POSITION has been added to Cycle 233 FACE MILLING (ISO: G233, option 19). This parameter allows you to define the … (New_and_Changed_Functions__1322094_02_A_ p72)
- [changed] Touch probe cycles ( option 17) Cycles 408 SLOT CENTER PRESET (ISO: G408) to 419 PRESET IN ONE AXIS (ISO: G419) take into account the machine parameter … (New_and_Changed_Functions__1322094_02_A_ p73)
- [changed] Program functions FUNCTION TCPM (option 9) supports peripheral milling (this means that it supports 3-D radius compensation with a constant position of the … (New_and_Changed_Functions__1322094_02_A_ p73)
- [changed] Programming Q parameters The functions M_CLOSE and M_TRUNCATE for FN 16: F-PRINT (ISO: D16) have the same effect as far as output to the screen is concerned. … (New_and_Changed_Functions__1322094_02_A_ p73)
- [changed] CAD Viewer CAD Import (option 42) outputs 2-D contours by default. CAD Import (option 42) outputs angles with PLANE SPATIAL instead of PLANE VECTOR. You can … (New_and_Changed_Functions__1322094_02_A_ p73)
- [changed] Manual touch probe cycles (option 17) If the axis angle does not match the tilt angle during manual probing, then the control no longer issues an error … (New_and_Changed_Functions__1322094_02_A_ p74)
- [changed] Tables If the active line of the preset table is reactivated, the control reads out the values again. The machine manufacturer defines which file types the … (New_and_Changed_Functions__1322094_02_A_ p74)
- [changed] Remote Desktop Manager (option 133) The setting Additional waiting time: has been added to the Switch-off/restart of a computer function for shutting down an … (New_and_Changed_Functions__1322094_02_A_ p74)
- [changed] HEROS functions The screensaver glideshow has been removed. (New_and_Changed_Functions__1322094_02_A_ p74)
- other: 7 Software 81760x-06 | New software options · State Reporting Interface (option 137) SRI offers an interface for acq · Monitoring of configured machine …

**81760x-07**

- [new] With the FUNCTION TCPM function, you can define a feed-rate limit of the compensation movements, see "FUNCTION TCPM (option 9)", Page 392 The FUNCTION TCPM … (1096887-27)
- [new] Using the compensation tables, the control can compensate for deviations in the tool coordinate system (T-CS) or working plane coordinate system (WPL-CS), also … (1096883-27)
- [changed] The control includes the QR parameters in the backup, see "Principle and overview of functions", Page 256 With the SQL EXECUTE and SQL SELECT SQL commands … (1096883-27)
- [changed] New point pattern cycle 224 DATAMATRIX CODE PATTERN for the creation of a DataMatrix code. New cycle 238 MEASURE MACHINE STATUS for monitoring machine … (1096883-27)
- [new] Programming or ISO Programming Using the compensation tables, the control can compensate for deviations in the tool coordinate system (T-CS) or working plane … (1263172-21)
- [new] OPC UA NC Server (options 56 to 61) OPC UA provides a standardized interface for secure data exchange between products independent of the manufacturer. … (New_and_Changed_Functions__1322094_02_A_ p76)
- [new] Display step The Display Step software option (option 23) is available in the standard control version. The display step of the axes is no longer limited to four … (New_and_Changed_Functions__1322094_02_A_ p76)
- [new] Machining cycles Cycle 224 DATAMATRIX CODE PATTERN (ISO: G224) This cycle allows the control to convert a text into a machine-readable Data Matrix code. The Data … (New_and_Changed_Functions__1322094_02_A_ p77)
- [new] Program functions FUNCTION CORRDATA allows you to activate a line of the compensation table. The compensation remains effective until the next tool change or the … (New_and_Changed_Functions__1322094_02_A_ p77)
- [new] Miscellaneous functions During retraction with M140 MB MAX, the control takes into account the safety clearances that can be defined by the machine manufacturer … (New_and_Changed_Functions__1322094_02_A_ p77)
- [new] Status displays The control displays a corresponding icon in the general status display when tool radius compensation is active. The control shows the POS HR tab … (New_and_Changed_Functions__1322094_02_A_ p79)
- [new] Moving the machine axes The HR 520 and HR 550 FS handwheels display the offset of handwheel superimpositioning from M118 (option 21). The handwheels also show … (New_and_Changed_Functions__1322094_02_A_ p79)
- [new] Tilting the working plane (option 8) The 3-D ROT: AXIS ANGLE SPA. ANGLE soft key has been added to the 3-D ROT menu. This soft key allows you to select whether … (New_and_Changed_Functions__1322094_02_A_ p79)
- [new] Tools Compensation tables allow you to compensate for tools during program run without having to edit the NC program or the tool tables. The compensation table … (New_and_Changed_Functions__1322094_02_A_ p79)
- [new] Machine-specific user parameters The optional machine parameter applyCfgLanguage (no. 101305) allows you to define whether the HEROS operating system adopts the … (New_and_Changed_Functions__1322094_02_A_ p80)
- [new] Machining cycles In Cycle 225 ENGRAVING (ISO: G225), you can program system variables that contain the following information for the current main program and the … (New_and_Changed_Functions__1322094_02_A_ p80)
- [new] Touch probe cycles ( option 17) If you run the Cycles 1410 PROBING ON EDGE (ISO: G1410), 1411 PROBING TWO CIRCLES (ISO: G1411), and 1420 PROBING IN PLANE (ISO: … (New_and_Changed_Functions__1322094_02_A_ p80)
- [new] Program functions You can limit the feed rate of the compensating movements in FUNCTION TCPM (option 9). (New_and_Changed_Functions__1322094_02_A_ p80)
- [changed] Miscellaneous functions If you program a precision stop of rotary axes using M134 or M135, the control no longer displays an error message. The control … (New_and_Changed_Functions__1322094_02_A_ p81)
- [changed] Programming Q parameters The FN 10 function also allows you to check QS parameters and texts for inequalities. You can use combined QS parameters in the SQL … (New_and_Changed_Functions__1322094_02_A_ p81)
- [changed] Test run (option 20) If you set the MEASURING soft key to ON, then the control displays the following additional information: Surface orientation of the … (New_and_Changed_Functions__1322094_02_A_ p81)
- [changed] Manual touch probe cycles (option 17) If, during probing, the axis angles do not match the tilt angles, then the Working plane is inconsistent menu allows … (New_and_Changed_Functions__1322094_02_A_ p82)
- [changed] MOD functions If the control is accessed from external, it displays a corresponding icon in the header. Limits that have been defined in the Traverse limits … (New_and_Changed_Functions__1322094_02_A_ p82)
- [changed] HEROS functions When user administration is active, the Liberating motion mode requires the NC.OPModeManual permission (i.e., at least the role of … (New_and_Changed_Functions__1322094_02_A_ p82)
- [new] HEIDENHAIN OPC UA NC Server (options 56 to 61) OPC UA provides a standardized interface for secure data exchange between products independent of the … (New_and_Changed_Functions__1322094-02.tx p84)
- [new] Touch probe cycles ( option 17) Cycle 238 MEASURE MACHINE STATUS (ISO: G238, option 155) This cycle allows the control to measure various components defined to … (New_and_Changed_Functions__1322094-02.tx p85)
- other: Reading system data", Page 285 · 8 Software 81760x-07 | New software options · Optimized contour cycles (option 167) The OCM cycles enable the trocho · Calling …

**81760x-08**

- [new] Overview of new and modified software functions Further information about the previous software versions is presented in the Overview of New and Modified … (1096887-28)
- [changed] Machining Cycles Cycle 225 ENGRAVING (ISO: G225) allows you to use system variables to engrave the number of the current week. Cycles 202 BORING (ISO: G202) … (1096887-28)
- [new] NC fundamentals The BLK FORM FILE function allows you to use STL files to define the workpiece blank, and optionally the finished part, by specifying the path of … (New_and_Changed_Functions__1322094_02_A_ p84)
- [new] Program functions FUNCTION MODE SET allows you to activate settings defined by the machine tool builder (e.g., changes to the range of traverse) from within the … (New_and_Changed_Functions__1322094_02_A_ p86)
- [new] Programming Q parameters In the mask file of the FN 16: F-PRINT (ISO: D16) function, you can define whether the control shows or hides blank lines for undefined … (New_and_Changed_Functions__1322094_02_A_ p86)
- [new] Test run (option 20) If you use TARGET in the BLK FORM FILE function to define a finished part, you can show or hide it by soft key in the Test Run operating … (New_and_Changed_Functions__1322094_02_A_ p87)
- [new] CAD Viewer When you double-click a layer in the CAD-Viewer, the control marks the first contour element of this layer. You can transfer the data from the CAD … (New_and_Changed_Functions__1322094_02_A_ p87)
- [new] Status displays If a feed-rate limit has been activated with the F MAX soft key, the control displays an exclamation mark after the feed-rate value in the … (New_and_Changed_Functions__1322094_02_A_ p87)
- [new] Error messages and help system If, after a hardware change or an update, an error occurs when the control is booting, the control will automatically open the … (New_and_Changed_Functions__1322094_02_A_ p88)
- [new] MOD functions A link to the Firewall settings HEROS function has been added to the External access MOD function. A link to the OPC UA NC Server - License … (New_and_Changed_Functions__1322094_02_A_ p88)
- [new] Remote Desktop Manager (option 133) Remote Desktop Manager (option 133) allows you to create private connections if user administration is active. Private … (New_and_Changed_Functions__1322094_02_A_ p88)
- [new] HEROS functions When user administration is active, the control automatically disables the LSV2 connections of the serial interfaces (COM1 and COM2) for security … (New_and_Changed_Functions__1322094_02_A_ p88)
- [new] NC fundamentals You can use the RND element (ISO: G24) to connect circles that are perpendicular to the machining plane instead of lying in the machining plane. (New_and_Changed_Functions__1322094_02_A_ p88)
- [changed] Use the parameter Q575 INFEED STRATEGY to define whether the control machines the contour from top to bottom or vice versa. The maximum input range of … (New_and_Changed_Functions__1322094_02_A_ p90)
- [changed] Touch probe cycles ( option 17) Cycles 480 CALIBRATE TT (ISO: G480) and 484 CALIBRATE IR TT (ISO: G484, option 17) can be used to calibrate a tool touch … (New_and_Changed_Functions__1322094_02_A_ p90)
- [changed] Miscellaneous functions The M109 function allows the control to keep the feed rate constant at the cutting edge, including during approach and departure. The … (New_and_Changed_Functions__1322094_02_A_ p90)
- [changed] Program run In Program run, single block operating mode, the control interprets the workpiece blank definition as a single NC block only. The control … (New_and_Changed_Functions__1322094_02_A_ p91)
- [changed] Moving the machine axes When you activate a handwheel with display, the control automatically activates the override potentiometer of the handwheel. In the … (New_and_Changed_Functions__1322094_02_A_ p91)
- [changed] Manual touch probe cycles (option 17) By default, the control calculates the basic rotation in the input coordinate system (I- CS). If the axis angles and … (New_and_Changed_Functions__1322094_02_A_ p91)
- [changed] HEROS functions If you enter a password or code number with Caps Lock active, then the control issues a message. The HEIDENHAIN OPC UA NC Server enables you … (New_and_Changed_Functions__1322094_02_A_ p92)
- other: 9 Software 81760x-08 | New software options · Renamed software options Option 146 has been renamed as Machine Vibrat · Machining cycles Cycle 277 OCM CHAMFERING …

**81760x-16**

- [changed] 1 Fundamentals | Control model, software, and features The parameters Q429 COOLANT ON and Q430 COOLANT OFF in Cycle 241 SINGLE-LIP D.H.DRLNG (ISO: G241 … (1303427-21)
- [new] Production data acquisition State Reporting Interface (software option 137) is no longer available. (New_and_Changed_Functions__1322094_02_A_ p94)
- [new] NC fundamentals Within circular paths C, CR and CT, the LIN_ syntax element is now available in order to superimpose a linear motion over the circular motion of … (New_and_Changed_Functions__1322094_02_A_ p94)
- [new] Program functions With the TRANS ROTATION function, you can rotate contours or positions around a rotation angle. The TRANS DATUM RESET function allows you to … (New_and_Changed_Functions__1322094_02_A_ p95)
- [new] Programming Q parameters Using the SYNTAX soft key, you can enclose path information in double quotation marks in order to use any special characters as part of … (New_and_Changed_Functions__1322094_02_A_ p95)
- [new] Calling an external NC program Using the SYNTAX soft key, you can enclose path information in double quotation marks in order to use any special characters as … (New_and_Changed_Functions__1322094_02_A_ p95)
- [new] File management Within the file manager, the control hides as well as files with a period at the start of the name. If necessary, you can display the files with … (New_and_Changed_Functions__1322094_02_A_ p95)
- [new] CAD Viewer In the CAD-Viewer, you can choose the machining planes YZ and ZX for milling. You use a selection menu to choose the machining plane. (New_and_Changed_Functions__1322094_02_A_ p95)
- [new] Status displays The TRANS tab of the additional status display contains the active shift in the working plane coordinate system WPL-CS. If the shift comes from a … (New_and_Changed_Functions__1322094_02_A_ p96)
- [new] NC fundamentals In order for the control to show the workpiece blank in the simulation, the workpiece blank must have minimum dimensions. The minimum dimensions … (New_and_Changed_Functions__1322094_02_A_ p96)
- [changed] Machining cycles Cycle 225 ENGRAVING (ISO: G225) has been enhanced: The parameter Q202 MAX. PLUNGING DEPTH allows you to define the maximum plunging depth. … (New_and_Changed_Functions__1322094_02_A_ p97)
- [changed] If a measurement in Cycle 238 MEASURE MACHINE STATUS (ISO: G238, option 155) was not performed correctly, for example at a feed rate override of 0 %, you can … (New_and_Changed_Functions__1322094_02_A_ p98)
- [changed] Touch probe cycles ( option 17) The Cycles 1420 PROBING IN PLANE (ISO: G1420), 1410 PROBING ON EDGE (ISO: G1410), 1411 PROBING TWO CIRCLES (ISO: G1411) have … (New_and_Changed_Functions__1322094_02_A_ p98)
- [changed] Programming Q parameters While program run is interrupted or has been canceled, you can edit Q and QS parameters with numbers 0 to 99, 200 to 1199 and 1400 … (New_and_Changed_Functions__1322094_02_A_ p98)
- [changed] File management If you exit an NC program by pressing the END key, the control opens the file manager. The cursor is on the NC program that was just closed. … (New_and_Changed_Functions__1322094_02_A_ p99)
- [changed] Manual touch probe cycles (option 17) If you use a manual touch-probe function to automatically probe a stud or hole with an opening angle of 360°, the … (New_and_Changed_Functions__1322094_02_A_ p99)
- [new] File management Within the file manager, the control hides system files as well as files and folders with a period at the start of the name. If necessary, you … (New_and_Changed_Functions__1322094_03_A_ p95)
- other: Cycles for Workpieces and Tools Cycle 1400 POSITION PROBING (ISO: G140 · 10 Software 81760x-16 | New software options · CAD Model Optimizer (option 152) This …

**81760x-17**

- [new] Machining cycles Cycle 277 OCM CHAMFERING (ISO: G277, option 167) monitors contour damage on the floor caused by the tool tip. This tool tip results from the … (New_and_Changed_Functions__1322094_02_A_ p103)
- [new] Touch probe cycles ( option 17) The log of Cycles 451 MEASURE KINEMATICS (ISO: G451) and 452 PRESET COMPENSATION (ISO: G452, option 48) contains diagrams with … (New_and_Changed_Functions__1322094_02_A_ p103)
- [changed] Test run (option 20) The simulation takes into account the following columns of the tool table: R_TIP LU RN The control takes into account the following NC … (New_and_Changed_Functions__1322094_02_A_ p104)
- [changed] CAD Viewer CAD-Viewer has been enhanced: Internally, CAD-Viewer always uses mm for its calculations. If you select inches as the unit of measure, CAD-Viewer … (New_and_Changed_Functions__1322094_02_A_ p104)
- [changed] Moving the machine axes If a handwheel is active, the control shows the contouring feed rate in the display during program run. If only the currently … (New_and_Changed_Functions__1322094_02_A_ p104)
- [changed] Tables The minimum input value of the FMAX column in the touch probe table has been changed from –9999 to +10. (New_and_Changed_Functions__1322094_02_A_ p104)
- [changed] Accessories The control no longer supports the ITC 750 additional operating station. (New_and_Changed_Functions__1322094_02_A_ p104)
- [changed] HEROS functions The HEROS tool Diffuse has been removed. In the Certificate and keys window you can select a file with additional public SSH keys in the … (New_and_Changed_Functions__1322094_02_A_ p105)
- [changed] Machine-specific user parameters The machine manufacturer uses the machine parameters allowUnsecureLsv2 (no. 135401) and allowUnsecureRpc (no. 135402) to … (New_and_Changed_Functions__1322094_02_A_ p105)
- [new] Programming Q parameters The FN 18: SYSREAD (ISO: D18) functions have been extended: FN 18: SYSREAD (D18) ID610 NR49: Mode of filter reduction of one axis (IDX) … (New_and_Changed_Functions__1322094_03_A_ p102)
- [changed] Test run (option 20) The simulation considers the following columns of the tool table: R_TIP LU RN The control considers the following NC functions in the … (New_and_Changed_Functions__1322094_03_A_ p104)
- [changed] HEROS functions The Diffuse HEROS tool was removed. In the Certificate and keys window you can select a file with additional public SSH keys in the … (New_and_Changed_Functions__1322094_03_A_ p105)
- other: Cycles for Workpieces and Tools Cycle 1416 INTERSECTION PROBING (ISO:  · Touch probe cycles ( option 17) Cycle 1416 INTERSECTION PROBING (ISO:  · Tables You …

**81760x-18**

- [new] Program functions Use the NC function TRANS RESET to reset all simple coordinate transformations simultaneously. (New_and_Changed_Functions__1322094_03_A_ p108)
- [new] Machine-specific user parameters In machine parameter safeAbsPosition (no. 403130), the machine manufacturer defines whether the SLP safety function is activated … (New_and_Changed_Functions__1322094_03_A_ p109)
- [new] Machining cycles You can also define subcontours as LBL subprograms within the complex SEL CONTOUR contour formula. The machine manufacturer can hide the cycles … (New_and_Changed_Functions__1322094_03_A_ p109)
- [changed] Touch probe cycles ( option 17) In the touch probe cycles 14xx, you can enter symmetrical tolerances "+-...." for the nominal dimensions. Cycle 441 FAST … (New_and_Changed_Functions__1322094_03_A_ p110)
- [changed] CAD Viewer CAD Import (option 42) subdivides contours that are not located in the working plane, into individual sections. CAD Viewer creates straight lines … (New_and_Changed_Functions__1322094_03_A_ p110)
- [changed] Test run (option 20) If a component has not been configured or cannot be monitored, the control displays the corresponding machining operation in gray in the … (New_and_Changed_Functions__1322094_03_A_ p110)
- [changed] Tables In the TABDATA WRITE, TABDATA ADD and FN 27: TABWRITE (ISO: D27) NC functions, you can enter values directly. If you create a table, and there is at … (New_and_Changed_Functions__1322094_03_A_ p111)
- [changed] HEROS functions The HEROS tool Diffuse has been added. You can compare and merge text files. The OPC UA NC Server has been enhanced as follows: The OPC UA NC … (New_and_Changed_Functions__1322094_03_A_ p111)
- [changed] Machine-specific user parameters Machine parameter CfgStretchFilter (no. 201100) has been removed. (New_and_Changed_Functions__1322094_03_A_ p111)
- other: Cycles Cycle 1274 OCM CIRCULAR SLOT (ISO: G1274, option 167) This cycl · 12 Software 81760x-18 | New software options · Pallet management The Pallet Management …

### C.TNC 640


**34059x-02**

- [new] Dynamic Collision Monitoring (DCM) (option 40) The machine manufacturer can define machine components and minimum distances that are to be monitored by the … (New_and_Changed_Functions__1322095_02_A_ p20)
- [new] CAD Import (option 42) The DXF Converter allows you to open DXF files on the third desktop. You can set a preset and extract contours or machining positions. The … (New_and_Changed_Functions__1322095_02_A_ p20)
- [new] Adaptive Feed Control (AFC) (option 45) AFC controls the contouring feed rate based on the spindle power. After a teach- in cut, AFC automatically adapts the … (New_and_Changed_Functions__1322095_02_A_ p20)
- [new] Machining cycles Cycle 225 ENGRAVING This cycle allows you to engrave a text in one plane. You can arrange the texts in a straight line or along an arc. Cycle … (New_and_Changed_Functions__1322095_02_A_ p21)
- [new] Touch probe cycles Cycle 460 CALIBRATION OF TS ON A SPHERE (ISO: G460) This cycle enables the automatic calibration of a 3-D touch probe using a calibration … (New_and_Changed_Functions__1322095_02_A_ p22)
- [new] Miscellaneous functions M197 allows you to avoid the rounding of outside contour corners at high contour speeds. (New_and_Changed_Functions__1322095_02_A_ p22)
- [new] Program functions FUNCTION TURNDATA BLANK (option 50) allows you to program the updating of the blank form for turning operations. The control automatically … (New_and_Changed_Functions__1322095_02_A_ p22)
- [new] Programming Q parameters Freely definable tables (*.tab) allow you to create your own tables for defining measuring positions, for example. You can access the … (New_and_Changed_Functions__1322095_02_A_ p23)
- [new] NC fundamentals The maximum number of characters for the tool name in TOOL DEF blocks (ISO: T) or TOOL CALL blocks (ISO: G51) has been increased from 16 to 32. … (New_and_Changed_Functions__1322095_02_A_ p24)
- [new] Machining cycles The parameter Q437 APPROACH POSITION has been added to Cycle 256 RECTANGULAR STUD (ISO: G256). This parameter allows you to define whether the … (New_and_Changed_Functions__1322095_02_A_ p24)
- [new] Touch probe cycles After optimizing the angular orientation with Cycle 451 MEASURE KINEMATICS (ISO: G451, option 48), a separate measurement series for position … (New_and_Changed_Functions__1322095_02_A_ p24)
- [changed] Program functions When programming the PLANE functions (option 8), you can also define SEQ by means of a Q parameter. If a value of 0 is entered in the Q … (New_and_Changed_Functions__1322095_02_A_ p25)
- [changed] Positioning w/ Manual Data Input operating mode If an NC block contains a tool radius compensation in Positioning w/ Manual Data Input operating mode, then … (New_and_Changed_Functions__1322095_02_A_ p25)
- [changed] Status displays If you use a spindle as a C axis, then the control displays the axis letter C instead of S2 in the distance-to-go display of this spindle. In … (New_and_Changed_Functions__1322095_02_A_ p25)
- [changed] Moving the machine axes You can use handwheel superimpositioning with M118 when FUNCTION TCPM (option 9) is active. (New_and_Changed_Functions__1322095_02_A_ p25)
- [changed] Manual touch probe cycles You can also use manual touch probe cycles in conjunction with mechanical probes or milling cutters. The ACTUAL POSITION CAPTURE … (New_and_Changed_Functions__1322095_02_A_ p26)
- [changed] Tilting the working plane (option 8) You can define up to three tilt angles in the 3-D ROT menu. You define the rotation using the spatial angles SPA, SPB … (New_and_Changed_Functions__1322095_02_A_ p26)
- [changed] Tools The maximum number of characters permitted in the NAME and DOC columns of the tool table has been increased from 16 to 32. The tool table provides a … (New_and_Changed_Functions__1322095_02_A_ p26)
- [changed] Machine-specific user parameters When you edit an NC block and press the up or down cursor key, the control goes to the next identical syntax element. The … (New_and_Changed_Functions__1322095_02_A_ p26)
- [new] Dynamic Collision Monitoring (DCM) (option 40) The machine tool builder can define machine components that are to be monitored by the control during all machine … (New_and_Changed_Functions__1322095-02.tx p18)
- other: 1 Software 34059x-02 | New software options · Remote Desktop Manager (option 133) Remote Desktop Manager allows you  · Cross Talk Compensation (CTC) (option …

**34059x-04**

- [new] The character set of the fixed cycle 225 Engraving was expanded by more characters and the diameter sign see "ENGRAVING (Cycle 225, DIN/ISO: G225)", page 290 New … (892905-22)
- [new] Conversational languages The Additional Language software option (option 41) is no longer available. The following conversational languages are also available … (New_and_Changed_Functions__1322095_02_A_ p28)
- [new] NC fundamentals The control supports up to four spindles. The BLK FORM CYLINDER function allows you to define a cylindrical workpiece blank by entering the … (New_and_Changed_Functions__1322095_02_A_ p28)
- [new] Touch probe cycles Cycle 4 MEASURING IN 3-D This cycle allows you to determine any position on the workpiece. You define the direction of approach in the cycle … (New_and_Changed_Functions__1322095_02_A_ p29)
- [new] Program functions FUNCTION TURNDATA CORR-TCS (option 50) allows you to compensate a turning tool in the tool coordinate system. This compensation is also … (New_and_Changed_Functions__1322095_02_A_ p29)
- [new] Adaptive Feed Control (AFC) (option 45) The following functions allow you to regulate AFC from within the NC program: FUNCTION AFC CUT BEGIN starts an AFC … (New_and_Changed_Functions__1322095_02_A_ p29)
- [new] Programming aids The PROGRAMMING AIDS soft key has been added to the soft-key row for special functions. This soft key opens a menu where you can add a comment … (New_and_Changed_Functions__1322095_02_A_ p30)
- [new] Tables If a table column is assigned the UPTEXT column type, then the control will convert the entered text to uppercase. (New_and_Changed_Functions__1322095_02_A_ p31)
- [new] MOD functions The Graphic settings group that allows you to modify the 3-D motion simulation has been added to the MOD menu. You can choose between the available … (New_and_Changed_Functions__1322095_02_A_ p31)
- [new] NC fundamentals If you use the ACTUAL POSITON CAPTURE key to create a linear block containing the current axis positions, then the linear block contains only the … (New_and_Changed_Functions__1322095_02_A_ p31)
- [changed] Machining cycles The German umlauts and the diameter symbol have been added to the character set of Cycle 225 ENGRAVING (ISO: G225). In Cycle 22 ROUGH-OUT … (New_and_Changed_Functions__1322095_02_A_ p32)
- [changed] DXF Converter (option 42) You can add folders to bookmarks within the file manager of the DXF Converter. If option 42 is active, then, by default, the … (New_and_Changed_Functions__1322095_02_A_ p33)
- [changed] Dynamic Collision Monitoring (DCM) (option 40) DCM monitors movements, in which the machine traverses more than one axis (e.g., movements with active … (New_and_Changed_Functions__1322095_02_A_ p33)
- [changed] Manual touch probe cycles You can use the manual touch probe cycle Probe centerline to determine the centerline between two holes or studs. At the end of the … (New_and_Changed_Functions__1322095_02_A_ p33)
- [changed] Tables When you import a tool table from the iTNC 530, the control loads all supported tool types and assigns the tool type Milling cutter,MILL to all … (New_and_Changed_Functions__1322095_02_A_ p33)
- [changed] MOD functions In the Display settings MOD group, you select which information the control is to show in the position display of the general and additional … (New_and_Changed_Functions__1322095_02_A_ p34)
- [changed] Machine-specific user parameters The control supports searching for language-specific names. Certain machine parameters are available only after you have … (New_and_Changed_Functions__1322095_02_A_ p34)
- other: New and changed cycle functions of software New and changed cycle func · Fundamentals New and changed cycle functions of software · 2 Software 34059x-04 | New …

**34059x-05**

- [changed] overview of the optional Q parameters that have been added in this software version. You can decide for yourself whether you would like to define optional Q … (892905-25)
- [new] NC fundamentals When you program in ISO programming mode using the alphabetic keyboard, the control automatically uppercases the first letter of an NC block. (New_and_Changed_Functions__1322095_02_A_ p36)
- [new] Machining cycles Cycle 270 CONTOUR TRAIN DATA (ISO: G270) This cycle allows you to add a type of approach, type of departure, and radius compensation to the data … (New_and_Changed_Functions__1322095_02_A_ p37)
- [new] Program functions FUNCTION FEED DWELL allows you to program a cyclic dwell time. Use the parameters D-TIME and F-TIME to define the dwell time and the machining … (New_and_Changed_Functions__1322095_02_A_ p37)
- [new] Programming Q parameters The FN 18: SYSREAD functions (ISO: D18) have been enhanced: FN 18: SYSREAD (D18) ID52 NR1: pocket number of a tool NR2: magazine number … (New_and_Changed_Functions__1322095_02_A_ p38)
- [new] Manual touch probe cycles The manual touch probe cycle Probe in plane allows you to measure the misalignment of a plane. After you have probed three points on … (New_and_Changed_Functions__1322095_02_A_ p38)
- [new] HEROS functions The option Allow USB storage devices has been added to the SELinux settings in the HEROS menu. If you deactivate this option, the control will … (New_and_Changed_Functions__1322095_02_A_ p39)
- [new] Machine-specific user parameters The machine parameter posAfterContPocket (no. 201007) allows you to define how the control moves to clearance height at the end … (New_and_Changed_Functions__1322095_02_A_ p40)
- [new] NC fundamentals The dialog for shutting down the control has been modified. You define whether the control shuts down or restarts. In the tool call, you can … (New_and_Changed_Functions__1322095_02_A_ p40)
- [changed] Machining cycles The character set provided by Cycle 225 ENGRAVING (ISO: G225) has been expanded. You can engrave the CE mark, the letter ß, and the symbol … (New_and_Changed_Functions__1322095_02_A_ p41)
- [changed] Touch probe cycles The input range of parameter Q305 NUMBER IN TABLE has been extended from +2999 to +99 999 in touch probe cycles. The parameter Q536 STOP … (New_and_Changed_Functions__1322095_02_A_ p41)
- [changed] Miscellaneous functions The function M116 (option 8) causes the control to interpret the feed rate of a rotary axis as mm/min or 1/10 inches/min instead of … (New_and_Changed_Functions__1322095_02_A_ p42)
- [changed] File management The COPY FIELD soft key has been added to the Last files pop-up window. This soft key allows you to save the path of the marked file to the … (New_and_Changed_Functions__1322095_02_A_ p42)
- [changed] Program run The behavior of the control in Program run, single block operating mode has been modified. When executing point pattern cycles and the function … (New_and_Changed_Functions__1322095_02_A_ p42)
- [changed] Status displays The basic rotation icon has been modified to match the 3-D basic rotation icon. The control-in-operation icon is shown on the screen as long … (New_and_Changed_Functions__1322095_02_A_ p42)
- [changed] Tilting the working plane (option 8) The PLANE RESET function does not affect Manual operation mode. To deactivate an active tilting function in Manual … (New_and_Changed_Functions__1322095_02_A_ p42)
- [changed] Manual touch probe cycles The number of positioning movements has been reduced in the manual calibration cycle Calibration of a TS in a ring. The Safety … (New_and_Changed_Functions__1322095_02_A_ p43)
- other: The column PITCH has been added to the tool management, see "Enter too · The column PITCH has been added to the tool management, see "Entering  · 3 Software …

**34059x-06**

- [changed] New Cycle 258 POLYGON STUD New probing cycles 600 and 601 for monitoring with a camera (option number 136) Cycle 291 INTERPOLATION TURNING, COUPLING (option … (892909-24)
- [changed] New cycle 258 POLYGON STUD New cycles 600 and 601, touch probe cycles for monitoring with a camera (option 136) Cycle 291 INTERPOLATION TURNING, COUPLING … (892909-25)
- [new] Program functions FUNCTION S-PULSE allows you to program a pulsing spindle speed (e.g., to avoid self-oscillations of the machine). The SCALE parameter defines … (New_and_Changed_Functions__1322095_02_A_ p46)
- [new] Adaptive Feed Control (AFC) (option 45) In the AFC-LOAD column of the tool table, you can define a spindle reference power for AFC in percent, which you … (New_and_Changed_Functions__1322095_02_A_ p46)
- [new] Programming Q parameters The FN 18: SYSREAD (ISO: D18) functions have been enhanced: FN 18: SYSREAD (D18) ID56 NR1: number of lines in the active tool table NR2: … (New_and_Changed_Functions__1322095_02_A_ p47)
- [new] Program run The control differentiates between an interrupted and a stopped NC program. When the control is in an interrupted state, it provides the following … (New_and_Changed_Functions__1322095_02_A_ p47)
- [new] Manual touch probe cycles You can transfer the values measured in a manual touch probe cycle into the preset table by defining a line number in the Number in … (New_and_Changed_Functions__1322095_02_A_ p48)
- [new] Tables The PATHNAME column type for tables has been added. For this type of column, you can use the SELECT soft key to select a file path from a pop-up window. … (New_and_Changed_Functions__1322095_02_A_ p48)
- [new] Machine-specific user parameters The optional machine parameter restoreCoolant (no. 201008) allows you to define whether the control restores the status of the … (New_and_Changed_Functions__1322095_02_A_ p48)
- [new] NC fundamentals The functions APPR LT, APPR LCT, DEP LT, and DEP LCT position all of the required axes simultaneously to an auxiliary point. The functions APPR … (New_and_Changed_Functions__1322095_02_A_ p48)
- [changed] Machining cycles A mirror image that was programmed using Cycle 8 MIRRORING (ISO: G28) is also effective for a subsequent tilting of the working plane … (New_and_Changed_Functions__1322095_02_A_ p49)
- [changed] Miscellaneous functions When tilted machining (option 8), M128, or FUNCTION TCPM (option 9) is active, the control saves the current positions of the rotary … (New_and_Changed_Functions__1322095_02_A_ p50)
- [changed] DXF Converter (option 42) DXF Converter outputs the FUNCTION MODE MILL or FUNCTION MODE TURN machining mode as a comment. (New_and_Changed_Functions__1322095_02_A_ p50)
- [changed] Program run A block scan takes into account a feed rate defined with M116 (option 8) in mm/ min for rotary axes. The setting of the feed-rate potentiometer … (New_and_Changed_Functions__1322095_02_A_ p50)
- [changed] Positioning w/ Manual Data Input operating mode The function PLANE RESET (option 8) always positions the rotary axes to their home position, regardless of … (New_and_Changed_Functions__1322095_02_A_ p50)
- [changed] Status displays When M136 is active, the control displays the feed rate in mm/1 or inches/1 in the general status display. In Manual Operation mode, the … (New_and_Changed_Functions__1322095_02_A_ p50)
- [changed] Tables The preset table shows the columns SPA, SPB, and SPC, regardless of the active tool axis. The maximum input range of the columns SPA, SPB, and SPC of … (New_and_Changed_Functions__1322095_02_A_ p51)
- [changed] Tools When you edit a tool in the tool table or tool management, the only table line that is locked for other access is the line containing this tool. If … (New_and_Changed_Functions__1322095_02_A_ p51)
- [changed] MOD functions The control checks the values defined in the Traverse limits MOD function for their validity. If a value is not valid, the control issues an … (New_and_Changed_Functions__1322095_02_A_ p51)
- other: New and changed cycle functions of software 34059x-06 New Cycle 258 PO · 4 Software 34059x-06 | New software options · Camera-based Visual Setup Control (VSC) …

**34059x-07**

- [new] 3-D radius compensation depending on the tool’s contact angle (option 92) This option allows you to compensate for tools depending on the tool contact angle. For … (New_and_Changed_Functions__1322095_02_A_ p54)
- [new] NC fundamentals The control supports programming of spatial arcs. Spatial arcs are defined based on three axes and are perpendicular to the working plane. (New_and_Changed_Functions__1322095_02_A_ p54)
- [new] Program functions FUNCTION DWELL allows you to program a dwell time in seconds or define the number of spindle revolutions for dwelling. FUNCTION TURNDATA … (New_and_Changed_Functions__1322095_02_A_ p54)
- [new] Programming Q parameters The FN 0 function allows you to assign the Undefined status to Q parameters. The FN 9 function allows you to compare QS parameters and … (New_and_Changed_Functions__1322095_02_A_ p55)
- [new] Test run Press the MEASURING soft key to have the control display the workpiece coordinates at the position of the mouse pointer in 3-D view and plan view. The … (New_and_Changed_Functions__1322095_02_A_ p56)
- [new] Program run If program run has been canceled, the control saves the point of interruption (i.e., the last active NC block). The SELECT LAST soft key allows you … (New_and_Changed_Functions__1322095_02_A_ p56)
- [new] Positioning w/ Manual Data Input operating mode The Positioning w/ Manual Data Input operating mode has been enhanced. The following functions are supported: … (New_and_Changed_Functions__1322095_02_A_ p56)
- [new] Status displays When a pulsing spindle speed is active, the control shows a corresponding icon in the general status display. When an NC block has been executed … (New_and_Changed_Functions__1322095_02_A_ p57)
- [new] Moving the machine axes You can connect up to five portable handwheels to the control. You can use the axis keys of the control to select the axis to be … (New_and_Changed_Functions__1322095_02_A_ p57)
- [new] Tilting the working plane (option 8) In the 3-D ROT menu, the control shows the active machine kinematics. (New_and_Changed_Functions__1322095_02_A_ p57)
- [new] Machine-specific user parameters During many face turning operations, it is not possible to maintain constant surface speeds because the maximum spindle speed … (New_and_Changed_Functions__1322095_02_A_ p58)
- [new] NC fundamentals Cylindrical workpiece blanks can also be defined by means of a diameter. You can program up to six axes in a linear block. You can use the … (New_and_Changed_Functions__1322095_02_A_ p58)
- [changed] Machining cycles Cycle 225 ENGRAVING (ISO: G225) has been enhanced. The parameter Q367 TEXT POSITION allows you to define the starting position based on the … (New_and_Changed_Functions__1322095_02_A_ p59)
- [changed] Touch probe cycles Cycles 460 CALIBRATION OF TS ON A SPHERE (ISO: G460), 462 CALIBRATION OF A TS IN A RING (ISO: G462), and 463 TS CALIBRATION ON STUD (ISO: … (New_and_Changed_Functions__1322095_02_A_ p60)
- [changed] Program functions FUNCTION TCPM (option 9) with selected AXIS SPAT takes into account an active mirror function that has been programmed with Cycle 8 … (New_and_Changed_Functions__1322095_02_A_ p60)
- [changed] Programming Q parameters If you determine the length of an undefined Q parameter by means of the function STRLEN, the control displays a value of –1. (New_and_Changed_Functions__1322095_02_A_ p60)
- [changed] Calling an external NC program If an NC program called with CALL PGM (ISO: %) includes an M2 or M30 function, then the control displays a warning. (New_and_Changed_Functions__1322095_02_A_ p60)
- [changed] Test run If you select a new NC program or start it with the RESET + START soft key, the control resets the tool data. In BLANK IN WORK SPACE view, the … (New_and_Changed_Functions__1322095_02_A_ p61)
- [changed] Dynamic Collision Monitoring (DCM) (option 40) The machine manufacturer defines the response of the control if a collision occurs due to a retraction that … (New_and_Changed_Functions__1322095_02_A_ p61)
- [changed] Program run If you modify the preset during a program run interruption, you can resume the NC program run only with a block scan or GOTO. Entering into an FK … (New_and_Changed_Functions__1322095_02_A_ p61)
- [changed] Pallets While a pallet table is being run, you can edit it only by using the EDIT PALLET soft key. (New_and_Changed_Functions__1322095_02_A_ p61)
- [changed] Status displays When a spindle speed limit defined with FUNCTION TURNDATA SPIN SMAX (option 50) is active, then the control displays SMAX instead of S in the … (New_and_Changed_Functions__1322095_02_A_ p61)
- [changed] Camera-based monitoring of setup situation (VSC) (option 136) If you press the SHOW AREAS soft key, the control shows all monitoring areas of an image. If … (New_and_Changed_Functions__1322095_02_A_ p62)
- [changed] Manual touch probe cycles The manual touch probe cycle Corner as preset allows you to probe both a circular stud and a hole on a straight line. The positions … (New_and_Changed_Functions__1322095_02_A_ p62)
- [changed] Tilting the working plane (option 8) If you activate the Tilt working plane function in the 3-D ROT menu, then the control uses the COORD ROT transformation … (New_and_Changed_Functions__1322095_02_A_ p62)
- [changed] Tables When you open the preset table, the cursor is on the line of the active preset. When you import a tool table from the iTNC 530, the control correctly … (New_and_Changed_Functions__1322095_02_A_ p62)
- [changed] Tools You are allowed to use the special characters % and , in tool names. When you create a new tool, the columns L and R of the tool table are empty and … (New_and_Changed_Functions__1322095_02_A_ p62)
- [changed] Camera-based Visual Setup Control (VSC) (option 136) If you press the SHOW AREAS soft key, the control shows all monitoring areas of an image. If you press … (New_and_Changed_Functions__1322095_01_A_ p62)
- other: Cycle 860 has been expanded by parameters Q510, Q511, and Q462. This m · 5 Software 34059x-07 | New software options · Touch probe cycles Cycle 444 PROBING IN …

**34059x-08**

- [new] Reading system data", Page 288 Running NC Programs New Global Program Settings function (option 44). The new Batch Process Manager function enables you to plan … (892903-27)
- [new] Renamed software options Option 42 was renamed as CAD Import. (New_and_Changed_Functions__1322095_02_A_ p66)
- [new] Machining cycles Cycle 276 THREE-D CONT. TRAIN (ISO: G276) This cycle allows you to machine a contour train of different depths. You define the contour in a … (New_and_Changed_Functions__1322095_02_A_ p67)
- [new] Program functions FUNCTION COUNT allows you to define a counter (e.g., for counting the number of produced workpieces) in the NC program. The control considers … (New_and_Changed_Functions__1322095_02_A_ p67)
- [new] Miscellaneous functions M138 allows you to select individual rotary axes for tilting, M128, or FUNCTION TCPM (option 9) even in turning mode (option 50). (New_and_Changed_Functions__1322095_02_A_ p68)
- [new] Programming Q parameters In the function FN 16: F-PRINT (ISO: D16), you can use a Q or QS parameter to define the path of the source file and the output file. … (New_and_Changed_Functions__1322095_02_A_ p69)
- [new] Programming aids You can turn an NC block into a comment by pressing the INSERT COMMENT soft key or the < key of the alphabetic keyboard. You can turn a comment … (New_and_Changed_Functions__1322095_02_A_ p69)
- [new] Test run Using the SELECT TRAVERSE RANGE soft key from within the BLANK IN WORK SPACE menu, you can select ranges of traverse defined by the machine manufacturer … (New_and_Changed_Functions__1322095_02_A_ p70)
- [new] CAD Viewer The CAD-Viewer can open STEP (*.stp and *.step) and IGES (*.igs and *.iges) files.CAD Import (option 42) additionally allows you to extract contours … (New_and_Changed_Functions__1322095_02_A_ p70)
- [new] Manual touch probe cycles In the manual touch probe cycle Corner as preset, you can compensate for angular misalignment by rotating the rotary table. The TCH … (New_and_Changed_Functions__1322095_02_A_ p71)
- [new] Tables You can manually edit the line 0 of the preset table. The machine manufacturer can disable the setting of presets for individual axes. The SERIAL column … (New_and_Changed_Functions__1322095_02_A_ p71)
- [new] Tools You can search for a tool name in the tool table by entering the name using the alphabetic keyboard. After entering the first character, the control opens … (New_and_Changed_Functions__1322095_02_A_ p71)
- [new] Machine-specific user parameters The machine parameter decimalCharakter (no. 100805) allows you to select whether the control uses a period or a comma as the … (New_and_Changed_Functions__1322095_02_A_ p72)
- [new] NC fundamentals You can change the sorting sequence within the pop-up window for tool selection by clicking the header of a column. The sorting sequence and the … (New_and_Changed_Functions__1322095_02_A_ p72)
- [changed] Machining cycles Cycle 225 ENGRAVING (ISO: G225) allows you to engrave the current count. Cycles 256 RECTANGULAR STUD (ISO: G256) and 257 CIRCULAR STUD (ISO: … (New_and_Changed_Functions__1322095_02_A_ p73)
- [changed] The OLD CYCLS soft key and the associated cycles have been removed: Cycle 1 PECKING (ISO: G83) Cycle 2 TAPPING (ISO: G84) Cycle 3 SLOT MILLING (ISO: G74) … (New_and_Changed_Functions__1322095_02_A_ p74)
- [changed] Touch probe cycles The machine manufacturer can define whether the offset R-OFFS of the tool table is taken into account for a tool without cutting edges. If … (New_and_Changed_Functions__1322095_02_A_ p74)
- [changed] Program functions In FUNCTION TCPM (option 9), you can define the tool reference point and the center of rotation. The control positions the tool to the tool … (New_and_Changed_Functions__1322095_02_A_ p74)
- [changed] Miscellaneous functions With M94, the control reduces the display value of all rotary axes to a value of less than 360°, even if the axes are not modulo … (New_and_Changed_Functions__1322095_02_A_ p74)
- [changed] Test run The machine manufacturer can change the tool change position for Test Run operating mode. The control displays the tool in blue during an air cut … (New_and_Changed_Functions__1322095_02_A_ p75)
- [changed] File management The control automatically detects a connected USB device. The soft key for connecting a USB device has been removed. (New_and_Changed_Functions__1322095_02_A_ p75)
- [changed] Program run When returning to the contour, the last axis positioned by the control is, by default, the tool axis. If the tool tip is positioned below the … (New_and_Changed_Functions__1322095_02_A_ p75)
- [changed] Status displays The control displays the spindle speed with at least four digits in the general status display. This means if the spindle speed is less than … (New_and_Changed_Functions__1322095_02_A_ p76)
- [changed] Moving the machine axes In a tilted working plane (option 8), you can also home axes (e.g., a tool magazine) that are not activated in the active kinematic … (New_and_Changed_Functions__1322095_02_A_ p76)
- [changed] Tools When you place the cursor into an input field in the form view of the tool management, the control selects the complete field contents. If you enter a … (New_and_Changed_Functions__1322095_02_A_ p76)
- [changed] Error messages and help system When displaying error messages, the control sorts them first by priority and, second (within the same priority), by time. … (New_and_Changed_Functions__1322095_02_A_ p76)
- [changed] Remote Desktop Manager (option 133) In the Remote Desktop Manager, all of the functions that you previously had to call by clicking the right mouse button … (New_and_Changed_Functions__1322095_02_A_ p77)
- [changed] (ISO: G831), and 832 THREAD EXTENDED (ISO: G832, option 50). Cycle 239 ASCERTAIN THE LOAD (ISO: G239, option 143) can adapt the maximum acceleration of the … (New_and_Changed_Functions__1322095-02.tx p89)
- other: 6 Software 34059x-08 | New software options · Global Program Settings (option 44) This option allows you to define v · Batch Process Manager (option 154) This …

**34059x-09**

- [new] NC fundamentals You can use the unlabeled key between PGM MGT and ERR to toggle between the operating modes. In the TOOL DEF (ISO: G51) function, you can define … (New_and_Changed_Functions__1322095_02_A_ p81)
- [new] Machining cycles Cycle 285 DEFINE GEAR (ISO: G285, option 157) This cycle allows you to define the geometry of gear teeth. If you define only the parameters Q540 … (New_and_Changed_Functions__1322095_02_A_ p81)
- [new] Touch probe cycles The following cycles allow you to ascertain the exact workpiece position and compensate it as needed during program run. The cycles save the … (New_and_Changed_Functions__1322095_02_A_ p81)
- [new] Program functions The PLANE functions (option 8) provide the alternative selection option SYM in addition to SEQ. If SYM is selected, the control calculates the … (New_and_Changed_Functions__1322095_02_A_ p82)
- [new] Programming Q parameters The following options have been added to the text file that defines the output format for FN 16: F-PRINT (ISO: D16): The character * … (New_and_Changed_Functions__1322095_02_A_ p82)
- [new] Global Program Settings (option 44) If you activate Handwheel superimp. in the Global Program Settings function, the control can show the values of the virtual … (New_and_Changed_Functions__1322095_02_A_ p83)
- [new] Positioning w/ Manual Data Input operating mode In Positioning w/ Manual Data Input operating mode, the HIDE soft key that enables you to skip NC blocks has been … (New_and_Changed_Functions__1322095_02_A_ p83)
- [new] Status displays In the general status display, the control displays a feed rate value of less than 10 with one decimal place and a feed rate value of less than 1 … (New_and_Changed_Functions__1322095_02_A_ p83)
- [new] Manual touch probe cycles The manual probing functions take an active M128 or FUNCTION TCPM (option 9) into account. When using the manual touch probe cycle … (New_and_Changed_Functions__1322095_02_A_ p84)
- [new] Tilting the working plane (option 8) The appearance of the 3-D ROT menu has been revised. You can individually activate the settings for Manual operation mode … (New_and_Changed_Functions__1322095_02_A_ p84)
- [new] Tools You define the tool material in the TMAT column of the tool table. In the CUTDATA column, you select the cutting data table for the cutting data … (New_and_Changed_Functions__1322095_02_A_ p84)
- [new] Machine-specific user parameters The machine parameter CfgProgramCheck (no. 129800) allows you to define for NC programs and pallet machining whether and when … (New_and_Changed_Functions__1322095_02_A_ p85)
- [new] NC fundamentals When you program linear blocks, the default is no longer that the control shows the syntax element &Z. In order to use & to program parallel-axis … (New_and_Changed_Functions__1322095_02_A_ p85)
- [new] Machining cycles The parameter Q367 SURFACE POSITION has been added to Cycle 233 FACE MILLING (ISO: G233). This parameter allows you to define the position of … (New_and_Changed_Functions__1322095_02_A_ p85)
- [changed] Touch probe cycles Cycles 408 SLOT CENTER PRESET (ISO: G408) to 419 PRESET IN ONE AXIS (ISO: G419) take into account the machine parameter chkTiltingAxes … (New_and_Changed_Functions__1322095_02_A_ p86)
- [changed] Program functions FUNCTION TCPM (option 9) supports peripheral milling (this means that it supports 3-D radius compensation with a constant position of the … (New_and_Changed_Functions__1322095_02_A_ p86)
- [changed] Programming Q parameters The functions M_CLOSE and M_TRUNCATE for FN 16: F-PRINT (ISO: D16) have the same effect as far as output to the screen is concerned. … (New_and_Changed_Functions__1322095_02_A_ p86)
- [changed] CAD Viewer CAD Import (option 42) outputs 2-D contours by default. CAD Import (option 42) outputs angles with PLANE SPATIAL instead of PLANE VECTOR. You can … (New_and_Changed_Functions__1322095_02_A_ p87)
- [changed] Manual touch probe cycles If the axis angle does not match the tilt angle during manual probing, then the control no longer issues an error message. … (New_and_Changed_Functions__1322095_02_A_ p87)
- [changed] Tables If the active line of the preset table is reactivated, the control reads out the values again. The machine manufacturer defines which file types the … (New_and_Changed_Functions__1322095_02_A_ p87)
- [changed] Remote Desktop Manager (option 133) The setting Additional waiting time: has been added to the Switch-off/restart of a computer function for shutting down an … (New_and_Changed_Functions__1322095_02_A_ p87)
- [changed] HEROS functions The screensaver glideshow has been removed. (New_and_Changed_Functions__1322095_02_A_ p87)
- other: 7 Software 34059x-09 | New software options · State Reporting Interface (option 137) SRI offers an interface for acq · Monitoring of configured machine …

**34059x-10**

- [changed] New point pattern cycle 224 DATAMATRIX CODE PATTERN for the creation of a DataMatrix code. New cycle 238 MEASURE MACHINE STATUS for monitoring machine … (892903-28)
- [new] Programming or ISO Programming (1261174-21)
- [new] OPC UA NC Server (options 56 to 61) OPC UA provides a standardized interface for secure data exchange between products independent of the manufacturer. … (New_and_Changed_Functions__1322095_02_A_ p90)
- [new] Grinding operations on milling machines (option 156) This option allows you to perform a grinding operation at the control. The control supports jig grinding … (New_and_Changed_Functions__1322095_02_A_ p90)
- [new] Display step The Display Step software option (option 23) is available in the standard control version. The display step of the axes is no longer limited to four … (New_and_Changed_Functions__1322095_02_A_ p91)
- [new] Machining cycles Cycle 224 DATAMATRIX CODE PATTERN (ISO: G224) This cycle allows the control to convert a text into a machine-readable Data Matrix code. The Data … (New_and_Changed_Functions__1322095_02_A_ p92)
- [new] Cycle 1032 GRINDING WHL LENGTH COMPENSATION (ISO: G1032, option 156) This cycle allows you to define the overall length of a grinding tool. Depending on whether … (New_and_Changed_Functions__1322095_02_A_ p93)
- [new] Program functions FUNCTION CORRDATA allows you to activate a line of the compensation table. The compensation remains effective until the next tool change or the … (New_and_Changed_Functions__1322095_02_A_ p93)
- [new] Miscellaneous functions During retraction with M140 MB MAX, the control takes into account the safety clearances that can be defined by the machine manufacturer … (New_and_Changed_Functions__1322095_02_A_ p93)
- [new] Batch Process Manager (option 154) In the operating modes Program run, single block and Program run, full sequence, the COLLISION CHECKING soft key has been … (New_and_Changed_Functions__1322095_02_A_ p94)
- [new] Global Program Settings (option 44) The GLOBAL SETTINGS INACTIVE soft key that allows you to deactivate all defined settings has been added to the form of the … (New_and_Changed_Functions__1322095_02_A_ p95)
- [new] Status displays The control displays a corresponding icon in the general status display when tool radius compensation is active. The control shows the POS HR tab … (New_and_Changed_Functions__1322095_02_A_ p95)
- [new] Moving the machine axes The HR 520 and HR 550 FS handwheels display the offset of handwheel superimpositioning from M118 or from the Global Program Settings … (New_and_Changed_Functions__1322095_02_A_ p95)
- [new] Tilting the working plane (option 8) The 3-D ROT: AXIS ANGLE SPA. ANGLE soft key has been added to the 3-D ROT menu. This soft key allows you to select whether … (New_and_Changed_Functions__1322095_02_A_ p95)
- [new] Tools Compensation tables allow you to compensate for tools during program run without having to edit the NC program or the tool tables. The compensation table … (New_and_Changed_Functions__1322095_02_A_ p95)
- [new] Machine-specific user parameters The optional machine parameter applyCfgLanguage (no. 101305) allows you to define whether the HEROS operating system adopts the … (New_and_Changed_Functions__1322095_02_A_ p96)
- [changed] Machining cycles In Cycle 225 ENGRAVING (ISO: G225), you can program system variables that contain the following information for the current main program and … (New_and_Changed_Functions__1322095_02_A_ p97)
- [changed] Touch probe cycles Based on the 3-D error (i.e., the deviation along the normal vector), Cycle 444 PROBING IN 3-D (ISO: G444) is able to differentiate … (New_and_Changed_Functions__1322095_02_A_ p97)
- [changed] Program functions You can limit the feed rate of the compensating movements in FUNCTION TCPM (option 9). (New_and_Changed_Functions__1322095_02_A_ p97)
- [changed] Adaptive Feed Control (AFC) (option 45) When running the AFC CUT BEGIN function in turning mode (option 50), the control checks for whether the programmed … (New_and_Changed_Functions__1322095_02_A_ p97)
- [changed] Miscellaneous functions If you program a precision stop of rotary axes using M134 or M135, the control no longer displays an error message. The control … (New_and_Changed_Functions__1322095_02_A_ p97)
- [changed] Programming Q parameters The FN 10 function also allows you to check QS parameters and texts for inequalities. You can use combined QS parameters in the SQL … (New_and_Changed_Functions__1322095_02_A_ p98)
- [changed] Test run If you set the MEASURING soft key to ON, then the control displays the following additional information: Surface orientation of the current position … (New_and_Changed_Functions__1322095_02_A_ p98)
- [changed] Status displays The icon displayed for FUNCTION MODE TURN (option 50) in the additional status display has been modified. On 19" and 24" screens, the control … (New_and_Changed_Functions__1322095_02_A_ p98)
- [changed] Manual touch probe cycles If, during probing, the axis angles do not match the tilt angles, then the Working plane is inconsistent menu allows you to tilt … (New_and_Changed_Functions__1322095_02_A_ p99)
- [changed] MOD functions If the control is accessed from external, it displays a corresponding icon in the header. Limits that have been defined in the Traverse limits … (New_and_Changed_Functions__1322095_02_A_ p99)
- [changed] HEROS functions When user administration is active, the Liberating motion mode requires the NC.OPModeManual permission (i.e., at least the role of … (New_and_Changed_Functions__1322095_02_A_ p99)
- [new] Cycle 1033 GRINDING WHL RADIUS COMPENSATION (ISO: G1033, option 156) This cycle allows you to define the radius of a grinding tool. Depending on whether an … (New_and_Changed_Functions__1322095_01_A_ p93)
- [new] HEIDENHAIN OPC UA NC Server (options 56 to 61) OPC UA provides a standardized interface for secure data exchange between products independent of the … (New_and_Changed_Functions__1322095-02.tx p106)
- [new] Cycle 1015 PROFILE DRESSING (ISO: G1015, option 156) This cycle allows you to dress a random-shape, rotationally symmetric profile of a grinding tool. You define … (New_and_Changed_Functions__1322095-02.tx p109)
- [new] Touch probe cycles Cycle 238 MEASURE MACHINE STATUS (ISO: G238, option 155) This cycle allows the control to measure various components defined to be monitored. … (New_and_Changed_Functions__1322095-02.tx p109)
- other: The jig grinding function (Option 156) allows to machine a workpiece u · 8 Software 34059x-10 | New software options · Optimized contour cycles (option 167) The …

**34059x-11**

- [new] Overview of new and modified software functions Further information about the previous software versions is presented in the Overview of New and Modified … (892909-29)
- [changed] Machining Cycles Cycle 225 ENGRAVING (ISO: G225) allows you to use system variables to engrave the number of the current week. Cycles 202 BORING (ISO: G202) … (892909-29)
- [new] NC fundamentals The BLK FORM FILE function allows you to use STL files to define the workpiece blank, and optionally the finished part, by specifying the path of … (New_and_Changed_Functions__1322095_02_A_ p102)
- [new] Touch probe cycles Cycle 485 MEASURE LATHE TOOL (ISO: G485, option 50) This cycle enables you to measure lathe tools with a tool touch probe. This cycle can only … (New_and_Changed_Functions__1322095_02_A_ p103)
- [new] Program functions FUNCTION MODE SET allows you to activate settings defined by the machine tool builder (e.g., changes to the range of traverse) from within the … (New_and_Changed_Functions__1322095_02_A_ p103)
- [new] Programming Q parameters In the mask file of the FN 16: F-PRINT (ISO: D16) function, you can define whether the control shows or hides blank lines for undefined … (New_and_Changed_Functions__1322095_02_A_ p104)
- [new] Test run If you use TARGET in the BLK FORM FILE function to define a finished part, you can show or hide it by soft key in the Test Run operating mode In Test … (New_and_Changed_Functions__1322095_02_A_ p104)
- [new] CAD Viewer When you double-click a layer in the CAD-Viewer, the control marks the first contour element of this layer. You can transfer the data from the CAD … (New_and_Changed_Functions__1322095_02_A_ p104)
- [new] Status displays If a feed-rate limit has been activated with the F MAX soft key, the control displays an exclamation mark after the feed-rate value in the … (New_and_Changed_Functions__1322095_02_A_ p105)
- [new] Error messages and help system If, after a hardware change or an update, an error occurs when the control is booting, the control will automatically open the … (New_and_Changed_Functions__1322095_02_A_ p105)
- [new] MOD functions A link to the Firewall settings HEROS function has been added to the External access MOD function. A link to the OPC UA NC Server - License … (New_and_Changed_Functions__1322095_02_A_ p105)
- [new] Remote Desktop Manager (option 133) Remote Desktop Manager (option 133) allows you to create private connections if user administration is active. Private … (New_and_Changed_Functions__1322095_02_A_ p106)
- [new] HEROS functions When user administration is active, the control automatically disables the LSV2 connections of the serial interfaces (COM1 and COM2) for security … (New_and_Changed_Functions__1322095_02_A_ p106)
- [new] NC fundamentals You can use the RND element (ISO: G24) to connect circles that are perpendicular to the machining plane instead of lying in the machining plane. (New_and_Changed_Functions__1322095_02_A_ p106)
- [changed] Use the parameter Q575 INFEED STRATEGY to define whether the control machines the contour from top to bottom or vice versa. The maximum input range of … (New_and_Changed_Functions__1322095_02_A_ p108)
- [changed] Touch probe cycles Cycles 480 CALIBRATE TT (ISO: G480) and 484 CALIBRATE IR TT (ISO: G484) can be used to calibrate a tool touch probe with cuboid probe … (New_and_Changed_Functions__1322095_02_A_ p108)
- [changed] Miscellaneous functions The M109 function allows the control to keep the feed rate constant at the cutting edge, including during approach and departure. The … (New_and_Changed_Functions__1322095_02_A_ p108)
- [changed] Program run In Program run, single block operating mode, the control interprets the workpiece blank definition as a single NC block only. The control … (New_and_Changed_Functions__1322095_02_A_ p109)
- [changed] Moving the machine axes When you activate a handwheel with display, the control automatically activates the override potentiometer of the handwheel. In the … (New_and_Changed_Functions__1322095_02_A_ p109)
- [changed] Manual touch probe cycles By default, the control calculates the basic rotation in the input coordinate system (I- CS). If the axis angles and tilt angles … (New_and_Changed_Functions__1322095_02_A_ p109)
- [changed] Tools In the form view of tool management, the input parameter TL is available for grinding and dressing tools. If the check box is enabled, the control … (New_and_Changed_Functions__1322095_02_A_ p110)
- [changed] HEROS functions If you enter a password or code number with Caps Lock active, then the control issues a message. The HEIDENHAIN OPC UA NC Server enables you … (New_and_Changed_Functions__1322095_02_A_ p110)
- other: Machining Cycles Cycle 277 OCM CHAMFERING (ISO: G277, option 167) This · 9 Software 34059x-11 | New software options · Renamed software options Option 146 has …

**34059x-16**

- [new] Cycles Cycle 1017 DRESSING WITH DRESSING ROLL (ISO: G1017, option 156) This cycle allows you to dress the diameter of a grinding wheel with a dressing roll. … (892909-2A)
- [changed] 1 Fundamentals | Control model, software, and features The Cycles 860 to 862 and 870 to 872 output an error message during active multiple plunging if … (1303406-21)
- [new] Camera-based inspection of the setup situation (VSC) Visual Setup Control (software option 136) is no longer available. (New_and_Changed_Functions__1322095_02_A_ p112)
- [new] Production data acquisition State Reporting Interface (software option 137) is no longer available. (New_and_Changed_Functions__1322095_02_A_ p112)
- [new] NC fundamentals Within circular paths C, CR and CT, the LIN_ syntax element is now available in order to superimpose a linear motion over the circular motion of … (New_and_Changed_Functions__1322095_02_A_ p112)
- [new] Machining cycles Cycle 1017 DRESSING WITH DRESSING ROLL (ISO: G1017, option 156) This cycle allows you to dress the diameter of a grinding wheel with a dressing … (New_and_Changed_Functions__1322095_02_A_ p113)
- [new] Program functions With the TRANS ROTATION function, you can rotate contours or positions around a rotation angle. The TRANS DATUM RESET function allows you to … (New_and_Changed_Functions__1322095_02_A_ p114)
- [new] Programming Q parameters Using the SYNTAX soft key, you can enclose path information in double quotation marks in order to use any special characters as part of … (New_and_Changed_Functions__1322095_02_A_ p114)
- [new] Calling an external NC program Using the SYNTAX soft key, you can enclose path information in double quotation marks in order to use any special characters as … (New_and_Changed_Functions__1322095_02_A_ p114)
- [new] File management Within the file manager, the control hides as well as files with a period at the start of the name. If necessary, you can display the files with … (New_and_Changed_Functions__1322095_02_A_ p114)
- [new] CAD Viewer In the CAD-Viewer, you can choose the machining planes YZ and ZX for milling. You use a selection menu to choose the machining plane. (New_and_Changed_Functions__1322095_02_A_ p114)
- [new] Status displays The TRANS tab of the additional status display contains the active shift in the working plane coordinate system WPL-CS. If the shift comes from a … (New_and_Changed_Functions__1322095_02_A_ p115)
- [changed] NC fundamentals In order for the control to show the workpiece blank in the simulation, the workpiece blank must have minimum dimensions. The minimum … (New_and_Changed_Functions__1322095_02_A_ p116)
- [changed] Machining cycles Cycle 225 ENGRAVING (ISO: G225) has been enhanced: The parameter Q202 MAX. PLUNGING DEPTH allows you to define the maximum plunging depth. … (New_and_Changed_Functions__1322095_02_A_ p116)
- [changed] If a measurement in Cycle 238 MEASURE MACHINE STATUS (ISO: G238, option 155) was not performed correctly, for example at a feed rate override of 0 %, you can … (New_and_Changed_Functions__1322095_02_A_ p117)
- [changed] The following cycles support machining with a FreeTurn tool: Cycle 811 SHOULDER, LONGITDNL. (ISO: G811, option 50) Cycle 812 SHOULDER, LONG. EXT. (ISO: G812 … (New_and_Changed_Functions__1322095_02_A_ p118)
- [changed] Touch probe cycles The Cycles 1420 PROBING IN PLANE (ISO: G1420), 1410 PROBING ON EDGE (ISO: G1410), 1411 PROBING TWO CIRCLES (ISO: G1411) have been … (New_and_Changed_Functions__1322095_02_A_ p118)
- [changed] Programming Q parameters While program run is interrupted or has been canceled, you can edit Q and QS parameters with numbers 0 to 99, 200 to 1199 and 1400 … (New_and_Changed_Functions__1322095_02_A_ p118)
- [changed] File management If you exit an NC program by pressing the END key, the control opens the file manager. The cursor is on the NC program that was just closed. … (New_and_Changed_Functions__1322095_02_A_ p119)
- [changed] CAD Viewer CAD Import (option 42) enables you to use a closed contour as workpiece blank for turning (option 50). (New_and_Changed_Functions__1322095_02_A_ p119)
- [changed] Manual touch probe cycles If you use a manual touch-probe function to automatically probe a stud or hole with an opening angle of 360°, the control returns … (New_and_Changed_Functions__1322095_02_A_ p119)
- [new] File management Within the file manager, the control hides system files as well as files and folders with a period at the start of the name. If necessary, you … (New_and_Changed_Functions__1322095_03_A_ p116)
- [new] Machining cycles Cycle 1017 DRESSING WITH DRESSING ROLL (DIN/ISO: G1017, option 156) This cycle allows you to dress the diameter of a grinding wheel with a … (New_and_Changed_Functions__1322095_01_A_ p110)
- other: 10 Software 34059x-16 | New software options · CAD Model Optimizer (option 152) This option allows you tho use the 3D · Touch probe cycles Cycle 1400 POSITION …

**34059x-17**

- [changed] Machining cycles Cycle 277 OCM CHAMFERING (ISO: G277, option 167) monitors contour damage on the floor caused by the tool tip. This tool tip results from the … (New_and_Changed_Functions__1322095_02_A_ p124)
- [changed] Touch probe cycles If KinematicsComp (software option 52) is active, the log of Cycle 451 MEASURE KINEMATICS (ISO: G451, option 48) shows the active … (New_and_Changed_Functions__1322095_02_A_ p124)
- [changed] Test run The simulation takes into account the following columns of the tool table: R_TIP LU RN The control takes into account the following NC functions in … (New_and_Changed_Functions__1322095_02_A_ p124)
- [changed] CAD Viewer CAD-Viewer has been enhanced: Internally, CAD-Viewer always uses mm for its calculations. If you select inches as the unit of measure, CAD-Viewer … (New_and_Changed_Functions__1322095_02_A_ p125)
- [changed] Moving the machine axes If a handwheel is active, the control shows the contouring feed rate in the display during program run. If only the currently … (New_and_Changed_Functions__1322095_02_A_ p125)
- [changed] Tables The minimum input value of the FMAX column in the touch probe table has been changed from –9999 to +10. (New_and_Changed_Functions__1322095_02_A_ p125)
- [changed] Tools The maximum input range of the LTOL and RTOL columns of the tool table has been increased from 0 - 0.9999 mm to 0.0000 - 5.0000 mm. The maximum input … (New_and_Changed_Functions__1322095_02_A_ p125)
- [changed] Accessories The control no longer supports the ITC 750 additional operating station. (New_and_Changed_Functions__1322095_02_A_ p125)
- [changed] HEROS functions The HEROS tool Diffuse has been removed. In the Certificate and keys window you can select a file with additional public SSH keys in the … (New_and_Changed_Functions__1322095_02_A_ p125)
- [changed] Machine-specific user parameters The machine manufacturer uses the machine parameters allowUnsecureLsv2 (no. 135401) and allowUnsecureRpc (no. 135402) to … (New_and_Changed_Functions__1322095_02_A_ p126)
- [new] Programming Q parameters The FN 18: SYSREAD (ISO: D18) functions have been extended: FN 18: SYSREAD (D18) ID610 NR49: Mode of filter reduction of one axis (IDX) … (New_and_Changed_Functions__1322095_03_A_ p124)
- [changed] Test run The simulation considers the following columns of the tool table: R_TIP LU RN The control considers the following NC functions in the Test Run … (New_and_Changed_Functions__1322095_03_A_ p126)
- [changed] HEROS functions The Diffuse HEROS tool was removed. In the Certificate and keys window you can select a file with additional public SSH keys in the … (New_and_Changed_Functions__1322095_03_A_ p127)
- other: Cycles for Workpieces and Tools Cycle 1416 INTERSECTION PROBING (ISO:  · 11 Software 34059x-17 | New software options · Advanced turning functions Software …

**34059x-18**

- [new] Program functions Use the NC function TRANS RESET to reset all simple coordinate transformations simultaneously. (New_and_Changed_Functions__1322095_03_A_ p130)
- [new] Machine-specific user parameters In machine parameter safeAbsPosition (no. 403130), the machine manufacturer defines whether the SLP safety function is activated … (New_and_Changed_Functions__1322095_03_A_ p131)
- [new] Machining cycles You can also define subcontours as LBL subprograms within the complex SEL CONTOUR contour formula. The machine manufacturer can hide the cycles … (New_and_Changed_Functions__1322095_03_A_ p131)
- [changed] Touch probe cycles In the touch probe cycles 14xx, you can enter symmetrical tolerances "+-...." for the nominal dimensions. Cycle 441 FAST PROBING (ISO: … (New_and_Changed_Functions__1322095_03_A_ p132)
- [changed] CAD Viewer CAD Import (option 42) subdivides contours that are not located in the working plane, into individual sections. CAD Viewer creates straight lines … (New_and_Changed_Functions__1322095_03_A_ p132)
- [changed] Test run If a component has not been configured or cannot be monitored, the control displays the corresponding machining operation in gray in the heatmap. (New_and_Changed_Functions__1322095_03_A_ p132)
- [changed] Tables In the TABDATA WRITE, TABDATA ADD and FN 27: TABWRITE (ISO: D27) NC functions, you can enter values directly. If you create a table, and there is at … (New_and_Changed_Functions__1322095_03_A_ p133)
- [changed] HEROS functions The HEROS tool Diffuse has been added. You can compare and merge text files. The OPC UA NC Server has been enhanced as follows: The OPC UA NC … (New_and_Changed_Functions__1322095_03_A_ p133)
- [changed] Machine-specific user parameters Machine parameter CfgStretchFilter (no. 201100) has been removed. (New_and_Changed_Functions__1322095_03_A_ p133)
- other: Cycles Cycle 1274 OCM CIRCULAR SLOT (ISO: G1274, option 167) This cycl · Software updates To install or update software version 18, a control w · Machining …

### C.TNC7


**81762x-17**

- [new] Text editor In the text editor mode, the control provides an auto-complete function when programming. The control suggests syntax elements matching your entries … (New_and_Changed_Functions__1373081_01_A_ p10)
- [new] Superimpositioning on circu- When programming circular paths with C, CR and CT, the LIN_ syntax lar paths element is now available in order to superimpose a … (New_and_Changed_Functions__1373081_01_A_ p11)
- [new] NC sequences You can save up to 200 successive NC blocks as NC sequences and insert them during programming using the Insert NC function window. In contrast to … (New_and_Changed_Functions__1373081_01_A_ p11)
- [new] FN 18: SYSREAD (ISO: D18) The FN 18: SYSREAD (ISO: D18) functions have been enhanced: FN 18: SYSREAD (D18) ID610 NR49: Mode of filter reduction of one axis (IDX) … (New_and_Changed_Functions__1373081_01_A_ p12)
- [new] ISO programs You can run and edit ISO programs. (New_and_Changed_Functions__1373081_01_A_ p12)
- [new] Context menu In the context menu of the Editor operating mode and the MDI appli- cation, the control offers the Insert last NC block function. With this function … (New_and_Changed_Functions__1373081_01_A_ p12)
- [new] Clamping situation In the Visualization options column of the Simulation workspace, you can show the worktable and, if necessary, the fixtures, in Workpiece mode … (New_and_Changed_Functions__1373081_01_A_ p12)
- [new] Navigation path If you execute an NC program or a pallet table or if you test it in the opened Simulation workspace, the control displays a navigation path in … (New_and_Changed_Functions__1373081_01_A_ p13)
- [new] Form workspace The control displays an icon of the selected tool type in the Tool Icon area. For the turning tools the icons also take into account the tool … (New_and_Changed_Functions__1373081_01_A_ p13)
- [new] Window You can change the size of windows. The control remembers the size until it is shut down. Applications In the Files, Tables and Editor operating modes, a … (New_and_Changed_Functions__1373081_01_A_ p14)
- [new] Additional operating station The control no longer supports the ITC 750 additional operating station. (New_and_Changed_Functions__1373081_01_A_ p14)
- [changed] Status overview on the In the status overview, the control displays the run time of the TNC bar NC program in mm:ss format. As soon as the run time of the NC … (New_and_Changed_Functions__1373081_01_A_ p15)
- [changed] Shut down If you shut down the control with still unsaved changes in NC programs and contours, the control displays the Close the program window. You can … (New_and_Changed_Functions__1373081_01_A_ p15)
- [changed] Input When you save an input value, the control removes superfluous zeros at the beginning of the input and at the end of the decimal places. The input range … (New_and_Changed_Functions__1373081_01_A_ p16)
- [changed] Tool call If you select the tool with the selection window when calling the tool with TOOL CALL, you can switch via an icon to the Tables operating mode. In … (New_and_Changed_Functions__1373081_01_A_ p16)
- [changed] Line L If you press the actual position capture key in the Editor operating mode or the MDI application, the control creates a straight line L with the … (New_and_Changed_Functions__1373081_01_A_ p17)
- [changed] Cycle 19 WORKING PLANE You can edit and execute Cycle 19 WORKING PLANE (ISO: G80, (ISO: G80, option 8) option 8), but you cannot insert it into an NC program … (New_and_Changed_Functions__1373081_01_A_ p17)
- [changed] The 3-D rotation window In the 3-D rotation window (option 8), if you enable a function in the (option 8) Manual Operation or Program run areas, the control … (New_and_Changed_Functions__1373081_01_A_ p17)
- [changed] FUNCTION PROG PATH If you define a grinding tool (option 156) with orientation 9 or 10, the (option 9) control supports circumferential milling in … (New_and_Changed_Functions__1373081_01_A_ p17)
- [changed] File management The control shows the occupied memory and total memory of the drives in the navigation bar of the file management. The control shows STEP … (New_and_Changed_Functions__1373081_01_A_ p18)
- [changed] FN 16: F-PRINT (ISO: D16) For a screen output with FN 16: F-PRINT (ISO: D16), the control displays a pop-up window. The Q parameter list The window Q … (New_and_Changed_Functions__1373081_01_A_ p18)
- [changed] Elements If you select the face of a closed contour, you can insert a radius or chamfer at each corner of the contour. Element information area In the … (New_and_Changed_Functions__1373081_01_A_ p18)
- [changed] Unit of measure Internally, CAD-Viewer always uses mm for its calculations. If you select inches as the unit of measure, CAD-Viewer will convert all values … (New_and_Changed_Functions__1373081_01_A_ p19)
- [changed] The Structure column in the The structure contains the NC functions APPR and DEP as structure Program workspace elements. The control shows comments in the … (New_and_Changed_Functions__1373081_01_A_ p19)
- [changed] Aligning the rotary table If you align the rotary table after a manual touch probe function, the control remembers the selected type of rotary axis … (New_and_Changed_Functions__1373081_01_A_ p20)
- [changed] Cycle 451MEASURE If KinematicsComp (software option 52) is active, the log of Cycle KINEMATICS (ISO: G451, 451 MEASURE KINEMATICS (ISO: G451, option 48) … (New_and_Changed_Functions__1373081_01_A_ p20)
- [changed] Batch Process Manager If you check the pallet table in Batch Process Manager (option 154) (option 154) with Dynamic Collision Monitoring (DCM, option 40) … (New_and_Changed_Functions__1373081_01_A_ p21)
- [changed] Tables operating mode The M and S statuses are highlighted in color only for the active applica- tion, and gray for the other applications. You can close all … (New_and_Changed_Functions__1373081_01_A_ p21)
- [changed] Secure connections The machine manufacturer uses the machine parameters allowUnse- cureLsv2 (no. 135401) and allowUnsecureRpc (no. 135402) to define whether … (New_and_Changed_Functions__1373081_01_A_ p22)
- [new] The Status workspace The TRANS tab of the Status workspace indicates the active shift in the working plane coordinate system WPL-CS. If the shift comes from a … (New_and_Changed_Functions__1373081_03_A_ p15)
- [changed] Code number When you enter a code number in the Settings application, the control displays a load icon. Network You can export and import existing network … (New_and_Changed_Functions__1373081_03_A_ p28)
- other: Cycle 1416 INTERSECTION PROBING (ISO: G1416) This cycle allows you to  · 1 New and Modified Functions | Changed functions 81762x-17 · 1 Software 81762x-17 | New …

**81762x-18**

- [new] The Text editor workspace In the Programming operating mode, the Text editor workspace is available. In the Text editor you can create and edit data of the … (New_and_Changed_Functions__1373081_01_A_ p25)
- [new] TRANS RESET Use the NC function TRANS RESET to reset all simple coordinate trans- formations simultaneously. (New_and_Changed_Functions__1373081_01_A_ p26)
- [new] FN 18: SYSREAD (ISO: D18) The FN 18: SYSREAD (ISO: D18) functions have been extended: FN 18: SYSREAD (D18) ID10 NR10: Counts the number of executions of the … (New_and_Changed_Functions__1373081_01_A_ p26)
- [new] Importing contours into It is possible to import NC blocks that contain NC functions for coordi- graphical programming nate transformation into the graphical … (New_and_Changed_Functions__1373081_01_A_ p27)
- [new] The Simulation settings The Optimized saving of STL (#152 / #1-04-1) toggle switch allows window you to output a simplified STL file. These STL files have been … (New_and_Changed_Functions__1373081_01_A_ p27)
- [new] Retracting the tap If the NC program stops during tapping, the control will display the Tool Retract button. When you select that button and press the NC start … (New_and_Changed_Functions__1373081_01_A_ p27)
- [new] The Form workspace Using the Configure the layout icon in the Form workspace, you can add or remove columns and arrange the areas in columns. Tool table Yo can … (New_and_Changed_Functions__1373081_01_A_ p28)
- [new] SLP safety function (safely In machine parameter safeAbsPosition (no. 403130), the machine limited position) manufacturer defines whether the SLP safety function … (New_and_Changed_Functions__1373081_01_A_ p28)
- [new] The Positions workspace If the handwheel is active, the control shows a symbol next to the selected axis in the Positions workspace. The symbol indicates whether … (New_and_Changed_Functions__1373081_01_A_ p29)
- [changed] Handwheel If you select Manual operating mode, the control deactivates the handwheel. (New_and_Changed_Functions__1373081_01_A_ p30)
- [changed] The Editor operating mode You can change the tab order in the Editor operating mode. The Program workspace On the title bar of the Program workspace, the … (New_and_Changed_Functions__1373081_01_A_ p30)
- [changed] Tool data The thread-turning tool turning tool type includes the parameter SPB- Insert (#50 / #4-03-1). Indexed tools In the Insert tool window, the Index … (New_and_Changed_Functions__1373081_01_A_ p31)
- [changed] SEL CONTOUR You can also define subcontours as LBL subprograms within the complex SEL CONTOUR contour formula. PATTERN DEF The Insert NC function window … (New_and_Changed_Functions__1373081_01_A_ p31)
- [changed] Cycle 225 ENGRAVING (ISO: The input value 1 has been added to parameter Q515 FONT in Cycle G225) 225 ENGRAVING (ISO: G225). Use this input value to select … (New_and_Changed_Functions__1373081_01_A_ p32)
- [changed] Cycle 800 ADJUST XZ Cycle 800 ADJUST XZ SYSTEM (ISO: G800) (#50 / #4-03-1) has been SYSTEM (ISO: G800) extended: (#50 / #4-03-1) The input range of the … (New_and_Changed_Functions__1373081_01_A_ p32)
- [changed] File functions If file functions are available for a selected folder or file, the control will display three dots below the icon. If you copy a file and then … (New_and_Changed_Functions__1373081_01_A_ p33)
- [changed] Component monitoring If a component has not been configured or cannot be monitored, the (#155 / #5-02-1) control displays the corresponding machining … (New_and_Changed_Functions__1373081_01_A_ p33)
- [changed] Formulas If you press the spacebar while using the Formula, String formula and Contour formula NC functions, the control displays all currently usable syntax … (New_and_Changed_Functions__1373081_01_A_ p34)
- [changed] The Contour settings The control will save the settings made in the Contour settings window window permanently. Only the Plane and Diameter programming … (New_and_Changed_Functions__1373081_01_A_ p34)
- [changed] CAD Import (#42 / #1-03-1) When you select contours and positions in CAD Viewer, you can rotate the workpiece using touch gestures. While you are using touch … (New_and_Changed_Functions__1373081_01_A_ p34)
- [changed] ISO programming In connection with ISO programming, the control provides the following functions: Auto-complete Color highlighting of syntax elements … (New_and_Changed_Functions__1373081_01_A_ p35)
- [changed] Comments and structuring You can insert line breaks within comments or structuring items. items The Structure column You can use the context menu to mark … (New_and_Changed_Functions__1373081_01_A_ p35)
- [changed] The Simulation settings In the Editor operating mode, the Simulation workspace can be open window for only one NC program at a time. If you want to open the … (New_and_Changed_Functions__1373081_01_A_ p35)
- [changed] Probe process When you select a manual touch-probe function, the control automati- cally suggests the probing direction last used for this function. After … (New_and_Changed_Functions__1373081_01_A_ p36)
- [changed] Tool measurement cycles Using the optional machine parameter maxToolLengthTT 48x (no. 122607), the machine manufacturer defines a maximum tool length for … (New_and_Changed_Functions__1373081_01_A_ p37)
- [changed] Cycle 451 MEASURE Cycles 451 MEASURE KINEMATICS (ISO: G451) (#48 / #2-01-1) and 452 KINEMATICS (ISO: G451) PRESET COMPENSATION (ISO: 452) (#48 / #2-01-1) … (New_and_Changed_Functions__1373081_01_A_ p37)
- [changed] Feed-rate limitation The button for feed-rate limitation and the associated functions (previ- ously FMAX) were renamed to F LIMIT. Execution cursor The … (New_and_Changed_Functions__1373081_01_A_ p37)
- [changed] Creating a new table When you create a new table in the file manager, the table does not contain information on the required columns yet. When you open the … (New_and_Changed_Functions__1373081_01_A_ p38)
- [changed] OPC UA NC Server Within the OPC UA menu item, a button is available to manually start or (#56-61 / #3-02-1*) restart the OPC UA NC Server. The OPC UA NC … (New_and_Changed_Functions__1373081_01_A_ p39)
- [changed] Login as a function user Your IT administrator can set up a function user to facilitate connectivi- ty to the Windows domain. Connecting to a Windows If you … (New_and_Changed_Functions__1373081_01_A_ p39)
- [changed] Display of the machine In the List workspace, you can toggle between a structure and a table parameters view of the configuration editor. StretchFilter … (New_and_Changed_Functions__1373081_01_A_ p39)
- [new] The Text editor workspace The Text editor workspace is available in the Editor operating mode. In the Text editor you can create and edit data of the following … (New_and_Changed_Functions__1373081_03_A_ p33)
- other: User's Manual as integrated product aid: TNCguide · TNCguide You can open TNCguide for the current context. Context-sensit · Hardware requirements To install or …

**81762x-19**

- [new] Remember position of Use the Remember position icon to select whether the control will windows remember the position of a window the next time a window is … (New_and_Changed_Functions__1373081_03_A_ p51)
- [new] Insert NC function window For some NC functions, the Insert NC function window offers the possibility of inserting the start and end of the NC function into the … (New_and_Changed_Functions__1373081_03_A_ p51)
- [new] Cylindrical grinding with Use FUNCTION MODE GRIND (#156 / #4-04-1) to activate cylindrical FUNCTION MODE GRIND grinding and select a kinematic model. Cylindrical … (New_and_Changed_Functions__1373081_03_A_ p52)
- [new] Programming a straight line L Press the actual position capture key to program a straight line L with with actual position capture the actual positions of all … (New_and_Changed_Functions__1373081_03_A_ p52)
- [new] Cycle 1011 DRESSING This cycle dresses the front face or shaft face of a grinding wheel. SIDE A/I (ISO: G1011) You define the dressing operation and the number … (New_and_Changed_Functions__1373081_03_A_ p53)
- [new] Control structures (e.g., with The control provides the NC functions for programming the control IF or ELSE) structures. The control provides the following NC … (New_and_Changed_Functions__1373081_03_A_ p54)
- [new] Navigation path In the file management, you can open a history of up to 20 previously used paths. You can edit the current navigation path. User-defined filters … (New_and_Changed_Functions__1373081_03_A_ p54)
- [new] Set up fixtures If the active unit of measure is inch, the control converts mm to inches (#140 / #5-03-2) within the Set up fixtures function. Advanced checks … (New_and_Changed_Functions__1373081_03_A_ p54)
- [new] Variable: named parameters The control provides the named parameters variable type. The variable name of named parameters consists of a freely chosen designation … (New_and_Changed_Functions__1373081_03_A_ p56)
- [new] Auto draw In Editor mode, the control provides the Auto draw toggle switch. If you select an NC block and activate the toggle switch, the control will show a … (New_and_Changed_Functions__1373081_03_A_ p56)
- [new] NC function G79|G00 The control calls the most recently programmed machining cycle at the position you defined in the NC block with G79|G00. The control moves to … (New_and_Changed_Functions__1373081_03_A_ p57)
- [new] Issuing notifications with With the FUNCTION REPORT NC function, the controls issues FUNCTION REPORT notifications under program control. You can define the … (New_and_Changed_Functions__1373081_03_A_ p57)
- [new] The Workpiece options The control provides the following functions also in the Manual and column Program Run operating modes: Reset the workpiece Remove the … (New_and_Changed_Functions__1373081_03_A_ p58)
- [new] Set up the workpiece You can choose an NC program within the Set up the workpiece (#159 / #1-07-1) function. The control generates the 3D model from the … (New_and_Changed_Functions__1373081_03_A_ p58)
- [new] The Table workspace In the header of the Table workspace, the control may show the "Rules not met" filter icon, if applicable. The control displays only the rows … (New_and_Changed_Functions__1373081_03_A_ p59)
- [new] F limited toggle switch The F limited toggle switch has been removed. Test status of the axes You can reset the test status of individual axes or all axes in the … (New_and_Changed_Functions__1373081_03_A_ p59)
- [new] Menu item Adjustment of In the Adjustment of analog voltage offset menu item, the control analog voltage offset shows all connected analog axes, whether the axes … (New_and_Changed_Functions__1373081_03_A_ p60)
- [new] Configuration editor In the Configuration window, the Display in tree view toggle switch has been added. With this toggle switch, you can display the machine … (New_and_Changed_Functions__1373081_03_A_ p61)
- [new] HEROS menu The Hostkey HEROS tool has been added; the control can use this tool to display a unique ASCII graphic, similar to a fingerprint. If you config- ure a … (New_and_Changed_Functions__1373081_03_A_ p62)
- [changed] The Status workspace The MON tab of the Status workspace has been renamed as COMPMON (#155 / #5-02-1). The machine manufacturer defines the contents to be … (New_and_Changed_Functions__1373081_03_A_ p63)
- [changed] Marking an area using identi- If you press SHIFT + UP or SHIFT + DOWN during editing, the control cal syntax elements will jump to the next NC block with the … (New_and_Changed_Functions__1373081_03_A_ p63)
- [changed] Preselecting a tool with TOOL You can no longer program L and R in the TOOL DEF NC function. The DEF control issues an error message during execution. (New_and_Changed_Functions__1373081_03_A_ p63)
- [changed] PATTERN DEF The control shows a matching icon for the selection options of the PATTERN DEF NC function. Support for *.hp point files The control no longer … (New_and_Changed_Functions__1373081_03_A_ p64)
- [changed] Cycle 24 SIDE FINISHING If the sum of the finishing allowance for the side Q14 and the radius of (ISO: G124 ) the finishing mill is smaller than the sum of … (New_and_Changed_Functions__1373081_03_A_ p64)
- [changed] Cycle 1000 DEFINE RECIP. The following parameters have been added to Cycle 1000 DEFINE STROKE (ISO: G1000) RECIP. STROKE: (#156 / #4-04-1) Q1003 … (New_and_Changed_Functions__1373081_03_A_ p65)
- [changed] NC sequences You can save up to 2000 sequential NC blocks as one NC component. You can save user-defined folder icons for the subfolders of the NC … (New_and_Changed_Functions__1373081_03_A_ p65)
- [changed] The Update TAB / PGM The control supports only tables with UTF-8 character encoding. With function the Update TAB / PGM function, the control changes the … (New_and_Changed_Functions__1373081_03_A_ p66)
- [changed] The Text editor workspace Pressing the ENT key inserts a line break in the Text editor workspace. Note regarding external file If the currently opened file … (New_and_Changed_Functions__1373081_03_A_ p66)
- [changed] Process monitoring By default, the control displays six monitoring tasks. (#168 / #5-01-1) Process monitoring shows suggestion type notes (e.g., Consider … (New_and_Changed_Functions__1373081_03_A_ p67)
- [changed] Q parameter list window Via the Global search toggle switch you can choose whether the control searches all columns of the Q parameter list window or only … (New_and_Changed_Functions__1373081_03_A_ p67)
- [changed] Working plane is inconsis- The 3-D ROT Apply status function no longer transfers only the current tent! (#8 / #1-01-1) window positions of the rotary axes to … (New_and_Changed_Functions__1373081_03_A_ p68)
- [changed] Touch probe cycles 42x and The control saves the status of the message in parameters Q180 to 43x Q182 before it issues the measurement report. If you … (New_and_Changed_Functions__1373081_03_A_ p68)
- [changed] Block scan If program run is canceled during a program section repeat or program loop, the control provides the number of the repeat as point of interrup- … (New_and_Changed_Functions__1373081_03_A_ p69)
- [changed] Table filters The control displays user-defined filters under the All filter. You can select and deselect the user-defined filters. When you tap or click a … (New_and_Changed_Functions__1373081_03_A_ p69)
- [changed] Rotational speed in the The display of the handwheel always shows the spindle speed of the display handwheel current channel (for example for a machine with … (New_and_Changed_Functions__1373081_03_A_ p69)
- [changed] Self-test of the control If the self-test of the control is active, the control displays an icon in the information bar. Internally monitored axes Internally … (New_and_Changed_Functions__1373081_03_A_ p70)
- [changed] PKI Admin The PKI Admin has been reorganized. The Advanced settings tab has been removed and the settings were transferred to the appropriate tabs. The VNC … (New_and_Changed_Functions__1373081_03_A_ p70)
- [changed] Configuration editor The key combination CTRL + F opens the Search column in the config- uration editor. No longer needed machine parameters of the TNC 640 … (New_and_Changed_Functions__1373081_03_A_ p70)
- [changed] Connection to Windows To join a Windows domain, the IT administrator must set up a function domain user. You can no longer join a Windows domain with a … (New_and_Changed_Functions__1373081_03_A_ p70)
- other: 3 Software 81762x-19 | Modified or enhanced software options · OPC UA NC Server (#56-61 / #3-02-1*) · Software option On controls with SIK2, you can enable up …

**81762x-20**

- [changed] The Collision Monitoring The software option Collision Monitoring (#40 / #5-03-1) is no longer software option available. To be able to use the Dynamic … (New_and_Changed_Functions__1373081_03_A_ p75)
- [new] Workspaces When you select the title of a workspace, the control will open a selec- tion menu. This selection menu allows you to switch workspaces without … (New_and_Changed_Functions__1373081_03_A_ p76)
- [new] Settings in the Program In the Edit area, the Automatically adapt cycle comments toggle workspace switch was added. If the toggle switch is active, the control … (New_and_Changed_Functions__1373081_03_A_ p77)
- [new] The Open in text editor The Open in text editor button of the Files operating mode opens the button selected file in the Text editor workspace. You can open any … (New_and_Changed_Functions__1373081_03_A_ p77)
- [new] FN 18: SYSREAD (ISO: D18) Use FN 18: SYSREAD (D18) ID59 NR100 to read whether there are axis movements during a tool call with the same tool number. (New_and_Changed_Functions__1373081_03_A_ p78)
- [new] Start page If process monitoring has not been activated or was reset for an NC program, the control will display a start page in the Process Monitoring … (New_and_Changed_Functions__1373081_03_A_ p79)
- [new] Parameter learning The parameter learning function allows you to set the Process Monitor- ing parameters without knowing the required values. You can define the … (New_and_Changed_Functions__1373081_03_A_ p80)
- [new] The Hover mode The Hover icon was added to replace the Rotate mode. If Hover is active and you swipe across the CAD model during touch operation, the control … (New_and_Changed_Functions__1373081_03_A_ p80)
- [new] Outputting texts with With the FUNCTION PRINT NC function, you can output any text from FUNCTION PRINT an NC program to a pop-up window or to a file. The Search … (New_and_Changed_Functions__1373081_03_A_ p80)
- [new] The Visualization options With the Rewind function, you can wind simulated machining steps column forward and backward in order to check the traverses in an NC … (New_and_Changed_Functions__1373081_03_A_ p81)
- [new] The Manual traverse Using the optional machine parameter userControl (no. 144101), the function machine manufacturer defines if the control will display the … (New_and_Changed_Functions__1373081_03_A_ p81)
- [new] Code numbers The Home operating mode icon shown in the TNC bar changes if the code number 123 is enabled. The icon additionally shows an unlocked padlock to … (New_and_Changed_Functions__1373081_03_A_ p81)
- [new] Open context menu For all objects in the configuration editor, the control provides an icon that opens the context menu. Objects are shown by a folder icon. The … (New_and_Changed_Functions__1373081_03_A_ p82)
- [new] HEROS menu The HEROS tool Virtual keyboard was added. The virtual keyboard is also available if only the operating system is active and the NC software is … (New_and_Changed_Functions__1373081_03_A_ p82)
- [changed] Workspaces As an alternative to using the Move icon in the title bar, you can move workspaces by dragging the title area. Scaling windows If a window is … (New_and_Changed_Functions__1373081_03_A_ p83)
- [changed] The Positions workspace If dressing mode FUNCTION DRESS is active, the control shows a corre- sponding symbol in the Positions (#156 / #4-04-1) workspace. … (New_and_Changed_Functions__1373081_03_A_ p83)
- [changed] Configuration files During booting, the control checks whether all the needed configuration files are available in the TNC:\config folder and on the HOME: … (New_and_Changed_Functions__1373081_03_A_ p83)
- [changed] Measure unbalance If the Inch unit of measure is active in the control, the radial position will be displayed in inches and the mass in pounds in the Result … (New_and_Changed_Functions__1373081_03_A_ p84)
- [changed] The Program workspace In the NC program, cycles have multiple lines but just one block number. The control will automatically remove any additional block … (New_and_Changed_Functions__1373081_03_A_ p84)
- [changed] Grinding tools With the V_MAX parameter, you can define the maximum allowed (#156 / #4-04-1) cutting speed in addition to the maximum speed indicated with … (New_and_Changed_Functions__1373081_03_A_ p84)
- [changed] Q255 DWELL TIME In Cycle 204 BACK BORING, the input value of this parameter was extended from zero to four decimal places. (New_and_Changed_Functions__1373081_03_A_ p85)
- [changed] The PLANE functions The control can execute PLANE functions containing MOVE and the ABST syntax element. This increases compatibility with earlier controls. … (New_and_Changed_Functions__1373081_03_A_ p85)
- [changed] Q1012 INCR. For the following cycles, the default value of this parameter was COMPENSATION changed from +0 to +1: Cycle 1032 GRINDING WHL LENGTH COMPENSATION … (New_and_Changed_Functions__1373081_03_A_ p86)
- [changed] The Text editor workspace The control highlights formatting characters in color within *.a files that are used for the function FN 16: F-PRINT (ISO: D16). … (New_and_Changed_Functions__1373081_03_A_ p86)
- [changed] MONITORING SECTION Within the MONITORING SECTION NC function, the PAUSE and RESUME syntax elements were added. With MONITORING SECTION PAUSE and MONITORING … (New_and_Changed_Functions__1373081_03_A_ p87)
- [changed] Parameterization of monitor- During the parameterization of monitoring tasks, the control displays ing tasks the number of available Parameter settings and … (New_and_Changed_Functions__1373081_03_A_ p88)
- [changed] FUNCTION POLARKIN polar The control permits execution of the POLARKIN NC function only in kinematics FUNCTION MODE MILL milling mode. (New_and_Changed_Functions__1373081_03_A_ p88)
- [changed] Context menu The context menu of the Document workspace was removed. Cutting data calculator The representation of the cutting data calculator was improved. … (New_and_Changed_Functions__1373081_03_A_ p89)
- [changed] The Simulation settings While programming, you can select table values, such as tools, from window a selection window. By default, the control will display … (New_and_Changed_Functions__1373081_03_A_ p90)
- [changed] The Change the preset The control displays a symbol in front of the row of the current preset, window like in the preset table. If you apply the changes made … (New_and_Changed_Functions__1373081_03_A_ p90)
- [changed] Machine parameters The optional machine parameter trackAsync (no. 122503) for machine manufacturers has been removed from software version 19 SP1. (New_and_Changed_Functions__1373081_03_A_ p90)
- [changed] The Job list workspace During program run, you can move the rows of the job list to change the execution sequence. However, you cannot manipulate the current … (New_and_Changed_Functions__1373081_03_A_ p90)
- [changed] Buttons The buttons in the function bar of the Program Run operating mode will always be displayed, even when no NC program is selected. The Program … (New_and_Changed_Functions__1373081_03_A_ p91)
- [changed] Network drives You can use the Single Sign On setting not only for private networks, but also for all networks. The control uses the respective user's logon … (New_and_Changed_Functions__1373081_03_A_ p91)
- other: 4 Software 81762x-20 | New software options · Process Tracking Interface (#3-04-1*) · Software option PTI 8 This software option provides an interface for r …

### C.TNC7 basic


**81762x-19**

- [new] Remember position of Use the Remember position icon to select whether the control will windows remember the position of a window the next time a window is … (New_and_Changed_Functions__1443541_01_A_ p11)
- [new] Insert NC function window For some NC functions, the Insert NC function window offers the possibility of inserting the start and end of the NC function into the … (New_and_Changed_Functions__1443541_01_A_ p11)
- [new] Programming a straight line L Press the actual position capture key to program a straight line L with with actual position capture the actual positions of all … (New_and_Changed_Functions__1443541_01_A_ p11)
- [new] Control structures (e.g., with The control provides the NC functions for programming the control IF or ELSE) structures. The control provides the following NC … (New_and_Changed_Functions__1443541_01_A_ p12)
- [new] Navigation path In the file management, you can open a history of up to 20 previously used paths. You can edit the current navigation path. User-defined filters … (New_and_Changed_Functions__1443541_01_A_ p12)
- [new] Set up fixtures If the active unit of measure is inch, the control converts mm to inches (#140 / #5-03-2) within the Set up fixtures function. Advanced checks … (New_and_Changed_Functions__1443541_01_A_ p12)
- [new] Variable: named parameters The control provides the named parameters variable type. The variable name of named parameters consists of a freely chosen designation … (New_and_Changed_Functions__1443541_01_A_ p13)
- [new] Auto draw In Editor mode, the control provides the Auto draw toggle switch. If you select an NC block and activate the toggle switch, the control will show a … (New_and_Changed_Functions__1443541_01_A_ p13)
- [new] NC function G79|G00 The control calls the most recently programmed machining cycle at the position you defined in the NC block with G79|G00. The control moves to … (New_and_Changed_Functions__1443541_01_A_ p14)
- [new] Issuing notifications with With the FUNCTION REPORT NC function, the controls issues FUNCTION REPORT notifications under program control. You can define the … (New_and_Changed_Functions__1443541_01_A_ p14)
- [new] The Workpiece options The control provides the following functions also in the Manual and column Program Run operating modes: Reset the workpiece Remove the … (New_and_Changed_Functions__1443541_01_A_ p15)
- [new] Set up the workpiece You can choose an NC program within the Set up the workpiece (#159 / #1-07-1) function. The control generates the 3D model from the … (New_and_Changed_Functions__1443541_01_A_ p15)
- [new] Cycle 1403 RECTANGLE This cycle determines the center, width and length of a rectangle. The PROBING (#17 / #1-05-1) control probes two opposing touch points in … (New_and_Changed_Functions__1443541_01_A_ p16)
- [new] The Table workspace In the header of the Table workspace, the control may show the "Rules not met" filter icon, if applicable. The control displays only the rows … (New_and_Changed_Functions__1443541_01_A_ p16)
- [new] F limited toggle switch The F limited toggle switch has been removed. Test status of the axes You can reset the test status of individual axes or all axes in the … (New_and_Changed_Functions__1443541_01_A_ p16)
- [new] Menu item Adjustment of In the Adjustment of analog voltage offset menu item, the control analog voltage offset shows all connected analog axes, whether the axes … (New_and_Changed_Functions__1443541_01_A_ p17)
- [new] Configuration editor In the Configuration window, the Display in tree view toggle switch has been added. With this toggle switch, you can display the machine … (New_and_Changed_Functions__1443541_01_A_ p18)
- [new] HEROS menu The Hostkey HEROS tool has been added; the control can use this tool to display a unique ASCII graphic, similar to a fingerprint. If you config- ure a … (New_and_Changed_Functions__1443541_01_A_ p19)
- [changed] The Status workspace The MON tab of the Status workspace has been renamed as COMPMON (#155 / #5-02-1). The machine manufacturer defines the contents to be … (New_and_Changed_Functions__1443541_01_A_ p20)
- [changed] Marking an area using identi- If you press SHIFT + UP or SHIFT + DOWN during editing, the control cal syntax elements will jump to the next NC block with the … (New_and_Changed_Functions__1443541_01_A_ p20)
- [changed] Preselecting a tool with TOOL You can no longer program L and R in the TOOL DEF NC function. The DEF control issues an error message during execution. (New_and_Changed_Functions__1443541_01_A_ p20)
- [changed] PATTERN DEF The control shows a matching icon for the selection options of the PATTERN DEF NC function. Support for *.hp point files The control no longer … (New_and_Changed_Functions__1443541_01_A_ p20)
- [changed] Cycle 24 SIDE FINISHING If the sum of the finishing allowance for the side Q14 and the radius of (ISO: G124 ) the finishing mill is smaller than the sum of … (New_and_Changed_Functions__1443541_01_A_ p21)
- [changed] NC sequences You can save up to 2000 sequential NC blocks as one NC component. You can save user-defined folder icons for the subfolders of the NC … (New_and_Changed_Functions__1443541_01_A_ p21)
- [changed] The Update TAB / PGM The control supports only tables with UTF-8 character encoding. With function the Update TAB / PGM function, the control changes the … (New_and_Changed_Functions__1443541_01_A_ p22)
- [changed] The Text editor workspace Pressing the ENT key inserts a line break in the Text editor workspace. Note regarding external file If the currently opened file … (New_and_Changed_Functions__1443541_01_A_ p22)
- [changed] Q parameter list window Via the Global search toggle switch you can choose whether the control searches all columns of the Q parameter list window or only … (New_and_Changed_Functions__1443541_01_A_ p22)
- [changed] Working plane is inconsis- The 3-D ROT Apply status function no longer transfers only the current tent! (#8 / #1-01-1) window positions of the rotary axes to … (New_and_Changed_Functions__1443541_01_A_ p23)
- [changed] Touch probe cycles 42x and The control saves the status of the message in parameters Q180 to 43x (#17 / #1-05-1) Q182 before it issues the measurement … (New_and_Changed_Functions__1443541_01_A_ p23)
- [changed] Block scan If program run is canceled during a program section repeat or program loop, the control provides the number of the repeat as point of interrup- … (New_and_Changed_Functions__1443541_01_A_ p24)
- [changed] Table filters The control displays user-defined filters under the All filter. You can select and deselect the user-defined filters. When you tap or click a … (New_and_Changed_Functions__1443541_01_A_ p24)
- [changed] Rotational speed in the The display of the handwheel always shows the spindle speed of the display handwheel current channel (for example for a machine with … (New_and_Changed_Functions__1443541_01_A_ p24)
- [changed] Self-test of the control If the self-test of the control is active, the control displays an icon in the information bar. Internally monitored axes Internally … (New_and_Changed_Functions__1443541_01_A_ p25)
- [changed] PKI Admin The PKI Admin has been reorganized. The Advanced settings tab has been removed and the settings were transferred to the appropriate tabs. The VNC … (New_and_Changed_Functions__1443541_01_A_ p25)
- [changed] Configuration editor The key combination CTRL + F opens the Search column in the config- uration editor. No longer needed machine parameters of the TNC 640 … (New_and_Changed_Functions__1443541_01_A_ p25)
- [changed] Connection to Windows To join a Windows domain, the IT administrator must set up a function domain user. You can no longer join a Windows domain with a … (New_and_Changed_Functions__1443541_01_A_ p25)
- other: 1 Software 81762x-19 | Modified or enhanced software options · OPC UA NC Server (#56-61 / #3-02-1*) · Software option On controls with SIK2, you can enable up …

**81762x-20**

- [changed] The Collision Monitoring The software option Collision Monitoring (#40 / #5-03-1) is no longer software option available. To be able to use the Dynamic … (New_and_Changed_Functions__1443541_01_A_ p29)
- [new] Workspaces When you select the title of a workspace, the control will open a selec- tion menu. This selection menu allows you to switch workspaces without … (New_and_Changed_Functions__1443541_01_A_ p30)
- [new] Settings in the Program In the Edit area, the Automatically adapt cycle comments toggle workspace switch was added. If the toggle switch is active, the control … (New_and_Changed_Functions__1443541_01_A_ p31)
- [new] The Open in text editor The Open in text editor button of the Files operating mode opens the button selected file in the Text editor workspace. You can open any … (New_and_Changed_Functions__1443541_01_A_ p31)
- [new] FN 18: SYSREAD (ISO: D18) Use FN 18: SYSREAD (D18) ID59 NR100 to read whether there are axis movements during a tool call with the same tool number. (New_and_Changed_Functions__1443541_01_A_ p32)
- [new] The Hover mode The Hover icon was added to replace the Rotate mode. If Hover is active and you swipe across the CAD model during touch operation, the control … (New_and_Changed_Functions__1443541_01_A_ p32)
- [new] Outputting texts with With the FUNCTION PRINT NC function, you can output any text from FUNCTION PRINT an NC program to a pop-up window or to a file. The Search … (New_and_Changed_Functions__1443541_01_A_ p33)
- [new] The Visualization options With the Rewind function, you can wind simulated machining steps column forward and backward in order to check the traverses in an NC … (New_and_Changed_Functions__1443541_01_A_ p33)
- [new] The Manual traverse Using the optional machine parameter userControl (no. 144101), the function machine manufacturer defines if the control will display the … (New_and_Changed_Functions__1443541_01_A_ p33)
- [new] Code numbers The Home operating mode icon shown in the TNC bar changes if the code number 123 is enabled. The icon additionally shows an unlocked padlock to … (New_and_Changed_Functions__1443541_01_A_ p34)
- [new] Open context menu For all objects in the configuration editor, the control provides an icon that opens the context menu. Objects are shown by a folder icon. The … (New_and_Changed_Functions__1443541_01_A_ p34)
- [new] HEROS menu The HEROS tool Virtual keyboard was added. The virtual keyboard is also available if only the operating system is active and the NC software is … (New_and_Changed_Functions__1443541_01_A_ p35)
- [changed] Workspaces As an alternative to using the Move icon in the title bar, you can move workspaces by dragging the title area. Scaling windows If a window is … (New_and_Changed_Functions__1443541_01_A_ p36)
- [changed] The Status workspace On the Tool tab, the control shows the tool life in minutes instead of in hours and minutes. This format is also used in tool … (New_and_Changed_Functions__1443541_01_A_ p36)
- [changed] Configuration files During booting, the control checks whether all the needed configuration files are available in the TNC:\config folder and on the HOME: … (New_and_Changed_Functions__1443541_01_A_ p36)
- [changed] The Program workspace In the NC program, cycles have multiple lines but just one block number. The control will automatically remove any additional block … (New_and_Changed_Functions__1443541_01_A_ p37)
- [changed] Tool management If a tool has been saved to the pocket table, you cannot reset the corre- sponding row in tool management. The tool must be removed from the … (New_and_Changed_Functions__1443541_01_A_ p37)
- [changed] Q255 DWELL TIME In Cycle 204 BACK BORING, the input value of this parameter was extended from zero to four decimal places. (New_and_Changed_Functions__1443541_01_A_ p37)
- [changed] The PLANE functions The control can execute PLANE functions containing MOVE and the ABST syntax element. This increases compatibility with earlier controls. … (New_and_Changed_Functions__1443541_01_A_ p37)
- [changed] The Text editor workspace The control highlights formatting characters in color within *.a files that are used for the function FN 16: F-PRINT (ISO: D16). … (New_and_Changed_Functions__1443541_01_A_ p38)
- [changed] FUNCTION POLARKIN polar The control permits execution of the POLARKIN NC function only in kinematics FUNCTION MODE MILL milling mode. (New_and_Changed_Functions__1443541_01_A_ p38)
- [changed] Context menu The context menu of the Document workspace was removed. Cutting data calculator The representation of the cutting data calculator was improved. … (New_and_Changed_Functions__1443541_01_A_ p39)
- [changed] The Simulation settings While programming, you can select table values, such as tools, from window a selection window. By default, the control will display … (New_and_Changed_Functions__1443541_01_A_ p39)
- [changed] The Change the preset The control displays a symbol in front of the row of the current preset, window like in the preset table. If you apply the changes made … (New_and_Changed_Functions__1443541_01_A_ p40)
- [changed] Q1146 EXTRUSION LENGTH In Cycle 1493 EXTRUSION PROBING (ISO: G1493), the input value of this parameter was expanded from ±99 to ±9999.99. The Calibration … (New_and_Changed_Functions__1443541_01_A_ p40)
- [changed] The Job list workspace During program run, you can move the rows of the job list to change the execution sequence. However, you cannot manipulate the current … (New_and_Changed_Functions__1443541_01_A_ p41)
- [changed] Buttons The buttons in the function bar of the Program Run operating mode will always be displayed, even when no NC program is selected. The Program … (New_and_Changed_Functions__1443541_01_A_ p41)
- [changed] The Table workspace In the dialog bar of the Table workspace, the control displays the unit of the selected table cell. If you insert the contents of the … (New_and_Changed_Functions__1443541_01_A_ p42)
- [changed] Importing table contents The Confirm import window has been revised: The control opens the window for every import. The window shows the entire content to be … (New_and_Changed_Functions__1443541_01_A_ p43)
- [changed] Enabling software options To enable a software option for a control with SIK2, you need a key file with SIK2 (from the machine manufacturer, for example). If … (New_and_Changed_Functions__1443541_01_A_ p44)
- other: 2 Software 81762x-20 | New software options · Process Tracking Interface (#3-04-1*) · Software option PTI 8 This software option provides an interface for r …
