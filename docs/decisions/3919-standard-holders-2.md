## #3919 / #4239 - Holder library: shrink-fit chucks, BT50 and CAT50 (second slice); the tapers as spindle mounts

**What was decided.**

1. **Shrink-fit chucks** (`crates/tools/src/holder_library_shrink.rs`)
   on each taper of the library but TTS: BT30, BT40, CAT40, SK30, SK40,
   ISO30, HSK-A63, HSK-F63, HSK-E40, HSK-E32, BT50 and CAT50. A new
   clamp kind, `Clamp::Shrink { bore, nose_d, body_d }`, with the names
   "<taper> Shrink <bore> mm" ("BT40 Shrink 12 mm").
2. **BT50 and CAT50** (`crates/tools/src/holder_library_fifty.rs`), each
   with ER16-ER40 collet chucks and every Weldon bore of the existing
   tables (6-20 mm, 1/4-3/4 in), default gauge length 100.
3. **#4239: the tapers are spindle mounts.**
   `camcraft_post::machine::SPINDLE_MOUNTS` lists each taper of the
   holder library, in its order (13 names). The post crate does not
   depend on the tools crate, so the list stays a hand list. The app
   test `every_library_taper_is_a_spindle_mount`
   (`crates/app/tests/machine_default_holder.rs`) keeps the two lists
   equal: a new taper in the library without its mount fails that test.
   A mount that the library does not have is refused by its name, as
   before.
4. **The picker groups the shrink-fit bores** like the Weldon bores
   (#2018): one row "BT40 Shrink" for each taper, resolved to the bore
   that takes the tool's shank (`holder_library::shrink_for`).

### The shrink-fit outline

The form is the DIN 69882-8 shrink chuck: a 4.5-degree outer taper from
the nose diameter d2 to the body diameter d3, for each clamping diameter
d1. The outline is our own envelope:

- The nose face is drawn at d2/2 + `SHRINK_MARGIN`. A cone at tan(4.5°)
  rises from it to the body radius d3/2 + `SHRINK_MARGIN`.
- The 45-degree flare envelope of `holder_library.rs` runs down from the
  flange's front face. The outline is the larger of the cone/body and
  the flare at each height. The flare rises faster than the cone and the
  body, so the two cross at most once (`envelope`).
- So a short chuck (its cone runs into the flare before it reaches the
  body) is drawn whole, and the shortest gauge length is only the flange
  front + `MIN_BODY`. An ER or Weldon entry still needs its whole flare
  below the flange.
- `SHRINK_MARGIN` = 2.0 mm (radius). Reason: the makers' tables agree
  on d2 and d3, but one maker's "standard version" chuck is wider. Its
  SK30 16 mm chuck has a Ø30 nose (the table: 27) and a Ø37.06 body
  (the table: 34), reached 45.9 above the nose face. That is 1.53 mm
  more in radius; 2.0 covers it with room. The test
  `a_wider_makers_chuck_stays_inside` pins it.
- The collision bands of the spun profile hold each climbing segment at
  its larger radius, so the stack laws judge the nose cone as a band at
  its top radius: never smaller than the steel.

### Shrink-fit table (mm)

d1 = bore, d2 = nose Ø, d3 = body Ø.

| d1 | d2 | d3 |
|---|---|---|
| 3 | 11 | 15 |
| 4 | 14 | 22 |
| 5 | 16 | 22 |
| 6, 8 | 21 | 27 |
| 10, 12 | 24 | 32 |
| 14, 16 | 27 | 34 |
| 18, 20 | 33 | 42 |
| 25, 32 | 44 | 53 |

HSK-E32 has its own d3 from 6 to 12 mm (the Ø32 flange): 26 (6, 8: d2
21; 10, 12: d2 24). Its 14 and 16 mm rows are the common 27 / 34.

Bores and default gauge lengths (the shortest length the table lists
for the bore; A = gauge line to nose face):

| Taper | Bores (A) |
|---|---|
| BT30 | 3, 4, 5, 6, 8, 10, 12, 14, 16, 20 (80) |
| SK30, ISO30 | 3, 4, 5, 6, 8, 10, 12, 14, 16, 20 (80) |
| BT40 | 3, 4, 5 (80); 6-20 (90); 25, 32 (100) |
| SK40 | 3-20 (80); 25, 32 (100) |
| CAT40 | 6, 8, 10, 12, 16, 20 (80); 25, 32 (100) |
| HSK-A63 | 3, 4, 5 (80); 6-12 (70); 14, 16 (75); 18 (95); 20 (100); 25 (115); 32 (120) |
| HSK-F63 | 3-8 (80); 10 (85); 12, 14 (90); 16 (95); 20 (80); 25 (115); 32 (120) |
| HSK-E40 | 3 (60); 4 (70); 5-10 (80); 12-20 (90) |
| HSK-E32 | 3, 4, 5 (60); 6, 8, 10 (70); 12 (80); 14, 16 (90) |
| BT50 | 6, 8, 10, 12, 16, 20 (100); 25, 32 (120) |
| CAT50 | 6, 8, 10, 12, 14, 16, 18, 20, 25, 32 (100) |

("3-20" includes 3, 4, 5, 6, 8, 10, 12, 14, 16, 18 and 20.)

Sources (makers' published catalogue tables; the standard itself was
not read):
- Rineck "Werkzeugtechnik - Shrink fit chucks, Metric" catalogue (as
  distributed by Tyson Tool, tysontool.com/ShrinkFit-Metric.pdf), the
  "standard execution 4,5°" tables for HSK-A63, HSK-E32, HSK-E40,
  HSK-F63, SK30 and SK40 DIN 69871 AD/B, BT30, BT40 and BT50 JIS B 6339:
  columns Ø d1, Ø d2, Ø d3, A, l1, l2. Cross-check: A - l1 is the gauge
  line to the flange front of each taper (SK 19.1, BT30 22, BT40 27,
  BT50 38, HSK-A63 / F63 26, HSK-E 20), the same as the taper tables.
- The same maker's CAT40 table (tysontool.com/Complete-CV40-TTCL.pdf,
  "Shrink fit chuck CAT40 AD/B, standard execution 4,5°"): 6-32 mm, the
  same d2 / d3; A - l1 = 19.1.
- Techniks "ShrinkFIT Tools" catalogue (techniksusa.com), CAT40 and
  CAT50 metric tables: the same d2 / d3 for 5-32 mm (its 4 mm row is
  slimmer), an independent agreement. The CAT50 bores and the 100 mm length are from this table.
- The wider maker's sample for the margin: Haimer shop pages "Shrink
  Fit Chuck Standard Version, DIN ISO 7388-1 SK30" (30.445.16: D2 /
  interference diameter 1.181 in, body diameter 1.459 in, length of
  body diameter 1.807 in, taper angle 4.5°) and "... DIN 69893-5
  HSK-E40" (E40.145.16: 1.181 in, body 1.299 in at 0.75 in).

### 50-size tapers (mm)

"Standoff" is the gauge line to the flange's back face; "front" is the
gauge line to the flange's front face.

| Taper | Gauge Ø | Flange Ø | Standoff | Flange thickness | Front |
|---|---|---|---|---|---|
| BT50 | 69.85 | 100.0 | 3.0 | 35.0 | 38.0 |
| CAT50 | 69.85 (2.750 in) | 98.425 (3.875 in) | 3.175 (0.125 in) | 15.875 (0.625 in) | 19.05 (0.750 in) |

Sources:
- BT50: Rego-Fix "Spindle interface norms" sheet
  (RFAG_TD_Normen_Spindelschnittstellen_EN), BT MAS 403 row BT 50: D1
  69.85, D2 100, L1 3, L2 101.8, L3 35 (the same sheet the BT30 / BT40
  rows of #793 came from). Haimer "Technical specifications JIS B
  6339-2" page, BT50: D1 69.85, D3 100, L2 38 (gauge line to flange
  front: 3 + 35). The two agree.
- CAT50: Haimer "Steep taper ASME B5.50 CAT40 CAT50" page, inch table:
  D1 2.75, D4 3.875, L2 0.75, L3 0.625. The CAT40 column of the same
  table (1.75 / 2.5 / 0.75 / 0.625) is the ASME B5.50 Table 1 size 40
  of #793, so the columns read as gauge Ø, flange Ø, gauge line to
  flange front, flange thickness.

The default gauge length of each 50-taper ER and Weldon entry is 100,
a typical catalogue length (the shrink-fit tables above list 100 for
BT50 and CAT50). The user sets the real gauge length.

### Shrink-fit chucks in the component library

The built-in component library (#3645, `component_library::builtins`)
turns each holder entry with a taper into a read-only adaptive item. A
shrink-fit chuck maps so:

- machine side: its taper, as for the ER and Weldon entries;
- workpiece side: `CYL <bore>`, a plain bore. The chuck grips the shank
  directly, with no collet. `CYL` takes a plain shank of the bore
  (within the bore slack) and a Weldon shank of that size, and refuses
  all other shanks. A Weldon shank in a shrink-fit chuck is normal shop
  practice (h6 shanks, the flat is only unused);
- geometry: `ComponentGeometry::holder` at the library gauge length,
  the same conservative outline that the holder library draws (never
  smaller than the steel). No bore stop: the tables state no bore
  depth, so a solid tool needs a stated stickout, as in a Weldon entry.

Dropped: a new `SHRINK <bore>` connection code. It would mate exactly
the same shanks as `CYL`, and each tool file and machine side would
need to know a second name for one bore. Dropped also: leaving the
shrink-fit entries out of the catalogue. The mapping is not in doubt,
and a missing entry would make the user draw the chuck by hand.

**Why.** Shrink-fit chucks are the issue's group 4; CAT50 and BT50 are
in its "Then" list. Without them a user with such a holder had to draw
it by hand, and the collision checks judged a hand-made shape. #4239: a
machine file could not state the tapers of the first slice.

**Considered and dropped:**
- **CAT30.** The published values disagree on the flange Ø (1.750,
  1.812 or 1.875 in), and no source gave the flange thickness or the
  gauge line to the flange. Left out until a table with all of them is
  found.
- **Inch shrink-fit bores** (1/4-1 in). The inch tables (Techniks CAT40
  / CAT50) give a different d2 / d3 for the same bore at each length,
  and the body grows with the length. One row for each bore is not a
  conservative outline for all of them. Left out; a metric table row
  is used for each metric bore only.
- **Odd metric shrink-fit bores** (7, 9, 11, 22 mm): only on some
  tapers in one table. Left out.
- **A grouped "<taper> Shrink" mount rule that reads the library.** The
  post crate would need the tools crate (a new crate edge for one list).
  The hand list with the app test that keeps it equal is the smaller
  change.
- **SK50, Morse, NMTB, hydraulic and milling chucks, ER50, TG, SF32**:
  later slices. SF32 waits for the owner's measurements.
- **The camcraft-mcp skill row of `list_holder_library`** names only
  the first tapers. It is a process file; the MCP tool description and
  `docs/agents/mcp-tools.md` carry the new text.

**Cost to change later.** Low. A shrink-fit size is one row; a taper is
one `BORES` line; the margin is one constant. A tool that references a
library holder by name reads the new numbers on the next load.
