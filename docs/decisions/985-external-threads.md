## #985 - External threads on a boss at the operation level

**What was decided**

1. **One flag, one list.** `OpParams::Thread` gains `external: bool`. The
   feature list keeps its shape (`Hole`): on a boss `radius` is the MAJOR
   radius (CAD draws an external thread at its nominal diameter),
   `entry_z` the boss's top and `floor_z` its shoulder. File key
   `external`, additive, written only when set (no project-version step);
   the stamp digests it only when set, so no internal thread goes Stale.
2. **Bosses are recognised, not picked (yet).** `drilling::recognize_bosses`
   is the hole recogniser's own weld, circle fit and rim-span law with the
   facing vote inverted: closed round upright walls whose facets face AWAY
   from their axis. Stepped walls on one axis are NOT folded (a stepped
   shaft is two bosses; folding "narrow radius, whole span" would thread
   into the wider step). "Detect holes" (button and MCP `detect_holes`)
   reads bosses when the operation says external. A round base is a boss
   too, so the Thread page's list gets a per-row Remove button (draft edit,
   committed by Apply as one undoable `EditOperationParams`).
3. **Three readings judge a tool beside a boss; none asks the silhouette.**
   The flat footprint region cannot judge here: the boss rises from a
   wider base, and the base's outline would read as a wall around a tool
   that stands above it.
   - *Crest law*: the tangency verdict reads a region holding only each
     boss's MINOR circle (major radius less 0.6134 x pitch). The teeth may
     reach the design root and no further. This is #1071's lesson read for
     a boss: the drawn wall is not the wall the crests are judged against,
     because the helix cuts past it by design.
   - *Root law*: the solid law with the cutter narrowed by the EXTERNAL
     depth (`thread_op::root_cutter`): tooth roots, tip, shank and holder
     against the whole part - the boss's drawn wall and its shoulder
     included, the tip with no allowance.
   - *Neighbour law* (new, `thread_boss::neighbours`): the crest ring at
     the tool's full radius over its cutting length against every facet
     that is not the threaded boss's own wall, with the gouge tolerance
     only. The root law grants the thread depth radially at EVERY wall;
     inside a hole there is no other wall, beside a boss there can be (a
     rib, a second boss). A facet is the boss's own when all its vertices
     stand within the boss's radius + 0.05 mm of its axis. Which boss a
     run of feeds threads is read from the path's structure (the boss its
     helix arcs are centred on), never from nearness, so a neighbouring
     listed boss is not excused. Rapids excuse nothing.
4. **The air law (the external analogue of #992).** The mill feeds down
   BESIDE the boss and cuts only the thread's depth, so at generate every
   position the feeds take the tool through is read in the stock the
   program ahead leaves (the same lazy prior grid): each column under the
   tool's full disc, more than one grid cell outside the boss's drawn
   wall, must stand at or below the tip there. Otherwise generate refuses
   by name with the place, the heights, and how far out and how deep to
   clear. Unknown (nothing ahead cut, unreadable grid, a posed operation)
   is refused. One cell of slack at the boss wall is the lattice's
   resolution; a part wall left standing in the sweep is refused by the
   same reading.
5. **The door's boss half** (create, edit, retool, and again at generate):
   a boss no thicker than the thread is deep; a thread longer than the
   mill reaches beside a boss (the cutting length when the shank is wider
   than the tooth roots, cutting length + shank when only the holder is);
   the infeed against the external depth. A stated bottom is judged at
   generate, where it resolves.
6. **The program simulation sweeps an external thread at its external
   root** (`thread_op::sim_radius`): the internal depth at the tool's
   coarsest pitch read 0.07 x pitch of the boss as cut away and the post
   gate called it a gouge. Internal threads and taps keep
   `tool_law::cutting_radius` untouched.
7. **The kernel's ride is used as #1005 pinned it** (`external: true`):
   right-hand climb orbits clockwise from the top down, quarter-arc leads
   one tool radius outside the centre circle, feeds only below the top.

**Why**

The issue's "Done means": external threads a modelled boss, verifies clean
against the part and the stock left before it, posts helical G2/G3 with Z;
a boss thinner than the depth and one with no cleared air are refused by
name; pinned by app tests and goldens.

**Sources (public)**

- ISO 68-1 / ISO 965-1 / DIN 13: external design root 17/24 H = 0.6134 P
  (M8 x 1.25 root Ø6.466, M10 x 1.5 root Ø8.160).
- Machinery's Handbook, "Thread milling"; Smid, "CNC Programming
  Handbook", thread milling chapter: external thread milling orbits
  outside the work from clear air, tangential arc entry, climb direction.
- Sutherland-Hodgman polygon clipping; point-to-polygon distance
  (O'Rourke, "Computational Geometry in C").

No HSMWorks / Fusion material, output or documentation was consulted.

**Considered and dropped**

- *The model's section at depth with the thread depth as allowance* (the
  issue's sketch): it grants the thread depth at every wall in the
  section, so a rib 0.7 mm inside the crest sweep passes, and a coarse
  boss tessellation moves the verdict by its chord sag. The exact minor
  circle plus the neighbour law has neither gap.
- *Keeping the silhouette in the crest region*: a base edge within one
  tool radius of the orbit convicts every move of a legitimate thread.
- *A full-cutter solid law against "the part without the boss"*: needs a
  closed mesh with the boss removed, which chamfered or bored bosses do
  not give.
- *Nearest-wall ownership of a tool position*: between two close bosses
  it excuses the wrong one for part of the orbit.
- *Reading the whole ring out to the farthest reach at the lowest tip*
  for the air law: refuses stock under the lead-out station that the tool
  only ever passes over high. The per-position reading is exact.
- *Folding stepped bosses like counterbores*: see 2.
- *A viewport pick for bosses in this PR*: the list editor (Remove) makes
  detection usable; a pick through the one pick rule (#692) is filed as a
  follow-up.

**What it would cost to change later**

- `external` is in the file grammar and the MCP params; renaming needs a
  reader alias.
- A near-net or cast stock with nothing machined ahead is refused (as a
  cored hole is under #992); acquitting it needs the raw stock grid.
- Posed (3+2) external threads need a frame-aware prior reading.
- The simulation still shows the boss at its drawn diameter (the thread
  form is not drawn; the roots stand on the wall).
- The neighbour law samples every 0.25 mm and walks the facets the path's
  box reaches; a model with very many facets around the boss wants a
  spatial index.
