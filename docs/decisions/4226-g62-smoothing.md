## #4226 - Heidenhain DIN/ISO post: the G62 filter mode and the T range

**What was decided.**

- `heidenhain.cps` writes the smoothing ask as
  `G62 T<tol> P01 <0|1> P02 0`. `P01 0` is finish and `P01 1` is rough,
  from `PARAM SmoothingMode`. `P02` is the rotary-axis tolerance. This
  post refuses rotary work, so it writes `P02 0`. That is the value that
  the control inserts when a program gives T only.
- The control profile decides if `P01` is written. The resolver
  (`heidenhain_profiles.rs`) has a new flag `g62_mode`:
  - `no`: TNC 410, TNC 310, TNC 426 280 462/470, TNC 320 340 55x-01/-02
    (no cycle 32), and TNC 426/430 280 472-477 (cycle 32 with T only).
  - `yes`: iTNC 530 340 420/422, TNC 320 from 340 55x-03, TNC 620,
    TNC 640, TNC7.
  - `option2`: iTNC 530 340 49x and 606 42x. The word is written only
    when the machine file lists option 2 in `installed_options`.
- The largest T is a new flag `g62_t_max_um` (1 µm grid, 0 = no cycle 32):
  - 10 mm ("0.0000 to 10.0000"): TNC 320 77185x-05 and later, TNC 620
    81760x-05 and later, TNC 640 34059x-08 and later, TNC7.
  - 99999.999 mm ("0 to 99999.9999", on the 1 µm grid of the post): iTNC
    530 34049x-05 to -08 and 606 42x, TNC 320 34055x-04 to 77185x-04,
    TNC 620 34056x-02 to 81760x-04 (and 73498x), TNC 640 34059x-01 to -07.
  - A manual that states no range takes 10 mm, the narrowest documented
    range: TNC 426/430 280 472-477, iTNC 530 340 420/422 and 34049x-01 to
    -04 (the Picomax control 340490-03 is here), TNC 320 34055x-03, TNC 620
    34056x-01.
- A T above the largest T is refused by name. A positive T that rounds to
  `T0` on the 1 µm grid is refused too: the control reads T 0 as the
  tolerance of the machine builder. A stated control without cycle 32
  (TNC 410) is refused when the program asks for smoothing. Before, the
  post wrote a `G62` that this control does not have.
- A machine with no controller identity keeps the TNC 640 34059x-07 level
  of this post: `P01` is written, T 0 to 99999.9999.
- Where `P01` is not written and the ask is rough, the post writes T only
  and gives a counted warning. The control then uses its finish filter.
  That is the more accurate filter, not a less safe one.

**Fehlmann Picomax 55 (iTNC 530, NC 340490-03).** The output is
byte-identical: `G62 T<tol>`. The machine file lists no installed options.
The iTNC 530 340 49x-03 ISO manual says that "The P01 and P02 parameters
are only available if on your machine you have software option 2 active
(HSC machining)". The research does not prove that the current output is
wrong for this control. The only change on this machine: a rough ask now
gives a warning, and a T above 10 mm is refused.

**Sources** (the HEIDENHAIN user manuals in `docs/research/heidenhain/`
`inventory.md`; section "TOLERANCE (cycle 32, DIN/ISO: G62)"):

- ISO form and the T-only rule: iTNC 530 ISO 533 188-23 (34049x-04,
  example `G62 T0.05 P01 0 P02 5`, "the TNC inserts the two remaining
  parameters with the value 0"); TNC 640 ISO 892909-26 (34059x-08, example
  `G62 T0.05 P01 1 P02 3`).
- Option 2: iTNC 530 diniso/bhb manuals 340 49x-03 to -06 and 606 42x-01.
- Effective only with the HSC filter (word accepted): iTNC 530 340 420
  (360 994-23).
- T only on the TNC 426/430: 318 179-24 (280 472, example `G62 T0.05`).
- T ranges: the cycle manuals 670 388-20..-25 (iTNC), 679 220 / 1096959
  (TNC 320), 679 295 / 1096886 / 1303427 (TNC 620), 892905-20..-28 and
  1303406 (TNC 640), 1358775 (TNC7).
- No cycle 32 on the TNC 410: 345 940-22 contents ("not TNC 410").

**Considered and dropped.**

- Write `P01` without `P02`: no manual shows that form. Every example
  has the two words.
- Write `P01` on the Picomax: the option 2 note says the control may not
  take it. That would change a tested output on a guess.
- Refuse a rough ask where `P01` is not written: it would refuse
  programs that post today. The finish filter is the more accurate one.
- One 10 mm bound for every control: simpler, but the ask was to follow
  the documented range by version.

**Cost to change later.** One flag value in the profile table and one
test row for each control. A machine file that lists option 2 on an iTNC
530 gets `P01` with no code change.
