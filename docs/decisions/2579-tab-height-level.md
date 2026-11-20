## #2579 - 2D Contour tabs: the tab top is a cut level

**What was decided.** A closed-ring 2D Contour with tabs gets one more cut
level at `bottom_z + tab_height`, when the stepdown ladder has no level
there. `ContourParams::ring_levels` adds it. The closed contour
(`contour_region`) and the whole-model contour (`contour_whole`) read it.
A ladder level within 1e-6 mm of the tab top becomes the tab top itself.

**Why.** A level carries the tabs when it is below `bottom_z + tab_height`.
The last level above that plane cuts the whole ring. The tab therefore
stood up to that level: 4 mm with a 4 mm stepdown and a 1.5 mm tab height.
With a level at the tab top, the tab is as tall as the user set, for a
stepdown larger or smaller than the tab height. The issue's "Done means"
asks for this pass.

The snap of a near level has the same cause. Repeated subtraction gives
levels such as `-3.0000000000000004`. That level is below a tab top at
`-3.0`, so it carried the tabs and the tab was one stepdown too tall
(0.2 mm stepdown, 1 mm tab height: 1.2 mm).

**Considered and dropped.**

- A note that names the real tab height, with no new level. The issue
  permits it only "if a level must be skipped". No level must be skipped
  here, and a note leaves the tab too strong.
- A change of the stepdown so that a level lands on the tab top. This
  changes each level of the cut for one parameter. The added level changes
  only the passes near the tab.
- The tab level in `ContourParams::levels` for each caller. The open runs
  (`contour_along_chain`, `contour_open_run`, `contour_open_run_sided`)
  have no tabs. A level there is a pass with no purpose.
- A change to a tab top at or above Top Z. Each level carries the tabs
  then, and no level is added. The tab is as tall as the cut. This is not
  the reported defect.

**Cost to change later.** Small. The rule is one function,
`ContourParams::ring_levels`, with two callers. The test
`crates/motion/tests/contour_tab_height.rs` measures the tab height from
the posted moves.

**Known effect.** A program with tabs whose stepdown ladder had no level at
the tab top gets one more pass. Its G-code changes. A program without tabs
does not change.

The shipped example `examples/first-router-part` is such a program: 4 mm
stepdown, 12 mm depth, 3 mm tabs. Its program gets one whole pass at Z-9.
The project file is the command build, saved again (`WRITE_GOLDEN`, the
decision of #1917). The save also writes what `main` changed since the
last save: file version 68 to 71, the tool numbers, the drill undersize
warnings and the start of the first rapid. A file with only the new pass
does not pass the verification gate of its own test, because the cache
stamp does not agree.
