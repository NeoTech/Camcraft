## #2869 - Example case multiaxis/wrapped: a grooved bar on the A axis

**Decided.** The case `examples/catalog/multiaxis/wrapped` is a 20 mm x 20 mm
bar along X with one round-bottom ring groove at X0. Three operations use the
catalog ball mill `BN-0600`: Rotary Offset (rough), Rotary Parallel (finish)
and Rotary Contour (rings over the groove). The machine is the built-in
`Tormach 770MX + microARC 4`. A setup `Wrap` selects the `Tormach PathPilot`
post. The case test pins the three saved kinds and A motion in each posted
section. The full regenerate and rebuild checks are in the heavy lane.

**Why.**
- The rotary kinds refuse a model that is not a shape of revolution. A groove
  that wanders along X (a barrel cam) is refused with a message that names
  #2772. A ring groove is a shape of revolution, and it still shows A motion in
  each of the three kinds.
- Rotary Parallel finishes each free end face down to the A centreline. The
  flutes must reach the centreline from the stock radius. At radius 15 mm the
  8 mm shank of `BN-0600` touched the end face, and verification blocked the
  post. At radius 10 mm (stock radius 11 mm) the 15 mm flutes reach, and the
  catalog tool works without an edit.
- Without a setup, the post refused the program: "tilted move commands B -
  the machine states no B rotary". With a setup on the machine's own post,
  `Tormach PathPilot`, the program has A words only. #4422 tracks the
  refusal without a setup. The MCP surface
  selects a post only per setup (`set_setup_post`), so the script adds one
  setup.
- The rotary generators scale with the facet count. With 72 sectors and a
  0.5 mm axial step (4176 facets), Rotary Parallel took 56 s and Rotary
  Contour 18 s in release, past the 60 s case budget. A 30 mm bar with 48
  sectors (1632 facets) took 17 s alone, but 59 s beside the other heavy
  catalog cases. The final 20 mm bar with 36 sectors and a 1 mm step (1080
  facets) and a 0.2 mm finish tolerance takes about 14 s in the parallel heavy
  lane and about 30 s alone in debug.
- About 30 s alone in debug is too close to the 60 s budget beside the other
  cases of the debug suite. The `2d/slots` case (about 40 s alone) set the
  precedent: the two full checks move to the heavy lane, and a quick test
  pins the committed program.

**Dropped.**
- A sine (barrel-cam) groove: refused by the rotary kinds (#2772).
- A 30 mm x 50 mm bar: the project file was 17 MB, and the shank touched the end
  faces.
- A 20 mm x 30 mm bar: too slow beside the other heavy catalog cases.
- A tool edit (`flute_len` 20 mm, `shank_d` 6 mm): not needed at the smaller
  radius.

**Cost to change.** The project file is about 3.7 MB. Most of it is the Rotary
Parallel path: its raster samples each pass about every 1.1 mm, and it steps
the A rapids between passes. A coarser finish stepover did not make the path
smaller. A different part or kind changes only this case folder and its test.
