## #2724 - A Drill or Thread op is rated by the lower of the material row and the running feed

**What was decided.**

- The feed rating of a Drill or Thread op (`operation::feed_rating::rated_feed`)
  is the lower of two values when the material in force on the op's setup
  is the document's material:
  - the feed of the tool's cutting-data row for that material;
  - the tool's running feed.
- The post clause (`feed_rating::problems`) and the seed of a new or
  retargeted Drill or Thread op (`feed_rating::seed_into`) use the same
  rating. Thus a seeded op is never refused by the check that follows it.
- When the setup's stock states a material that is not the document's
  material, the row of the stock material alone is the rating, as before.
- A running feed that is not a positive, finite number does not lower the
  rating. The row stays the rating then, as before.

**Why.**

- The running feed follows the document's material: `SetWorkMaterial` and
  `ChangeTool` set it, and `cutting_rows::follow_running` does not move a
  row that names the material. When the user lowers the running feed below
  that row, the lowered value is the limit that the user set on the tool.
  Before this change an op at the row feed posted with no refusal.
- The seed takes the same value. A seed of the row feed would make an op
  that the post check then refuses.
- The running feed is stated for the document's material. A stock with its
  own material (#2015) has its own row. The running feed of another
  material is no limit for it: a plain lower-of-two would refuse a legal
  program and seed a feed that is too low.

**Dropped.**

- Lower of the two for the check only, with the seed kept on the row: a new
  op would be seeded at a feed that its own post refuses.
- Lower of the two for every material in force: wrong for a stock that
  states its own material (see above).
- Move the named row with the running feed in `follow_running`: that edits
  the user's cutting-data table without a word, and a later material change
  could not restore the row.
- A change of the existing ops' feeds when the running feed drops: the post
  refuses by name instead, so the user decides (Prime Directive: no silent
  change of a stated value).
