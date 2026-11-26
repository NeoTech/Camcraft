# #3071: hobby router frames as machine files

Six library machines on the **Grbl 1.1** post. Each value comes from the
maker's own public page (read 2026-11-26). A value that no maker page
states is not written: the unstated-limits law checks nothing for it.

## Sources, value by value

### Carbide 3D Shapeoko 5 Pro (4x4, 4x2, 2x2)

Source: Carbide 3D, "Shapeoko 5.1 Pro Specifications",
`https://carbide3d.com/shapeoko/shapeoko5pro-specs/`. The table fills its
cells with a script from the figures in the page source (mm values).

| Value | Page figure | File |
|---|---|---|
| Machine travel, 4 ft axis | `1249-12` mm | 1237 mm |
| Machine travel, 2 ft axis | `635-12` mm | 623 mm |
| 4x4 travel (X) x (Y) | 4 ft x 4 ft | 1237 x 1237 |
| 4x2 travel (X) x (Y) | 4 ft x 2 ft | 1237 x 623 |
| 2x2 travel (X) x (Y) | 2 ft x 2 ft | 623 x 623 |
| Total Z travel | `165 - 10` mm | 155 mm |
| Router or spindle | the owner supplies a 65 mm router or spindle | not stated |
| Controller | Carbide Motion (Grbl) | Grbl 1.1 post |
| Homing | inductive sensors, dual Y homing | no `safe_retract` (see below) |

Carbide 3D's launch post ("Introducing Shapeoko 5 Pro",
carbide3d.com/blog) gives the same Z travel: 6 in (155 mm).

### Inventables X-Carve 2021 1000 mm

Source: Inventables' machine page for the 2021 upgrade,
`https://easel.com/partners/machines/inventables-x-carve-nov2021` (Easel is
Inventables' own software). The page says the model is no longer sold.

| Value | Page figure | File |
|---|---|---|
| Cuttable area | 29.5 x 29.5 x 4.5 in | 749.3 x 749.3 x 114.3 mm |
| Router | Makita, 1.25 HP (800 W), 10,000 to 30,000 rpm | 10000..30000 rpm, 0.8 kW |
| Controller | X-Controller, a Grbl controller, USB | Grbl 1.1 post |

### Inventables X-Carve Pro (4x4, 4x2)

Source: `https://easel.com/partners/machines/inventables-x-carve-pro`.

| Value | Page figure | File |
|---|---|---|
| 4x4 cuttable area | 48 x 48 x 4 in | 1219.2 x 1219.2 x 101.6 mm |
| 4x2 cuttable area | 48 x 24 x 4 in | 1219.2 x 609.6 x 101.6 mm |
| Spindle | 2 HP (1.5 kW) VFD, 8,000 to 24,000 rpm | 8000..24000 rpm, 1.5 kW |
| Controller | Grbl 1.1h (Inventables firmware), USB | Grbl 1.1 post |

## Decisions

- **Chosen: the Grbl 1.1 post for the two makers.** Both name a Grbl
  controller. The post writes a tool change as an M0 stop, with the tool in
  a comment, because Grbl 1.1 has no M6. Carbide Motion and the X-Carve Pro
  tool changer can take a change in their own way; that is outside this
  file. Dropped: the grblHAL post, because no maker names grblHAL.
- **Chosen: no `safe_retract`.** The Shapeoko page names homing switches,
  but no page states where machine Z0 sits after homing. Without that, a
  `G53 G0 Z0` lift is a guess about which way is up. The post lifts in the
  work offset to the proven clearance height and refuses an unknown one by
  name. Dropped: `safe_retract: 0` like the AltMill files, because those
  came from a machine's own `$$` settings.
- **Chosen: no axis rates.** No maker page states a feed or a rapid rate
  for these frames. Reseller pages give some figures; they are not the
  maker's words, so they are dropped.
- **Chosen: travel is the stated travel either way from the work zero**,
  as every shipped machine writes it (`-T..T`).
- **Chosen: a model name without parentheses** (`Inventables X-Carve 2021
  1000 mm`). The Grbl post writes the model as the first comment line, and
  Grbl comments cannot hold `(` or `)`, so the post drops them.
- **Not done here: Onefinity.** Onefinity's compare page
  (`https://www.onefinitycnc.com/compare-product`) gives Gen 2 Elite figures
  per series only, and the series page fills its table by script that the
  read did not show. The current controller is "Redline CNC Controller
  powered by Realtime CNC motion software". No built-in post is confirmed
  for its dialect, and the older Buildbotics and Masso controllers are not
  confirmed either. The Onefinity frames stay open on #3071.
