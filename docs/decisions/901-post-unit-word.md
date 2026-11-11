## #901 - Fanuc, Haas, Mazak, Siemens 840D and Doosan DNM posts state the unit they write

**Decided.** Unit word only. All five mill posts now state the unit they
actually write - millimetres - before any motion:

| Post | Unit word | Where |
|---|---|---|
| `fanuc.cps` | `G21` | own block at the end of `onOpen` (after `%`, the O word and name, the inverted-axis notes) |
| `haas.cps` | `G21` | own block at the end of `onOpen` |
| `mazak.cps` | `G21` | own block at the end of `onOpen` |
| `doosan-dnm.cps` | `G21` | own block at the end of `onOpen` |
| `sinumerik.cps` | `G710` | inside each section's opening block (`N.. G17 G710 G90 G54 G40 G80`) |

The posts do not read `machine.units` and refuse nothing: where the
program's unit is chosen (a document-level setting, per the
maintainer's 2026-11-11 decision) and inch output are #902. A machine
file stating inch still gets a mm program - which now says G21 / G710,
so it is correct and self-describing, not silent.

**Why.** These posts always write millimetre numbers; a control left in
inch by power-on or by the last program would read every coordinate
25.4x too long. RS-274 / ISO 6983 G21; the Fanuc-family manuals put
G20/G21 in a block of its own at the program's start; on a Haas, G21
also checks the control's unit setting and alarms on a mismatch. On the
840D, `G710` rather than `G71`: the family's programming manual has G71
switch only geometric data, so F would still read in the basic system's
unit on an inch-based control; G710 switches lengths and feeds, and our
feeds are mm/min. The 840D's word rides the existing preamble block
instead of a new numbered block, so the N numbering of every existing
program stays unchanged (only the new word differs in the goldens).

**Dropped.** A named refusal of an inch machine file (the first plan;
dropped by the maintainer's scope change - the machine file stops
choosing the unit in #902). Inch output (#902). A separate numbered
`G710` block in the 840D's `onOpen` (renumbers every line of every 840D
program). `G71` on the 840D (feeds stay in the basic system's unit).

**Known edge.** The 840D states G710 at the first section's preamble; a
program with no section at all (only the end-of-program `G53 Z` lift,
machine coordinates) carries no unit word. Not reachable from a real
posting (the gate posts operations), so left as is.

**Cost to change.** #902 swaps the fixed word for the document's unit
(G20 / G700 with scaled numbers for inch).
