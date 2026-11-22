# Five-axis wall and surface paths

**WORK IN PROGRESS:** This held branch is not a finished catalog case.
The project generates, and `cam.verify()` returns a clean result.
Posting refuses the first tilted feed because the initial A angle is unknown.
The opening rapid moves have no tool orientation.
Issue [#3300](https://github.com/NeoTech/Camcraft/issues/3300) tracks this blocker.
There is no committed `five-axis.nc` program yet.
No safety check is bypassed.

This case is intended to show Swarf, 5-Axis Contour, and 5-Axis Flow on one original faceted cylinder.
The cylinder has a 12 mm radius and extends from X = -24 mm to X = 24 mm.
Its generator divides the circumference into 16 sections and the length into six sections.
The closed mesh includes both end caps.
The OBJ generator writes each vertex once and indexes the shared vertex list to avoid the import defect tracked in [#3298](https://github.com/NeoTech/Camcraft/issues/3298).

All three operations cut the upper tilted facets between Z = 11 mm and Z = 5 mm.
They use the built-in **Tilt-head 5-axis** machine and a 15-degree lead angle.
The clearance plane is Z = 25 mm.
The paths show three strategies over the same model; they do not describe a production machining sequence.

| Operation | What to inspect |
| --- | --- |
| Swarf wall | The side passes along the tilted faceted wall. |
| Contour surface | The contour passes over the upper surface. |
| Flow surface | The flow passes follow the surface across the facets. |

Inspect the tool orientation in each generated operation.
After the posting defect is fixed, inspect the A/B words in each section of `five-axis.nc`.
The saved project embeds the mesh, so it opens without a source-file link.

The tool starts from the BN-0600 catalog ball-nose cutter.
The case sets its flute length to 60 mm and its shank diameter to 6 mm.
It selects no holder.
Contour and Flow disable their optional shank-avoidance search.
These choices keep the example small and give the modeled cutter room above the wall.
They do not define a physical tool assembly.

`build.camscript` generates all three operations and requires `cam.verify()` to return a clean result before saving and posting.
The catalog tests require regeneration, a script rebuild, and an NC byte comparison.
Those tests and the catalog gate cannot pass while the NC program is missing.
This case does not demonstrate posed stock-removal simulation or a run on a physical machine.

To regenerate the mesh and reproduce the posting refusal, run these commands from the repository root:

```sh
python3 examples/catalog/multiaxis/five-axis/model/gen_five_axis_obj.py
cargo xtask catalog --case multiaxis/five-axis
```

After the defect is fixed and the NC program is generated, run these checks:

```sh
cargo xtask catalog --check --case multiaxis/five-axis
cargo test -p camcraft-app --test example_catalog catalog_multiaxis_five_axis
```

The geometry and the generator are original analytic constructions.
They use no HSMWorks model, output, or asset.
