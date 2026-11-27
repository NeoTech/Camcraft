# Sinker (Sinker Burn, Sinker Side Burn)

A 60 x 40 x 25 mm clean-room block with one blind rectangular cavity.
The cavity is 20 x 12 mm and 8 mm deep (floor `Z17`), centred at `(5, 0)`.
A sinker EDM burns it with one rectangular electrode.

The machine is the shipped template `Sinker EDM 3-axis (template)`.
The template states no conditions and no codes.
The build script adds them with `edit_machine`.
The two conditions and all of the code words are invented for this example.
They are not the codes of any control.
Before you use a real machine, take its conditions and codes from the control's manual.

| Condition | Role | Spark gap per side | Undersize per side |
|---|---|---|---|
| `R1` | rough | 0.10 mm | 0.25 mm |
| `F1` | finish | 0.03 mm | 0.03 mm |

## What it shows

1. **Electrode (`E-19.5x11.5`):** `electrode_from_faces` makes the electrode from the cavity's floor and four walls.
   The `R1` undersize makes it 0.25 mm smaller per side: 19.5 x 11.5 mm.
2. **Sinker Burn (`Rough burn`, condition `R1`, pass `rough`):** The electrode sinks at `(5, 0)` in one servo block, `M101 Z17.25 F50.`.
   A rough pass leaves 0.15 mm (0.25 undersize - 0.10 gap) on each wall and on the floor.
   Thus the electrode tip stops at the floor + 0.15 + 0.10 = `Z17.25`.
3. **Sinker Side Burn (`Side burn +X`, condition `F1`):** The electrode sinks again to the floor + the finish gap, `Z17.03`.
   This takes the floor to its drawn depth.
   Then the electrode moves 0.22 mm toward +X and back, at that depth: two `M102` blocks.
   The move puts the +X wall at its drawn size (0.25 undersize - 0.03 finish gap = 0.22).
   The other three walls keep the 0.15 mm rough allowance.

## What to look at

- In `sinker.nc`, find the two `(OPERATION: ...)` sections.
  Each section selects its condition (`ER1`, `EF1`) before its burn.
- Only the side burn section has `M102` blocks.
  The first goes to `X5.22`, and the second comes back to `X5.`.
  `Y` and `Z` do not change during the lateral move.
- A straight burn moves the electrode in Z only.
  A side burn moves one wall out by a stated amount.
  CamCraft refuses an amount larger than the undersize minus the condition's gap.
- The post refuses a Side Burn if the machine does not state the `side` code.

## Files

- `model/gen_sinker_obj.py` — watertight voxel-CSG generator for the block and its cavity.
- `model/sinker.obj` — generated quad mesh imported by `build.camscript`.
- `build.camscript` — headless script that builds `sinker.camproj` and posts `sinker.nc` through CamCraft's undoable commands.
- `case.toml` — operation kinds, machine and expected result.
- `sinker.camproj` — committed project with generated toolpaths.
- `sinker.nc` — committed program, checked byte for byte in `crates/app/tests/example_catalog.rs`.

## Rebuild and verify

From the repository root:

```sh
cargo xtask catalog --case edm/sinker
cargo test -p camcraft-app --test example_catalog
```
