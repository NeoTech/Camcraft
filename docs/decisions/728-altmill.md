## #728 - Sienci AltMill (2x4, 4x4, 4x8) machine files

**Decided:** three built-in machines, "Sienci AltMill 2x4", "Sienci AltMill
4x4" and "Sienci AltMill 4x8" (`assets/machines/sienci-altmill-*.mch`),
posting through the existing grblHAL post (#710). No AltMill post file.

- **No `altmill.cps`.** The AltMill's SLB-EXT runs grblHAL, so the grblHAL
  post already speaks its dialect, the way every Tormach mill shares
  `tormach.cps`. A separate post would only be needed for a different
  firmware (as Carvera's Smoothieware needed `carvera.cps`, #707).
  *Dropped:* an AltMill profile or option inside the post.
- **No ATC or manual tool-change mode, and no rack field.** gSender and
  ncSender both take over on `T<n> M6` and run whatever is set up there:
  their own manual change, a RapidChange or Sienci ATC routine, or a pass to
  the firmware's `TC.macro` (owner of a 4x4 with an 8-pocket RapidChange
  rack and ncSender, 2026-11-10). The grblHAL post's plain `T<n> M6` on a
  line of its own is right for all of them. Which tools sit in a rack is the
  sender's and the ATC's business. *Dropped:* a post option picking manual
  or ATC, and a machine-file rack field that refuses tools outside it
  (proposed in the issue text, made redundant by the sender).
- **The first change is posted even when that tool is loaded.** The
  controller knows its current tool (`$G` reports `T3`), but the post cannot
  know it when the program is written. A tool swapped by hand between jobs
  would otherwise run at the wrong length. The sender or ATC may skip or
  re-measure a same-tool change. *Dropped:* skipping the first `M6`.
- **No length word.** The owner's `$G` shows `G43.1` active: the controller
  applies the length its tool setter measured. That confirms #710's choice to
  post no G43, G43.1 or G49.
- **`safe_retract: 0.0` (lift `G53 G0 Z0`).** The 4x4's `$22=79` enables
  homing, requires it at startup, and sets machine origin 0 at the homed
  position. Z homes up (`$23=1` inverts X only) and soft limits are on
  (`$20=1`, Z 0 to -170). So machine Z0 is the homed top, 1.5 mm
  (`$27`) under the switch. The 2x4 and 4x8 use the same controller and
  homing. *Dropped:* the generic grblHAL file's no-homing work-offset lift
  (the AltMill always homes); a margin below Z0 (soft limits already allow
  Z0).
- **Units mm.** The SLB-EXT runs `G21` and Sienci's figures are metric.
- **Travel.** Written as the full stroke either way from the work zero
  (`-L..L`), as every shipped machine is. The 4x4 uses the controller's soft
  limits (`$130-$132`: 1260, 1248, 170 mm), which sit just inside Sienci's
  1265 x 1251 x 174 mm cutting area: a program the gate passes must not trip
  a soft limit. The 2x4 (1265 x 635 x 174) and 4x8 (1265 x 2460 x 210) use
  Sienci's spec pages until an owner's `$$` confirms them. Earlier web
  figures (4x8 Y 2640 mm, Z 5.5 in) were wrong and are not used.
- **Axis speeds.** 2x4 and 4x4: 15000 mm/min X and Y, 6000 Z (spec page;
  the 4x4's `$110-$112` agree). 4x8: Y 25000, Z 6000, and X 15000. The 4x8
  page says "16,000mm/min (590 IPM)", and 590 IPM is 15000 mm/min, so the
  lower figure is taken, matching the 4x4's same X hardware. grbl's max rate
  is both the feed and rapid ceiling, so both are stated equal.
- **Spindle 1200-24000 rpm, power unstated.** The spindle is an add-on
  (Sienci's 1.5 kW or 2.2 kW kit, an ATC spindle, or the owner's own). The
  range is the owner's `$30`/`$31`, matching their spindle (PwnCNC 1.5 kW
  110 V ER20, 80 mm, air-cooled, 24,000 rpm). The power is left unstated
  rather than picking one owner's spindle or one kit.
- **Coolant Flood M8, Mist M7**, as the generic grblHAL file: grblHAL's core
  lists both, and an unwired output does nothing.
- **No rotary.** The owner's controller has an A axis set up (`$103`, the
  Vortex), but the grblHAL post refuses any A word until #714. The files
  state three axes.

**Cost to change later:** each figure is one number in one file, pinned by
`altmill_machines.rs`. The 4x4 goldens (`golden/nc/grblhal/altmill_4x4_two_tool*.nc`)
move only if the grblHAL post or the 4x4 file changes.
