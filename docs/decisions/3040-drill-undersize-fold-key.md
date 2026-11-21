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
- Key `222`: open PR #2836 moves `UndercutLift` to `222`. The next free key
  is `223`, so the two PRs do not collide when the two merge.
- A test that builds one value of each `Warning` kind and asserts distinct
  keys: dropped for this fix. It does not stop a new kind from being left
  out of the test, and the issue asks for the pin of the one collision.
  An exhaustive key table (an enum of group kinds) is a larger refactor.

**Cost to change later.** None. The key is display-only. The persisted
cache stores each `Warning`, not the fold key.
