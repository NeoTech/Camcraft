# #2867 - Example case 3d/rest

## Chosen

- The case uses two 3D Pockets: a 12 mm end mill (`EM-1200-2FL`), then a
  6 mm end mill (`EM-0600-4FL`). The second pocket is limited by the
  re-machining mask: CamCraft simulates the program ahead and cuts only
  where material still stands. No new parameter or code was needed.
- The part is a plate with two ribs and an 8 mm channel between them. The
  walls are vertical, so the 12 mm tool leaves a rest region with a clear
  shape (the channel and its two mouths), and the 6 mm tool leaves no
  thin skims on the outer walls.
- The README shows the rest region as a text top view with coordinates.
- The test `catalog_3d_rest::the_small_tool_cuts_only_the_rest_region` pins
  the claim of the issue ("the second operation cuts only that region") on
  the committed program.

## Why

- The earlier blocker note on #2867 said that no 3D operation has a
  rest-stock input. The 3D Pocket reads the stock of the program ahead
  (M3.1 D), which is rest machining by the previous operations. That is the
  workflow of the case.
- A height-field mesh with ramped walls leaves a small staircase on each
  wall. The second pocket could then cut thin skims on all walls, and the
  rest region would not be clear.

## Dropped

- `rest_rough` on the 3D Pocket. It closes the staircase of the same tool.
  It is not a second, smaller tool.
- Catalog screenshots (`cargo xtask catalog --screenshots`). The command
  pushes to an images branch. The README text view and the pin show the
  region without it.
- A 3D Finish after the rest pass. The case shows rest machining only.
  `3d/rough-finish` shows the finish operations.
