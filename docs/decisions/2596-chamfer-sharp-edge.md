## #2596 - a no-pick 2D Chamfer leaves the model's sharp edges alone when the model carries a chamfer

**Decided.** A 2D Chamfer with no picks (its source is the whole model)
reads the model's own edges at the top plane:

- If at least one top edge stands over a modelled bevel (a modelled
  chamfer or round), the op rides only the bevelled edges. A sharp edge
  beside them (the pocket rim of #2596) is sharp by design and is not
  cut. To break it, the user picks it.
- If no top edge carries a bevel (a sharp brick, a sharp pocket block),
  the op breaks every top edge as before. This keeps the #303 law: the
  default chamfer on a finished sharp part posts with its declared break.

- If a bevelled edge would lose its nesting when the sharp edge around
  it is left out (a modelled chamfer on a pocket in a sharp plate), the
  op refuses by name and asks for picks. Alone, the region would read the
  rim as a boss, and the ride would travel against its material (#1555).
  The refusal is safer than a ride in the wrong direction.

Verify side: on a model that carries a modelled bevel at any declared
edge (`camcraft_sim::stock_sim::models_a_bevel`), a no-pick chamfer gets
no edge-break credit at all. Its ride is judged against the model, so a
cut into a sharp edge, or a bevel cut wider than the modelled one, is a
gouge. This also catches a ride cached by an older build.

**Why.** The model is the finished part. Where the designer modelled
some chamfers and left other edges sharp, the sharp edges are intent,
and a cut there removes model material (Prime Directive). Where the
model has no chamfer at all, it states no intent, and #303 accepted the
no-pick op as "break every edge".

**Dropped.**
- Refuse every no-pick chamfer on a model with no modelled bevel (the
  strict reading of the issue's "Done means"). It reverses the #303 law
  and its tests (`edge_break_gate.rs`, the L-boss and pocket tests of
  `chamfer_modelled_edge.rs`). That needs the maintainer's word; filed as
  a follow-up.
- An operation parameter "auto edges: modelled only / every top edge".
  The pick list already gives the user the second choice per edge, and a
  new parameter touches the project file, the UI and MCP.
- A user-visible note that lists the edges a no-pick op skipped. The
  op shows only its toolpath; filed as a follow-up.

**Cost to change.** The rule is in `chamfer_ride::ride_loops` (one
fallback line) and in `stock_sim_breaks::program_breaks` (one guard).
