# Wrapped A axis: a grooved bar

This original example shows continuous 4-axis work on a round bar.
The A axis turns during the cutting moves.
The case uses three rotary kinds: Rotary Offset, Rotary Parallel and Rotary Contour.

The model is a cylinder along X, 20 mm across and 20 mm long, from X−10 to X10.
A round-bottom ring groove wraps once around the bar at X0.
The groove profile is an arc of radius 5 mm, and the groove is 1.5 mm deep at its center.
The generator checks that the mesh is closed and that each facet faces out.

The built-in `Tormach 770MX + microARC 4` machine supplies the A axis along X.
The setup `Wrap` posts with the `Tormach PathPilot` post.
The script sets a cylinder stock along X and fits it to the model with a 1 mm margin.
The stock is 22 mm across and 22 mm long.
All three operations use the 6 mm ball end mill `BN-0600`, without a holder.

The rotary kinds read Clearance, Retract, Top and Bottom as radii about the A axis.
Rotary Contour is different: it reads Top and Bottom as stations along the axis.

| Order | Operation | Kind | What it does |
| --- | --- | --- | --- |
| 1 | Wrap rough | Rotary Offset | Removes the stock from radius 11 mm in radial levels of 1 mm and leaves 0.3 mm |
| 2 | Wrap finish | Rotary Parallel | Passes along X and steps around the bar; it also finishes the two end faces |
| 3 | Groove rings | Rotary Contour | Rings around the axis from X−5 to X5, over the groove |

Rotary Parallel finishes each free end face down to the A centreline.
The tool must reach the centreline from the stock radius.
The 15 mm flutes of `BN-0600` reach past the 11 mm stock radius, so the shank stays clear.
A thicker bar or shorter flutes make the verification report a shank collision on an end face.
The mesh is coarse (36 sectors, 1 mm axial steps in the groove) to keep the case regeneration short.

Inspect the three toolpaths in the project.
Inspect the A words in `wrapped.nc`.
Each operation turns A inside its cutting moves.
The post writes a move that turns A with X, Y and Z in inverse-time feed (`G93`), then goes back to `G94`.
A counts on past 360 degrees: Groove rings ends near A1620.
The model mesh is embedded in `wrapped.camproj`; the project has no external model link.

The rotary kinds plan contact for a shape of revolution.
They refuse a groove that wanders along X around the bar (a barrel cam).
Issue #2772 tracks that limit, so this case uses a ring groove.

The script requires a clean verification result before it saves the project or posts the program.
Stock simulation cannot judge a rotary program; the radial checks at generation and the gouge and collision checks still run.
No physical CNC machine was run.
The example has no chuck or fixture model.

From the repository root, regenerate the model and rebuild the case:

```sh
python3 examples/catalog/multiaxis/wrapped/model/gen_wrapped_obj.py
cargo xtask catalog --case multiaxis/wrapped
```

Check the catalog and its project tests:

```sh
cargo xtask catalog --check
cargo test -p camcraft-app --test example_catalog
```

The full rebuild of this case is in the heavy lane:

```sh
cargo test --release -p camcraft-app --test example_catalog -- catalog_multiaxis_wrapped --ignored
```
