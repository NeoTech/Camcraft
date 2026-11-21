## #3102 - Heidenhain G205: the retraction feed Q208 is a number, not FMAX

**What was decided.**

- `heidenhain.cps` writes `Q208=+99999.999` in the G205 UNIVERSAL PECKING definition. It wrote `Q208=FMAX` before.
- 99999.999 mm/min is the top of the documented Q208 input range. It asks for the fastest retract that the control permits. The control holds a programmed feed to the machine's own limit.
- The Heidenhain cycle tests now fail on any Q value that is not a signed number. This covers G200, G205, G206 and G207, not only Q208.
- The golden file `crates/app/tests/golden/nc/heidenhain/drill_peck.nc` changes in this one word.

**Why.**

- The user's iTNC 530 programming station (NC software 606425 004 SP20) prefixed the whole `N100 G205` block with `ERROR =` on a DIN/ISO import. With only `Q208=+60` in place of `Q208=FMAX`, the station read the complete block (#3102).
- HEIDENHAIN "iTNC 530 Cycle Programming", ID 670388-25 (NC software 60642x-04), cycle 205: Q208 is the "Feed rate for retraction", input range 0 to 99999.999. The manual lists FMAX, FAUTO and PREDEF as dialogue inputs, but its own NC example writes the number `Q208=99999`.
- The TNC 640 manual that the post was written from (892905-25) also accepts the number. One output is correct for the two controls.
- `Q208=0` was not chosen: it retracts at the plunge feed Q206. That is safe but slower, and it changes the requested retract behaviour.

**Considered and dropped.**

- A machine property for the controls that accept FMAX. No control is known that refuses the number, so the property adds a choice and no safety.
- The machine's maximum feed from the `.mch` file. It is not set on each machine file, and the control already holds the feed to its own limit.
- A refusal of G205 on the iTNC 530. A supported number exists, so a refusal takes a working cycle away.

**Cost to change later.**

- The value is the constant `RETRACT_FEED_MAX` in `assets/posts/heidenhain.cps`, beside `qword`.
- The check on the physical Fehlmann (NC software 340490-03) is still open (#3102). If that control refuses the number too, the post must refuse G205 there.
