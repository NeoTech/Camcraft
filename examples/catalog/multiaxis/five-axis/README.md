# Five-axis wall and surface paths

This case shows Swarf, 5-Axis Contour, and 5-Axis Flow on one original faceted cylinder.
The three operations cut the same tilted upper facets, so you can compare how each strategy sets the tool axis.

The cylinder has a 12 mm radius and extends from X = -24 mm to X = 24 mm.
Its generator divides the circumference into 16 sections and the length into six sections.
The closed mesh includes both end caps.
The OBJ generator writes each vertex once and indexes the shared vertex list.
This avoids the import defect tracked in [#3298](https://github.com/NeoTech/Camcraft/issues/3298).

All three operations cut the tilted facets between Z = 11 mm and Z = 5 mm.
They use the built-in **Tilt-head 5-axis** machine and a 15-degree lead angle.
The clearance plane is Z = 25 mm.
The paths show three strategies over the same model.
They do not describe a production machining sequence.

| Operation | What to inspect |
| --- | --- |
| Swarf wall | The passes run along the cylinder's straight rulings (parallel to X). The tool lies along the ruling, so its flank touches the wall along a line. Look for `B90.` / `B-90.` on these passes, with `A0.` or the 15-degree lead (`A15.` / `A-15.`). |
| Contour surface | The waterline passes go around the part at constant Z. The tool axis leans from the contact normal by the lead angle, so `A` and `B` change around each ring. |
| Flow surface | The passes follow the surface direction across the facets. The `A` and `B` words change smoothly along each pass. |

In `five-axis.nc`, each operation section starts with a rapid move that states `A` and `B` before the first tilted feed.
The post needs that opening pose because the machine states rotary rate limits.
`G43.4` turns on tool centre point control, so the X, Y and Z words are tool-tip positions.

The saved project embeds the mesh, so it opens without a source-file link.

The tool starts from the BN-0600 catalog ball-nose cutter.
The case sets its flute length to 60 mm and its shank diameter to 6 mm.
It selects no holder.
Contour and Flow disable their optional shank-avoidance search.
These choices keep the example small and give the modeled cutter room above the wall.
They do not define a physical tool assembly.

`build.camscript` generates all three operations and requires `cam.verify()` to return a clean result before it saves and posts.
The catalog tests require regeneration, a script rebuild, and an NC byte comparison.
This case does not demonstrate posed stock-removal simulation or a run on a physical machine.

To regenerate the mesh and rebuild the case, run these commands from the repository root:

```sh
python3 examples/catalog/multiaxis/five-axis/model/gen_five_axis_obj.py
cargo xtask catalog --case multiaxis/five-axis
```

To check the case, run these commands:

```sh
cargo xtask catalog --check --case multiaxis/five-axis
cargo test -p camcraft-app --test example_catalog catalog_multiaxis_five_axis
```

The geometry and the generator are original analytic constructions.
They use no HSMWorks model, output, or asset.
