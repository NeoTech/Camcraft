## #2789 - The STEP readers of the app read through solidcore's Part 21 parser

**Where the readers live.** The tool coordinate-system reader
(`step_csys` + `step_csys_read`), the length-unit reader (`step_units`) and
the syntax locator (`step_diag`) moved from `crates/app/src/geometry/import/`
to `camcraft-geometry`. They read text and give domain values; their `use`
lines name none of the five things that `app` holds. The app keeps:

- `step_csys.rs` as a thin wrapper: the nesting scan, the empty and UTF-8
  checks, and the map to `ImportError`. It re-exports the types, so the
  callers (`tool_body*`, MCP) did not change.
- `step_depth.rs` unchanged in code (the nesting law, see below).

The parser comes through `camcraft_kernel::p21`, a one-line re-export of
`solidcore_step::p21` in the kernel. The solidcore pin thus stays in one
manifest (`crates/kernel/Cargo.toml`). No new edge between our crates: the
app already depends on geometry, and geometry on the kernel (native only).
Dropped: a direct `solidcore-step` dependency in the app or in geometry (a
second copy of the pin to keep in step), and the kernel's own unit reader
(`solidcore_step::units`, other refusal wording than the pinned one).

The browser build (wasm32) has no solidcore. There `read_tool_csys` refuses
by name ("STEP files open in the desktop app only"), as the model import
already does, and the locator passes each record (the import refuses the
file after it).

**Entity id defined twice: refused.** ruststep's reader kept the last
definition; solidcore's parser keeps the first. A file with a duplicate id
is malformed (ISO 10303-21: an instance name is unique), and the two
readings can place the tool body at two places. The reader now refuses the
file by name ("entity #N is defined more than once ..."). No tracked
fixture has a duplicate id (scan of every tracked `.step`/`.stp`).
Dropped: keep the first (silent change of reading), and refuse only when
the duplicate is an entity the reader reaches (more code, and the same
file then reads in one place and not in another). The model import keeps
the kernel's reading (first wins, as before #2789 on its side).

**DATA sections.** A file with no DATA section is refused with the old text
"file has no DATA section": the parser reads it as a file with no
entities, so the reader asks the locator when the parser gives none.
Several DATA sections are now read as one (ISO 10303-21 edition 3 permits
them; the kernel reads them so). The old reader read only the first and
ignored the others without a word. No tracked fixture has two.

**Syntax errors.** A syntax refusal of the tool-body read now has the
locator's wording, the same as the model import ("syntax error in DATA
section: entity #N at line L cannot be parsed: ..."), else the parser's
("STEP syntax error at line L: ..."). The old text was "syntax error:"
and ruststep's message. No test asserted the old text.

**The nesting limit.** `step_depth::MAX_NESTING = 64` and `check` stay the
law in front of every read. solidcore's parser has its own private cap of
64, but it refuses only a parameter read deeper than 64: an empty
innermost `()` one level past the cap is never read as a parameter, so the
parser takes it. The scan refuses it. Pinned on both sides:
`the_parser_alone_takes_an_empty_list_one_past_its_depth_cap`
(camcraft-geometry) and `an_empty_list_one_past_the_cap_is_refused_by_the_scan`
(app). The scan's table was written against ruststep's tokenizer; re-audit
it when the solidcore pin moves.

**A length unit that no context states: refused (review of PR #2918).**
The parser maps some ISO 10303-21 short names to long names (`A2PL3D`,
`CRTPNT`, `DRCTN`, `LNGUNT`, `CNBSUN`, `LMWU`, ...) but not the short forms
of `GLOBAL_UNIT_ASSIGNED_CONTEXT` and `SI_UNIT`.
- Old reading (ruststep): it mapped no short names. A short-name tool file
  gave no placement, and the tool body fell back to its numeric placement
  ("the file names no coordinate systems").
- Reading at the reviewed head: the CIP is found, the unit context is not,
  and the reader took 1 mm per unit. An inch or metre file put the tool
  25.4 or 1000 times too small, with no refusal.
- New reading: when no context states a length unit but the file has one
  (a `LENGTH_UNIT`, an `SI_UNIT` of the metre, or a `CONVERSION_BASED_UNIT`
  over a `LENGTH_MEASURE_WITH_UNIT`), the read is refused by name ("length
  unit #N is in the file, but no GLOBAL_UNIT_ASSIGNED_CONTEXT states a
  length unit ..."). The maintainer confirmed: refuse.
- A file with no unit record at all keeps 1 mm per unit, as the ruststep
  reader did (`mm_per_unit` gave `None`, read as 1.0). Angle units alone
  (radian, degree) are no length unit.
- Pins: `a_short_name_tool_file_in_inches_or_metres_is_refused_by_name`,
  `a_short_name_tool_file_with_a_long_unit_context_reads_at_its_scale`,
  `a_long_name_tool_file_in_inches_reads_at_twenty_five_point_four`,
  `a_tool_file_with_no_unit_record_keeps_millimetres` (`step_csys_read`) and
  `a_length_unit_that_no_context_states_is_refused_by_name` (`step_units`).
- Dropped: read the short-name context and SI unit (no public source in
  the repository or the solidcore checkout states those two short names,
  and a guessed name is a guessed scale); refuse every coordinate-system
  read that finds no unit context (it would also refuse the old
  no-unit-record files, which read the same as before).
- The model import has the same gap in the kernel's `length_units`; that
  is #2919, a separate fix.

**Cost to change later.** Low. The readers are behind
`camcraft_geometry::step_csys::read_tool_csys`, `step_units::length_unit_mm`
and `step_diag::locate`; a change of reading is local to those files and
their suites (`step_csys_read`, `step_units`, `step_diag` in
`crates/geometry/tests/`).
