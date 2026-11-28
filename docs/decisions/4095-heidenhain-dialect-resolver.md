## #4095 - HEIDENHAIN posts: one dialect resolver for both posts (first slice)

**What was decided.**

- One resolver in the post crate, `camcraft_post::post::heidenhain_dialect`
  (table in `heidenhain_profiles.rs`). It reads `controller.identity`
  (`model`, `nc_software`, `installed_options`) and gives a profile name and
  a flag set. The post engine puts the result into the script global
  `machine.heidenhainDialect`: the flags, or `{ "error": "<why>" }`. Each
  post decides if it serves the control. The code is in `post`, not in
  `app`: it works on domain values only (AGENTS.md "app holds the
  document").
- Canonical values (`dialects.md` section 1):
  - `model`: an exact HEIDENHAIN model name of the table (`TNC 310`,
    `TNC 410`, `TNC 426`, `TNC 430`, `iTNC 530`, `TNC 320`, `TNC 620`,
    `TNC 640`, `TNC7`, `TNC7 basic`). Each other name is refused by name.
    `TNC 124` and `TNC 128` are refused as paraxial controls.
  - `nc_software`: 6 digits, `-`, 2 release digits, optional ` SP<n>`
    (n = 1..999, no leading zero). Release `00` is refused. No prefix match.
  - `installed_options`: option numbers as decimal strings with no leading
    zero (`"0"`, `"8"`, `"19"`). Absent means unknown. This slice parses and
    exposes them; no flag reads them yet.
- Rows match the model, the 6-digit identifier (an exact list for the old
  controls and iTNC 530 340 420/422, a family plus a variant range for the
  others) and the release. The release lists name only the releases that
  the research names, as #4172 did. An unnamed release (TNC 640 -03 or -12
  to -15, for example) is refused, never assumed.
- The flags of this slice, with their sources (`differences.md` = `docs/
  research/heidenhain/differences.md`; "B2"/"B3" are its sections; manual
  IDs and pages are in those rows):

  | Flag | Values | Source |
  |---|---|---|
  | `languageIso` | yes / no / unknown | B2 "DIN/ISO programs" (345 940-22; 318 179-24 title; 679 226-20; 1373081_03 p17; 1358773-20). `dialects.md` section 3 for the unknown cells (TNC 426 280 462/470, TNC 320 340 551-01 to -03, TNC 620 340 56x-01) |
  | `decimals` | 3 / 4 | B2 "Coordinate decimals" (331 645-22 printed p228; 340 135-22 spec) |
  | `toolCallByName` | yes / no | B2 "Tool name in quotes" (311 062-21; 345 940-22) |
  | `toolNameMax` | 0 / 16 / 32 | B2 "Name length" (1322095-02 p24: 32 from TNC 640 34059x-02; TNC7 has the TNC 640 -16 functions, section B). 16 elsewhere |
  | `m140` | yes / no | B2 "M140" (340 135-22 p6; 360 994-23 p20; 550 671-20 p17) |
  | `m136m137` | yes / no / unknown | section B, iTNC 530 340 420 baseline (360 994-23 / 369 475-21: M136/M137); section B, TNC 410 (345 940-22: M136 "not TNC 410"). Unknown elsewhere |
  | `drillC200` | base / q211 / q395 | B2 "Cycle 200 (DRILLING) Q list" and the B3 rows that it cites |
  | `cycle247` | none / datum / presetTable / unknown | B2 "Cycle 247 DATUM SETTING" (340 135-22 p6; 375 738-24 p10; 345 940-22 p6, p19) and `dialects.md` section 3 (TNC 320 from 340 55x-03) |
  | `dwell` | cycle9 / functionDwell / unknown | B2 "Cycle 9 DWELL TIME" and "`FUNCTION DWELL`" (1322095-02 p65; 1322094_03 p48) |

- **DIN/ISO post** (`assets/posts/heidenhain.cps`):
  - No controller identity: the output of today (#4094).
  - A stated control that the resolver refuses, or whose `languageIso` is
    not `yes`, is refused by name at the program start. Nothing is written.
  - `drillC200` replaces the #4172 table (`drillLevel()`): `q395` writes
    `Q395`, `q211` does not, `base` (no Q211) refuses G200 by name.
  - `cycle247` other than `presetTable` refuses `G247 Q339=n` by name.
  - `decimals`: the post writes 3 decimals, which every profile takes. It
    refuses a profile with fewer (none exists).
  - `dwell`: no branch. DIN/ISO always writes `G04 F` (#4093), also on a
    `functionDwell` control. The tests pin this.
  - `toolCallByName`/`toolNameMax`: no branch. Both posts write the tool
    number form (`T<n>` / `TOOL CALL <n>`), which every contouring control
    takes (section A, "Common to ALL controls"). A later slice reads them
    when a post writes a tool name.
- **Klartext post** (`assets/posts/heidenhain-klartext.cps`):
  - The resolver replaces the fixed language check. The post refuses by
    name a profile that does not confirm `m140`, `cycle247: presetTable`
    and `m136m137` (the words that it writes), and it rounds coordinates to
    the profile's `decimals`.
  - The manual quill procedure stays verified for iTNC 530 NC 340490-03
    only (#4114). That is an installation fact (the M92/M140 retract and
    the stop behaviour), not a language fact, and the DIN/ISO post has the
    same check. So the Klartext post still posts only for that NC software.
    The refusal now names the resolved profile.
  - The refusal text for a machine with no identity does not change. The
    recorded refusal goldens hold the script line numbers, so the lines
    above that refusal do not move.

**Why.**

- The issue asks for one shared resolver and a refusal by name. Cycle 247
  changes its meaning between versions, so a silent fallback can move a
  datum (`dialects.md` section 1).
- The output of the shipped machine does not change. The Fehlmann Picomax
  55 (iTNC 530, NC 340490-03) resolves to `itnc-49x`: 4 decimals, M140,
  M137, cycle 247 as a preset row, G200 without Q395. This is the output of
  today for both posts.

**Behaviour changes for controls that no shipped machine file states.**

- Now refused by the DIN/ISO post, at the program start:
  - TNC 320 340 551-01 to -03 and TNC 620 340 56x-01: the research does not
    show DIN/ISO programs on them (#4172 posted G200 at the Q211 level).
  - Malformed or unknown identities that do not drill: before, only G200
    was refused. Now the whole program is refused.
- Now posted by the DIN/ISO post (refused before): iTNC 530 340 420, TNC
  426/430 280 474 to 280 477, TNC7 81762x-17 to -20. The research gives
  their cycle 200 level (B3). On iTNC 530 340 420 and TNC 426/430
  280 476/477, G247 is refused (cycle 247 sets a datum value there).

**Dropped.**

- `drill.c205`, `oldCycles`, `tilt`, `tcpm.*`, `planeAxial`, `planeSym`,
  `feedFzFu`, `arcs.*`, `q.*`, `programName`, `blockForm` as a flag, and
  the option gating. No post writes these words today. They come with the
  slice that writes them.
- A fallback to the lowest profile of a model with a warning: refused, as
  `dialects.md` section 1 recommends.
- Widening the Klartext post to other iTNC 530 releases in this slice: the
  manual procedure is verified for one release (#4114).

**Cost to change later.** One table row for each new profile or release.
A new flag is one field in `Flags`, one value for each profile, and one
table test.
