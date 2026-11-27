# Rest Machining (two 3D Pockets, 12 mm then 6 mm)

A 60 x 50 x 4 mm clean-room aluminum plate with two parallel ribs on top.
Each rib is 13 mm wide, 30 mm long and 10 mm high (top at `Z14`). The ribs
stand at `X-17..-4` and `X4..17`, from `Y-15` to `Y15`. The channel between
them is 8 mm wide. The stock is a 60 x 50 x 18 mm box (top at `Z18`).

## What it shows

1. **3D Pocket (`3D Pocket D12`, `EM-1200-2FL`):** Roughs the stock down to
   the plate in 2 mm levels, with a 6 mm stepover and 0.2 mm stock to leave.
   The levels are `Z16`, `Z14.2` (0.2 mm above the rib tops), `Z14`, `Z12`,
   `Z10`, `Z8`, `Z6` and `Z4.2` (0.2 mm above the plate). The 12 mm tool
   plus 0.2 mm on each side needs 12.4 mm, so it cannot go into the 8 mm
   channel below the rib tops.
2. **3D Pocket (`3D Pocket D6 rest`, `EM-0600-4FL`):** The same pocket with a
   6 mm tool. CamCraft simulates the program ahead of it and cuts only where
   material still stands (re-machining). This operation has no rest switch:
   the stock of the earlier operation limits it.

## The rest region

The 12 mm pass leaves this material, seen from the top (`#` is a rib, `R`
is the rest region):

```text
  Y
  25 +-------------------------------------------+
     |                                           |
  15 |      +---------+ R R R +---------+        |
     |      |#########|R R R R|#########|        |
     |      |#########|R R R R|#########|        |
     |      |#########|R R R R|#########|        |
 -15 |      +---------+ R R R +---------+        |
     |                                           |
 -25 +-------------------------------------------+
    -30    -17       -4   0   4        17       30  X
```

- In the channel: `X-4..4`, `Y-15..15`, from `Z14.2` down to the plate.
- At the two mouths of the channel (`Y` near `-15` and `15`): the corners
  next to the rib ends that the 12 mm radius cannot reach.

The 6 mm operation cuts only there. All its feed moves have the tool centre
inside `X-2.2..2.2`, `Y-17.7..17.7`. Between the ribs the centre stays inside
`X-0.79..0.79`: 3 mm tool radius plus 0.2 mm stock to leave from each wall.
The test `catalog_3d_rest::the_small_tool_cuts_only_the_rest_region` pins
this on the committed program.

## What to look at

- NC line 1573 starts `3D Pocket D6 rest`. It has no level at `Z16`: the
  12 mm tool cleared it. Its first level is at `Z14`, in the channel only.
  Its last level, at `Z4.2`, starts at NC line 2580.
- The 6 mm program is 1,421 mm of cut, about 1.7 minutes. The 12 mm program
  is 7,581 mm, about 7.8 minutes. The rest pass does not cut the large open
  areas again.
- The two operations report `entry:` notes. The helix and the ramp do not
  fit at some pass starts, so they ramp along the pass.
- `cam.verify()` must report no finding before the post. The script stops
  if it does.

## Files

- `model/gen_rest_obj.py` — generator for the watertight test solid (a plate
  with two ribs, vertical walls).
- `model/rest.obj` — generated triangle mesh imported by `build.camscript`.
- `build.camscript` — headless script that builds `rest.camproj` and posts
  `rest.nc` through CamCraft's undoable commands.
- `rest.camproj` — committed project with generated toolpaths.
- `rest.nc` — committed Fanuc NC program, verified byte for byte in
  `crates/app/tests/example_catalog.rs`.

## Rebuild and verify

From the repository root:

```sh
cargo xtask catalog --case 3d/rest
cargo test -p camcraft-app --test example_catalog catalog_3d_rest
```
