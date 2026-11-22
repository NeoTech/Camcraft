## #3040 - DrillUndersize gets its own fold key in the Warnings pane

**What was decided.** `fold_warnings_located` (`crates/ui/src/operations.rs`)
groups `Warning::DrillUndersize` under key `223`. Before, it used `216`, the
key of `Warning::AvoidHit` with `FixturePart::Shank` (the avoid range is
`216..=220`). A new pin folds one `DrillUndersize` with one `AvoidHit` for
each `FixturePart` and asserts one line for each.

**Why.** A group prints only the line of its first warning. When the
undersize hole came first, the blocking "Do not touch" shank hit was folded
into the advisory line as "repeated on 1 more move", and the pane did not
show it.

**Considered and dropped.**
- Key `222` for `DrillUndersize`: it was kept for `UndercutLift`, so the
  next free key `223` is used.
- A test that builds one value of each `Warning` kind and asserts distinct
  keys: dropped for this fix. It does not stop a new kind from being left
  out of the test, and the issue asks for the pin of the one collision.
  An exhaustive key table (an enum of group kinds) is a larger refactor.

**Addendum (merge of main, 2026-11-21).** Main added `UndercutLift`
(#1444, commit 5452810e5) with the key `217`. That is the key of an
`AvoidHit` on `FixturePart::Holder`, the same fault as #3040. The merge
gives `UndercutLift` the key `222`. The new pin
`an_undercut_lift_never_hides_an_avoid_hit` folds one `UndercutLift`, one
`DrillUndersize` and one `AvoidHit` for each part, and asserts one line for
each. The note above said that open PR #2836 moves `UndercutLift` to `222`.
That was not correct: #2836 adds the key `217`, as main does.
Dropped: keep `217` and leave the fault to a new issue. The merged tree
then has a known way to hide a "Do not touch" hit.

**Cost to change later.** None. The key is display-only. The persisted
cache stores each `Warning`, not the fold key.
