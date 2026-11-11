## #743 - the simulated tool is drawn as the revolved outline the laws judge (part 1)

**What was decided.**

1. *The simulation draws the judged outline, not the body's own facets.*
   The tool at the playhead is one revolved body built from two reads the
   collision laws already make: the judged tool record
   (`tool_body::judged`) for the cutter and shank, and the band stack the
   fixture laws sweep (`tool_body::stack`) for everything above the shank -
   the holder cylinder, an imported body's revolved envelope (#368) or a
   drawn holder's bands (#93). Before this, a tool with a body drew the
   body's raw facets (an imported mesh with its flats and slots at a fixed
   turn; a drawn profile with its cones and arcs).
2. *A cone or an arc is therefore drawn stepped.* The laws read a drawn
   cone as one cylinder at its larger end (`SpunProfile::bands`), so the
   simulation now shows that cylinder. It looks coarser than the drawing,
   and it is exactly what is checked. The tool editor's preview still
   shows the drawn outline.
3. *One mesh, cached, keyed by the outline itself.* `SimToolCache` holds
   the mesh of the one tool at the playhead. The key is the judged
   outline, so any edit that moves a judged number rebuilds it and nothing
   else does; it empties when the tool or operation is gone, the tool is
   refused, or the transport closes. It lives in `State` behind a `Mutex`
   because the scene is collected from `&State`.
4. *A tool that cannot be drawn draws nothing and is logged once.* A body
   that no longer fits its tool, or a degenerate outline (non-finite,
   negative, running back down the axis, no radius, past the body caps),
   gives no body and one `tracing::warn!` per distinct reason. Before, a
   misfitting body was still drawn.
5. *No new command, no MCP tool.* Playback position is ephemeral; nothing
   here writes the world.

**Why.** The issue asks for "the same profile the collision laws judge, so
what you see is what is checked". Calling the laws' own two functions is
the only way the picture cannot drift from the verdict; a second
derivation of the holder shape for display would be a second truth.

**Considered and dropped.**

- *Keep drawing the body's facets and add the envelope as a second,
  translucent shell.* The viewport's model parts have no translucent
  class, and two bodies on screen leave it unclear which one is judged.
- *Draw the smooth drawn outline for a spun holder.* Prettier, but it
  shows a cone where the law sweeps a cylinder - a tool that looks clear
  of a clamp can be reported as striking it.
- *A cache keyed by `doc_version`.* Every command bumps it, so the mesh
  would be rebuilt after edits that do not touch the tool, and the app
  layer could not test the cache without the UI.
- *A per-tool map of meshes.* Only one tool is at the playhead; a map
  would need its own eviction on tool/op deletion for no visible gain (a
  rebuild is a few hundred triangles).
- *Drawing the lathe insert kinds.* Their profile is a stand-in envelope;
  they keep the axis marker, as before.

**Known gap, filed not fixed.** The laws that read one holder cylinder
(shank reach, gouge and the rest) judge the cylinder *enclosing* the body
(`HolderProfile::enclose`), which is wider than the band stack drawn here
wherever the body is stepped. This slice does not touch any law; the
difference is filed as #835.

**Cost to change later.** Small: `JudgedOutline::new` is the one place the
drawn shape is assembled, and `world::sim_tool_tris` the one caller.
Part 2 of #743 (STEP body to an editable DXF half-profile) is untouched.
