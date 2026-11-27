## #4094 - Heidenhain DIN/ISO: the G200/G205 parameter set follows the controller identity

**What was decided.**

- `assets/posts/heidenhain.cps` gets a small helper, `drillLevel()`. It reads
  `machine.controllerIdentity` (model and NC software) and gives one of two
  parameter levels for cycles 200 (DRILLING) and 205 (UNIVERSAL PECKING):
  - `q379`: G200 without `Q395`; G205 with `Q379`/`Q253`, without `Q208` and
    `Q395`. For iTNC 530 340 422, 34049x-01 to -08 and 60642x-01 to -03;
    TNC 640 34059x-01/-02; TNC 620 34056x-01 to -04 and 73498x-01/-02;
    TNC 320 34055x-01 to -06.
  - `q208`: the output of today (`Q379`, `Q253`, `Q208`, `Q395`). For iTNC 530
    60642x-04; TNC 640 34059x-04 to -11 and -16 to -18; TNC 620 81760x-01 to
    -08 and -16 to -18; TNC 320 77185x-01, -02, -04 to -06 and -18.
- **One guard for the NC software string (#4172 review).** The post parses
  the complete canonical identifier of `dialects.md` section 1, with no
  prefix match: `^\d{5}\d-\d{2}( SP[1-9]\d{0,2})?$`. The first five digits
  are the family and the sixth is the variant. Then one table row must
  match all of: the model, the family, the variant range, and a release that
  the research names (never `-00`). The variant ranges: iTNC 530 34049 +
  `0..4` and 60642 + `0..5` (from `dialects.md`); 340 422 is the one
  variant `2` of family 34042; the TNC 640/620/320 families take `0..5`.
  The research gives no variant range for these, so the HSCI range of the
  iTNC 530 is used; a variant above it is refused. The release lists are
  the releases that the research names: TNC 640 -03 and -12 to -15, for
  example, are not named and are refused. 340 422 has no named release in
  the research, so each release `-01` to `-99` takes `q379`: Q379 exists
  from 340 422 on, and Q208 never exists before 60642x-04.
  - The first version matched a prefix (`/^(\d{5})(\d)-(\d{2})/`). It read
    `340490-03junk` and `340490-030` as release 03, and it accepted
    `340490-00` and `340499-03`. Each of these is now refused by name.
- A stated identity that is not in this table (for example iTNC 530 340 420,
  TNC 640 34059x-03, TNC 426, TNC7, or an NC software that does not parse) is
  refused by name for G200/G205: "the cycle 200/205 parameter set of <model>
  NC <nc> is not known to this post - refused, not posted". This follows
  `docs/research/heidenhain/dialects.md`: "An unknown model or a version
  outside the table: the post refuses by name".
- On the `q379` level the post drops `Q395` only when it asks `Q395=0` (the
  depth is to the tool tip, the only reference of the old cycle). It drops
  `Q208` only when it asks the top of the range (`99999.999`, "no slower
  than the control's own retract"). Any other ask is refused by name
  ("G205 Q208=<v> cannot be written for <model> NC <nc>: its cycle 205 has
  no Q208"). The CAM does not ask for either value today, so this guard is
  pinned with a script variant: the test changes the constant
  `RETRACT_FEED_MAX` (the asked Q208) to 500. The guard compares the ask
  with a second, fixed constant `OLD_CYCLE_RETRACT_FEED` (99999.999), not
  with `RETRACT_FEED_MAX`. A first version compared the ask with
  `RETRACT_FEED_MAX` itself, so the variant changed both sides and the
  guard never fired (a winbuild gate found this). The test also asserts
  that the variant changed the script, so a stale replace cannot pass.
- **No controller identity: keep the output of today (`q208`).**

**Why (no identity).**

- The post is specced from the TNC 640 manual 892905-25 (34059x-07). That
  level writes `Q208`/`Q395`, and the 606425-04 programming station accepted
  the blocks (#3095).
- The research names the failure of these words on an older control as a
  load-time refusal: "The control probably refuses the block on load, as an
  alarm with no motion" (`picomax-55-review.md`, defects 2 and 3). The words
  change no tool path: `Q395=0` is the tool-tip reference that the old cycle
  always uses, and `Q208` is a retract feed along the hole axis. So a
  machine with no stated control gets a visible alarm at worst, not a
  wrong cut. This is different from cycle 247 (PRESETTING), where the research
  shows a silent change of meaning.
- To refuse G200/G205 for each machine without an identity would refuse all
  drilling on each DIN/ISO machine file that exists today. No shipped machine
  file that names a HEIDENHAIN control has no identity.

**Dropped.**

- An open upper release bound for the q208 families (TNC 640/620/320): a
  newer release is not in the research, so it is refused by name until a
  research row adds it. A refusal is the safe failure.
- A variant range of `0..9` for the TNC 640/620/320 families: the research
  names no variant above 5.

- Writing the `q379` level when there is no identity: it is not confirmed
  that a newer control accepts a G205 block without `Q208`/`Q395`.
- `Q373` (TNC 640/620 -16, TNC 320 771851-18, TNC7): out of scope. These
  controls keep the output of today; TNC7 is refused (not in the table).
- Option 19 on the TNC 620 (cycles 201-205): out of scope, for #4095.
- The dwell (`FUNCTION DWELL` / `G04 F`): #4093, claimed by a contributor
  (PR #4158). This PR does not change the dwell lines.

**Cost to change later.** The table is one function. The dialect resolver of
#4095 replaces `drillLevel()` and keeps the two word sets.
