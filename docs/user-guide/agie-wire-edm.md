# Post an Agie Classic Cut 2S job

CamCraft writes one native ISO geometry file per work and one SBL and SBR for the complete job. The setup uses cold die steel (material 0001), Berco Cut 0.25 mm wire and the resident QN2 Fine Ra 0.30 µm technology. Use this recipe only when those settings match the work and the installed control. The controller calculates the wire offsets and trim passes from that technology. CamCraft displays a wire-center preview with a representative finishing gap of 0.005 mm.

## Post from the Wire EDM workspace

1. Mount **AGIECUT Classic 2S (template)** as the setup machine. In the machine editor, open **EDM** and add a condition with ID **AGIE_QN2**, role **rough**, wire diameter **0.25 mm** and preview gap **0.005 mm**. Apply this change to the mounted machine. Choose **Agie native wire EDM** in the post picker.
2. Define axis-aligned rectangular stock at most 350 × 250 mm in plan. Choose a stock thickness of 15, 20, 40 or 80 mm. This span describes the stock. Verify the installed work datum and the available axis travel on Agie Control. CamCraft refuses profiles with a stated X or Y travel limit because its preview cannot check the full resident path against that limit. Put the first, unrotated work origin on the stock top. Its XY position determines the job coordinates.
3. For a single work, use one modeled workpiece and one **Wire Contour**. For two punches on the same stock, use the grouped workflow below. For a die opening, follow the selection and piercing instructions below. For a punch, select the complete punch model. Use a punch with straight sides, vertical walls and flat ends at the stock planes. Angled faces are supported. Leave at least 0.4 mm of stock around the nominal polygon. Acute corners can need more stock: the post also checks the intersection of the two offset face lines and the swept wire at that corner. CamCraft checks each corner with the rough offset for the selected stock thickness. The stock margin includes the 0.125 mm wire radius. Some reentrant contours are supported. A tight inside corner fails if the rough offset leaves more than 0.002 mm at the nominal vertex. Short segments and closely spaced faces fail if adjacent rough paths could meet. The punch must have a straight outer face that allows the lead to stay entirely on the waste side. The punch must have one boundary and no cavities, steps or other bodies. Select a 0.25 mm wire and condition **AGIE_QN2**. Set the work planes to the stock top and bottom, and specify how the punch is captured. The capture setting confirms support for the punch when it is released. Keep the wire vertical. Taper, four-axis, burnout, explicit skims, tabs and fixture models are unsupported.
4. Place the centre of a predrilled 2 mm threading hole in the waste. The entire hole must be inside the stock. Allow a perpendicular approach to a straight face with at least 3 mm of lead and at least 3 mm of straight face on each side of the entry. Generate the operation.
5. Open **Post**, choose **One program** and keep the general post settings at their defaults. Under **Agie technology**, set **Separation cut length (mm)** to **1.0**. For a closed contour, zero leaves the separation unspecified and blocks the post. This length applies to the separation segment that Agie Control generates. Check its position, trims and release sequence in the control’s job preview. Enter the parent folder and a fresh **Job folder name** such as `PUNCH40`. Use up to eight uppercase letters or digits, beginning with a letter. The post publishes the entire directory after validation. To retain an earlier job, give the new one a different name.

### Group two captured punches

Use a single setup with 20 mm stock and two complete modeled punches. Create a **Wire Contour** for each solid, select **Punch** and the same wire tool, and enter its own threading hole. Confirm capture separately for each punch. Keep a clear band along X or Y between the complete cutting areas. The check includes each solid, threading hole, lead, and the furthest native rough corner. A shared corridor or another model prevents export.

Generate both operations. Select a new job name of up to seven letters and digits. The post appends `1` and `2` to the name for the two ISO resources. The work names `CUT1` and `CUT2` follow program order. Both share the stock origin. In the control preview, check the profile and separation sequence for both works and check each threading location. The control can complete the profile passes on both punches before it starts their separation cuts.

Run the synthetic example from the project root with a fresh `PAIR20` destination.

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/punch-pair.camscript
```

The left rectangular punch starts at X−30 Y0. The right five-sided punch starts at X1 Y0. Copy all four files in the directory together.

### Select a die opening

Use a 20 mm thick retained plate with one through-opening, planar caps and vertical walls. In the Wire Contour Geometry tab, choose **Edges** and Shift-click the connected edges of its upper opening rim. Set the wire side to **Die**. Set **Freed piece captured** when the setup can hold the released slug. Place the complete 2 mm threading hole inside the opening, with at least 3 mm between its centre and the entry face. Leave 3 mm of straight face on each side of the perpendicular entry. The supported opening is a convex polygon with corners that the rough wire can reach within 0.002 mm.

Run the original example from the project root with a fresh output directory:

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/die-plate.camscript
```

The example selects the complete upper opening rim on a 60 × 60 mm plate. The native `DIE20` job starts at X−5.5 Y0 and enters the straight wall at X−12.5 Y0. Inspect the **inside** compensation and the 1 mm rough separation in Agie Control. The controller completes the separation before it makes the three full trim passes. Arrange support for the slug and a clear trim path after release.

### QN2 offsets by stock thickness

Agie Control calculated these wire-centre offsets on the straight outside faces of a synthetic rectangular punch. All dimensions are in millimetres. The native job sends nominal geometry. The resident technology applies each offset.

| Stock thickness | Rough | Trim 1 | Trim 2 | Trim 3 |
| ---: | ---: | ---: | ---: | ---: |
| 15 | 0.245 | 0.155 | 0.140 | 0.134 |
| 20 | 0.245 | 0.155 | 0.140 | 0.134 |
| 40 | 0.260 | 0.170 | 0.140 | 0.130 |
| 80 | 0.266 | 0.176 | 0.143 | 0.133 |

The clearance check rounds the rough offset plus the wire radius up to the next tenth of a millimetre. For these four heights this gives 0.4 mm on a straight face. At a pointed corner the post checks the miter for that height separately. Other heights require their own control measurements.

## Try the synthetic punch

From the project root, run the supplied script with a local CamCraft desktop CLI build.

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/punch.camscript
```

The script imports an original 40 × 20 × 20 mm rectangular punch, places it in 80 × 60 × 20 mm stock and sets the threading point at X−30 Y0. Its post call selects the native separation with `options: {separationLengthMm: 1.0}`. Stock top is Z0 in the Agie job. The `PUNCH40` directory contains the nominal contour in `PUNCH40.ISO`, its import reference in `PUNCH40.SBR` and the piece, start point and technology in `PUNCH40.SBL`. Existing destinations are refused. Edit the script’s output name to make another job.

For an oblique entry, run `cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/oblique-punch.camscript` with a fresh `POLY20` destination. This pentagonal punch has a 3:4 sloping entry face. Its threading hole at X−20.5 Y6 approaches X−12.5 Y0 perpendicular to the middle of that face.

Run `cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/slope-punch.camscript` for a 7:10 entry face. The `SLOPE20` job keeps the drilled hole at X−20.5 Y6. Its perpendicular entry falls between five-decimal ISO coordinates. CamCraft can move the entry up to 0.00001 mm toward the waste while preserving the lead, separation room and rough-pass clearance.

Run `cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/notched-punch.camscript` for a 20 mm wide, 1 mm deep reentrant notch. The `NOTCH20` job enters at X−20 Y0 from the hole at X−30 Y0. The nominal notch tip is at X0 Y9.

## Post a straight open cutoff

Use 20 mm cold die steel, Berco Cut 0.25 mm and AGIE_QN2. Model the retained solid as one full-height prism. Select the straight X- or Y-parallel wall edge that reaches opposite faces of the rectangular stock. In **Geometry**, choose **Left** or **Right** so the kerf lies on the waste side as the selected chain is traversed. Clear **Start hole stated**. The native post extends each end of the nominal cut 4 mm into air. It uses a separate perpendicular approach at each end.

Clamp both resulting pieces independently. Keep the clamps clear of the entire wire path, the 4 mm tails and both 3 mm approaches. The rough pass severs the offcut before three further trims. A catcher alone allows the offcut to move. Center the stock on the midpoints of the installed X and Y strokes. In **Passes**, confirm **Both pieces clamped through all passes** and **Stock centered in installed XY travel**. The post checks the air tails, approach points, work origin and wire envelope against the centered 350 × 250 mm travel. Use a run-out clearance of 3.6 mm or less.

Choose **Agie native wire EDM**, set the separation cut length to zero and choose a new output directory. The controller alternates travel across the part on successive passes. Inspect the formal check and the plotted four cutting passes before machining.

Run either original example from the repository root:

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/cutoff-lower.camscript
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/cutoff-upper.camscript
```

The examples select opposite sides of synthetic 80 × 60 × 20 mm stock. Each writes one native job, named `LOWER20` or `UPPER20`.

## Open the job in Agie Control

Keep all three files together. In the control’s file manager, open **Imp/exp** and import the ISO object into **ISO Sys**. Then import the SBL into **SBL Sys**, keeping the matching SBR beside the SBL. In the workpiece list choose **New workpiece → Script Files**, select the imported SBL and confirm with **OK**.

Open the new piece and check the configuration data against the intended setup. Run **FORMALCHECK** and inspect both errors and warnings. Save and close the piece, then run **GRAFICHECK** on that piece. Check the installed XY datum, available axis travel, entry from the drilled hole and compensation on the waste side of each face. In the tested 20 mm punch job, the main rough pass and three trims left a 1 mm section. The control then returned to cut and trim that section. Check the sequence and arrange support for release. CamCraft does not command a manual pause before the section cut.

### Punch from a placed DXF racetrack

Create box stock, import one DXF profile, and place it on the stock face. Use
the complete profile as the source for a captured vertical Wire Contour punch.
The drawing must contain exactly two parallel straight lines and two tangent
semicircular ARC entities of the same radius. Keep the radius between 0.5 and
20 mm. For native posting, use 20 mm stock and place the straights parallel to
X or Y. The cap centers and radius must fit the five-decimal native grid.
Place the threading hole outside a straight side, at least 3 mm from it, with
at least 3 mm of straight face on both sides of the perpendicular entry.

The verified preview encloses each circular cap within 0.000025 mm. The native
ISO file emits the nominal semicircles using their exact centers. The control
applies its own wire offset. Regenerate the operation after moving the sketch.

Run the translated, rotated example from the repository root with a fresh
destination named `RACE20`.

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/racetrack.camscript
```

A drawing with one DXF `CIRCLE` can also produce a 20 mm punch. Use a radius
from 6 to 15 mm. Place the 2 mm threading hole on a horizontal or vertical
radius, at least 3 mm beyond the nearest circular edge.
The native file starts at that edge and defines the circle with two nominal
semicircles. The control applies the 1 mm separation along the circumference.
The whole rough wire and threading hole must fit inside the stock.

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/circle-punch.camscript
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/north-circle.camscript
```
