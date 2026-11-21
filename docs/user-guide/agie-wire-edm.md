# Post an Agie Classic Cut 2S job

CamCraft writes the three native Agie files directly. The setup uses cold die steel (material 0001), Berco Cut 0.25 mm wire and the resident QN2 Fine Ra 0.30 µm technology. Use this recipe only when those settings match the work and the installed control. The controller calculates the wire offsets and trim passes from that technology. CamCraft displays a wire-center preview with a representative finishing gap of 0.005 mm.

## Post from the Wire EDM workspace

1. Mount **AGIECUT Classic 2S (template)** as the setup machine. In the machine editor, open **EDM** and add a condition with ID **AGIE_QN2**, role **rough**, wire diameter **0.25 mm** and preview gap **0.005 mm**. Apply this change to the mounted machine. Choose **Agie native wire EDM** in the post picker.
2. Define axis-aligned rectangular stock, 20 mm thick and at most 350 × 250 mm in plan. This span describes the stock. Verify the installed work datum and the available axis travel on Agie Control. CamCraft refuses profiles with a stated X or Y travel limit because its preview cannot check the full resident path against that limit. Put the first, unrotated work origin on the stock top. Its XY position determines the job coordinates.
3. Use one modeled workpiece and create one **Wire Contour** on the complete punch model. Use a convex punch with straight sides, vertical walls and flat ends at the stock planes. Angled faces are supported. Leave at least 0.4 mm of stock around the nominal polygon. Acute corners can need more stock: the post also checks the intersection of the two offset face lines and the swept wire at that corner. On the tested 20 mm QN2 job, the rough wire centre ran 0.245 mm outside the punch. The stock margin includes the 0.125 mm wire radius. Additional bodies, selected subprofiles, holes, steps and concavities are refused. Select a 0.25 mm wire and condition **AGIE_QN2**. Set the work planes to the stock top and bottom, and specify how the punch is captured. The capture setting confirms support for the punch when it is released. Keep the wire vertical. Taper, four-axis, die cuts, burnout, explicit skims, tabs and fixture models are unsupported.
4. Place the centre of a predrilled 2 mm threading hole in the waste. The entire hole must be inside the stock. Allow a perpendicular approach to the interior of a straight outside face with at least 3 mm of lead and at least 3 mm of straight face on each side of the entry. Generate the operation.
5. Open **Post**, choose **One program** and keep the general post settings at their defaults. Under **Agie technology**, set **Separation cut length (mm)** to **1.0**. Zero leaves the separation unspecified and blocks the post. This length applies to the separation segment that Agie Control generates. Check its position, trims and release sequence in the control’s job preview. Enter the parent folder and a fresh **Job folder name** such as `PUNCH40`. Use up to eight uppercase letters or digits, beginning with a letter. The post publishes the entire directory after validation. To retain an earlier job, give the new one a different name.

## Try the synthetic punch

From the project root, run the supplied script with a local CamCraft desktop CLI build.

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/punch.camscript
```

The script imports an original 40 × 20 × 20 mm rectangular punch, places it in 80 × 60 × 20 mm stock and sets the threading point at X−30 Y0. Its post call selects the native separation with `options: {separationLengthMm: 1.0}`. Stock top is Z0 in the Agie job. The `PUNCH40` directory contains the nominal contour in `PUNCH40.ISO`, its import reference in `PUNCH40.SBR` and the piece, start point and technology in `PUNCH40.SBL`. Existing destinations are refused. Edit the script’s output name to make another job.

For an oblique entry, run `cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/oblique-punch.camscript` with a fresh `POLY20` destination. This pentagonal punch has a 3:4 sloping entry face. Its threading hole at X−20.5 Y6 approaches X−12.5 Y0 perpendicular to the middle of that face.

Run `cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/slope-punch.camscript` for a 7:10 entry face. The `SLOPE20` job keeps the drilled hole at X−20.5 Y6. Its perpendicular entry falls between five-decimal ISO coordinates. CamCraft can move the entry up to 0.00001 mm toward the waste while preserving the lead, separation room and rough-pass clearance.

## Open the job in Agie Control

Keep all three files together. In the control’s file manager, open **Imp/exp** and import the ISO object into **ISO Sys**. Then import the SBL into **SBL Sys**, keeping the matching SBR beside the SBL. In the workpiece list choose **New workpiece → Script Files**, select the imported SBL and confirm with **OK**.

Open the new piece and check the configuration data against the intended setup. Run **FORMALCHECK** and inspect both errors and warnings. Save and close the piece, then run **GRAFICHECK** on that piece. Check the installed XY datum, available axis travel, entry from the drilled hole and compensation on the outside of the punch. In the tested 20 mm job, the main rough pass and three trims left a 1 mm section. The control then returned to cut and trim that section. Check the sequence and arrange support for release. CamCraft does not command a manual pause before the section cut.
