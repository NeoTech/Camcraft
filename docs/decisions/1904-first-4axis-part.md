## #1904 - "Your first 4-axis part": the 3+1 indexed walkthrough on the 770MX + microARC 4

- **What was decided:**
  1. The page is `site/content/guide/getting-started/first-4axis-part.md` (`weight = 6`, after the router part). It is linked from the Getting started index, from `first-run.md` next to the router first part, and from the Multi-axis index. The maintainer's scope note on the issue asks for this place and title. The plasma first part (#1905) is not on `main` yet, so the link sits next to the router part.
  2. The part is a 3/4 in round bar, 1.5 in out of the chuck: one 0.015 in flat on each of four faces (A-90, A0, A90, A180) with a 1/4 in end mill, then a cross slot on the A0 face with a 1/8 in end mill. The holding, the work offset and the cut sizes are those of the tester kit `docs/tester-kits/tormach-microarc-4th/`, with two more faces.
  3. Each operation is a typed Slot with its own **Tilted plane (3+2)**. One setup holds all five. CamCraft has no index at setup level, and a tilted operation refuses a picked face or edge ("a tilted plane cannot yet index a picked selection").
  4. The companion files are `examples/first-4axis-part.camproj`, `examples/first-4axis-part.nc` and `examples/models/first-4axis-part.obj` (made by `gen_first_4axis_part_obj.py`). `crates/app/tests/first_4axis_part.rs` builds the project through the app's commands and holds the program to the post. `site/tests/test_first_4axis_part.py` holds the page to the UI sources, the project and the program.
  5. The builder uses only values that a user can type in the UI: tools from the New tool starter, stock from Fit to Models with margin 0, heights as Top of stock plus an offset, the Slot defaults elsewhere.
  6. The face opposite the top is documented as it works today: Tool axis A 0, B 180, with the plane's X and Y exchanged. Issue #2133 holds the two problems. The page links it.
  7. The continuous 4-axis walkthrough is not written. `tormach.cps` refuses an A that changes inside an operation, and #796 needs PathPilot Hub results first. The page says so and lists the refusals.

- **Why:**
  - The tester kit's holding and cuts already have a safety review, and shallow cuts keep a first 4-axis run low-risk.
  - Slot is the only planar kind that needs no pick, so it is the kind that works under a tilt on all four faces today.
  - A box stock is used because a Cylinder stock always stands along Z and the Stock page turns stock in yaw only.
  - The stock simulation reports a tilted program as not judged. The page states this and sends the user to the PathPilot simulator and an air cut.

- **What was considered and dropped:**
  - **One setup for each index angle:** CamCraft has no such control. Part alignment turns the model, not the A axis.
  - **Pockets or holes on the faces, with picked geometry:** refused under a tilt (above). Drill takes no tilt.
  - **A model that is only a core below the cuts (the kit's model):** a user guide needs the finished part as the model. The finished-part mesh passes the same checks.
  - **A page under Multi-axis:** the scope note places it in Getting started.
  - **Exact PathPilot screen labels in the Hub steps:** not written, because the repository holds no checked source for them.

- **Cost to change later:** Low. The page is additive Markdown. The project and the program regenerate with `WRITE_GOLDEN=1 cargo test -p camcraft-app --test first_4axis_part`. When #2133 changes the A180 input or the plane axes, the page's Flat A180 row and the builder change with it. When #796 lands, the continuous walkthrough is a second page.

- **PR #2135 review correction:** The first model had square-ended flats from X-0.875 to X-0.125 across the full chord. The 1/4 in tool cuts round ends, so the model was up to 0.0117 in below the real part near each end. The generator now makes the round ends (the skin as lines along X, with 1 degree samples on the flats), and the project holds the new model. `the_model_is_the_bar_as_the_program_cuts_it` compares the model with the surface that the cached toolpaths leave, on a grid over each of the four faces. Changing the operations to cut square ends was dropped: it needs more passes for no gain on a first part.
