## #361 - Vendor STEP tool models: the lathe holder reaches the user

**Where the issue stood.** The 2026-11-08 triage split #361 into #368 (a
milling body judged by its revolved envelope), #369 (a lathe holder body
judged against the revolved stock), #370 (alignment from ISO 13399
coordinate systems) and #371 (the tool editor's Body section). All four
shipped by v0.18.0. What the issue asks for first - asymmetric lathe
tooling - was only half there: the law (#369) and its insert zone (#396)
existed, but no door led to them. The tool editor placed every body along
a spindle axis, so a turning tool's Apply body was refused with "lathe
holders are placed in the turning tool frame", which nothing in the GUI
could do; and the MCP server had no tool-body tool at all (the
camcraft-mcp skill listed it as a known gap).

**What was decided.**

1. **This slice makes the lathe holder reachable; it adds no new law.**
   The tool editor's Body section places a lathe-held tool's file in the
   turning tool frame (radial and spindle directions by preset or picked
   face normal, tip by numbers or picked corner, optional insert zone) and
   queues the existing `SetToolBody::lathe`. Five MCP tools cover both
   frames: `get_tool_body`, `set_tool_body`, `set_lathe_tool_body`,
   `set_lathe_insert_zone`, `clear_tool_body`. Dropped: a new judging law
   for non-revolved bodies in milling - a spinning tool's swept body is
   its revolved envelope, which #368 already judges; only a lathe holds a
   body still, and #369 judges that.

2. **Which frame a body takes follows the tool kind, as the fit laws
   already refuse by** (`tool_body_lathe_view::frame_choice`): a lathe
   steel always the turning frame, a spun tool always the spindle axis, a
   boring bar either - the editor shows a frame switch only for the bar.
   Dropped: a frame switch on every tool (it would offer placements the
   command then refuses).

3. **A reduced-fidelity STEP is refused as a tool body, at one door**
   (`tool_body_file`). Found while reading: every body-file path used
   `load_model`, which drops #864's `coarse_mm`. A vendor assembly is
   exactly the file that busts the facet budget, and a facet of a convex
   wall stands inside the true surface, so a coarsely meshed holder reads
   smaller than the holder is - a collision check that is too generous
   with nothing said. The collision laws carry no allowance for a body's
   own chord error, so the only verdict that cannot be too generous is a
   refusal by name (with what to do: simplify the model). Dropped for
   this slice: growing the body's envelope and clearance bands by the
   excess (a change to the judged numbers; filed as a follow-up). This
   stays clear of #904, which tightens the laws that read a
   reduced-fidelity *part*.

4. **MCP: two set tools, not one with a frame argument.** The two
   placements share no argument but the tool and the path; one schema
   with both sets would let an agent mix them. `set_tool_body` with
   neither `axis` nor `tip_offset` means "align by the file's coordinate
   systems" and refuses by name when the file names none - it never
   falls back to +Z at the origin, which the editor may do because a
   person then sees and corrects the placement. Lengths are mm, like the
   rest of the MCP surface.

5. **The lathe section is short rows, and has no Fit button.** The tool
   editor window does not scroll; the first layout (a caption over every
   control) pushed Apply body off a 720 px screen once the insert zone
   was on. Rows are label-then-control with the explanation in the hover
   text, the tip is one row of three narrow fields (`fields::num_narrow`,
   the same unit lens), and Fit stays the preview's double-click.

6. **The fixture is our own.** `testdata/models/lathe_holder_offset.step`
   is an extruded L (an offset holder: head and shank, 8 planar faces)
   written by a small script, modelled away from the turning frame so the
   placement is exercised. No vendor file was read or committed.

**What it would cost to change later.** The MCP tool names and argument
shapes are the public surface; the rest is pane state. Replacing the
refusal in (3) with a grown envelope touches `tool_body_file::accept`
(carry the chord instead of refusing) and the two envelope builders.

**Still owed on #361** (filed as sub-issues): #1037 the lathe holder
judged against workholding and fixtures, not only part and stock (P1 - a
body present reads as "judged"); #1038 the lathe body drawn in the
simulation and viewport; #1039 a lathe placement from a file's ISO 13399
coordinate systems; #1040 a tool assembled from several vendor bodies
(clamping unit plus head, the ISO 26622/26623 interfaces); #1041 a
reduced-fidelity body judged with a grown envelope instead of refused.
Imported bodies in the tool library file ride #889.
