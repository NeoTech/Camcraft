## #4451 - The tool library preview draws a thread mill's teeth

**Decided.** The tool library window's preview (the 3D model and the side
view) draws a thread mill's cutting length as its teeth, in a new
display-only module (`crates/ui/src/egui_app/tool_library_dialog/thread_form.rs`):

- the core at the tooth roots: the crest radius minus the external ISO
  root depth (`EXTERNAL_DEPTH_PER_PITCH` x pitch), held to at most 80 % of
  the crest radius;
- one V tooth of the stated form angle (`tip_angle`) for each row: a
  multi-form mill has `thread_mill::rows` rows at its pitch, a
  single-form mill has one tooth over its stated tooth height
  (`flute_len`), sized at its coarsest pitch;
- the steel above the cutting length stays as it is: the shank of
  `Tool::profile`, or the neck of a landed drawn cutter.

**Why.** A thread mill states no neck field: a neck rides the drawn cutter
outline (`thread_mill.rs`). So the stated geometry is the crest diameter,
the pitch, the form angle, the cutting length and the shank, and the drawn
outline when there is one. `Tool::profile` stays the crest cylinder,
because the reach laws read that envelope (#894). The teeth stay inside
the crest cylinder, so the plain mesh's box still fits the view.

**Dropped.** A change to `Tool::profile` in `camcraft-tools`: it is the
envelope that collision and verify read, and the issue is display only.
A new neck field on the tool: that is a model change, not a preview fix.
The flutes of the other kinds, a finer mesh and separate colours are
#4452.

**Cost to change later.** Low. The module is private to the window. The
Properties side preview (`tool_preview::paint` with no steel override)
still draws the crest cylinder; #4452 can call `thread_form::toothed`
there too.
