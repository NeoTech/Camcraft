# Post an Agie Classic Cut 2S job

CamCraft writes the nominal contour, resource manifest and piece script directly. The job opens with zero erosion steps. It requests cold die steel (material 0001), Berco Cut 0.25 mm wire and a target finish of Ra 0.30 µm. Check the effective work technology on the installed control before assigning any cutting steps. The control calculates the offsets and trims. The CamCraft wire-center preview uses a 0.005 mm representative finishing gap.

## Post from the Wire EDM workspace

1. Mount **AGIECUT Classic 2S (template)** as the setup machine. In the machine editor, open **EDM** and add a condition with ID **AGIE_QN2**, role **rough**, wire diameter **0.25 mm** and preview gap **0.005 mm**. Apply this change to the mounted machine. Choose **Agie native wire EDM** in the post picker.
2. Define axis-aligned rectangular stock, 20 mm thick and at most 350 × 250 mm in plan. The stock and work origin together must fit that span. Verify the installed work datum and the available axis travel on Agie Control. CamCraft refuses profiles with a stated X or Y travel limit because its preview cannot check the full resident path against that limit. Put the first, unrotated work origin on the stock top. Its XY position determines the job coordinates.
3. Use one modeled workpiece and create one **Wire Contour** on the complete punch model. Use an axis-aligned rectangular punch with vertical sides and flat ends at the stock planes. Leave at least 0.4 mm of stock beyond each face. On the tested 20 mm QN2 job, the rough wire centre ran 0.245 mm outside the punch. The stock margin includes the 0.125 mm wire radius. Additional bodies, selected subprofiles, holes, steps and concavities are refused. Select a 0.25 mm wire and condition **AGIE_QN2**. Set the work planes to the stock top and bottom, and specify how the punch is captured. The capture setting confirms support for the punch when it is released. Keep the wire vertical. Taper, four-axis, die cuts, burnout, explicit skims, tabs and fixture models are unsupported.
4. Place the centre of a predrilled 2 mm threading hole in the waste. The entire hole must be inside the stock. Allow a perpendicular approach to the interior of a straight outside face with at least 3 mm of lead and at least 3 mm of straight face on each side of the entry. Generate the operation.
5. Open **Post**, choose **One program** and keep the general post settings at their defaults. Under **Agie technology**, set **Separation cut length (mm)** to **1.0**. Zero leaves the separation unspecified and blocks the post. This length applies to the separation segment that Agie Control generates. Check its position, trims and release sequence in the control’s job preview. Enter the parent folder and a fresh **Job folder name** such as `PUNCH40`. Use up to eight uppercase letters or digits, beginning with a letter. The post publishes the entire directory after validation. To retain an earlier job, give the new one a different name.

## Try the synthetic punch

From the project root, run the supplied script with a local CamCraft desktop CLI build.

```sh
cargo run -p camcraft-app --bin camcraft-cli -- script examples/agie/punch.camscript
```

The script imports an original 40 × 20 × 20 mm rectangular punch, places it in 80 × 60 × 20 mm stock and sets the threading point at X−30 Y0. Its post call selects the native separation with `options: {separationLengthMm: 1.0}`. Stock top is Z0 in the Agie job. The `PUNCH40` directory contains the nominal contour in `PUNCH40.ISO`, its import reference in `PUNCH40.SBR` and the piece, start point and requested quality in `PUNCH40.SBL`. Existing destinations are refused. Edit the script’s output name to make another job.

## Open the job in Agie Control

Keep all three files together. In the control’s file manager, open **Imp/exp** and import the ISO object into **ISO Sys**. Then import the SBL into **SBL Sys**, keeping the matching SBR beside the SBL. In the workpiece list choose **New workpiece → Script Files**, select the imported SBL and confirm with **OK**.

Select the newly created piece. After a repeat import, an earlier piece can retain its machining steps. Match the new work’s ISO geometry and threading hole to the setup. Check that the new work shows **[0]**. At this point **FORMALCHECK** should report undefined technology.

1. Open **Work Details** on the new work. Verify steel **0001**, wire **Berco Cut 0.25**, stock height **20 mm** and the effective work quality. The requested **CCB030** values are **Ra 0.30**, **Te 1.50** and **Tkm 1.00**, with three options disabled. Check the two options the control displays. A resident quality with the same name can hold different values.
2. If any visible value differs, use **Quality → Define** on this work to select the requested values. Confirm the effective settings before proceeding.
3. Open **Technology**, select **TECCUT** and choose **Assign** on the matching row. Inspect the generated steps. The expected rough and trim offsets are **0.245, 0.155, 0.140 and 0.134 mm**, in that order.
4. Run **FORMALCHECK**. Inspect errors and warnings. Save the piece and run **GRAFICHECK**. Check the installed XY datum and travel, the approach from the hole and compensation outside the punch. Verify the 1 mm separation section, trimming sequence and support for release. CamCraft does not command a manual pause before the section cut.
