## #2033 - 2D rest machining reads the stock of the previous operations

**What was decided**

- The rest group of the 2D Pocket has a mode, `RestSource`: `EnteredTool`
  (the default, the behaviour of #1489) or `PreviousOperations`. The mode is
  one field of `RestParams`. It rides the existing `EditOperationParams`
  command, the project key `rest.source` and the MCP `edit_operation`
  parameter. A file without the key reads as the entered tool.
- The two modes never mix. `rest_pocket` and `rest_plan` refuse a group in
  `PreviousOperations` mode by name. `rest_pocket_previous` refuses a group in
  `EnteredTool` mode by name. No path falls back to the entered tool.
- The stock is the grid that `rest::prior_state` already builds for the 3D
  Pocket: the setup's stock, cut by the cached paths of the earlier
  operations in program order, each with its own tool shape (flat, ball,
  bull-nose, cone). No new stock backend is added (#586 and #637 stay apart).
- The motion crate builds a `RestStock` (the contract of #2031) from the grid:
  one `StockFloor` for each cut depth of the Pocket's level ladder.
  - A column stands at a cut depth when its top is more than 1e-6 mm above it.
    A floor allowance of the earlier operation is material. A wall allowance
    is material, because those columns were not cut.
  - The standing columns grow by one cell diagonal. The grid samples the
    stock, so material thinner than a cell beside a standing column is not
    proved absent.
  - A column with no readable top (NaN) makes the whole pocket uncertain.
    The part of the pocket outside the grid is uncertain. Uncertain stands.
- The rest area is the union of the standing material over all cut depths.
  The grid is a height field, so this union is the reading at the deepest
  cut. A roughing pass that stops above the floor leaves the whole pocket,
  and the path says so in a note.
- The passes are the entered-tool mode's own passes (`emit_rest`): rings,
  offset rings or zigzag rows. Each move below the top of the cut is a feed
  or a plunge. A rest area that is too large costs air cuts, never a rapid
  into stock.
- The stock must be established. Generation refuses by name when an earlier
  operation of the setup has no toolpath, has a stale toolpath, or is
  suppressed with a cut on record. A tilted Pocket is refused at the edit and
  again at generation.
- No earlier cut at all is not a refusal. The stock is then known (it is
  uncut), the whole pocket is cut, and a note says it.
- Invalidation uses the existing program-ahead digest (`rest::prior_digest`),
  which each unposed operation already has in its stamp. A changed path,
  tool, depth or order of an earlier operation makes the rest pass stale.
  The stamp of the Pocket gets one byte (0xD8) only in the new mode.
- A suppressed earlier operation does not change the digest. Two existing or
  new refusals cover it: generation refuses in this mode, and the post gate
  (`posting_suppressed`) refuses a path that was generated against a
  suppressed cut.

**Why**

- The entered tool is no proof. A typed diameter says nothing about the depth
  that the earlier operation reached (#1490 item 3).
- The carried-stock infrastructure already decides the order, the tool
  shapes, the carry across setups and the refusals. A second reading of
  "what was cut before" could disagree with the post.
- One conservative rest area for all levels keeps the proven pocket passes
  and the level ladder unchanged.

**Considered and dropped**

- A separate field on `OpParams::Pocket` for the mode. It changes each Pocket
  literal in the repository. A field of `RestParams` changes eleven.
- A different rest area for each level. The pocket emitter takes one set of
  passes for all levels. It needs a new level walk. The cost of the union is
  air cuts on the upper levels when only the lower levels have material.
- A film tolerance of one cell (as `rest2d` drops its 0.02 mm film). It
  removes a band along each wall from the rest area that the grid cannot
  prove absent. Consequence of the strict reading: in this mode the rest pass
  follows each wall that an earlier tool rode, about 1.5 cells wide.
- Lifting the Adaptive refusal with `rest_adaptive`. The 2D Pocket's Adaptive
  strategy is retired (`POCKET_ADAPTIVE_RETIRED`), so no door leads to it
  from this operation. `rest_stock_from_columns` is public for a later
  consumer.
- A stamp term for suppression of an earlier operation. The post gate and
  the generation refusal already hold the result.
- To leave a suppressed earlier cut out of the simulation and continue. It
  needs a second walk beside `rest::ahead_of`, which keeps suppressed cuts by
  design. The refusal is simpler and says what to do.

**Cost to change later**

- The per-level rest area needs a level-aware emitter; `RestStock` already
  holds one floor for each level, so the stock side does not change.
- When the stock moves to the field engine (#586, #637), `Columns` already
  reads the two engines. The NaN rule matters more there: a field column
  whose march cannot finish answers NaN.
