## #2769 - Fanuc post: Manual Guide i blank block (G1902 / G1900)

**Decided.**

- The Fanuc post has a new boolean option `manualGuideBlank` ("Manual Guide i blank (G1902 / G1900)", group "Header & comments"). Its default is off. With the option off, the post output does not change.
- With the option on, the post writes one blank block per program. The block comes after `G20` / `G21` and before the first operation's notes, `G90 G54` and `T.. M6`. The call is in `onSection`, before `startPreamble()`.
  - A box stock gives `G1902 B D H I J K`. B, D and H are the X, Y and Z sizes. I, J and K are the work origin minus the stock's minimum corner, in program units.
  - A round bar along Z, with its axis on the work origin, gives `G1900 D L K`. K is the positive-Z end face minus the work origin (`StockZTop - OriginZ`), in program units. A top-face origin gives K0. An origin above the end face would need negative K and receives a refusal comment instead of a blank block.
  - Any other stock gives the comment `(NO BLANK BLOCK - <reason>)` and no block. The reasons are: no shape stated, stock from a model, stock from the setup before, stock off the machine axes, a bar along X or Y, a bar off the work origin, no bar diameter, an end face below the work origin, no bounds, no size, and a WCS turned about Z.
- The app adds three PARAMs to each stream after `StockZTop`, when the setup has a stock: `StockShape`, `StockShapeBottom` and (for a bar) `StockShapeDiameter`. The code is in a new module, `crates/app/src/posting_stock_shape.rs`. `posting.rs` gets two lines only.
- The Heidenhain Klartext post refuses each PARAM that is not on its positive list. It now ignores the three new PARAMs, as it ignores `TopOfStock`. Its output does not change: its `BLK FORM` reads `StockZBottom`. Without this, `klartext_builtin` failed with "unsupported parameter StockShape".
- A tester kit for the Robodrill is in `docs/tester-kits/fanuc-mgi-blank/`. A test pins it byte for byte to the post output.

**Why.**

- Off by default: a Fanuc control without Manual Guide i can alarm on G1902 (issue rule).
- A comment, not a warning: the issue asks for a comment. A warning blocks the write in the app, and the blank is only for the simulation.
- New PARAM names, not `StockZBottom`: the app sends `StockZBottom` only when the machine states a controller identity (the Klartext blank law). A change there moves the Klartext input. A new name changes the input of no shipped post.
- `StockShapeDiameter` is the true diameter. The faceted cylinder mesh gives bounds that are a little smaller when the bar turns about its own axis.
- The call is in `onSection`, not in `startPreamble`. Open PR #3280 changes `startPreamble`. This keeps the two PRs in different functions.
- The bar's centre comes from the middle of `StockXY*`. The 32-segment cylinder is symmetric, so the middle is the axis. `invert.rs` and `wcs_flip.rs` already mirror `StockXY*` and `OriginX/Y`, so the block stays correct on an inverted or flipped X or Y.

**Dropped.**

- `G1906` (a bar along X): it needs the machine's X-axis work-rotation parameters. The research comment plans it after the kit test. It posts a comment for now.
- `G1901`, `G1903`, `G1904`, `G1907` (hollow and square bars): CamCraft has no such stock shape.
- A guard for an inverted Z axis: the engine refuses an inverted Z before the script runs, so the guard was dead code.
- A second blank for a later setup with a different stock: the program writes the first section's stock only. Manual Guide i draws one blank per program.

**Bar end-face reference.** Fanuc B-63874EN/05, printed page 193 (PDF page 215), defines G1900 K from the work origin to the positive-Z end face. The lower-end analogy with G1902 was dropped because it shifts the simulated bar by its length. Tests pin a top-face origin (K0), an origin below the top face (K7), and refusal of negative K.

**Not verified on a control.** Manual Guide i reads the blank in the control's input unit, not the unit of `G20` / `G21` (manual, as the research comment reports). An inch program on a metric control can draw the wrong size in the simulation. The kit README says so and asks the user for the control's unit.

**Cost to change.** The end-face distance and its refusal guard live in `assets/posts/fanuc.cps`; the bar tests and NC fixture pin their output. The PARAM names are in the stream doc (`docs/CNC-INTERMEDIATE.md`) and in one app module.
