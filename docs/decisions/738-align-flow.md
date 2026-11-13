## #738 - Setup page alignment: one flow, moves compose, picks you can see

**What was decided.**

1. **Moves compose on the current pose.** Laying the part on a face
   (`AlignPartToSetup::lay_on_plane`) and on three points
   (`lay_on_points`) now read the pick through the setup's held
   alignment and compose the lay onto it (`pick::lay_on_plane_in`,
   `pick::lay_on_points_in`). They used to return the part to its
   import pose first, so a face lay lost an earlier axis turn's heading.
   The axes (`turn_in_plane`) already composed and are unchanged. A
   re-applied pick is a zero step and gives the held alignment back bit
   for bit (`onto_held` snaps a step within 1e-9 mm / the identity-rotation
   tolerance to nothing), so "already holds the part that way" still
   greys Apply out. Undo is unchanged: each move is one
   `AlignPartToSetup`, which stores the previous alignment and restores it
   exactly. The pure `pick::lay_on_plane` / `lay_on_points` keep their
   absolute reading (the step from the import pose) for their tests and
   readouts.
2. **One flow on the page.** The "Part alignment" row is now a step that
   composes too: its header says "each move starts from where the part
   sits now" (with the whole flow in its tooltip), the combo is "Start
   from a preset" and each preset flips or turns the part from where it
   sits now, about the parts' centre as held. The typed row is "Turn by"
   (relative, about the WCS origin or the part centre). It queues the
   new `CommandKind::TurnPart` -> `AlignPartToSetup::turn_by` ->
   `setup_turn::turned_by`. "As modelled" is the one reset, named as
   such in its tooltip; it still goes through the absolute
   `CommandKind::AlignPart`. The face / points / axes Apply tooltips say
   "from where it sits now (one undo step)".
3. **Pick status in words.** The rows read "1 face selected", "2 points
   selected", "1 edge selected" (`ui::pick_words::selected`, plurals by
   count). Point lines say where each point is ("on 'part' (x, y, z)
   mm"), the axis rows lead with the count, the face row's side readout
   says "of the picked face", the viewport-pick refusals say "that pick
   is an edge", and an app refusal shown on the page goes through
   `pick_words::plain`, which strips " 0x<hex>" ids.
4. **Picks you can see.** `ui::align_pick_viz` paints the collecting
   draft's picks as screen-space egui shapes over the 1 px GPU outline:
   every facet of a picked face filled with the pick violet at alpha
   96/255, each picked point a 6 px filled disc (alpha 210) with a light
   rim, each picked edge a 4 px stroke (the hover outline is 1 px).

**Why.** The maintainer's report: the preset row silently overrode the
pickers, each pick move reset the part, the status showed "Face
0xccc7c...", and the picks were hard to see. Composition is what a
re-clamp means at the machine: you move the part from where it is.

**Considered and dropped.**
- Making `AlignPartToSetup::turn` / `preset` / `angles` relative. Their
  absolute "from as modelled" reading is pinned by #606's turn tests and
  used by many setup tests as "hold the part this way"; the page now uses
  the new `turn_by` instead, so nothing that reads them changes.
- Keeping the preset row absolute and only relabelling it "start from a
  preset" (a reset step). That still discards picker moves when applied
  after them - the exact complaint - so the row composes instead.
- Drawing thicker GPU lines or a filled GPU overlay. A wgpu line has no
  width and a new GPU pass is a bigger change; the screen-space egui
  path is the one the origin triads and boundary marks already use.
- Changing the app's refusal wording (`"{feature} of '{entity}'"`) to
  drop ids everywhere. Many app tests and MCP replies read it; the page
  strips ids at display instead.

**Cost to change later.** Low. A different fill alpha, disc size or edge
width is a constant in `align_pick_viz`. Going back to reset-first lays
is a one-line change in each `AlignPartToSetup` constructor (call the
absolute `pick::lay_on_*`), with `align_compose.rs` to update.
`commands/setup_align.rs` was split (its following-stock half moved to
`commands/align_stock.rs`) and `state.rs`'s alignment arms moved to
`state_align.rs` to keep both from growing.
