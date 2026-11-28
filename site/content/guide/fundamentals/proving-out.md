+++
title = "Proving out a program"
description = "Simulation, verify and what its refusals mean, a dry run, single block and the first-article check."
weight = 7
+++

## The principles

A program is a claim that a path is safe and removes the right material. Prove it in
stages, each one cheaper than a crash:

1. **Simulate** the stock being cut, and compare the result with the model.
2. **Dry run** at the machine with the spindle off and the tool well above
   the part (for example with Z offset raised), watching the whole path for
   anything unexpected.
3. **Single block** the first cuts: one line at a time, with feed and
   rapid overrides turned down, a hand on the stop, watching distance to go.
4. **First article.** Cut one part, measure it, and only then run the batch.

Software checks the model, stock, tool and machine facts it was given. It
cannot see a clamp you forgot to enter or a wrong tool length.

## What CamCraft does

- **Simulate** plays the program against the stock. **Deviate** colours
  where material is left or where the program cut into the part, and **Frame
  rest** and **Frame gouge** go to the worst spot of each. **Paths** shows
  the cached toolpaths together.
- **Verify** runs at **Post**. It writes the program only when every
  operation is generated and current, none carries a warning, and the whole
  program passes: no gouge into the part, no move strikes a fixture, no rapid
  runs through stock, and no move exceeds a limit the mounted machine states.
  See [Posting](@/guide/check-and-post/posting.md).
- **What a refusal means.** The message names the operation and the
  problem. A gouge refusal means the simulated cutter removed model
  material; a warning refusal names the flagged moves, which are marked in
  the viewport. The table of common messages is in
  [Reading a refusal](@/guide/reference/refusals.md) and
  [Troubleshooting](@/guide/reference/troubleshooting.md).
- **Why refuse rather than warn.** A warning on a dialog gets clicked
  through and the program is posted anyway. Where the output could drive
  the tool into the part or a fixture, CamCraft writes nothing, and leaves any
  existing file untouched. The two named exceptions are advisory and show on the
  operation without holding the post: a scallop note (a finish quality
  matter) and a probe search distance.
- **"Not judged" is not "fine".** If a check cannot run to completion, for
  example a hidden-material scan that hits its budget, that is reported as
  not judged and holds the post, never counted as zero.
- **Setup sheets** list each setup's tools in change order, with estimated
  time, for the operator.

A green gate is not a guarantee. Run the dry run, single block and
first-article steps above on any new program.

## Stock simulation in CamCraft

These points describe what the stock simulation does in this release:

- When the Stock row of the active setup is hidden in the Browser, the
  line under the simulation transport says: "The stock is hidden, so the
  material removal is not shown. Show the Stock row in the Browser to see
  it."
- The field engine cuts long passes and deep plunges of a wide tool in
  pieces. A short move of a wide tool also cuts. Before, the field engine
  refused moves that the column engine cut.
- A stock side a few float steps longer than a whole number of cells gets
  no extra column, slab or verdict cell.
- A cut floor that lies on the sample lattice meshes to a valid closed
  solid by marching tetrahedra, also when its samples read tiny values of
  both signs.
- The closed-mesh check refuses an edge on four or more facets, not only
  an odd count. Such a mesh falls back to marching tetrahedra.
- Stock export writes a machined stock mesh as a closed solid, also where
  its corners are closer than the float precision of the STL file. It
  welds and checks the corners that merge in the file before it writes.
- A vertical milling rapid that crosses the finished part is blocked.
  Older finish caches must be generated again.
