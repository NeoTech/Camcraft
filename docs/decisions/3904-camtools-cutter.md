## #3904 - .camtools carries the drawn cutter (version 5)

**Decided.** A `.camtools` tool definition carries its drawn cutter
(`ToolCutter`) as `cutter`, the `[z, r, arc]` points that a `.camproj`
tool stores. A file with one or more cutters says version 5
(`CUTTER_VERSION`). A file without a cutter keeps version 4 or lower.
The reader judges a cutter with `cutter_profile::check`, the same check
as the project loader. A cutter that disagrees with its tool refuses the
whole file by tool name. `ImportTools` sets the cutter on Add, Update and
Replace, and undo restores the previous cutter in the same step. A library
tool without a cutter keeps the document's cutter on Update (the same law
as the body).

**Why.** Since #852 the stack laws read the neck of a drawn cutter. A
version-4 reader would skip the unknown key and judge a different tool, so
it must refuse the file. This follows the `HOLDER_REF_VERSION` precedent
(#2016). The form mill kind (#3369) keeps its outline in this field.

**Dropped.**
- Additive key with no version step (the `.camproj` choice of #93 slice
  2): an older build drops the neck silently.
- Refuse at export when the cutter disagrees with its tool: the commands
  keep the two in agreement, and the reader refuses such a file by name.
- Tie the cutter to the "holders" choice of the import: the cutter is the
  tool's own shape, not its holder.

**Project file.** No `.camproj` change: it carries `cutter` since #93
slice 2.

**Cost to change later.** Low: one key and one version constant.
