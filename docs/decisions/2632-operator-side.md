## #2632 - the operator side of the work envelope

**What was decided**

- A machine file may state `general.operator_side`. The words are `"-X"`,
  `"+X"`, `"-Y"` and `"+Y"`: the world axis that points from the envelope
  to the operator. The field is optional. An old file and an old project
  load unchanged.
- The defaults for a file that states nothing:

  | Machine kind | Default | Reason |
  |---|---|---|
  | mill, mill-turn | `-Y` | The operator stands at the front. +Y runs away from the operator. |
  | plasma, waterjet (gantry tables, routers) | `-Y` | The same axis convention as a mill. |
  | wire EDM, sinker EDM | `-Y` | The tank door is at the front. |
  | lathe | `+Y` | The lathe view shows the turning plane (+X up, +Z right) from +Y. The viewer is the operator. |

- A wrong word refuses the file by name. It never reads as the default.
- The viewport draws one arrow with the envelope, in the envelope colour.
  The arrow starts at the middle of the lower edge of the operator-side
  face and points out of the envelope. The text "Operator side" is screen
  text just past the tip. Both are outside the envelope, so they do not
  cover the part or the toolpath.
- The switch "Show operator side" is in the View door, below "Show work
  envelope". It is view state like the envelope box: on by default, no
  command, not saved. It is disabled while the envelope is off.
- The arrow is built when its input changes (the placed envelope and the
  side). Each other frame reads the kept arrow. The scene pass places the
  envelope one time for the envelope lines and for the arrow.

**Why**

- The maintainer asked for the arrow and the text (issue #2632).
- Axis words have one meaning on each machine kind. The words "front",
  "back", "left" and "right" do not: the front of a lathe is not a side of
  its turning plane.
- `-Y` as the front is general machine-tool practice for a vertical mill.

**Considered and dropped**

- The words `front` / `back` / `left` / `right` in the machine file. They
  are not clear for a lathe.
- A default for each library machine file. No library file needs a side
  that differs from the default of its kind.
- A kept preference for the switch. The envelope box is not kept, and the
  issue asks for ephemeral view state.
- A row for the field in the machine file editor. It is not in the issue.
  A user sets the field in the file text.
- A second floor scan for the text. The text reads the arrow that the
  scene pass kept, so a frame does one placement.

**Cost to change later**

- A new default for a kind is one line in `OperatorSide::default_for` and
  one row here.
- New words for the field need a reader that accepts the four axis words
  too, because files can state them from this version on.
