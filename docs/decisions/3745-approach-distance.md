## #3745 - The approach distance outside the stock, default 2.0 mm

**What was decided:** Add `approach_distance` (mm) to the operation's
`links.approach`. The default is 2.0 mm (`DEFAULT_APPROACH_DISTANCE`). It is
the clear air between the tool's edge and the stock boundary where the entry
of an Inside or Outside pass goes down. Zero is legal. A negative or
non-finite value is refused when the command is made (`Approach::validate`,
through `Links::validate` and `EditOperationParams`). The project file writes
the key only when the value is not the default, so the format version does
not change and earlier files load the default. The digest adds bytes only
for a value that is not the default, so the digest of each earlier operation
stays the same. The Lead in / Lead out page shows the row "Approach
distance" on 2D Offset, 2D Pocket and 2D Adaptive when Pass starts at is
Inside or Outside. MCP sets it through `links.approach.approach_distance`.

**Why:** The maintainer asked for an approach that starts "outside the stock
boundary with a given value". 2.0 mm is the same clearance that
`DEFAULT_PLUNGE_HEIGHT` keeps above the material. It is a known value in the
project, it is clear of the stock edge with a typical stock tolerance, and it
adds little air cut time. The serde default law (absent is the default) keeps
old files and their digests unchanged, so no migration is necessary and the
change is not breaking.

**Considered and dropped:**
- A default of the tool radius or the tool diameter: the issue pin measures
  "the tool radius plus the approach distance", so the tool radius is
  already in the rule. A second radius would double it.
- A default of 0: the tool edge then touches the stock boundary, and a stock
  that is a little larger than its model gets a cut at the descent.
- A project format version step: not necessary, because the field is
  optional and absent means the default.
- The row on 3D Pocket, 3D Finish, Bore and Circular: the entry sub-issue
  #3746 decides which kinds read the value. The row can follow there.

**Cost to change later:** Change `DEFAULT_APPROACH_DISTANCE`. A file that
stored the old default writes no key, so it reads the new default. Its
digest changes only if the default changes.
