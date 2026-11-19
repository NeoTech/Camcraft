## #2598 - Drill and Thread ops take the tool's feed; a feed above it is refused at the post

**What was decided.**

- A new Drill or Thread operation takes its feed from its tool: the
  cutting-data row for the material in force, else the tool's running
  feed. A Drill sets its feed and its plunge feed to that value. A Thread
  sets its edge feed to it and holds its plunge feed at or under it. This
  runs where every new op is seeded (`cutting_rows::seed_feed_for_tool`:
  the ribbon quick-add, MCP `add_operation`, the feature recognizer).
- Giving an op a new tool (`EditOperationLinks`, MCP `set_tool`) seeds the
  same way, inside that command. Undo restores the old feeds.
- A program with a Drill or Thread op that feeds above its tool's feed is
  refused at the post, by name (`operation::feed_rating::problems`, a
  clause of `posting::verification_report`): "D: feeds 300 mm/min, above
  the 115 mm/min that tool DR-0600 states - ...". A Drill is judged on the
  feeds of its cached path (not tapping cycles, which feed at pitch x rpm).
  A Thread is judged on its stated edge and plunge feeds, because its
  posted arcs are scaled to the tool centre.
- Old project files: nothing changes on load. A file saved with the old
  1200 / 300 seed keeps those numbers, and its first post is refused with
  the line above. The user lowers the op's feed (or raises the tool's
  feed). No project-file field or meaning changed, so `PROJECT_VERSION`
  stays.

**Why.** The Prime Directive: the output is correct, or it is visibly
refused. The cause was the shared seed (1200 / 300) and a feed seed that
moved only milling kinds, only for a named material row. A Drill feeds the
whole tool end axially and a thread mill's feed is stated at its edge, so
the tool's feed is the right number for both. A refusal at the post reads
the op as it stands, so it covers old files, later hand edits and a tool
edit, with no change on load. It has the same per-move form as the
machine's feed ceiling, the existing post law for a feed that is too high.

**Considered and dropped.**

- An advisory `verify::Warning` on the toolpath (like `DrillUndersize`):
  it does not hold the post, and an old file's cached path does not carry
  it, so an old file would still post 2-3x over without a word at the post.
- Rewriting old files' feeds on load: it silently changes a stated value.
- Judging milling kinds against the tool's feed: their feed moves with the
  stepover and chip thinning (`cutting::suggest`); the tool's feed is a
  starting point there.
- Making `suggest_cutting_data` / `apply_cutting_data` speak for drills and
  thread mills: the cutting-data table is a milling table. Not needed for
  this fix; a separate enhancement if wanted.
- Re-seeding the ops when the tool's own feed is edited (`ChangeTool`): it
  would overwrite hand-set op feeds. The post clause judges the new value.
- Re-seeding Drill and Thread ops on a material change (`SetWorkMaterial`,
  `SetStockMaterial`, `ApplyCuttingData`, which use
  `cutting_rows::reseeded_params`): these keep #890's milling-only rule.
  If a new material's row states a lower feed, the post clause names the
  op, and the user lowers its feed.

**Cost to change later.** The clause is one function and one line in the
gate; making it advisory is a change of that line. The seed is one call in
`cutting_rows`.
