## #1976 - Five Haas 3-axis machine definitions

**What was decided.** Add the five configurations requested in #1976 to the
built-in machine library. Use original, strict JSON in CamCraft format.
Use metric units, full-stroke program-coordinate travel bands (`-L..L`),
and machine-coordinate Z `safe_retract: 0.0` for top home.
Keep `general.vendor` as `Camcraft`, as the built-in convention requires.
The library labels identify the Haas models.

Use the requested CT40 configuration as CamCraft `CAT40` for VF-2,
VF-2SS, Mini Mill and DM-1. Use `BT30` for DT-1.
Use the existing `CAT40 ER32` and `BT30 ER32` holder entries as defaults.
These defaults are CamCraft choices; the Haas pages do not prescribe them.
Record the changer pockets, without counting the additional loaded spindle
position represented by `+1` in some Haas specifications.
State only flood coolant with `M8`. Do not imply that optional air blast
or through-spindle coolant is installed.

The Mini Mill entry follows the requested 406 x 356 x 381 mm redesigned
configuration. Its 400 ipm cutting and rapid cap is the published EDU
baseline, converted to 10,160 mm/min. It assumes the optional 10-pocket
changer, flood coolant and rigid tapping are installed. It is not a claim
that every Mini Mill has those options. The current standard Mini Mill
lists faster rates: 650 ipm cutting and 800 ipm rapid. Older Mini Mill
travel figures and Super Mini Mill specifications do not define this entry.

State the Mini Mill spindle power as 5.6 kW. The Haas CNC Guide, Issue 18,
lists 7.5 hp for this model; conversion gives about 5.6 kW. Do not retain
the 7.5 kW requested in #1976: that value treats horsepower as kilowatts
and overstates the published power.

The EDU page contains conflicting spindle information. Its redesigned-model
overview states 8,000 rpm, while its specification and standard-feature
rows still state 6,000 rpm. Retain the 8,000 rpm required by #1976 and
supported by the overview. Do not treat the conflicting page as a uniform
specification for every generation. The EDU page does not state rigid
tapping in its extracted standard-feature list; that installed option is
an explicit configuration assumption required by the issue.

Convert published inch feedrates with 25.4 mm/in rather than the rounded
metric display. VF-2SS cutting speed, 833 ipm, converts to 21,158.2 mm/min.
Use the issue's 21,158 mm/min, rounded down by 0.2 mm/min.
The other requested rate conversions are exact at whole mm/min.
Use the issue's published metric travel figures as given.

**Why.** Each definition supplies the existing parser, picker and machine
checker with one explicit configuration. No new post or parser behavior
is needed. Machine selection and post selection remain separate choices.
Tests must check the registry, parser, mirror, and stated software limits.
They do not certify the physical machine or a complete post/control pair.

**Sources.** Haas Automation publishes all sources below. Consulted
2026-11-16. These web pages state no document revision or publication date.
The table names identify the source sections. The guide below is Issue 18.
No redistribution license was identified. Only numeric facts and short control identifiers inform
these original definitions; no vendor files, manuals, images or substantial
prose are copied into this contribution.

- [VF-2 product page](https://legacy-aem.haascnc.com/machines/vertical-mills/vf-series/models/small/vf-2.html):
  Travels, Spindle, Feedrates, Tool Changer and Standard Features/The Haas
  Control. Supports the requested travels, 8,100 rpm, 22.4 kW, 650 ipm
  cutting, 1,000 ipm rapid, 20 pockets and rigid tapping. Spindle Taper
  lists CT40 among alternatives; this entry selects CT40.
- [VF-2SS product page](https://legacy-aem.haascnc.com/machines/vertical-mills/vf-series/models/small/vf-2ss.html):
  Travels, Spindle, Feedrates, Tool Changer and Standard Features/The Haas
  Control. Supports the requested travels, 12,000 rpm, 22.4 kW, 833 ipm
  cutting, 1,400 ipm rapid, 30 pockets plus spindle and rigid tapping.
- [DT-1 product page](https://legacy-aem.haascnc.com/machines/vertical-mills/drill-tap-mill/models/dt-1.html):
  product description, Travels, Spindle, Feedrates, Tool Changer, Tapping
  and Standard Features/The Haas Control. Supports BT30, the requested
  travels, 10,000 rpm, 11.2 kW, 1,200 ipm cutting, 2,400 ipm rapid,
  20 pockets plus spindle and rigid tapping.
- [DM-1 product page](https://www.haascnc.com/machines/vertical-mills/drill-tap-mill/models/dm-1.html):
  product description, Travels, Spindle, Feedrates, Tool Changer, Tapping
  and Standard Features/The Haas Control. Supports the 40-taper variant,
  requested travels, 10,000 rpm, 11.2 kW, 1,200 ipm cutting, 2,400 ipm
  rapid, 18 pockets plus spindle and rigid tapping.
- [Mini Mill product page](https://legacy-aem.haascnc.com/machines/vertical-mills/mini-mills/models/minimill.html):
  redesigned-model overview, Travels, Spindle and Feedrates. Establishes
  the current standard-model comparison described above.
- [Haas CNC Guide, Issue 18](https://haas.co.uk/wp-content/uploads/2026/02/Haas-CNC-Guide-Issue-18.pdf):
  the Mini Mill specification on printed page 10 states 7.5 hp, which converts
  to about 5.6 kW. This source corrects the power unit in #1976.
- [Mini Mill-EDU product page](https://legacy-aem.haascnc.com/content/haascnc/en/machines/vertical-mills/mini-mills/models/minimill-edu.html):
  redesigned-model overview, Travels, Spindle, Feedrates, Tool Changer,
  and Coolant Management. Supports the EDU baseline and optional changer.
  See the spindle conflict and installed-option assumptions above.
- [Mill M-code list](https://haascnc.com/service/service-content/guide-procedures/mill---m-codes.html):
  the `M08 / M09` row identifies coolant on/off. CamCraft uses `M8` for
  flood and the existing post handles coolant off.

**Considered and dropped.** Do not add optional coolant systems, alternate
spindles, unverified minimum RPM, spindle body dimensions, or tool envelope
limits. Those values need their own source and configuration evidence.
Do not add ignored JSON provenance or controller-capability keys. Keep
sources and limitations in this decision record. Do not change the Haas
post: the issue requests machine data and registration only.

The DT-1 and DM-1 pages state a 5,000 rpm tapping limit. The current machine
schema stores rigid-tapping availability as a boolean, not a cycle-specific
spindle ceiling. These definitions do not add enforcement of that ceiling.

**Cost to change later.** Low for corrected data: update the affected
configuration, its source note and numeric pins together. Modeling a
cycle-specific tapping ceiling would require separate schema and checker
work. Adding machine data does not establish that behavior.
