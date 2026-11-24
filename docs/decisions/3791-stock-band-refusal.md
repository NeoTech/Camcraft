## #3791 - Name the stock band when no cutter position fits

**What was decided.** Enrich the existing narrow-tool refusal in the app's
generation job for stock-source pocket-family operations with kept parts
and a collected stock-minus-part region. Keep the generator's refusal,
error location and path behavior. Reapply the operation's area controls
only to the diagnostic. Name the stock band only when the whole controlled
region erodes to nothing and the existing `unreached_pieces` band check
agrees.

Name the band only when the area surrounds kept material (PR #3805
review). A piece of the stock-minus-part region is a band when its outer
loop is a loop of the stock outline and material stands inside that loop.
A closed pocket in a part whose outline equals the stock outline, and a
pocket around an island, are bounded by part walls. They keep the
generator's pocket refusal. A strip of stock beside a part that is flush
with the other stock sides also keeps it: that strip is not around the
part.

Report the minimum side margin only for a rectangular outer boundary and
rectangular inner boundary, each checked against its bounding-box area.
Measure the controlled ring rather than the original stock. Curved or
other bands receive the tool size, stock to leave and corrective advice
without an unsupported width claim. Use the document's display units and
name stock to leave per wall.

**Why.** The motion generator sees a region without its stock-source
meaning. Its generic pocket refusal happens before the successful-path
band note can run. The app already holds the context needed to say that
the uncut area surrounds the part. This changes the explanation while
preserving the refusal.

**Considered and dropped.** Changing the generator's fit law could change
which jobs produce paths, which this wording fix does not require.
Guessing arbitrary band widths from their bounding boxes would report
misleading dimensions for curved, concave or multiple-island regions.
Changing true-pocket messages or unrelated refusals would expand the
scope beyond the reported stock-source failure.

**Cost to change later.** A general band-width metric could replace the
rectangular reader without changing generation or refusal behavior.
