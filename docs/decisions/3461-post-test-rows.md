## #3461 - Per-post test rows come from the registry and the post manifest

**What was decided.** This PR does item 2 of #3461 only. Item 2 is the
conformance and comment-escape rows. Items 1, 3 and 4 stay open on the
issue (`Refs #3461`, not `Fixes`).

- `crates/app/tests/conformance.rs` has no table with one row for each
  post. The rows come from `posting::BUILTINS` and the four family lists
  of `posting.rs` (`LATHE_POSTS`, `PLASMA_POSTS`, `WIRE_POSTS`,
  `SINKER_POSTS`). A milling dialect is each registry entry that no
  family lists, except Klartext (its machine needs facts that the
  generic VMC does not have). Fanuc's goldens stay at the top of
  `golden/nc/`, and each other mill has a subdirectory with its stem as
  the name.
- The extension check reads each post's own `extension = "..."`
  manifest line, which is the row source next to the post. Every post
  must declare `nc`, except the posts in a short sorted table `NOT_NC`.
  This table holds the seven posts whose control reads a different file
  type, each with its citation.
- The registry check now asserts a partition: the names are unique,
  each entry has a shipped file, each family name is in `BUILTINS`, each
  entry is in exactly one family, and Fanuc is entry 0.
- `crates/post/tests/comment_escape.rs` counts the `.cps` files that it
  finds and checks that each one gives a judged result. It does not
  compare with a fixed number.

**Why.** Before this change, two new-post branches conflicted in
`conformance.rs` (`POSTS` and `EXTENSIONS` both end at the same line).
A trial merge with `git merge-tree` of two simulated branches off
470d1a27c showed this conflict. In `comment_escape.rs` both branches
changed `26` to `27`. Git merged the two identical changes with no
conflict, and the result was a count one too low, so main became red
after the second merge. With this change, the same row edit finds no
table in either file, so a new post changes neither file.

Item 2 was chosen first because it has no behaviour change. It touches
only test files, and only one open PR (#2646) touches these tables.

**Dropped.**
- Stacking on #2646 (Hurco post). The rule says to stack on an open PR
  that changes the same lines. But #2646 is a draft `feat` with
  `needs-rework` and `conflicts`, and it merges only at a cut. A stack
  would hold this tooling fix until the cut. When #2646 is refreshed,
  it drops its `POSTS` / `EXTENSIONS` rows and its count change.
- A full name list in the registry check. The old test spelled out every
  name, so a new post edited it. Each post's own suite (`script_*`,
  `sinker_burn`, `wire_post` and others) already uses its registry name.
  A rename fails there.
- A check that each file in `assets/posts/` is in `BUILTINS`.
  `waterjet-iso.cps` ships for the engine but is not in the app
  registry, so this check would fail. That design question belongs to
  item 1.
- Machine kind for each post in `comment_escape.rs` from the `serves`
  manifest line. Most shipped posts do not declare `serves`, so the
  stem rules stay.
- Per-post behaviour rows (`REFUSED`, the synth-stream `match`) stay.
  A new post adds a row there only when it refuses a golden or cannot
  tap.

**Cost to change later.** Low. The derived functions are small and are
in one test file. If item 1 moves the registry into each post (a
manifest name, or discovery from `assets/posts/`), `registry()` reads
that source instead.
