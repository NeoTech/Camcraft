## #1617 - a closed STEP solid wound inward is turned outward at import

- **Chosen:** after a STEP model is meshed, each body (a connected set of
  facets) that is closed, manifold, consistently wound and has a clearly
  negative signed volume is reversed facet by facet, and the import says
  so in a new `Graded::notes` / `ImportModel::notes` list. The note reaches
  the MCP and script `import` result (`notes`) and the status line. The
  verdict reuses `pick::winding::winding_of`, the same closed/consistent/
  signed-volume test the face picks already refuse on, so the importer and
  the picks agree on what "inward" means.
- **Why per body, after meshing:** the welded mesh is what the
  operations read; a body with a gap, a lone face, a non-manifold edge or
  mixed winding has no outside to trust and is left as the file says.
  Working on the finished mesh keeps `step.rs` (rewritten by open PRs
  #1576 and #1638) untouched.
- **Overlap with PR #1576:** that PR adds the same `notes` plumbing
  (`Graded`, `ImportModel`, `v_import`, `cam.d.ts`, the status line) for
  its flipped faces. The code lines here are the same as its lines where
  possible, so the second PR to merge resolves by keeping one copy of the
  plumbing and both note sources.
- **Dropped:** a STEP-entity rule ("flip when no face was flipped") - wrong
  for open shells, as the issue says; flipping STL/OBJ bodies too - out of
  this issue's scope (those files are the mesh), and a change in what an
  STL import means.
- **Cost to change later:** low. `step_outward.rs` is one call in
  `parse_model_graded_within`; removing it restores the old import.
