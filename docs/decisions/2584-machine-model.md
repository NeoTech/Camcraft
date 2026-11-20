## #2584 - sim: the generated machine model (blocks and cylinders)

**What was decided**

- `camcraft_sim::machine_model` has its own plain input (`MachineSpec`,
  `PartSpec`). `camcraft-sim` gets no edge to `camcraft-post`. The caller
  (#2586) maps `MachineTree` into the input.
- The module is forward kinematics only. A joint `axis` is the direction
  in which the part itself moves or turns for a positive axis value. The
  caller reverses the direction of a part that moves the table.
- A part frame is the parent frame, then the part `offset`, then the
  joint. With no offset and each joint at zero, the spindle nose is at the
  origin and the spindle runs up +Z (ISO 841).
- Sizes come only from stated numbers:
  - a head: a cylinder of the spindle diameter and length over the nose;
  - a linear part: a block as long as its travel span, with a square
    section of 0.25 of the span;
  - a root or table part: a block with the linear travel of the machine
    along X and Y, and a thickness of 0.1 of the larger side.
- Head dimensions generate bodies for fixed, linear and rotary joints.
  A linear head with travel also keeps its slide block above its spindle.
- The tool station gives one more body of its own kind, `ToolEnvelope`:
  the room of the largest tool below the nose. It is not steel. A check
  selects bodies by `BodyKind`.
- A part with no number gets no body and a named note: a rotary part, a
  linear part with no travel, a head with no spindle size, a table with
  no X and Y travel.
- Parts that carry the head stand over the spindle body. All other parts
  hang below Z = 0. Thus the bodies of one stack touch and do not overlap
  at the zero pose.
- Each body has `BodySource::Generated`. `BodySource::judged_on()` gives
  the words "judged on a generated model" for a check.
- `pose` needs one value for each linear and rotary part. It refuses a
  missing value. It does not judge travel limits.
- Round faces are circumscribed polygons of 32 sides, as in
  `crate::chuck`: the drawn body is never smaller than the stated one.

**Why**

- The sign law of a table axis and the rotary conventions are in
  `camcraft-post` (`direction`, `moving_side`, the trunnion fields). A
  second copy of those laws in `camcraft-sim` could disagree with the
  post, and a wrong pose is a silent wrong answer.
- `travel` is in program coordinates, and the file states no machine
  home. Thus the model cannot know where a part is in the machine. The
  `offset` field lets the caller place each branch (work offset, tool
  length) when it knows them.
- A missing pose value that reads as zero would show a body where the
  machine does not put it.
- Accepted spindle and tool dimensions must produce their bodies. The
  joint kind changes their pose, not whether the bodies exist.

**Considered and dropped**

- A named refusal for spindle or tool dimensions on moving parts:
  dropped, because the existing joint frames can pose these bodies.
- A side flag (tool or table) in the input, with the sign applied in the
  module: dropped, because it puts a post convention in two crates.
- A body for a rotary part with a guessed size: dropped, the issue
  forbids a guessed size.
- The tool station as the size of a steel body: dropped. It states the
  largest tool, not steel.
- A check of the axis values against `travel` in `pose`: dropped. The
  limit judgment is #2585.
- `thiserror` error types: dropped. The crate's generators
  (`chuck`, `fixture`) refuse with a `String` that names the part.

**Not pinned**

- `pose` has a last check that each posed vertex is finite. The input
  checks ahead of it stop each non-finite value, so no test can reach it.
  It stays as a backstop.

**Cost to change later**

- The ratios (`SECTION_RATIO`, `BED_RATIO`) and the stack rule are in one
  function each. A change moves the pinned boxes in
  `crates/sim/tests/machine_model.rs`.
- A later restriction on moving heads would change input validation and
  the moving-head pins in `crates/sim/tests/machine_model.rs`.
- #2591 (the optional `body` key) adds a `BodySource` variant. Each
  `match` on `BodySource` then needs the new arm.
