## #772 - Lathe posts: constant surface speed is per section

**Decided:** both lathe posts (`fanuc-lathe.cps`, `tormach-lathe.cps`)
reset `sCss = 0` in `endPreamble()`, so each section states its own
spindle mode: a section with a `SurfaceSpeed` PARAM posts `G96`, one
without posts `G97 S<rpm>`. In the Fanuc post the reset sits before the
live-tool branch's early return, so a live-tool section clears it too;
in the Tormach post it sits beside `spinRev = false`.

**Why:** `SurfaceSpeed`, like `SpindleDirection`, `Coolant` and
`AfterOperation`, is a per-operation ask; carrying it over makes a
later drill on centre run under `G96` at the machine-max clamp (`G50 S`
/ `D`) instead of its own rpm. The issue's "Done means" names exactly
this reset.

**Considered and dropped:** resetting in `onSection()` / `startPreamble()`
instead - the PARAMs arrive before SECTION begin (PARAMs, TOOL, SECTION
order), so a reset there would wipe the section's own ask. Emitting an
explicit `G97` at section end - redundant, the next section's spindle
line already states its mode.

**Cost to change later:** none; no golden has two sections where one
states CSS, so no posted program in the conformance set changed.
