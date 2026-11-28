## #3012 - Pattern folder: linear and grid copies at the post

**Decided.**

- A pattern folder is a row of the operation list, like a Pause entry. It
  is a `PatternDef` component with an `OperationMeta` position. It is not
  an `Operation`, so the tool, freshness, sim and rest walks do not see it.
- The members are ordinary operations of one setup. They stand directly
  after the folder, in folder order (the row law). The commands keep that
  order: create, set members and reorder gather the members after their
  folder. A member moves only inside its folder. A drop onto a member is
  refused by name. The post refuses a hand-edited file that breaks the
  row law, by name.
- Copy numbers are 1-based. Copy 1 is the members' own place. A grid
  counts along `dir1` first, then the next row along `dir2`.
- Directions are unit vectors in the XY plane of the setup's WCS. The
  command stores them normalised. Z is refused: a pattern copies parts
  side by side. Two parallel grid directions are refused.
- Limits: 1-1000 copies along one direction, at most 10000 copies, a
  spacing above 0 and at most 100 m. At least one copy stays on.
- Only milling operations are members. A probe is refused: its touch sets
  the work offset.
- By tool: each member is one section over every enabled copy. The new
  `cl::transform::copies` joins the shifted paths. The link rises straight
  up to the link plane, crosses, and goes straight down to the next copy's
  start. The link plane is the op's clearance height, or higher when the
  path's end or start is higher. One section means one `T.. M6`.
- By part: every member cuts one copy, then the next copy, one section
  each. The after-action and the setup-change stop ride the last section
  of the operation only.
- Every copy posts in the setup's WCS. The moves shift; the work offset
  does not. A turned or flipped WCS is refused by name at the post.
- Suppressed folder: no copies. The members post once as plain
  operations.
- The verification gate judges every shifted copy against the setup's
  machine travel (`MachineCheck::check_path`) and its fixtures
  (`fixture_struck::op_hits`). The stock sim walks each member once, at
  copy 1. A not-judged line names the folders whose other copies the sim
  did not judge.
- Per-operation files refuse a by-part folder by name: one file per
  operation cannot hold the interleaved order. By tool posts there.
- Deleting a member operation (`DeleteOperation`, `DeleteEntity`) takes it
  out of its folder; undo puts it back at the same place.
- `DeletePatternFolder` ungroups: the members keep their rows.
- Project file: a new entity kind `pattern`, no version rung. A document
  without folders writes the same bytes. A reader from before #3012 meets
  the unknown kind word and refuses the file by name. The display name
  (label) is saved with the folder.
- `get_job` lists `pattern_folders` only when the document has one.

**Why.** The row law gives one order for the list, the stock sim and the
post, so the sim does not judge a program in a different order from the
one that posts. The clearance-plane link is the general CAM practice to
move between parts, and it keeps the shipped posts from writing a tool
change between copies. The travel and fixture checks for each copy keep
the Prime Directive: a copy that runs into a vise or past a limit blocks
the post.

**Dropped.**

- Per-copy work offsets (G54, G55, ...): the issue asks for the parent
  setup's WCS for all copies.
- A Z direction or a stacked pattern: no safe link law without more design.
- Copies in a turned WCS: refused instead of a mapping that no test
  covers yet.
- A serpentine (zig-zag) copy order: row order is easier to read and to
  number. It can come later as an order option.
- Delete with members: delete the operations one by one; a batch option
  can come later.
- Stock simulation of every copy: it needs a stock and model per copy;
  the not-judged line says what is not judged.
- UI: the browser shows a folder as a plain row. A page for folders is
  separate work.

**Cost to change later.** The file format is additive (`pattern` rows).
A serpentine order or per-copy offsets are new `PatternOrder` or
`PatternDef` fields with a default that reads as today.
