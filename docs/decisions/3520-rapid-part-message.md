## #3520 - Name rapid contact with the part

**What was decided.** The part-solid law emits `Warning::RapidInPart { at }`.
Its message says that the tool meets the part on the named rapid move.
`RapidInStock` keeps its stock and clearance-plane message.
Both findings block the post.
MCP names the new kind `rapid_in_part`.
The UI groups part findings separately from stock findings.
Stock acquittal rules continue to apply only to stock findings.

**Why.** A rapid can meet the part on a vertical lift or drop.
That contact does not establish travel below the clearance plane through stock.
The two findings need different messages and must retain their move locations.

**Considered and dropped.** Changing the stock message would hide the cause
that the stock law establishes.
Keeping one kind would mix the two causes and could let a stock acquittal
remove a part finding.
Changing the collision checks is unnecessary; this change names their result.

**Compatibility and cost to change later.** The new variant uses the existing
snake-case serialization rule.
Old cached `RapidInStock` findings remain readable.
Their original cause is unavailable until verification runs again.
A later change must update the exhaustive warning consumers and their tests.
