# #4081 slice 1: the laser machine kind and the laser head

The laser profile cut of #4081 comes in three slices, each a draft PR.
This slice adds the two kinds that the operation needs. It adds no
operation and no post.

## Decisions

- **Chosen: a `laser` machine kind, declared only.** A laser table has the
  X/Y/Z inventory of a mill or a waterjet, so only the stated word makes
  one (as for plasma and waterjet). It is an X/Y gantry with an optional Z
  (the focus height), no spindle, no live tool, no coolant list and no
  rotary axis. The words `laser`, `laser cutter` and `laser cutting` read
  as the kind.
- **Chosen: one door law for the two X/Y gantries.** The waterjet and the
  laser doors are the same rules with a different kind word, so
  `no_spindle::xy_gantry_refusals` and `Machine::xy_gantry_sane` hold them
  once. The waterjet sentences do not change.
- **Chosen: a laser head tool kind with the beam kerf as its diameter**,
  rpm 0 and no corner radius, as for the torch and the waterjet nozzle.
- **Chosen: no cutting data rows on a laser head in this slice.** A laser
  recipe (power, pierce time, assist gas for each material and thickness)
  would add a field to `CuttingRow`, its project DTO and its UI table.
  Slice 2 states the power and the feed on the Laser Profile operation
  instead. A laser head with rows is refused by name: "a laser head has no
  cutting data rows: the laser operation states the power and the feed".
  Dropped: plain rows (they need an rpm above 0, which a beam has not) and
  a new recipe type now (its fields need the operation to settle first).
- **Chosen: until slice 2, a laser machine refuses each operation by
  name.** No family admits a laser. The create-time gate says "the setup's
  machine is a laser cutter, which cuts only laser operations", and the
  program check says "<op>: a laser machine admits only laser operations".
  No built-in post serves a laser, so nothing posts. The tool law test
  states that every operation refuses a laser head; slice 2 changes that
  to "only the Laser Profile accepts it", as the waterjet case reads.
- **Chosen: an original glyph** (`assets/ui/svg/tool-laser.svg`): a solid
  head, a thin beam and a burst where the beam meets the sheet, so it
  does not read as the waterjet nozzle. `cargo xtask icons` rasterised it;
  the other PNGs did not change.
- **The UI stays quiet.** The New Tool combo offers a tool kind only when
  an operation on a selectable machine accepts it, so the laser head is
  not offered before slice 2. The machine editor's Kind combo offers
  `laser`, because a user can write the machine file first.

## Next slices

2. The Laser Profile operation on `motion::sheet` (kerf offset, side and
   direction for each chain, leads and pierce), on DXF sketch chains.
3. A laser post (Grbl 1.1 laser mode from the public Grbl wiki), the MCP
   reach through `add_operation` / `edit_operation`, and the docs.
