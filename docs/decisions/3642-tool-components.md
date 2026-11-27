## #3642 - Tool components: the ISO 13399 model, the insert and holder codes, and the fit checks

**What was decided.**
- The model is pure data in `camcraft-tools`, in six files:
  `insert_codes.rs` (the letter tables), `insert_iso1832.rs` (the insert
  code), `insert_seat.rs` (the seat and the ISO 5608 holder code),
  `connection_code.rs`, `component.rs` and `component_fit.rs`. There is no
  new crate edge, no ECS and no file I/O.
- `ToolComponent::new` checks a component when it is made. The fields are
  private, so a component that exists is valid. The id is a plain string;
  #3644 and #3645 decide how ids are made and stored.
- Validation rule (#4096 review): a component checks each value that
  it is given, also the values built in code. `InsertSpec` and
  `InsertSeatSpec` keep their public fields (the parsers and the
  stacked #3643 build them), and each has a `check()`. `InsertSpec::check`
  refuses a tolerance letter off the table or in lower case, a size off
  01 to 99, a corner that the shape cannot have, a suffix character that
  the parser changes or refuses, and a hand that the suffix does not
  state. Then it confirms that the value is the parse of its own text,
  which closes the whole class. `InsertSeatSpec::check` refuses an
  insert size off 01 to 99; its letters, its style and its thickness are
  types that hold only valid values. `ToolComponent::new` runs both
  checks and names the component, the seat port and the position.
  `check_insert_fit` runs them again as a guard.
- Connections are normalized, not refused: `ConnectionCode::canonical()`
  is the value that the code's text parses to (`Hsk { form: 'a' }` is
  `HSK-A63`, `Custom(" Shop pallet ")` is `CUSTOM:Shop pallet`).
  `ToolComponent::new` stores the canonical form, and `takes` compares
  canonical forms. An invalid code mates nothing. Thus validation, the
  text round trip, the component and the fit agree. A strict refusal of
  a non-canonical typed value was dropped: the parser already accepts
  the same text in any case, so the typed value follows the same rule.
- One `ConnectionCode` type serves both sides (ADINTMS and ADINTWS). A
  parent's workpiece side and a child's machine side then compare
  directly. The text form is our own (`BT40`, `HSK-A63`, `ER32`,
  `CYL 12`, `WELDON 12`, `RECT 25x25`, `CUSTOM:<name>`). Sizes must be in
  the published series, so a typo such as `BT45` is refused.
- Coupling rule: the same code fits, with diameters within the holder
  library's `BORE_SLACK` (0.02 mm). A plain bore or a collet also takes a
  Weldon shank of its size. A Weldon bore takes only a Weldon shank,
  because its screw needs the flat.
- Insert rule, in this order: shape, clearance, size code, thickness (only
  when the seat states one), clamping against the fixing hole, hand. A
  neutral insert, or one that states no hand, fits each hand of holder. A
  handed insert must match the seat's hand. The tolerance class is not
  checked.
- Clamping against fixing: C takes an insert with no hole. M, P and D take
  a cylindrical hole. S takes a countersunk hole. A special fixing (X)
  fits no standard clamp.
- The ISO 1832 parser reads the metric layout only. An inch code is too
  short for it and is refused by name. Position 7 is a nose radius, a
  sharp corner (`00`), a round insert (`M0`, or `00` on shape R), or a
  milling wiper facet (two letters). What follows position 7 is kept as
  written. The hand is read from it only in the plain forms (`R`, or an
  edge letter and then `R`/`L`/`N`).
- The inscribed circle is derived from the shape: each edge is the sum of
  the two tangent lengths `r / tan(a / 2)` at its corners. The size code
  is the edge cut to whole mm after 0.02 mm, so a 6.35 mm triangle (edge
  10.998 mm) reads as `11`. Parallelograms, rectangles and round inserts
  get no inscribed circle.
- The "round trip" of the model is through its text: an insert code, a
  holder code and a connection code each parse back to the same value.
  No serde derive was added: `Tool` has none, and the file format belongs
  to #3645.

**Why.** The issue asks for a pure model in `camcraft-tools` with named
refusals. The rules above are general machining practice; the code
tables are the ones tool vendors publish in their catalogs. No standard
text is copied.

**Considered and dropped.**
- Serde on the components now. Dropped: #3645 owns the
  `camcraft-components/1` format and its version rules.
- A collet range on an ER chuck's workpiece side (as the built-in holder
  library does). Dropped: in the component model the collet is its own
  component, so the chuck takes an `ER<n>` collet and the collet takes the
  shank.
- An ISO 6261 boring bar code parser. Dropped: not in the issue. A boring
  bar's seat can be made directly as an `InsertSeatSpec`.

**Cost to change later.** Low. Nothing outside the new files reads the
model yet. A changed rule changes the pinning tests in
`component_fit_tests.rs` and `insert_designation_tests.rs`.
