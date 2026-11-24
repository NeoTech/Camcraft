## #3526 - Face with 'Do not touch': a blocked level is refused, not noted

**What was decided.** When a "Do not touch" keep-out takes every row of a
facing level, the Face operation is refused by name. The refusal gives
Cut Z and the depth of the lowest level that the face can cut. It tells
the user to set Cut Z to that depth, to use a smaller clearance, or to
remove the item. Before, the operation generated and only a note said
"blocked N facing level(s)".

**Why.** The keep-out of a level covers the heights from that level up to
the clearance plane. Thus the blocked levels are always the lowest levels,
and a blocked level always leaves the whole face above Cut Z. The Prime
Directive asks for "incomplete" or a refusal. A toolpath has no
"incomplete" state: `Toolpath` carries notes only, and nothing in the app
or the post reads a note as a state. A refusal is visible, blocks the post,
and gives a direct workaround (Cut Z at the reached depth faces clean).

**Dropped.**
- An "incomplete" state on `Toolpath`. That is a change to the cache, the
  project file, the status line and the post gate in several crates. It is
  too large for this defect, and a Face that stops above Cut Z is not a
  useful partial result.
- A stronger note only. A note does not block the post and does not mark
  the operation, which is the defect.

**Cost to change later.** If a toolpath "incomplete" state ships, the
refusal in `face_keep::Tally::finish` can become that state. The tally
already holds the reached depth and Cut Z.
