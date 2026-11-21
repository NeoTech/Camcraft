## #1880 - Import SVG drawings as 2D sketch profiles

**What was decided.**

- The SVG reader (`camcraft_geometry::svg`) gives the same `dxf::Document` as the DXF reader.
  The sketch chaining, nesting, `SketchData`, the `Sketch` entity and the project file are
  shared and do not change. The project file stores an SVG profile as it stores a DXF one.
- The reader has its own small XML tokenizer. The rules forbid a new dependency in this PR,
  and an SVG file needs only elements and attributes. A DOCTYPE entity is never expanded.
- A file states a physical size when its `width` or `height` has a unit (mm, cm, in, pt, pc,
  px). A unitless or missing size is no physical size, and the import asks for the unit of
  one drawing unit (the DXF `MissingUnits` law). CSS reads a unitless width as px, but that
  is a guess about the author's intent, and the issue says "never guesses silently".
- A caller's unit for a file that states its size is refused (`SvgError::SizeStated`). For
  DXF a caller's unit wins over the file's. For SVG two sizes would disagree silently.
- The Y flip puts the page's bottom edge on y = 0 (y' = page height - y), so the drawing
  sits in the first quadrant as it looks in the editor. With no height and no viewBox, the
  flip is about the user-space origin.
- An elliptical arc, circle or rounded-rect corner whose mapped shape is a circle stays an
  analytic `Arc`/`Circle`. A Bezier, and an ellipse that maps to a true ellipse, becomes lines
  within 0.005 mm (`CHORD_TOLERANCE_MM`), counted by Wang's second-difference bound. No
  flattened vertex lies closer than 1.5 x the default join tolerance to its neighbour, so
  chaining does not reject the short pieces.
- A zero radius in an arc command is refused as a "degenerate arc", as the issue asks. The
  SVG specification would draw a straight line there. An arc with equal end points draws
  nothing, as the specification says.
- A zero-size rect, circle or ellipse draws nothing and is counted as skipped ("rect
  (empty)"). A negative size is refused.
- A top-level `g` is a layer, named by `inkscape:label`, else `id`. Other elements are on
  layer "0". Chains join only on one layer, as for DXF.
- `use` of an element in the same file is drawn (16 levels at most, which also stops a loop).
  A `use` of another file is skipped and counted. Text, images, nested `svg`, `style`,
  `clip-path` and `mask` are counted by kind in `summary.skipped`. `display:none` content is
  counted as "hidden". Fill and stroke are not read.
- `ImportSvg` is its own command in `commands/import_svg.rs`. It wraps an `ImportDxf` value
  (a new `ImportDxf::placed` constructor), so apply and invert are the same code. It repeats
  the 10-line setup and stock check of `ImportDxf::build` instead of a change to `build`:
  open PR #2773 changes those lines of `build`, and a stack on that PR would tie SVG import
  to an unrelated Stock page feature.
- The MCP tool is `import_svg` in a new file `mcp/svg_tools.rs`, and the script verb is
  `cam.importSvg`. They take `import_dxf`'s options.

- The missing-size refusal is a `CommandError::Needs` (a precondition the user can fix), and
  every other SVG refusal is the one new variant `CommandError::Svg`. A typed
  `SvgUnitsMissing` variant was dropped to keep `commands/mod.rs` (already over 400 lines)
  from growing more.
- The "posted 2D Contour golden" of the issue's Done means waits for #1864 (2D Contour on
  sketches, PR #2716 open): on `main` a 2D Contour on a profile is accepted but does not
  generate. It is filed as #3042.

**Dropped.**

- A shared `import_sketch` tool with a `format` field: two tools follow the file type, as
  `import_model` and `import_dxf` do, and the DXF tool does not change.
- Biarc fitting of Beziers (fewer, analytic segments): more code and risk; lines within
  0.005 mm cut the same part.
- A crate such as `roxmltree` or `usvg` (both are in `Cargo.lock` through the UI): a new
  dependency of `camcraft-geometry` needs the maintainer.
- Nested `<svg>` viewports, CSS stylesheets, text to outlines and embedded images: the issue
  puts them out of scope; they are counted, not drawn.

**Cost to change later.** The reader is one module behind `svg::parse`. A move to a library
parser, or biarcs, changes only that module and its tests. Accepting a caller's unit for a
sized file is a one-line change in `svg::parse_with_limits`.
