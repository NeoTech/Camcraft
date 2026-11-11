## #793 - Built-in taper holder library (BT30, BT40, CAT40; ER and Weldon)

**What was decided.** A built-in holder library in
`crates/tools/src/holder_library.rs`, a Rust table like the tool catalog
(`tools::catalog()`): 36 entries, each a steep-taper shank with an ER
collet chuck or a Weldon side-lock nose, drawn as a spun profile (#93).
The tool editor's Profile tab gets a "Library holder" picker with a gauge
length field and "Apply holder", which runs one undoable
`SetToolBody::library` (built on `SetToolBody::spun`), so the holder
lands as a drawn profile and stays editable point by point. The
collision checks judge it through the existing body path
(`tool_body::judged` / `stack`); no new collision code.

Entries:
- BT30: ER16, ER20, ER25, ER32 (gauge 60 mm); Weldon 6, 8, 10, 12, 16 mm
  and 1/4, 3/8, 1/2 in (gauge 60 mm).
- BT40: ER16, ER20, ER25, ER32 (gauge 70 mm); Weldon 6, 8, 10, 12, 16,
  20 mm and 1/4, 3/8, 1/2, 5/8, 3/4 in (gauge 63 mm).
- CAT40: ER32 (gauge 70 mm); Weldon 1/4-3/4 in and 6-20 mm (gauge 63.5
  mm, 2.5 in).

### Dimension table (mm)

Tapers. "Standoff" runs from the gauge line to the flange's back face;
"front" is the gauge line to the flange's front (tool-side) face.

| Taper | Gauge Ø (D1) | Flange Ø | Standoff | Flange thickness | Front | Source |
|---|---|---|---|---|---|---|
| BT30 | 31.75 | 46.0 | 2.0 | 20.0 | 22.0 | JIS B 6339 / MAS 403 BT |
| BT40 | 44.45 | 63.0 | 2.0 | 25.0 | 27.0 | JIS B 6339 / MAS 403 BT |
| CAT40 | 44.45 (1.750 in) | 63.5 (2.500 in) | 3.175 (0.125 in) | 15.875 (0.625 in) | 19.05 (0.750 in) | ASME B5.50-2009 Table 1, size 40 |

The BT values are the MAS 403 / JIS B 6339 table as tooling makers
publish it (e.g. Rego-Fix's "Spindle interface norms" sheet: BT30
31.75 / 46 / 2 / 48.4 / 20, BT40 44.45 / 63 / 2 / 65.4 / 25). The flange
front-face distances (BT40 27 mm) also match the e1 column of published
BT holder tables. The CAT40 numbers are read from ASME B5.50-2009 Table 1
(gauge diameter A, flange diameter H, the 0.125 in gauge-line-to-flange
and 0.625 in flange in Detail B, with the 60-degree V-groove).

ER collets and nuts (ISO 15488 collets; DIN 6499 standard round nut,
not the slim "mini" nut):

| Size | Collet range Ø | Nut thread | Nut Ø | Nut length |
|---|---|---|---|---|
| ER16 | 0.5-10 | M22×1.5 | 32 | 17.5 |
| ER20 | 1-13 | M25×1.5 | 35 | 19.5 |
| ER25 | 1-16 | M32×1.5 | 42 | 20.0 |
| ER32 | 2-20 | M40×1.5 | 50 | 22.5 |

The collet ranges and threads are the ISO 15488 / DIN 6499 sizes. The
nut's outside diameter and length are not fixed by ISO 15488. These are
typical values for the standard round nut, rounded up where makers
differ, so the drawn nut is never smaller than a common one.

Weldon bores (DIN 1835 form B shanks, DIN 6359 side-lock holders) and
typical body diameters:

| Bore | 6 | 8 | 10 | 12 | 16 | 20 mm | 1/4 | 3/8 | 1/2 | 5/8 | 3/4 in |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Body Ø | 25 | 28 | 35 | 42 | 48 | 52 mm | 25.4 | 31.75 | 34.925 | 44.45 | 44.45 mm |

### How a profile is drawn

The profile runs tip-up, from the nose face to the gauge line:

- the nose (an ER nut at its outside diameter over its whole length, or
  the Weldon body);
- the body up to the flange's front face (an ER chuck's body is drawn as
  wide as its nut);
- the flange as a full cylinder at its outside diameter;
- the standoff at the gauge diameter, grown by the 7/24 cone over its
  length;
- closed to the axis at the gauge line.

The taper inside the spindle is left out: the stack laws put the spindle
housing on the profile's top, which is the gauge line, where the spindle
face is.

The nose face sits at the tool's stickout (flute plus shank length) by
default; the command's `projection` overrides it. The gauge length
(gauge line to nose face) defaults to the entry's value and is the
picker's field. A gauge length too short to hold the flange, the nose
and 1 mm of body is refused by name.

**Why.** The flange is what strikes clamps and vises: a 35 mm holder
cylinder misses walls that a BT30 flange hits (pinned by
`the_bt30_er20_flange_strikes_a_wall_the_cylinder_misses`). Every
simplification errs outward: the 60-degree gripper groove, the drive
slots, the nut's chamfers and the body's neck are all enveloped, never
cut away. The library uses the existing drawn-profile body, so the
judging, saving, undo and editing paths are the ones #93 already pins.

**Small BT sizes (BT10/15/20/25): not included.** JIS B 6339-1 covers
the automatic-tool-changer shanks, and JIS B 6339-2 is identical to ISO
7388-2. Both, like ASME B5.50 (sizes 30, 40, 45, 50, 60), start at size
30. No public standard table for a BT10-BT25 shank was found. Those
names are used by makers of small high-speed spindles, with dimensions
that differ from maker to maker. Drawing one from a guess would put an
unchecked flange into a collision check. A user with such a spindle
draws the holder in the Profile table. An entry can be added once a
public standard is found.

**Other choices.**
- *Rust table vs an asset file.* The tool catalog is a Rust table
  (`tools::catalog()`). A table keeps the dimensions under unit test, and
  there is no file to go missing at runtime.
- *Shank check.* An ER chuck takes its collet range; a Weldon holder
  takes only its own bore (±0.02 mm). Anything else is refused by name
  at the command and shown live in the picker.
- *ER sizes per taper.* ER16-ER32 on BT30 and BT40, ER32 on CAT40, as
  scoped. ER40 needs a BT40 body wider than its flange and is left for
  #792.
- *Weldon bores on BT30.* Up to 16 mm and 1/2 in: larger side-lock
  bodies are wider than the 46 mm flange.
- *The Weldon clamp screw.* Its head sits about flush with the body and
  is not drawn.

**MCP.** No MCP tool sets a tool's holder or body today, so #799
("MCP: set a tool's holder from the built-in holder library") tracks a
`set_tool_holder` tool that takes a library entry name.

**Cost to change later.** Low. The library is data plus one outline
function. A changed dimension changes only the drawn points of the
holders picked after it. An already-applied holder is saved as its
drawn points, so a project never shifts under a library edit.
