## #3920 - plunge_depth_tab pins the pecked tab window dives

**What was decided**

`plunge_depth_tab` pinned the three tab window dives that the verifier of
#3679 named in the `2d/contour-tabs` catalog case. PR #3866 (#3678) merged
first and made each of those dives peck, so the verifier names none, and
main went red after PR #3847 merged. The test header said that the test
changes on purpose when #3678 lands.

The test now pins two things:

- The verifier names no `PlungeTooDeep` finding for the case.
- The path pecks: tab windows 1 to 3 each dive Z4, Z2, Z0 from the Z6
  stock top, each feed at most the 2 mm stepdown below the depth before it.

**Why**

The generator fix is correct: each peck feeds one stepdown. A test that
only asserts "no finding" also passes when the verifier stops seeing the
dives. The second assertion reads the path itself, so it fails if the
pecks go or if the verifier misses them.

**Considered and dropped**

- Only assert no finding. Dropped: see above.
- Delete the test. Dropped: it is the only end-to-end pin of the #3679
  verifier on the reporter's case.

**Cost to change later**

Low. One test file.
