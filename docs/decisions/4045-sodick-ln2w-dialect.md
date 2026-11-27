## #4045 - Sodick wire: an LN2W post dialect and a Sodick AQ600L machine file

**What was decided.**

- A second post script, `assets/posts/sodick-ln2w.cps` ("Sodick Wire EDM
  (LN2W)"), stands beside the Mark 21 script. The Mark 21 script
  `sodick-wire.cps` has no change, so its output is byte-identical.
- Each cut starts in the order that the owner of an AQ600L (LN2W) gave:
  `G00 X Y U0. V0.` (one line), `Z0.`, `T91`, `T84`, `T94`, the `C` word,
  then the first `G01`. The owner corrected the first order after his
  round-2 dry run (PR #4065 comment, 2026-11-25): `U0. V0.` goes on the
  rapid to the start point, not on a line of its own before the cut. This
  supersedes the order in the #4045 issue comment. `T84` / `T85` and
  `T94` / `T89` come from `edm.flushing` and `edm.tank`, as in the Mark 21
  script.
- The post writes no `G20` or `G21`. The owner's control alarms on `G21`
  (`E00170`). Thus the script declares `programUnits = "mm"`, and the
  engine refuses an inch document by name. A header note says that the
  unit setting of the control must be millimetres.
- The post writes vertical 2-axis cuts only. It refuses by name each
  taper: a section with `WireTaper`, an angle word, or a U V lean. The
  taper words (`TP` / `TN`, `G51` / `G52` `A`, `G50`) wait for #4075.
- A skim pass takes the `edm.skim_flushing` word (`T85` on the AQ600L)
  and its own `C` word before its first move.
- A new machine file, `assets/machines/sodick-aq600l.mch` ("Sodick
  AQ600L"), names the LN2W post in `controller.post`. Its U/V travel is
  80 x 80 mm (+-40 mm), taper +-20 degrees. It states no
  `edm.taper_angle`. Its example guide span is 100 mm, so a 20 degree
  lean (100 x tan 20 = 36.4 mm) stays inside the 40 mm half travel.
- The goldens are the EDM test parts square and 2-axis
  (`mkinney/cam-tools-and-fixtures`, `samples/edm-validation`, commit
  `c687f53f9`), copied to `testdata/models/edm/`. They are our own parts.
  The taper test part is pinned as a refusal by name. The three STEP
  files have rows in the kernel STEP bench (`step_agreement`).
- The header keeps the line `NOT VERIFIED ON A CONTROL`.

**Why.**

- The engine evaluates one script text and has no include. A dialect
  option in the Mark 21 script cannot be the default post of a machine
  file, because `controller.post` names a registry entry. A second script
  gives the AQ600L its default post and cannot change the Mark 21 output.
  The shared helper functions are copied.
- The path stays the wire-centre path with the kerf in it. The owner's
  start sequence has no `G41` / `G42` and no `H` word.
- The review of head `b12db2f06` found four faults in the taper output:
  - `TP` / `TN` came from one machine value (`edm.table_to_part_bottom`)
    for each setup. A part on a 5 mm fixture posted heights 5 mm too low.
    CamCraft has no table height for each setup.
  - The machine file stated `mitre` corners with no source. A control
    that rounds the lower corner cuts a different surface from the one
    that CamCraft checked.
  - The post mapped a positive stream angle to `G51`. The app signs the
    angle by `edm.taper_angle.positive`, so a `top_right` machine leaned
    to the wrong side.
  - A nonzero angle below 0.0005 degree rounded to `A0.`.
  Each of the four can give a wrong surface with no refusal. The safe
  answer is a refusal by name until each is stated and confirmed (#4075).
- The U/V travel follows the review, which cites Sodick's AQ400L / AQ600L
  brochure, page 7: 80 mm on each taper axis.

**Sources.**

- Sodick Inc., "Wire EDM Machine Operation Training, LN1W / LN2W Control"
  (part number 6300015, January 2012). This session could not open the
  document (the web request was blocked). The facts are those that the
  repository already records from it: #4045 (body), #1500 (comments),
  and the decisions #1496, #1497, #2649 and #2686.
- Sodick, AQ400L / AQ600L brochure, page 7, as the review of PR #4065
  cites it (U/V 80 x 80 mm). This session could not open it.
- The statements of the owner of a Sodick AQ600L with the LN2W control,
  relayed on #4045, #1500 and PR #4065 (2026-11-25).
- No other CAM product's post or output was read.

**Not confirmed. Check before the PR is flagged ready, or at the control.**

- **The AQ600L figures.** X/Y 600 x 400 mm, U/V 80 x 80 mm and taper
  +-20 degrees. Check them against Sodick's published AQ600L
  specification. The thickness at which the 20 degree rating applies is
  not recorded.
- A program with no `G40` at the start. The owner's sequence and the
  manual's example have none. A header note says that no wire offset is
  used.
- The guide heights 0 and 100 mm and the conditions `C001` / `C002` /
  `C903` in the machine file are examples. The file says so.

**Dropped.**

- A dialect option in `sodick-wire.cps`: see "Why".
- `G40` on the `G00` (the Mark 21 form): it is not in the owner's
  sequence.
- `U0. V0.` on a line of its own before the first cut: the owner
  corrected it (see "What was decided").
- A constant taper as `TP` / `TN` with `G51` / `G52` `A` and `G50`, and
  the machine key `edm.table_to_part_bottom` with the stream parameter
  `PARAM WireBottom`: written on head `b12db2f06`, removed after the
  review (see "Why"). #4075 holds the work to post a taper again. The
  removed code is in the history of PR #4065.
- `edm.taper_angle` in the AQ600L file: no source states its corner rule
  or the lean side of a positive angle.
- `G41` / `G42` with `H` registers for each pass: the wire laws judge the
  generated wire-centre path, and no stored `H` value ties a control
  offset to that path (#1498). A `COMP` record is refused by name.
- 4-axis cuts (`G74` / `G75`, U V profiles): #2130 is open for them. The
  post refuses them by name.
- An inch program: no unit word can be written.

**Cost to change later.** The start order is one function (`startCut`).
A taper is one refusal in `onSection` and one in `cutOk`; #4075 replaces
them with the taper words. Each change moves the goldens under
`crates/app/tests/golden/nc/sodick-ln2w/`.
