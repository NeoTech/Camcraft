## #2888 - Siemens Sinumerik 810M post and EMCO VMC-200 machine

### What was decided
1. **Separate 810M dialect post (`assets/posts/sinumerik-810m.cps`, registry `"Siemens 810M"`):**
   - Header/footer & extension: opens with `%MPF<1..9999>` (`extension = "mpf"`), ends with `M30`. Refuses program numbers outside `%MPF1`-`%MPF9999` rather than clamping.
   - Comments: parenthesized `( ... )` comments only (`;`, nested `(`, `)`, `%`, non-ASCII replaced by spaces); no 840D `;` line comments. Auto headers cap at 39 inner characters (41 chars displayed on the 810M CRT screen).
   - Sequence numbers & line length: `showSequenceNumbers` defaults to `"off"` to conserve limited 810M RAM (31–128 KB typical); when enabled, enforces the 4-digit `N1`-`N9999` limit (refuses past `N9999` rather than writing 5-digit N words). Every block enforces the 810M GA3 120-character limit.
   - Units (`programUnits = "mm"`, `endUnits = "note"`): writes `G71` and `G94`. On Sinumerik 810M, `G70`/`G71` scale geometry coordinates only and do not switch `F` feedrate units (no `G700`/`G710` exist), so inch documents and inch end-unit switches are refused/noted rather than emitting `G70` with metric feed assumptions.
   - Work offsets (`workOffsetMax = 4`): supports settable zero offsets `G54`-`G57` only. Offsets >= 5 (`G58`/`G59`, which on 810M are programmable additive shifts, plus absent `G505`/`G158`/`G159` tables) are refused by name.
   - Safe retract & tool change: `G0 G53 Z<safeRetractZ> D0` (refuses named machines omitting `safe_retract`), followed by `T<n> M6` then `D<n>` on its own block (`D1`-`D99` range checked), avoiding Alarm 3006 on combined blocks and letting the OEM ATC complete before activating offset `D`.
   - Cutter compensation & arcs: plain `G41`/`G42`/`G40` on linear moves in `G17` (no `NORM`/`KONT` 840D keywords); `G2`/`G3` use incremental `I`/`J`/`K` across `G17`/`G18`/`G19` (no `CR=`), restoring `G17` at section end. Dwell writes `G4 F<seconds>`.
   - Canned cycles via `L81`-`L89` subroutines with `R00`-`R11` parameters: explicit `R..` assignment blocks followed by `L8x P1` (rather than mixing modal `G8x` + all R words on one line, avoiding Alarm 3006 and restart hazards): `L81` (drill, dwell 0), `L82` (drill/spot with `R04` dwell s), `L83` (`PECK` `R11=0` chip-break / `DEEP` `R11=1` full retract, `R01` peck, `R05=0`), `L84` (`R08=-3` with `R09` pitch when rigid, refusing bottom dwell; `R08=-4` floating with `R04` dwell when floating/no encoder), `L85` (`REAM`), `L86` (`BORE` with `R07`), `L89` (`BACKBORE`). `FINEBORE`, probing, rotary/tilt (`A`/`B`/`C`), and smoothing (`CYCLE832` absent) are refused/noted cleanly.
2. **Machine profile (`assets/machines/emco-vmc-200.mch`, registry `"EMCO VMC-200"`):**
   - Public EMCO VMC-200 handbook specifications: X/Y/Z travels 420 / 330 / 240 mm (`-L..L`), rapid 10.5 m/min (`10500 mm/min`), working feed 4000 mm/min, spindle 50-4500 RPM, `safe_retract: 0.0`, Flood coolant `M8`, 12-tool ATC (`number_of_tools: 12`), paired `controller.post: "Siemens 810M"`. Left `rigid_tapping` and `spindle_mount` (SK40) unstated per `docs/MACHINE-ONBOARDING.md`.
3. **Owner air-cut kit (`docs/tester-kits/sinumerik-810m-vmc200/`):**
   - Short 52-line (`<1.1 KB`) metric program (`air_cut_vmc200.mpf` + `.camproj` + `README.md`) combining a 2D contour (`T1`, 2500 RPM, `G2` corners) and a two-hole peck cycle (`T2`, 1800 RPM, `L83 P1`), designed for single-block dry running with G54 Z shifted +50 mm above stock.

### Why
- The 840D (`sinumerik.cps`) dialect uses `CYCLE832`, `;` comments, `%_N_..._MPF`, `G700`/`G710`, `G505`+, and ISO-mode G8x blocks that alarm on Sinumerik 810M GA3 / EMCO WinNC.
- Explicitly invoking `L81`-`L89 P1` after assigning `R00`-`R11` and separating `T M6` from `D` avoids 810M Alarm 3006 ("wrong block structure"), prevents modal R-parameter staleness on block search, and keeps every line <= 120 characters.

### Considered and dropped
- Allowing `G70` inch output or emitting `G58`/`G59` for offsets 5-6: dropped because `G70` does not convert `F` feedrate on 810M and `G58`/`G59` are incremental programmable shifts, creating silent crash/scrap hazards.
- Emitting `N10` every block by default: dropped because `N` is limited to 4 digits (`9999`) and adds ~20-25% overhead to 31-128 KB 810M user memory; kept opt-in via `showSequenceNumbers` with hard `N9999` refusal.
- Setting `rigid_tapping: true` on `emco-vmc-200.mch`: dropped because stock VMC-200 variants vary by spindle encoder fitment; left unstated per Camcraft's unstated-capability rule while `sinumerik-810m.cps` declares `floatingTapping = true`.
- Setting `spindle_mount`: dropped because VMC-200 uses SK40/ISO40, which is not in `SPINDLE_MOUNTS` (`BT30/BT40/CAT40/TTS`).

### Cost to change later
- Low and localized to `assets/posts/sinumerik-810m.cps`, `assets/machines/emco-vmc-200.mch`, and their goldens/tests (for example, adding a variant `.mch` with `rigid_tapping: true` or toggling `G81`-`G89` modal vs `L81`-`L89 P1` cycle formatting).
