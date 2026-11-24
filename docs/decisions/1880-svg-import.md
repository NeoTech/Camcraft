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
- A `visibility` presentation attribute or inline declaration is reported by name, and its
  subtree is skipped. This prevents hidden contours from becoming cuts without evaluating
  inheritance and child overrides. Full visibility evaluation was dropped from this fix;
  adding it later needs explicit inherited state and regression tests for child overrides.
- A `transform-origin` presentation attribute or inline declaration is refused by name
  before geometry is drawn. Ignoring the origin can move the cut. Applying it was dropped
  from this fix because it needs CSS length, reference-box and inheritance rules.
- An inline `style` is read against an allow-list (`SAFE_PROPERTIES` in `svg/facts.rs`):
  paint, marker, text and rendering-hint properties, and `display`, `visibility`,
  `clip-path` and `mask`, which the reader handles by name. `vector-effect` is permitted
  only with `none` or `non-scaling-stroke`. Each other property is refused by name. Thus
  a vendor-prefixed transform, an SVG 2 geometry property (`width`, `r`, `d`) and a
  property that the reader does not know stop the import. They do not import a shape at
  the wrong size. The check is on the root and on `g`, `a`, `use` and the shape elements.
  An element that is skipped whole and counted (for example `text`) is not checked,
  because nothing of it is imported. A declaration whose name is empty or has a space in
  it is not CSS, and it is dropped as a browser drops it. Dropped: a list of refused
  properties (each new property was a new silent case), and `filter` on the allow-list
  (a filter can move what the user sees, so it is refused).
- The style and the transform of an element are read one time and kept by node index, and
  a map gives the element of an `id`. A `use` can draw an element again, but it does not
  parse them again. The shape data (`d`, `points`, the lengths) is read at each visit, so
  each visit charges those attribute bytes against a work budget of 4 x the byte budget.
  A file that goes above it is refused by name. One pass over a file is never above 1 x.
  Dropped: a cache of the parsed path commands (memory for each reused path).
- A `<style>` sheet is checked with the allow-list of the inline styles, with no selector
  matching (`svg/sheet.rs`). The reader does not know which element a rule matches. Thus
  `display` and `visibility` are refused by name with any value, and each property that is
  not on the list is refused. A sheet of paint rules (`.cls-1{fill:none;stroke:#000}`)
  imports and is counted as `style`. `clip-path`, `mask` and a marker in a sheet are
  counted, as the inline ones are. Refused by name: an at-rule other than `@charset`,
  `@namespace` and `@font-face` (`@import` and `@media` included), a rule inside a rule,
  and text that is not a whole rule. The last one is necessary: this scan is not a full
  CSS tokenizer, and a part that it cannot read must stop the import. An
  `<?xml-stylesheet?>` instruction and a `<link rel="stylesheet">` are refused by name.
  Dropped: selector matching (a CSS engine in the geometry crate), and a refusal of each
  `<style>` element (the usual Illustrator export has a paint sheet).
- Each element name has one of four results: drawn, silent, refused by name, or counted in
  the summary by its name. The animation elements (`animate`, `set`, `animateTransform`,
  `animateMotion`, `animateColor`, `discard`) are refused at each place in the tree, hidden
  and unreferenced content included, because they can name their target with `href`. A
  child of a shape or of a `use` that is not `title`, `desc`, `metadata` or `style` is
  counted as `<child> in <parent>`. An unknown element at a walked place is counted by its
  name. The scan of the children is charged against the work budget. Dropped: a refusal of
  each unknown element (an editor's private elements would stop the import).
- A reference to an entity that is not predefined (`&name;`) is refused by name in content
  and in an attribute value. A browser expands a DOCTYPE entity into elements or into a
  value (`display="&d;"`), and this reader does not expand it. The exception is a namespace
  declaration (`xmlns`, `xmlns:*`): its value is not read, and old Illustrator files write
  it with an entity. Dropped: entity expansion (the size of the result has no bound).
- A reference is a run after an `&` that ends at a `;` (`svg/reference.rs`). It is decoded
  only if it is one of the five predefined entities or a character reference to a character
  that XML permits, with any count of leading zeros. Each other reference is refused by
  name: a name with a character outside ASCII, a run that is not an XML name (`&a=b;`,
  `&#X41;`), a code point that is not a character (`&#0;`). A browser puts a value there,
  or does not draw the file. Dropped: a list of the characters of an XML name (a list that
  is too short lets a reference through as text).
- The inline style and the sheet use one CSS scanner (`svg/css.rs`). It reads an unquoted
  `url(` token as CSS Syntax 3 does: the token ends at the first `)` that is not escaped,
  and a quote, a `(`, a `;`, a brace or a `/*` in it starts nothing. A bad url token (a
  quote, a `(` or a gap in the address) is refused by name in a sheet and in the inline
  style of an imported element, because a style is not read in part. If the same style has
  `display:none`, the shape is hidden as in a browser, and nothing of it is imported.
  Dropped: only the browser rule (drop the declaration, read the next one) with no
  refusal; a later difference from a browser would then import a shape without a message.
- A `preserveAspectRatio` value that is not in the SVG grammar is refused. Before, an
  unknown alignment fell back to xMin/yMin, which moves the drawing.
- A marker property with a value other than `none` is counted as `marker`: the marker shape
  is not cut geometry.
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
