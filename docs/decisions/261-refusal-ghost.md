## #261 - a refused generation shows the moves it had built (part 3)

**Decided.** A refusal raised with a path in hand may carry that path as a
display-only *diagnostic ghost*. It rides the carrier part 2 built for the
refusal's spot - `GenError` (new `Ghosted { msg, at, ghost }` variant) ->
`CommandError` (new `MachiningGhost`) -> `State::feedback_ghost` - never a
second channel. The viewport draws it dashed, in the colour-vision palette's
gouge colour dimmed (rapids dimmer still), with no chevrons, under the
refusal's spot cross; it is read only through `State::refusal_ghost`, which
retires it with the refusal sentence (next command, regenerate, draft edit).
It is never stored on the operation, never cached, simulated, verified or
offered to the post: an op with a ghost has no toolpath for every law.

**Adopters.** Only where the path already exists when the refusal is raised,
so no generator computes anything new:
- the ladders' centre-cutting refusal (`approach::refusal` on the finished
  builder): 2D pocket / adaptive / offset, contour (rings and open chains),
  slot (both kinds), bore, circular, and the 3-D ladder (Pocket3D, and
  Multi-Axis Rough, whose stamp keeps the ladder's positions);
- the plunge law's after-the-fact audit (`plunge_law::audit*`): the 3-D
  finishes' and multiaxis kinds' centre-cut guard, Rotary Offset's bar audit,
  the posed audit, and the facer's air-entry fallback - the ghost is the
  whole audited path, beside the spot part 2 already named.

**Not adopted (owed, by design).** Generators that refuse before building
anything carry no ghost (parameter doors, drilling, bore/thread-mill hole
doors, region and tool gates). Rotary Offset's wrap-space ladder drops its
ghost (`GenError::without_ghost`): its moves are in the development, not
where the part is, and mapping them out would mean changing what the
generator computes. A refusal inside a generator that stops early (returns
before `finish()`) has no path to hand back without restructuring it.

**Bounded.** `Ghost::MAX_MOVES` = 100 000 (the overlay's vertex budget is
smaller still); a longer path keeps its first moves and says `truncated`. A
ghost with any non-finite move is dropped, never drawn. Its `Debug` prints
only its size, so logs and test messages never dump a path. A 3+2 tilt maps
the ghost to world through the same `world_path` the real path takes.

**MCP / script.** The `generate` error text adds the ghost's move count
("a partial path of N move(s) was built before the refusal; it is not kept"),
never the moves.

**Dropped.** Caching the partial path on the operation as a second kind of
toolpath (a refused path one step from being posted is the wrong failure
mode); a generic "generate anyway" switch; a separate ghost channel beside
the refusal's spot.

**Cost to change later.** Small: the ghost is one optional payload on the
refusal; dropping it is removing the variant and its two UI reads. Adopting
it in another generator is one `.with_ghost(&path)` at a refusal that holds
a path.
