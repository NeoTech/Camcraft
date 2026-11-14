## #931 - work offsets as an index, the post supplies the word (slice 1: post crate and scripts)

**Decided.** The stream's `PARAM WorkOffset` is the setup's 1-based offset
number. The engine reads the script's manifest once at load
(`camcraft_post::post::work_offset::Manifest`): a script with
`workOffsetIndex = true` receives the number and states `workOffsetMax`
(6 when unstated); the engine refuses a higher number by setup name. A
script without the flag receives the word `G54`..`G59` for 1-6, exactly as
before, and is refused any higher offset by setup name - an unflagged
script cannot raise its own maximum. The engine also reads the legacy
words `G54`..`G59` in a stream as 1-6 (streams and tests written before
#931); any other value (`G53`, 0, 2.5, `G54.1 P3`) is refused by setup
name before the script sees it. `work_offset::work_offsets(cps)` returns
the maximum and the names (`workOffsetName(n)`) for slice 2's display;
it never feeds the stream.

The app is unchanged in this slice: `Dump::wcs` still takes the word app
computes, and `dump` writes it into the stream as its index (`G55` -> 2).
Slice 2 turns the field into the index.

**Each dialect's maximum and its source.**

| script | max | words | source |
|---|---|---|---|
| tormach.cps | 9 | G54..G59, G59.1..G59.3 | LinuxCNC G-code reference (linuxcnc.org/docs/html/gcode/g-code.html): "G54-G59.3 Select Coordinate System (1 - 9)", and the G10 L2 table mapping P7/P8/P9 to G59.1/G59.2/G59.3. The script already cites this reference as the interpreter PathPilot is built on |
| tormach-lathe.cps | 9 | as above | same reference, already cited by the script |
| fanuc.cps, doosan-dnm.cps, mazak.cps, fanuc-lathe.cps | 6 | G54..G59 | `G54.1 Pn` extended offsets are an option on these controls; no public manual was read for them in this slice |
| haas.cps | 6 | G54..G59 | Haas' public G154 page refused an automated fetch (HTTP 403), so `G154 Pn` was not verified and is not written |
| sinumerik.cps | 6 | G54..G57, G505, G506 | 840D sl Fundamentals (as #934): G505..G599 are settable frames, but how many exist is the machine's configuration (machine data), so the post states only the six #934 already writes |
| heidenhain.cps | 6 | preset rows 1..6 (`G247 Q339=n`) | the preset table's length is the machine's; rows 1..6 as #51 set |
| carvera.cps | 6 | G54..G59 | no extended offsets verified for the firmware |
| tormach-plasma.cps | 6 | G54..G59 | the plasma manual and guidelines the script cites state no more; PathPilot plasma runs on LinuxCNC, but the plasma sources were not checked for G59.x |

**Why.** One law for the index, in the engine, so a dialect's refusal is
the same sentence for every script and an out-of-range offset can never
fall through to a script's default word (offset 9 silently posting at
offset 1 is a part cut at the wrong zero). Extended words only with a
cited public source: a guessed word posts a program a control rejects or,
worse, reads differently.

**Probe register arithmetic.** The Fanuc-family probe write
(`#5221 + 20 x slot`) reads the offset number (`wcsIndex - 1`) instead of
parsing the word back (`slotOf`), and Sinumerik's `$P_UIFR[n]` uses the
number directly (offset n is settable frame n) instead of `frameOf`. Both
give the same register for offsets 1-6; every such dialect declares max 6,
so the 1-6 register map is never exceeded. The probe's own
`SetWorkOffset` stays 0-based (0..=5) and `docs/CNC-INTERMEDIATE.md` now
says so beside the 1-based `WorkOffset`; making both 1-based was
considered and dropped for this slice (it would change the probe record
and every probing script for no user-visible gain).

**Considered and dropped.** Storing the flags as `Script` fields like
`coolantModes`: `js.rs` is already past 400 lines, so the work-offset
manifest lives in its own module and reads through one `eval_src` seam.
Changing `Dump::wcs` to a number now: it would touch app, which slice 2
owns.

**Test change.** `sinumerik_work_offsets::sinumerik_refuses_a_work_offset_that_names_no_settable_frame`
now expects the engine's refusal of `G53` ("not a work offset number")
instead of the script's: the engine refuses it before any script runs.

**Cost to change later.** Raising a dialect's maximum is two lines in its
script (the `workOffsetMax` value and `workOffsetName`) plus a cited
source and a test row in `crates/post/tests/work_offset_index.rs`.
