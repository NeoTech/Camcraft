## #4339 - Convert standalone tools to component stacks in one step

**What was decided.**

- The conversion works on the document's tools. A `.camtools` library
  converts after its import (import, convert, export). "My library" holds
  components, not tools, so it has no tools to convert.
- The proposed stack is: the tool's library holder as the built-in holder
  component `builtin:holder:<entry>`, then a collet that fits the holder
  and the shank (only a built-in collet: its document copy first, then
  the library's own; never a user adapter or extension), then
  the tool itself as a solid cutting item `user:tool:<tool name>` on a
  plain shank of its own shank diameter. A Weldon or shrink-fit holder
  takes the shank directly and gets no collet. The stick-out of the
  cutting item is the holder reference's projection.
- The old default stick-out of a library holder is the whole tool length
  (flute length plus shank length). Then no shank sits in the holder, and
  the stack resolver refuses that ("the stickout leaves no shank in the
  bore"). For such a tool the cutting item takes one shank diameter more
  of shank (in the bore), and the gauge length stays. When the projection
  is longer than the whole tool, the longer shank also fills the gap
  between the old shank end and the holder nose: steel outside the
  holder is added, never taken away. The new shank length is
  `projection - flute length + shank diameter`. The row is
  "different" and names the shank length change, so it converts only when
  the user ticks it. Kept the gauge length, not the shank length: the
  gauge length is what the posted tool lengths and the setup read; the
  hidden shank length only changes the drawn tool. Dropped: a shorter
  stick-out with the same record (it changes the gauge length) and "no
  match" for these tools (most tools of an old library have the default).
- The tool record does not name an insert or a cutter body. So an
  indexable tool also converts as one solid cutting item: its record stays
  exactly the same. A split into insert and body stays a hand step in the
  stack group.
- "Same" compares the derived tool record (all fields), the gauge length
  (holder gauge length plus projection, to 1e-6 mm) and the holder body
  (`tool_convert::outline_difference`). The holder body is the same only
  when the drawn profile and the bands are bit for bit the library
  holder's. Another outline that covers the old one everywhere is a
  difference ("holder outline: ..."), so the row is "different". An
  outline that is narrower anywhere, as the collision laws read the bands
  (`tool_body::checked_profile`), is "no match": a conversion never gives
  the collision checks a smaller holder. Example: a tool whose library
  holder reference has a longer gauge length than the entry's own. The
  library draws that holder longer, but the built-in holder component has
  the entry's gauge length, so the stack's flange sits lower and the
  outline is narrower there. Such a tool is "no match" (before this rule
  it was "different" by the gauge length only). Dropped: a tolerance on the
  outline (a "same" row with another outline), and a body copied from the
  library entry over the stack's own (a file read derives the body from
  the stack again, so the copy would not stay).
- A converted tool no longer counts as a user of its library holder. Its
  body comes from the stack (`library: None`), so `holder_users` and
  `tools_using_holder` do not list it, and the holder view's "used by"
  count goes down by one. The built-in library holders are read-only, so
  no holder edit is lost now. But a later change of a built-in holder
  (a new release) reaches an unconverted tool at the next file read,
  because its body is drawn from the library entry each time. A converted
  tool gets the change only when the user syncs the holder component from
  the library (the component table shows "update available"). The
  preview says this.
- A stack that a `ChangeTool` law refuses (a body in the flutes, an
  operation that the record strands) is "no match" with that reason.
- A lathe tool is "no match": its holder is not a built-in spindle holder
  component.
- Built-in copies join the document with their library link (as "Add from
  library" records it). The per-tool cutting item has no link.
- The MCP tool `convert_tools_to_components` has `dry_run` for the
  preview. Without `tools` it converts only the "same" rows, the same
  default as the window.
- The window's preview is computed when it opens, on Refresh and after a
  convert, never on a still frame. When the history moved after the
  preview, "Convert selected" is disabled until Refresh.

**Why.** The user asked for one step for a large library in the old
format, with no silent change of a tool. Each choice keeps the tool record,
the holder outline and the posted program the same, or shows the
difference first. The conversion changes the holder body and the shank
length that the collision checks read only when the row says so.

**Dropped.** A user holder component with the tool's own gauge length for
a "different" gauge row: it would add one user component for each tool and
hide a real difference. A conversion of tool library files without an
import: it needs a second, file-level command and no document undo.

**Cost to change later.** Low. The matcher is one function
(`tool_convert::row`). An insert-and-body split can be added there when the
tool record names them.
