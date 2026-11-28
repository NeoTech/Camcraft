## #3919 - Holder library: SK50 and ER50 (third slice); what is still left out

**What was decided.**

1. **SK50** (DIN 69871 / DIN ISO 7388-1 size 50) joins
   `crates/tools/src/holder_library_din.rs`, after SK40, with ER16-ER50
   collet chucks and every Weldon bore of the existing tables (6-20 mm,
   1/4-3/4 in), default gauge length 100 (the BT50 cut of slice 2).
2. **ER50** joins the one ER table (`holder_library::ER_SIZES`) and sits
   on each 40 and 50 taper: BT40, CAT40, SK40, BT50, CAT50 and SK50,
   default gauge length 100. There is no standalone "ER50 chuck" (the
   standalone gauge table stops at ER40), and no ER50 on the 30 tapers
   or HSK.
3. **SK50 is a spindle mount.** `camcraft_post::machine::SPINDLE_MOUNTS`
   has 14 names, SK50 after SK40 (the library's order). The app test
   `every_library_taper_is_a_spindle_mount` keeps the two lists equal.

Both are drawn by the existing laws of `holder_library.rs`: the flange
as a full cylinder, the 45-degree flare envelope from the body to the
flange front, the body as wide as the nut, the profile stops at the
gauge line.

### SK50 (mm)

| Taper | Gauge Ø | Flange Ø | Standoff | Flange thickness | Front |
|---|---|---|---|---|---|
| SK50 | 69.85 | 97.5 | 3.2 | 15.9 | 19.1 |

Sources (the standard's table as tooling makers reproduce it; the
standard itself was not read):
- Haimer, "Taper DIN ISO 7388-1 (formerly DIN 69871) SK30, SK40,
  SK50" product page, dimension table, row SK50: D1 69.85 (gauge Ø),
  D4 97.5 (flange Ø), L2 19.1 (gauge line to flange front), L3 15.9
  (flange thickness). The SK30 / SK40 rows of the same table are the
  slice-1 numbers (3919-standard-holders.md), so the columns read the
  same way. Standoff = L2 - L3 = 3.2, the same as SK30 / SK40.
  https://www.haimer.com/en/products/tool-holders/taper-din-iso-7388-1-sk30-sk40-sk50

### ER50 (mm)

| Size | Collet range Ø | Nut thread | Nut Ø | Nut length (drawn) |
|---|---|---|---|---|
| ER50 | 5-34 | M64×2 | 78 | 40 |

ISO 15488 does not fix the nut's outside diameter and length. These
are standard round-nut (DIN 6499 form) catalogue values, rounded up so
that the drawn nut is never smaller than a common one (the rule of
#792):
- Collet range: Rego-Fix "ER System" brochure, ER sizes up to ER50 for
  diameters up to 34 mm; ISCAR "ER Collet Type DIN 6499" sheet
  (datafileENM/INFO/ER.pdf), row ER-50: collet Ø52, length 60, largest
  collet group 26-34. The smallest shank: Techniks' ER50 metric collet
  set covers 5-34 mm, and Maritool gives an ER50 capacity of
  0.218-1.339 in (5.5-34 mm). 5 mm is the smallest of these.
- Nut: Maritool "ER50 collet nut", Ø78 (3.07 in), M64×2. A distributor
  listing of the Rego-Fix Hi-Q / ER standard nut: Ø78, height 35.3,
  M64×2. Techniks ER50 high-speed nut: Ø78, length 35. A listing of the
  Rego-Fix Hi-Q range gives 39 as the largest height of the range. The
  nut is drawn 40 long: the largest of these, rounded up.

The ER50 nut (Ø78) is wider than a 40-taper flange (Ø63-63.55). The
body is drawn at the nut's width up to the flange front, with no flare,
and the flange steps in from it (test
`a_bt40_er50_body_wider_than_its_flange_is_drawn_whole`). The real
body is as wide as the nut or narrower, so the outline is never
smaller than the steel.

The default gauge length of each ER50 entry is 100, a typical catalogue
length. The shortest that the library draws is the flange front + the
40 mm nut + 1 mm of body (BT40: 68). The user sets the real gauge
length.

**Why.** SK50 and ER50 are in the issue's list. Their numbers are in
published tables, so the library can draw them by its laws.

**Considered and dropped (left out of this slice):**
- **SK50 shrink-fit chucks.** The shrink-fit tables of slice 2 (the
  Rineck metric catalogue, the Techniks CAT tables) have no SK50 table:
  the Rineck catalogue lists SK30 and SK40 DIN 69871 only. The d2 / d3
  columns are common, but the bores and the shortest gauge length of
  each bore are not known for SK50. Left out until such a table is
  found.
- **TG75 / TG100 collet chucks.** TG is a maker's trade system, not an
  ISO or DIN standard. No standard dimension table for the nut and body
  was found. Left out.
- **Morse taper (MT2-MT5) holders.** DIN 228 fixes the taper, which is
  inside the spindle. The steel below the spindle face (the collar, the
  wrench flats and the body of an MT collet chuck or end mill holder)
  has no standard table, and an MT5 holder must be at least as wide as
  its Ø44.4 gauge diameter even with a small nut. Without a table for
  that collar, a conservative outline is a guess. Left out.
- **NMTB 30 / 40** (ASME B5.18). No published table with the flange
  diameter and thickness and the gauge line to the flange was found.
  Left out, the same as CAT30 in slice 2.
- **Hydraulic and milling (end mill) chucks.** Their outer bodies are
  each maker's own design; no standard outline table was read. A user
  with such a chuck draws it or edits a library outline. Left out.
- **SF32** waits for the owner's measurements (see the issue).
- **A standalone ER50 chuck.** No typical straight-shank ER50 chuck
  head length was checked. Left out; the ER50 nut is on the tapers only.

**Cost to change later.** Low. A taper is one constant and one
`family` line; an ER size is one row of `ER_SIZES`. A tool that
references a library holder by name reads the new numbers on the next
load.
