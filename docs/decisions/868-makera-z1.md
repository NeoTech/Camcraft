## #868 - Built-in machine: Makera Z1, posted through the Carvera post

**Decided.** `assets/machines/makera-z1.mch` ships as a built-in
(`posting::MACHINES`, "Makera Z1"): mill, mm, X/Y 200 mm and Z 100 mm
strokes, spindle 0-13000 rpm and 0.15 kW, axes' top travel 1200 mm/min,
no tool changer stated, coolant `Air` M7 only. It posts through the
existing `carvera.cps` (Air M7 / M9 resolved by #869/#961).

**Sources (published only, read 2026-11-11).**
- makera.com/blogs/cnc-buying-guide/makera-z1-vs-carvera-air and
  .../makera-z1-review: work area 200 x 200 x 100 mm, 150 W spindle,
  lever quick tool change (manual, not an ATC), "AeroDust" blower for chips
  (Makera Studio "Auto Blow"), optional 4th-axis and laser modules.
- cnx-software.com Z1 review (2026-09-01): spindle 0-13,000 rpm, maximum
  travel speed 1,200 mm/min.

**Confirmed vs assumed.**
- Confirmed from Makera's own pages: work area, 150 W, manual quick change,
  air blower.
- Taken from the third-party review (Makera's pages fetched did not list
  them): 13000 rpm and 1200 mm/min. A different article says 15000 rpm; the
  lower, Makera-consistent 13000 is kept (conservative).
- Assumed: the blower is exposed to G-code as M7 / M9. Makera's pages say
  nothing about codes; the issue (user report) asks for Air M7 and the
  Carvera family uses M7. If the Z1 ignores M7 it is a no-op, not a
  motion risk.
- Travel is the full stroke either way from the work zero, the same
  convention as the Carvera files (any zero on the bed reachable).
  The strictly smaller alternative (stroke only on one side) was dropped:
  it would wrongly refuse a legal origin.
- Not published, so not stated (absent is not zero): lowest spindle speed,
  tool-changer count.

**G-code dialect: NOT confirmed.** Nothing public that was read states the
Z1 runs the Carvera's Smoothieware-based firmware or accepts its words
(carvera.cps assumptions: mm only, T0 = probe, no G41/G42, M6 tool change).
Makera pages say Z1 works with Makera Studio and third-party CAM (Fusion,
Onshape, Kiri:Moto) and mention "Jcode" files, undefined. The Z1 is
posted through carvera.cps because the maintainer's report asks for it and
it is the nearest published Makera dialect; the machine file's behaviour
is limited to travels, rpm, coolant, units. A Z1 owner must verify the
first program with the machine in the air. The Z1's manual tool change is
not specially handled: the Carvera post's M6 sequence is what is posted.

**Considered and dropped.** Inventing a separate `z1.cps` (no dialect facts
to put in it); adding 4th axis or laser (optional modules, not the base
machine); guessing a 15000 rpm ceiling (a refusal bound must not exceed what
Makera states).
