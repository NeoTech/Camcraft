## #3176 - Import empty layout blocks and object dictionaries

**Decision.** Accept an empty block definition for model space or a reserved paper space layout. Check each supplied owner against its named block record. Accept a dictionary entry when its target is another dictionary and its ownership chain ends at a root.

**Reason.** These records can accompany model-space geometry without adding a drawable object. An exporter can omit an owner field or list an empty dictionary only through its parent pointer. The reader can check both forms with a bounded, iterative traversal.

**Dropped.** Skipping either section as a whole would hide block geometry, external references or drawing objects. Checking only familiar dictionary names would miss equivalent metadata from other exporters.

**Cost to change.** Supporting another block or object type requires its own field checks and tests for its effect on imported geometry.
