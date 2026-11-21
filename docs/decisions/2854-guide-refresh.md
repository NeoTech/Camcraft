## #2854 - Describe the shipped features and their limits in the guide

**What was decided.** Update the existing guide pages for the features
listed in #2854. Describe the current merged implementation and its tests.
Explain the holder-reference, DXF-profile and protected-operation format
barriers beside their workflows. Keep the AI section to a short connection
pointer because #1949 owns the full guide.

**Why.** The next documentation build must explain the controls that ship.
Some merged slices store data without making it available for machining:
DXF chain generation still refuses, and plasma recipes do not yet change
operation parameters or output. The guide states these limits explicitly.
It also follows current Report code, whose issue buttons request the
clipboard screenshot; an older PR body described a separate copy button.

**Considered and dropped.** A separate release feature page would duplicate
the guide and split instructions between pages. A complete AI guide would
overlap #1949. New screenshots are unnecessary for these text changes.
No executable behavior, output golden, dependency or process file changes.

**Cost to change later.** Low. These are guide text and coverage tests.
The overlap with #2799 concerns the same files but different sections;
the second PR to merge refreshes its branch.
