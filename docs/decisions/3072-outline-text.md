## #3072 - Text in an outline font as closed shapes

**Decided.**

- We wrote our own TrueType outline reader, `camcraft_geometry::outline_text::ttf`. We wrote it from the public OpenType specification. It reads the table directory, `head`, `maxp`, `hhea`, `hmtx`, `OS/2` (cap height), `cmap` formats 4 and 12, `loca`, and `glyf` (simple and composite glyphs).
- A font with CFF (PostScript) outlines (`OTTO`, or a `CFF ` table and no `glyf`) is refused by name. A font collection (`ttcf`) is refused by name. Both refusals tell the user what to pick instead.
- Quadratic contour pieces are flattened to lines. The chord error is at most 0.005 mm, from the bound |a - 2c + b| / (4 n²). The bound reserves 0.001 mm for dropped near points. A piece can use up to 4096 lines. A curve that needs more lines is refused with a diagnostic.
- The text sketch uses a join tolerance of 0.0005 mm, not the 0.01 mm DXF default. A flattened point closer than 0.001 mm to the point before it is dropped. A small letter keeps its short segments, and the chainer never sees a segment shorter than its tolerance.
- `height` is the font's cap height, as in `insert_text`. The reader takes the `OS/2` `sCapHeight`, then the top of `H`, then 70 % of the em.
- The sketch must have only closed chains, each with a clear nesting. Outlines that cross or touch (a font with overlapping contours, or letters that run into each other) are refused by name. The refusal tells the user to add letter spacing or to pick a different font. We never send an unclear region to a pocket.
- The command is a new `InsertOutlineText`, with the new MCP tool `insert_outline_text`. We did not add a `font` field to `InsertText` or `insert_text`. The font is read once at construction, and the profile stores the shapes. The project file therefore needs no font and no format change.
- Composite glyphs apply their component matrix to offsets when `SCALED_COMPONENT_OFFSET` is set. A component with both scaled and unscaled offset flags is refused.
- Font reads use slices bounded by each declared table length. The `cmap` reader also checks each selected subtable length. Bytes after a short table cannot supply a missing field.
- A hostile font gives a clean error, not a hang or an abort. One glyph can make at most 4096 component visits (`MAX_PARTS`). The flattened points of the whole text are limited to 2,000,000 (`MAX_VERTICES`, the primitive limit of the sketch chainer). The budget is charged before each point is made. A glyph coordinate outside the 16-bit range of the format is refused.
- The reader places each glyph by its advance width only. It ignores kerning (`kern` and `GPOS`). It ignores the `USE_MY_METRICS` flag of a composite glyph: the advance is the composite's own `hmtx` entry. Text can therefore space differently from a word processor. Letter spacing adjusts the gap.
- A font whose glyphs are made of overlapping contours is refused ("cross or touch"). This is safe and stated, but it refuses many real fonts, mostly variable fonts and fonts exported without an overlap removal. A boolean union step (see "Dropped") would accept them.
- The tests use a font that is generated in memory (`outline_text::sample_font`, `#[doc(hidden)]`). It has our own box and curve glyphs. No font file is stored in the repository or licensed.
- Final review verification uses a clean merge of current upstream. The GPU bounds loop iterates over dimensions to clear the reported Clippy warning. All bounds checks stay in place.

**Why.** No new dependency: `ttf-parser` is in `Cargo.lock` only through eframe, and a direct dependency needs a maintainer decision. The reader separates character mapping from outline decoding. Each read checks the declared table or subtable boundary. A byte-flip and truncation sweep in the tests shows that it never panics. Most hobby sign fonts are `.ttf` files with TrueType outlines.

**Dropped.**

- Clamping the number of curve pieces to 64. At large heights, this silently exceeds the chord tolerance. The higher limit permits ordinary large letters and still bounds the work.
- Ignoring scaled component offsets. This moves the outline without a diagnostic. Applying the specified transform preserves the font placement.
- Limits on depth and points only. An empty component adds no point, so a composite tree of empty parts walked 64^8 times. Text at a large height could allocate gigabytes of points before the chainer limit applied.
- Checking table fields against the whole file. Trailing bytes can hide a corrupt table length. Bounded slices refuse these inputs.
- A direct `ttf-parser` dependency. It would also read CFF outlines, but it needs a new dependency.
- A CFF (Type 2 charstring) reader. This is a larger format. It can come later as its own issue if users ask for `.otf` CFF fonts.
- Silently joining overlapping contours with a boolean union. That changes the font's shapes without a word. A later issue can add an explicit union step.
- The use of glyph arcs or splines in the sketch. The sketch primitives are lines and arcs only. Flattening within 0.005 mm is well below a cutter's tolerance.
- Disabling the reported lint or changing the UI tests. These changes would hide the verification failures instead of clearing them.

**Cost to change later.** The reader is private to `outline_text`. A change to `ttf-parser` or a CFF reader changes that one module, and the command and tool stay the same. Toolpaths from profile chains (2D Pocket, Contour) are #1867's work. The text profile goes to those operations as closed chains, the same as a DXF profile.
