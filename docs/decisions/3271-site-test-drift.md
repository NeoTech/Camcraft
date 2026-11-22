## #3271 - Site tests skip the generated catalog block before the operation icon

**Decided:** the site tests (`test_guide_icons.py`, `test_guide_refresh.py`)
read an operation section's lead after one leading, well-formed
`<!-- catalog:examples:start -->` .. `<!-- catalog:examples:end -->` block
(`section_lead`). The icon and the toolpath picture must still be the first
content of the section after that block. The guide pages and the generator
stay as they are.

**Why:** `cargo xtask catalog` (`xtask/src/catalog_links.rs`, `render`)
writes the block directly under each operation heading, and
`cargo xtask catalog --check` fails when the block is not there. To move
the icon above the block, the generator would have to find and step over
the icon line, and its check would have to accept prose between the heading
and the block. That makes the generator read guide prose, which its own
comment forbids ("never eat guide prose"). With the test change, each check
keeps one owner: xtask owns the block under the heading, and the site test
owns the icon as the first line of the section's own content. The helper
skips one block only, and only with its end marker, so a missing icon, a
block lower in the section or an unclosed block still fails the icon check.

**Dropped:** move the icon above the block in each section and teach the
generator to insert the block after the icon line. It changes a process
file (`xtask/`) and the 44 sections, and it couples the generator to the
icon shortcode.

**Cost to change later:** low. A different block position needs one change
in `render` and in `section_lead`, and a `cargo xtask catalog` run.
