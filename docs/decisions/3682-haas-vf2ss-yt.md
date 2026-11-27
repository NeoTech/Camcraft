## #3682 - Haas VF-2SS YT 3-axis machine definition

**What was decided.** Add `assets/machines/haas-vf2ssyt.mch` (`"Haas VF-2SS YT"` in
`posting::MACHINES` and `embedded()`) to the built-in machine library. Follow
the #1976 Haas conventions: original strict JSON in CamCraft format, metric
units (`mm`, `degrees`), full-stroke program-coordinate travel bands (`-762..762`
on X, `-508..508` on Y, `-508..508` on Z), and machine-coordinate Z
`safe_retract: 0.0` for top home.

Keep `general.vendor` as `"Camcraft"` and `general.model` as `"HAAS VF-2SS YT"`,
matching the built-in naming convention checked by `haas_machines.rs`. Set
spindle limits to `12000.0` rpm, `22.4` kW (30 hp), and `rigid_tapping: true`.
Set X/Y/Z feedrate limits to `21158.0` mm/min (`833 ipm * 25.4 = 21158.2`
rounded down by `0.2` mm/min as in `haas-vf2ss.mch`) and rapid limits to
`35560.0` mm/min (`1400 ipm * 25.4`, exact). Record `30` tool-changer pockets
(excluding the `+1` loaded spindle position), `spindle_mount: "CAT40"` (CamCraft
identifier for CT40), `default_holder: "CAT40 ER32"`, Flood coolant (`M8`) only,
and controller `"post": "Haas"`.

**Why.** Supplies the existing parser, picker, and machine checker with the
extended 508 mm Y-axis VF-2SS YT travel envelope requested in #3682 while
keeping `haas-vf2ss.mch` (406 mm Y travel) unchanged and remaining consistent
with the five #1976 Haas definitions.

**Considered and dropped.**
- Recording `31` tools (`30+1` including the spindle): dropped because #1976
  records only changer pockets (`30`) and Haas publishes `30+1` for standard SMTC.
- Adding optional `15,000` rpm spindle, `50+1` tool changer, or through-spindle
  coolant (`M88`): dropped because these are machine options rather than the
  base configuration; users with installed options edit a copy.
- Using `35600.0` mm/min from the rounded metric display (`35.6 m/min`): dropped
  in favour of exact `1400 ipm * 25.4 = 35560.0` mm/min per #1976.

**Cost to change later.** Low: editing one `.mch` JSON file and its row in
`crates/app/tests/haas_machines.rs`.

**Sources.** Consulted 2026-11-25 from public Haas Automation material (only
numeric specifications and standard M/G identifiers were used; no vendor
`.mch`/`.cps` files, images, or substantial prose were copied):
- Haas Automation VF-2SSYT product page
  (`https://www.haascnc.com/machines/vertical-mills/vf-series/models/small/vf-2ssyt-i.html`):
  Travels (`30.00 in / 762 mm` X, `20.00 in / 508 mm` Y, `20.00 in / 508 mm` Z),
  Spindle (`30.0 hp / 22.4 kW`, `12,000 rpm` inline direct-drive, `CT 40` /
  `BT 40` / `HSK-A63`), Feedrates (`1,400 ipm / 35.6 m/min` rapids, `833 ipm /
  21.2 m/min` max cutting), Tool Changer (`30+1` SMTC), and Standard Features
  (rigid tapping, flood coolant).
- Haas Automation VF-SS series flyer
  (`https://www.haascnc.com/content/dam/haascnc/assets/seriesflyers/20.01_vfss_4-pager.pdf`,
  cited in #3682): VF-2SSYT travels `30" x 20" x 20"` (`762 x 508 x 508 mm`).
