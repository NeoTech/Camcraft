## #3690 - Feed words use the control's documented number format

**What was decided:** Keep the decimal point in the Fanuc, Doosan DNM,
and Mazak feed formatters. Apply the same rule to the separate capped
rotary feed path in Fanuc and Doosan DNM. Keep their existing precision,
conversion, rounding, and rate limits. Keep the other posts' feed forms.

**Why:** An integer feed word can have a different unit from a pointed
word. FANUC B-63784EN/01, section 5.3, page 155, gives an unpointed inch
feed a unit of 0.01 inch/min under standard decimal input. Calculator
input can change that interpretation. Its metric unit can depend on F41.
A pointed word identifies the physical unit directly. F41 is a 15i
setting; this finding does not mean that every FANUC control scales metric feeds.
The 0i-F mill manual also gives unpointed G94 inch feeds a unit of
0.01 inch/min. This is the applicable FANUC source for Doosan DNM.
Mazak's feed table has the same inch unit. The requested 2 inch/min
must therefore have the point in `F2.`.

The audit below concerns decimal presence. It does not assess fractional
feed precision or change tapping precision. Integer acceptance does not
mean that a control rejects a decimal point.

| Post | Public manual section | Decision and reason |
|---|---|---|
| `carvera.cps` | [Smoothieware G1, Feedrate][smoothie] | Keep. `F100` specifies 100 mm/min in the documented Smoothieware dialect. Carvera uses that firmware family. This is a dialect inference, not a separate Makera guarantee. |
| `doosan-dnm.cps` | [FANUC B-64604EN-2/01, section 5.2.1, page 77, Unit of F command][fanuc-mill] | Change ordinary and capped rotary feeds. Preserve the point so integer inch values do not use 0.01 inch/min units. |
| `fanuc.cps` | [FANUC B-64604EN-2/01, section 5.2.1, page 77][fanuc-mill]; [B-63784EN/01, section 5.3, page 155][fanuc] | Change ordinary and capped rotary feeds. Preserve the point in metric and inch programs. |
| `fanuc-lathe.cps` | [FANUC B-64604EN-1/01, section 4.4, page 111, Unit of F command][fanuc-lathe] | Keep. G99 already preserves the point. The supported live-tool G98 path is metric only. The 0i-F manual gives integer metric G98 feeds a unit of 1 mm/min. Do not apply the 15i F41 rule to this 0i-F lathe path. |
| `grbl.cps` | [Grbl v1.1, Jogging, How to Use][grbl] | Keep. The official reference maps `F100` to feed 100. Applying this to ordinary G1 follows its stated shared G1 semantics. |
| `grblhal.cps` | [grblHAL G-code and M-code reference, F][grblhal] | Keep. The reference accepts `F500` and defines G94 in the current linear units per minute. |
| `heidenhain-klartext.cps` | [TNC 640 User's Manual, ID892903-2A, Turning, Basic functions, Feed rate, page 564][klartext] | Keep. The manual gives a 200 mm/min move with `F200`. |
| `heidenhain.cps` | [TNC 640 DIN/ISO Programming, ID892909-25, section 14.2, Feed rate][heidenhain] | Keep. The manual gives a 200 mm/min move with `F200`. |
| `linuxcnc.cps` | [LinuxCNC G-code, section 4, G1; section 59, G93/G94/G95][linuxcnc] | Keep. Its example gives `F10` the numerical feed rate 10. The active mode supplies its unit. |
| `mach3.cps` | [Using Mach3Mill, sections 10.5.3.1 and 10.7.24.5, page 10-28][mach3] | Keep. The number grammar accepts integers. The G84 example explicitly uses `F150` for 150 mm/min. |
| `mach4.cps` | [Mach4 Mill GCode Programming, chapter 1, page 5][mach4] | Keep. The guide states that the point is optional for values, including feed rates. |
| `mazak.cps` | [MAZATROL MATRIX EIA/ISO Programming, sections 5-3, 7-2 and 7-3][mazak] | Change. Section 7-2 permits integer metric feeds, but section 7-3 gives unpointed inch feeds 0.01 inch/min units. Preserve the point in the common formatter. |
| `sinumerik.cps` | [SINUMERIK Fundamentals, 01/2008, section 7.1, pages 115-116][sinumerik] | Keep. The manual accepts integer F values and gives `F1000` as 1000 mm/min. |
| `tinyg.cps` | [TinyG Feedhold and Resume, Jogging][tinyg] | Keep. The official example uses `G91 G1 F100`. Acceptance is explicit; the magnitude follows the documented G1 unit semantics. |
| `tormach-lathe.cps` | [8L Programming, Words and Feed Rate (F)][tormach-lathe] | Keep. Words take real values, including integers. F uses the active feed mode and program units. |
| `tormach-plasma.cps` | [1300PL Programming, Words and Feed Rate (F)][tormach-plasma] | Keep. The manual accepts integer feed examples and defines F in the active units. |
| `tormach.cps` | [1100M Programming, Words and Feed Rate (F)][tormach] | Keep, including its rotary floor word. The manual accepts integer feeds in the active units. |

**Considered and dropped:** Do not add a point to every post without a
control requirement. That would change valid programs and unrelated
goldens. Do not change the shared rotary formatter for all dialects.
The affected posts retain the point at their own output boundary.
Do not change the Fanuc lathe's already pointed G99 formatter.

The catalog generator also migrated project versions, parameters, and
cached paths. Those changes were discarded. This fix changes the post
text, so the original projects were retained. The 97 regenerated NC
programs contain only 27,691 added F-word points. Project and CL files
remain unchanged.

**Cost to change later:** A new control variant needs its own public
manual evidence and a posting test. A precision change needs separate
pitch and rate checks. No project format or dependency changes are needed.

[smoothie]: https://smoothieware.org/g1.html
[fanuc]: https://www.douglasrudd.com/manuals/Fanuc-15i-Programming.pdf
[fanuc-mill]: https://www.drivesul.com.br/template/imagens/manuais/manuais-fanuc/fanuc-series-0i-model-f/FANUC%20Series%200i%20MODEL%20F%20-%20OPERATOR_S%20MANUAL%20-%20Machining%20Center%20System.pdf
[fanuc-lathe]: https://www.vmclathe.com/wp-content/uploads/2024/06/Fanuc-controller-%E5%8F%91%E9%82%A3%E7%A7%91%E8%8B%B1%E6%96%87.pdf
[grbl]: https://github.com/gnea/grbl/wiki/Grbl-v1.1-Jogging
[grblhal]: https://github.com/grblHAL/grblhal_docs/blob/main/content/markdown/04-Reference/02-complete-gcode-and-mcode-reference.md#f
[klartext]: https://content.heidenhain.de/doku/tnc_guide/pdf_files/TNC640/34059x-16/bhb/892903-2A.pdf
[heidenhain]: https://content.heidenhain.de/doku/tnc_guide/pdf_files/TNC640/34059x-07/diniso/892909-25.pdf
[linuxcnc]: https://linuxcnc.org/docs/html/gcode/g-code.html#gcode:g1
[mach3]: https://www.machsupport.com/wp-content/uploads/2013/02/Mach3Mill_1.84.pdf
[mach4]: https://www.machsupport.com/wp-content/uploads/2014/05/Mach4%20Mill%20GCode%20Manual.pdf
[mazak]: https://douglasrudd.com/manuals/Mazak%20EIA%20-%20Programming%20Manula%20for%20Mazatrol%20Matrix.pdf
[sinumerik]: https://cache.industry.siemens.com/dl/files/748/104433748/att_96792/v1/PG_Fundamentals_en-US.pdf
[tinyg]: https://github.com/synthetos/TinyG/wiki/TinyG-Feedhold-and-Resume
[tormach-lathe]: https://knowledgebase.tormach.com/8l/programming-8l
[tormach-plasma]: https://knowledgebase.tormach.com/1300pl/programming-1300pl
[tormach]: https://knowledgebase.tormach.com/1100m/programming-1100m
