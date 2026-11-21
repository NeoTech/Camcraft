# Post an Agie Classic Cut 2S job

CamCraft writes one native ISO geometry file per work and one SBL and SBR for the complete job. The setup uses cold die steel (material 0001), Berco Cut 0.25 mm wire and a quality selected from the installed control. The saved condition **AGIE_QN2** requests Fine Ra 0.30 µm through the native quality **CCB030**. **AGIE_BERCO_RA035** uses **CCB035** for its separate Ra 0.35 µm definition. Use this recipe only when those settings match the work and the installed control. CAMCraft imports the complete piece and each work with zero working steps. The operator checks the selected technology on that loaded piece before generating working steps in Agie Control. CamCraft displays a wire-center preview with a representative finishing gap of 0.005 mm.

## Post from the Wire EDM workspace

1. Mount **AGIECUT Classic 2S (template)** as the setup machine. In the machine editor, open **EDM** and add a condition with ID **AGIE_QN2**, role **rough**, wire diameter **0.25 mm** and preview gap **0.005 mm**. Apply this change to the mounted machine. Choose **Agie native wire EDM** in the post picker.
2. Define axis-aligned rectangular stock at most 350 × 250 mm in plan. Choose a stock thickness of 15, 20, 40 or 80 mm. This span describes the stock. Verify the installed work datum and the available axis travel on Agie Control. CamCraft refuses profiles with a stated X or Y travel limit because its preview cannot check the full resident path against that limit. Put the first, unrotated work origin on the stock top. Its XY position determines the job coordinates.
3. For a single work, use one modeled workpiece and one **Wire Contour**. For several punches on the same stock, use the grouped workflow below. For a die opening, follow the selection and piercing instructions below. For a punch, select the complete punch model. Use a punch with straight sides, vertical walls and flat ends at the stock planes. Angled faces are supported. Leave at least 0.4 mm of stock around the nominal polygon. Acute corners can need more stock: the post also checks the intersection of the two offset face lines and the swept wire at that corner. CamCraft checks each corner with the rough offset for the selected stock thickness. The stock margin includes the 0.125 mm wire radius. Some reentrant contours are supported. A tight inside corner fails if the rough offset leaves more than 0.002 mm at the nominal vertex. Short segments and closely spaced faces fail if adjacent rough paths could meet. The punch must have a straight outer face that allows the lead to stay entirely on the waste side. The punch must have one boundary and no cavities, steps or other bodies. Select a 0.25 mm wire and condition **AGIE_QN2**. Set the work planes to the stock top and bottom, and specify how the punch is captured. The capture setting confirms support for the punch when it is released. Keep the wire vertical. Taper, four-axis, burnout, explicit skims, tabs and fixture models are unsupported.
4. Place the centre of a predrilled 2 mm threading hole in the waste. The entire hole must be inside the stock. Allow a perpendicular approach to a straight face with at least 3 mm of lead and at least 3 mm of straight face on each side of the entry. Generate the operation.
5. Open **Post**, choose **One program** and keep the general post settings at their defaults. Under **Agie technology**, set **Separation cut length (mm)** to **1.0**. For a closed contour, zero leaves the separation unspecified and blocks the post. This length applies to the separation segment that Agie Control generates. Check its position, trims and release sequence in the control’s job preview. Enter the parent folder and a fresh **Job folder name** such as `PUNCH40`. Use up to eight uppercase letters or digits, beginning with a letter. The post publishes the entire directory after validation. To retain an earlier job, give the new one a different name.

### Release the loaded works in Agie Control

1. Import all geometry resources and the piece. Keep this piece open. Each work starts with zero working steps.
2. Open each work and inspect the effective quality in the controller editor. Check material, wire, technology height, Ra, Te, Tkm and both visible option boxes against the CAMCraft post receipt. The post fixes all three requested flags to off. The editor displays only the speed and high-quality boxes. A quality label by itself does not establish the selected parameters.
3. If a value differs, keep the work at zero steps. On the tested control, open **Kwaliteit → Definiëren** (quality definition) in that work. Select the entry with the correct displayed values, confirm the quality and confirm the work. Reopen that work and verify the selected values. A numerical suffix can distinguish definitions with the same base name.
4. In that work open **Technologie → TECCUT**. Check the **Vooraf geprogrammeerd** (preprogrammed) row against the required wire, material, height and targets. Choose **Toewijzen** (assign) only on a matching row. Inspect **Arbeidsstappen** (working steps). Check the main cut and each trim in order. Compare every generated wire offset with the receipt and the table below. Check the release sequence and the complete graphical path. Run the formal check and complete the normal machine setup before machining. Perform these checks again after importing or changing a job.

### Group independently captured punches

Use a single setup with 20 mm stock and two to nine complete modeled punches. Create a **Wire Contour** for each solid, select **Punch** and the same wire tool, and enter its own threading hole. Confirm capture separately for each punch. Leave a clear band along X or Y between the complete cutting areas of every pair. The check includes each solid, threading hole, lead, and the furthest native rough corner. A shared corridor or another model prevents export.

Generate all operations. Select a new job name of up to seven letters and digits. The post adds one digit to the name for each ISO resource. The work names `CUT1`, `CUT2` and so on follow program order. They share the stock origin. In the control preview, check each threading location and the profile and separation sequence. The control can complete the profile passes on every punch before it starts their separation cuts.

Run the synthetic example from the project root with a fresh `PAIR20` destination.

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/punch-pair.camscript
```

The left rectangular punch starts at X−30 Y0. The right five-sided punch starts at X1 Y0. Copy all four files in the directory together.

The three-punch example creates a five-file job. The grid example places nine independent punches on a 32 mm pitch and creates eleven files in the NINE20 directory.

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/punch-trio.camscript
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/punch-grid.camscript
```

### Select a die opening

Use a 20 mm thick retained plate with one through-opening, planar caps and vertical walls. In the Wire Contour Geometry tab, choose **Edges** and Shift-click the connected edges of its upper opening rim. Set the wire side to **Die**. Set **Freed piece captured** when the setup can hold the released slug. Place the complete 2 mm threading hole inside the opening, with at least 3 mm between its centre and the entry face. Leave 3 mm of straight face on each side of the perpendicular entry. The supported opening is a convex polygon with corners that the rough wire can reach within 0.002 mm.

Run the original example from the project root with a fresh output directory:

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/die-plate.camscript
```

The example selects the complete upper opening rim on a 60 × 60 mm plate. The native `DIE20` job starts at X−5.5 Y0 and enters the straight wall at X−12.5 Y0. Inspect the **inside** compensation and the 1 mm rough separation in Agie Control. The controller completes the separation before it makes the three full trim passes. Arrange support for the slug and a clear trim path after release.

### Group two die openings in one plate

Use one retained plate with exactly two through-openings, one setup, 20 mm stock and the same **AGIE_QN2** wire condition for both operations. Create one **Wire Contour** for each opening. Select each opening's upper rim, enter its own threading hole and confirm capture for its slug. Leave a clear band along X or Y between the two cutting areas, including the leads and wire clearance. Generate both operations and post to a new job folder with a name of up to seven characters.

Run the original two-opening example from the project root.

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/die-pair.camscript
```

The `DUODI20` folder contains two ISO files, one SBR and one SBL. The left work threads at X−35.5 Y0. The right work threads at X24.5 Y0. Check each inward offset and the release of both slugs in the control preview before running the job.

### Check the selected quality and wire offsets

Open each work in the Agie Control editor. Check material 0001, Berco Cut 0.25 mm, the actual technology height and the selected quality parameters. For **CCB030**, the requested Ra, Te and Tkm are 0.30, 1.50 and 1.00 µm. The request sets all three optional flags to off. Check the two visible boxes and the named work steps and their wire-centre offsets before machining. A formal check can pass even when an existing quality has different settings.

Agie Control calculated these wire-centre offsets on the straight outside faces of a synthetic rectangular punch. All dimensions are in millimetres. The native job sends nominal geometry. The resident technology applies each offset.

| Stock thickness | Rough | Trim 1 | Trim 2 | Trim 3 |
| ---: | ---: | ---: | ---: | ---: |
| 15 | 0.245 | 0.155 | 0.140 | 0.134 |
| 20 | 0.245 | 0.155 | 0.140 | 0.134 |
| 40 | 0.260 | 0.170 | 0.140 | 0.130 |
| 80 | 0.266 | 0.176 | 0.143 | 0.133 |

The clearance check rounds the rough offset plus the wire radius up to the next tenth of a millimetre. For these four heights this gives 0.4 mm on a straight face. At a pointed corner the post checks the miter for that height separately. Other heights require their own control measurements.

### Request Fine+ Ra 0.35 µm for a 20 mm punch

For one closed punch in 20 mm cold die steel, add a mounted machine condition named **AGIE_BERCO_RA035**. Use role **rough**, Berco Cut **0.25 mm** wire and a **0.005 mm** preview gap. Select **AGIE_BERCO_RA035** in the Wire Contour condition field and save the project. The native job requests Ra 0.35 µm, Te 1.50 µm and Tkm 1.00 µm. The request fixes all three optional flags to off. Check the targets and the two visible options in the editor.

The offline Agie Control selected T100 for the main cut and three trim passes. The straight-face wire offsets matched the 20 mm row above. The Ra value is the requested surface finish. Measuring the resulting surface requires a physical cut. Groups, dies, open cuts and other stock heights require their own qualification for this quality.

Run the example from the repository root into a fresh `FINE20` directory.

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/fine-punch.camscript
```

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

Choose **Agie native wire EDM**, set the separation cut length to zero and choose a new output directory. On Agie Control, [release the loaded works](#release-the-loaded-works-in-agie-control) before plotting. Inspect the formal check and all four plotted cutting passes. Each pass must cross the stock once, alternating direction.

Run either original example from the repository root:

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/cutoff-lower.camscript
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/cutoff-upper.camscript
```

The examples select opposite sides of synthetic 80 × 60 × 20 mm stock. Each writes one native job, named `LOWER20` or `UPPER20`.

## Open the job in Agie Control

Keep the complete job directory together. In the control’s file manager, open **Imp/exp** and import every ISO resource into **ISO Sys**. Then import the SBL into **SBL Sys**, keeping the matching SBR beside the SBL. In the workpiece list choose **New workpiece → Script Files**, select the imported SBL and confirm with **OK**.

Select the newly created row in the workpiece list. It can have a default name such as **Werkstuk:4**. Reimporting the same SBL creates a separate piece. Earlier copies retain their working steps. In the new piece, match every work to its expected ISO resource and geometry, and verify that each work shows **[0]** working steps. Follow [Release the loaded works in Agie Control](#release-the-loaded-works-in-agie-control) to inspect, select and assign the actual technology. Then run **FORMALCHECK** and inspect both errors and warnings. Choose **Sluiten** (close), then **Ja** (yes) at **Opslaan?** (save). Run **GRAFICHECK** on the saved piece. Check the installed XY datum, available axis travel, entry from the drilled hole and compensation on the waste side of each face. In the tested 20 mm punch job, the main rough pass and three trims left a 1 mm section. The control then returned to cut and trim that section. Check the sequence and arrange support for release. CamCraft does not command a manual pause before the section cut.

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

### Cut a convex DXF punch with tangent lines and arcs

1. Create 20 mm box stock in a Wire EDM setup. Import the complete closed DXF loop and place it on the top stock face. Use true LINE and ARC entities. Each joint must be tangent, with the material on the inside through every turn. Arcs can have different radii from 0.5 to 20 mm and sweep up to 180 degrees.
2. Add a **Wire Contour** with the complete placed drawing as its source. Select **Punch**, the Berco Cut 0.25 mm wire and the qualified machine condition. Set the heights to the two stock faces and confirm that the punch will be captured.
3. Place the 2 mm threading hole outside a straight parallel to X or Y. Its perpendicular projection must lie at least 3 mm from both ends of that straight. Leave at least 3 mm from the hole center to the straight. Other faces may be oblique.
4. Generate and verify the toolpath. Choose **Agie native wire EDM**, set the separation to 1 mm and choose a new output directory. On Agie Control, [release the loaded works](#release-the-loaded-works-in-agie-control) before checking the formal result and plot. The ISO preserves each nominal endpoint and arc center. The resident technology applies the compensation.

Placed joints and centers must be representable on the five-decimal ISO grid. The post checks the two emitted radii and tangent direction at every joint. The normal verification preview uses up to 2,048 tangent vertices to enclose the source arcs. Drawings can contain up to 128 curve segments.

Run the mixed-radius four-corner example or the three-corner example with an oblique side from the repository root.

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/rounded-punch.camscript
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/three-corner-punch.camscript
```
