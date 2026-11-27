# #4081 slice 2a: the laser profile generator in the motion crate

Slice 2 of #4081 (the Laser Profile operation) is split in two. This
part is the motion crate only: the generator and its laws. Slice 2b adds
the app operation (type, stored params, project DTO, family, ribbon,
UI page, MCP reach). This part does not depend on slice 1 (PR #4270).

## Decisions

- **Chosen: the laser rides the waterjet generator under its own
  voice.** A laser profile cut and an abrasive waterjet cut follow the
  same rules: the kerf offset on the waste side, holes before the
  outline that frees them, the pierce in the waste with a lead, tabs as
  uncut gaps, the tight-hole rule, open runs on a stated side, G41/G42
  and arc linearization. `motion::laser::laser_profile` converts its
  `LaserParams` into `WaterjetParams` with `Voice::LASER` and calls
  `waterjet_profile`. Dropped: a copy of the planner, the tab code and
  the emitter (about 500 lines that two issues would then have to keep
  equal), and a new generic "sheet profile" module (a larger move of
  reviewed code for no change in behaviour).
- **Chosen: `WaterjetParams` carries a `voice`.** The waterjet module
  wrote "waterjet profile" and "waterjet:" in about 20 places. Each
  refusal and note now reads the voice of its params. `Voice` gets a
  `short` word for the notes (`plasma`, `wire`, `waterjet`, `laser`).
  Each waterjet sentence is unchanged: the golden CL
  `waterjet_tabbed_plate_with_holes.cl` and the waterjet suites pass
  without edits.
- **Chosen: the pierce note names the tool.** Under the waterjet voice
  the note is the old one (pierce time and abrasive delay). Under any
  other voice it names the pierce time and the voice's tool ("beam and
  dwell records are not implemented"). A laser has no abrasive, so
  `as_sheet` sets the abrasive delay to 0.
- **Chosen: the power is a percent of the source's maximum**, more than
  0 and at most 100, refused by name outside that range. The Grbl laser
  mode reads its S word against `$30` (the public Grbl wiki, "Laser
  Mode"), so a fraction of the maximum is what a post can scale. Dropped:
  watts (the post cannot know the source's rating without a new machine
  field). The generator only checks the power and states it in a note;
  no CL record carries it yet (slice 3, the laser post).
- **Kept: the clearance plane must stand above the cut plane.** A laser
  table without Z still needs a beam-off travel; the app (slice 2b)
  resolves the heights, and the post (slice 3) decides what a Z word
  means on a machine without Z.
