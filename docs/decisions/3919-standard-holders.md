## #3919 - Holder library: HSK, DIN 69871 SK and router ISO30 holders (first slice)

**What was decided.** The built-in holder library gets seven more
tapers, each with ER collet chucks and Weldon side-lock holders, drawn
by the existing laws of `crates/tools/src/holder_library.rs` (full
flange cylinder, 45-degree flare envelope from the body to the flange
front, the profile stops at the gauge line, a Weldon body 1 mm wider in
radius for the clamp screw):

- `crates/tools/src/holder_library_din.rs`: SK30 and SK40 (DIN 69871 /
  DIN ISO 7388-1 form A/AD), and ISO30, the router spindles' name for
  the same size 30 taper and flange.
- `crates/tools/src/holder_library_hsk.rs`: HSK-A63, HSK-F63, HSK-E40,
  HSK-E32.

Entries (ER sizes from the one ER table, Weldon bores from the existing
DIN 6359 / inch bore tables):

| Taper | ER chucks (default gauge) | Weldon bores (default gauge) |
|---|---|---|
| SK30, ISO30 | ER16-ER32 (60) | 6-16 mm, 1/4-1/2 in (60) |
| SK40 | ER16-ER32 (70), ER40 (80) | 6-20 mm, 1/4-3/4 in (63) |
| HSK-A63, HSK-F63 | ER16-ER40 (100) | 6-20 mm, 1/4-3/4 in (80) |
| HSK-E40 | ER11-ER25 (70) | 6-12 mm, 1/4-1/2 in (65) |
| HSK-E32 | ER11-ER20 (60) | 6, 8 mm, 1/4 in (60) |

The ER and Weldon choices follow what each flange size carries (the
same cuts the BT30 / BT40 entries use). The default gauge lengths are
typical catalog lengths. They are only the start value: the user sets
the gauge length of the real holder.

Taper names have no space ("HSK-A63", not "HSK A63"). Reason: the
grouped Weldon pick ("HSK-A63 Weldon", #2018) reads a two-word name.

### Dimension table (mm)

DIN 69871 / DIN ISO 7388-1. "Standoff" is the gauge line to the
flange's back (spindle-side) face. "Front" is the gauge line to the
flange's front (tool-side) face.

| Taper | Gauge Ø | Flange Ø | Standoff | Flange thickness | Front |
|---|---|---|---|---|---|
| SK30 / ISO30 | 31.75 | 50.0 | 3.2 | 15.9 | 19.1 |
| SK40 | 44.45 | 63.55 | 3.2 | 15.9 | 19.1 |

Sources (the standard's table as tooling makers reproduce it; the
standard itself was not read):
- Haimer, "Taper DIN ISO 7388-1 (formerly DIN 69871) SK30, SK40,
  SK50" product page, dimension table: D1 31.75 / 44.45 (gauge Ø), D4
  50 / 63.55 (flange Ø), L2 19.1 / 19.1 (gauge line to flange front),
  L3 15.9 / 15.9 (flange thickness).
  https://www.haimer.com/en/products/tool-holders/taper-din-iso-7388-1-sk30-sk40-sk50
- "DIN ISO 7388-1 (DIN 69871) Technical informations" data sheet
  (SK40_DIN69871.pdf): D1 31.75 / 44.45, D6 50 / 63.55, L2 15.90, A
  3.20 (gauge line to the flange's back face). The two sources agree:
  3.2 + 15.9 = 19.1.
- ISO30 (router): Techniks "CNC Router Catalog 2016" lists its ISO 30
  holders with a Ø50 flange (C = 50 mm), the DIN 69871 size 30 flange.
  Holders that the same catalog lists as "CMS" ISO 30 (Ø46 flange) are
  a different, non-DIN flange and are not this entry.

HSK (DIN 69893-1 form A, -5 form E, -6 form F; ISO 12164-1). The plane
(contact) face of the flange seats on the spindle face, so it is the
gauge line: no standoff. "Collar" (f1) is the plane face to the collar's
tool-side face; it is drawn as a full cylinder at d1, so the gripper
groove, the orientation notch and the drive slots are enveloped.

| Taper | Flange Ø (d1) | Collar f1 | Taper gauge Ø (d2) | Taper length l1 |
|---|---|---|---|---|
| HSK-A63 | 63 | 26 | 48 | 32 |
| HSK-F63 | 63 | 26 | 38 | 25 |
| HSK-E40 | 40 | 20 | 30 | 20 |
| HSK-E32 | 32 | 20 | 24 | 16 |

Sources (the standard's table as tooling makers reproduce it; the
standard itself was not read):
- ISCAR, "HSK DIN 69893 (ISO 12164-1 Standard)", form A table, row A63:
  d1 63, d2 48, l1 32, f1 26.
  https://www.iscar.com/eCatalog/Ecat/datafileENM/INFO/1352.pdf
- GDP Tooling, "Identifying HSK Tool Holder Interface": DIN 69893-1
  row 63 (d1 63, l1 32, f1 26); DIN 69893-5 rows E32 (d1 32, l1 16, f1
  20) and E40 (d1 40, l1 20, f1 20); DIN 69893-6 row F63 (d1 63, l1 25,
  f1 26). https://gdptooling.com/identifying-hsk-tool-holder-inferface/
- Haimer, "Hollow taper shank DIN 69893 HSK-A, HSK-E, HSK-F" product
  page: A63 D1 63, D3 48.010 (d2), L1 32, L3 26 (f1); F63 D1 63, D3
  38.009, L1 25, L3 26; E32 / E40 D3 24.007 / 30.007.
  https://haimer.com/en/products/tool-holders/hollow-taper-shank-din-69893-hsk-a,-hsk-e,-hsk-f
- TAC Rockford HSK-E and HSK-F blank worksheets: E32 / E40 d1 32 / 40,
  f1 20; F63 f1 26. They agree with the rows above.

The taper gauge Ø and length are recorded for completeness only: the
taper is inside the spindle and is not drawn.

**Why.** These are the most common tapers that the library did not
have (issue groups 1 and 2). Without them, a user with such a spindle
had to draw the holder by hand, and the collision checks judged a
hand-made shape.

**Considered and dropped (for later slices of #3919):**
- **ISO20 and ISO25 router tapers.** No published table gave the gauge
  Ø, the gauge line to the flange, or the flange thickness. Only one
  maker's catalog gave a flange Ø (30 and 38, its own values). Left out
  until a standard or maker table with these numbers is found.
- **R8.** R8 is a de facto shank standard. No published standard
  describes anything below the spindle nose: each R8 holder's body is
  the maker's own. An R8 ER chuck drawn at nut width is the same shape
  as the existing standalone "ER.. chuck" entries, which an R8 user can
  pick today. Left out until a holder family with published body
  dimensions is chosen.
- **SF32** (a user's Fehlmann Picomax 55). Numbers that we have:
  - From a user's STEP model of the taper, drawn from their own caliper
    measurements (not committed): large end Ø30.8 at the top face, a
    7.6-degree half-angle taper, a Ø20.6 groove from 12.4 to 16.7 below
    the top face with 45-degree flanks, the drive dog Ø18 with flats
    9.4 wide, 42.5 to 54.25 below the top face.
  - From Fehlmann's public "Maschinenzubehör" catalog (fehlmann.com,
    pages 21-22): ER11 nut Ø19 (L 40 / 70), ER16 nut Ø28 (L 45 / 75),
    ER25 nut Ø42 (L 32-110), ER32 nut Ø49.5 (44), Weldon chucks D2
    25-52 with L 27-46.
  - Missing: the diameter and length of the shoulder between the
    taper's large end and the holder body, and the datum of the
    catalog's L (the ER drawings and the Weldon drawing start L at
    different faces). These set the outline below the spindle face, so
    SF32 waits for them.
- **Shrink-fit holders, CAT30 / CAT50 / BT50, Morse, NMTB, hydraulic
  and milling chucks, ER50, TG**: later slices.
- **Machine files.** `tooling.spindle_mount` still takes only BT30,
  BT40, CAT40 and TTS (`camcraft_post::machine::SPINDLE_MOUNTS`). The new
  tapers are selectable in the tool library and the tool editor; a
  machine file that states one is a follow-up issue.

**Cost to change later.** Low. Each taper is one `Taper` constant and
one `entries()` list. A tool that references a library holder by name
reads the new numbers on the next load.
