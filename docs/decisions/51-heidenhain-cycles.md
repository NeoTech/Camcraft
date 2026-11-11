## #51 - Heidenhain complete cycle and preset interface

**Decision.** Complete the existing Heidenhain DIN/ISO post with the approved
G200, G205, G206, G207 and G247 mappings. Preserve the older `TAP` contract as
rigid tapping. Add `FLOATINGTAP` as an explicit, persisted operation choice; it
is the only intent that selects G206. Keep preset selection distinct from the
control's G54 datum-shift word by applying the house mapping `G54..G59` to
preset rows `1..6`.

**Why.** The TNC 640 cycle manual distinguishes floating-holder tapping (G206,
§4.2) from rigid tapping (G207, §4.3). Machine capability cannot establish
that a floating holder is installed or requested, so a rigid operation must
never fall back to G206. The same manual says G247 selects a preset-table row
and resets coordinate transformations (§10.4); the application WCS slot is
not itself a controller preset row. The cycle manual's §2.1 describes one-shot
M99 on a positioning block, so the post calls each supported hole once with
M99; it does not use repeated M89 or assert an alternative ISO call spelling.

**Sources and limits.** HEIDENHAIN *TNC 640 User's manual for cycle
programming*, NC software 340590-07 / 340591-07 / 340595-07, English,
September 2016, document 892905-25:

- [Cycle manual PDF](https://content.heidenhain.de/doku/tnc_guide/pdf_files/TNC640/34059x-07/zyklen/892905-25.pdf)
- SHA256 of preserved reference copy: `fe12499d8840835126f801c868d8e5d108adb264fd872093971fe0eb1d8d050d`

Relevant source: §2.1 pp.58–60 (one-shot cycle calls and M99), §3.3
pp.81–82 (G200), §3.8 pp.95–98 (G205), §4.2 pp.111–113 (G206), §4.3
pp.114–116 (G207), and §10.4 p.275 (G247). These sections support cycle
names, Q inventories and call behavior. This decision does not claim
unverified ISO cycle-call grammar, controller execution, or byte-for-byte
trajectory equivalence. No controller was operated.

### Approved mapping

All depths and clearances use the entry surface as datum:

- `Q203 = EntryZ - OriginZ` (absolute surface in the shifted program frame).
- `Q200 = R - EntryZ` and `Q204 = R - EntryZ` (non-negative clearance).
  The existing explicit higher rapid hops and XY traverses are retained, so
  Q204 returns the native cycle to R; no extra guessed traverse height is
  introduced.
- `Q201 = BottomZ - EntryZ` (signed, normally negative).
- Origin shifts apply to absolute positioning and Q203, not to Q200/Q201/Q204.
  `Q395=0` keeps the stream's tool-tip bottom reference.

| Intent | Native cycle | Numeric Q inventory and fixed choices |
|---|---|---|
| `SPOT` / `DRILL` | G200 | Q200, Q201, Q206, Q202, Q210=0, Q203, Q204, Q211=P, Q395=0. A single plunge sets Q202 to total depth. |
| `DEEP` | G200 | Same Q values; Q202=Q, retaining the deep-hole full-return class. |
| `PECK` | G205 | Q200, Q201, Q206, Q202=Q, Q203, Q204, Q212=0, Q205=Q, Q258=0, Q259=0, Q257=Q, Q256=min(1 mm,Q), Q211=P, Q379=0, Q253=feed, Q208=FMAX, Q395=0. This is constant chip-break pecking with no decrement or deepened start. |
| `TAP` (legacy, rigid) | G207 | Q200, Q201, Q239, Q203, Q204. `Q239 = positive cutting feed / positive integral spindle rpm`; rigid-only range and four-decimal representability checks; no Q206 and no dwell. The current tool model is right-hand-only. |
| `FLOATINGTAP` (explicit floating holder) | G206 | Q200, Q201, Q206=positive feed in mm/min (0.001 mm/min resolution), Q211=P in seconds (0≤P≤0.5, inclusive, to avoid tool wedging), Q203, Q204. No Q239; spindle direction remains the handedness source. |
| explicit work offset | G247 definition | Q339 is the house row: G54→1, G55→2, G56→3, G57→4, G58→5, G59→6. No `WorkOffset` means leave the active preset unchanged. Emit only on row change. |

The G205 choices state the current stream's constant-peck/no-decrement policy;
they do not claim identical internal control trajectories. Q208 is the
manual's `FMAX` value, not a numeric Q value. Unsupported geometry, invalid
feeds, unsupported rigid dwell, inconsistent chain bottoms, or an unavailable
rigid spindle capability produce counted refusals, never a silent fallback.

### Backward compatibility and safety boundaries

- Persisted `tap` and stream `CYCLE TAP` remain rigid. The appended choice
  serializes as `floating_tap`; its stream marker is `CYCLE FLOATINGTAP`, while
  the script receives `CycleType=TAP` and `TappingMode=floating`.
- A post without `floatingTapping=true` receives a counted refusal for
  `FLOATINGTAP`. A machine with `rigidTapping=false` refuses rigid TAP; it
  does not authorize G206. A legacy TAP stream lacking spindle speed remains
  a named refusal rather than receiving an invented pitch.
- The engine supplies nullable `ChainFeedRate` from the first subsequent
  cutting move and `ChainBottomZ` from the expanded chain. The post uses the
  former when a legacy cycle has no feed and checks the latter when present.
- Native drilling replaces only its axial cutting chain: one M99 per hole,
  no duplicate G01 cut/dwell, no G80 or M89. Explicit rapid lifts and XY
  traverses remain. PECK/DEEP with `R != ENTRY` retain the original chain,
  because the generator starts its first infeed from R while the native
  control counts the constant peck from the surface. BORE and BACKBORE always
  retain their expanded chains.
- Probe moves, tilted/RTCP moves and control cutter compensation remain
  refused by name; these cycle additions do not broaden those capabilities.

Tests pin the mappings and boundaries through `post::engine::run` with actual
expanded chains. Test authorship is not a test-pass claim; the native gates and
any golden regeneration remain with the parent owner.

**Considered and dropped.** Do not infer floating mode from machine capability;
do not reinterpret G54..G59 as the TNC datum-shift words; do not discard
explicit rapid hops; do not map nonzero-R peck/deep cycles to a surface-based
native peck; do not apply origin subtraction to cycle increments; do not replace
boring chains; do not assert an alternative ISO call form without source evidence.

**Cost to change later.** Changing persisted mode or stream spelling requires a
migration/compatibility decision and new generator, serializer, parser, and post
regressions. Changing preset row numbering affects every explicit WCS post and
requires an owner-approved mapping plus new row-transition tests. Changing the
G205 fixed values or the R/entry fallback requires manual evidence and renewed
geometry-safety review before altering emitted cycles.
