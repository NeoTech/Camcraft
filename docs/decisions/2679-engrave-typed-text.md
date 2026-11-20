## #2679 - Typed text becomes a profile of engraving strokes (first slice)

**What was decided**

1. The first ride mode is single-line engraving. A typed string becomes
   tool-centreline strokes, not glyph outlines.
2. The strokes come from our own stroke font, drawn from scratch for this
   project (`crates/geometry/src/stroke_text/glyphs.rs`). The font has
   capital letters, digits, the space and `- _ . : / # + = ( )`. Each glyph
   is straight strokes on a 4 x 6 grid with cut corners. The zero has a
   slash.
3. `InsertText` makes the same `Sketch` entity that `ImportDxf` makes: line
   primitives in millimetres on the layer `text`, placed on the top face of
   the setup's stock at X, Y and an angle. `MoveSketch`, `DeleteEntity`, the
   project file and `get_job` work on it with no change.
4. A character with no glyph is refused by name. A lower-case letter is not
   written as a capital.
5. This PR is a first slice (`Refs #2679`). It ships the font, the command
   and the MCP tool `insert_text`. It does not ship the toolpath.

**Why**

- The issue says to use a font bundled in `assets/ui/fonts/`. That directory
  does not exist on `main`, and the repository holds no font file. The UI
  text comes from the default fonts inside the `egui` crates. To read their
  outlines, `camcraft-geometry` or `camcraft-app` needs a font parser and the
  font data as new dependencies. A new dependency waits for the maintainer
  (`needs-maintainer`), and the worker brief forbids a `Cargo.toml` change.
- The Engrave and Trace generators cut along the tool centreline. A
  centreline along a glyph outline gives hollow letters. A single-line stroke
  gives the solid mark that a part number needs, with a v-bit or a small end
  mill, in one pass for each stroke.
- A skeleton computed from outlines is a new algorithm (medial axis). A
  drawn stroke font gives the same ride mode with a table and no algorithm.
- Sketch chaining refuses a chain that branches. The glyphs are drawn so
  that a stroke end meets a different stroke only inside a segment. For
  example the arms of `Y` meet on a short flat, and the stem ends in that
  flat. `stroke_text::sketch` fails if a chain is rejected, and
  `each_glyph_chains_with_no_rejection_at_the_extreme_heights` pins each
  glyph. A text sketch can never hold a rejected chain.
- A silent change of the typed text is a wrong mark on a part. A refusal by
  name is safe.
- On `main`, generation from the chains of a profile is refused by name
  (`sketch_model::generation_problem`). #1869 adds Engrave and Trace on
  sketch chains, and it depends on #1864 (PR #2716). The toolpath, the
  verify and the golden CL of this issue need #1869. This slice does not
  touch those functions, so it does not conflict with that work.

**Considered and dropped**

- Glyph outlines from the `egui` default fonts: needs two new dependencies
  and a decision of the maintainer. It can come later as a second font
  source; `TextSpec` is the place for a font choice.
- A public-domain stroke font (Hershey): the clean-room rule permits it, but
  a data file needs a licence entry and a parser. Our own table is 47 short
  rows.
- Lower-case glyphs: 26 more drawings. Part numbers are capitals and digits.
  A later PR can add them to the table.
- The text string as an editable parameter of the Engrave operation: this
  needs a new schema field and a project version step. PR #2716 also steps
  the project version. In this slice the text is fixed when it is placed;
  to change it, delete the profile and place it again.
- A script verb `cam.insertText`: the MCP tool calls the command directly.
  The verb can follow with the UI.
- Placement on a picked model face: a `Sketch` lies on the top face of the
  setup's stock only (`sketch::face_frame`). Other faces need a change of
  that law.
- New wording for the refusals that say "DXF profile": a text profile reads
  as a "DXF profile" in those messages. The wording is used in many tests
  and in PR #2716, so it stays for now.

**Cost to change later**

- A new glyph or a changed glyph is one table row; the golden string in
  `crates/geometry/tests/stroke_text.rs` moves if its glyphs move.
- A saved project stores the strokes, not the text. A font change does not
  change a saved project.
- An outline font can be added behind `stroke_text` without a change to
  `InsertText` or to the project file.
