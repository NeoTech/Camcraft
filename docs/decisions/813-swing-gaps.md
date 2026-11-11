## #813 - the index-swing check's follow-ups: an unstated axis vector, old projects

Follow-ups from the review of PR #810 (#804). Sources (behaviour only):
RS-274 / ISO 6983 rotary lettering (A about X, B about Y, C about Z); the
#804 decisions (`804-a-swept-radius.md`).

**A work-turning rotary with no `axis` vector: judge by its letter, refuse
when it has none.**
- **Decided:** the clause reads the line a rotary turns about from its
  stated `axis` vector; with none, from its RS-274 letter. An `A` is about
  X, so a user's table-A file that never states the vector is judged like
  the microARC's. A `B` or `C` is off X and is outside this clause. A
  work-turning rotary with no vector and no RS-274 letter (say `U`)
  refuses every op by name, asking for its `axis`; when a machine holds
  both, that refusal wins.
- **Why:** the post writes a leaning tool axis as an A turn about X
  (`a = atan2(-y, z)`) whatever the file says, so an `A` with no vector
  is still an A about X on the machine. Passing it unjudged was the
  silent pass the Prime Directive forbids. A rotary with neither a vector
  nor a standard letter could be on any line, X included; guessing either
  way could pass a crash, so it refuses.
- **Dropped:** refusing every unstated vector, including a named `A`. That
  makes a user fix a file whose meaning is already fixed by the standard.
  The `rotary_letter` gate's "unstated checks nothing" law was also
  dropped here: it governs which letter a rotary kind posts, where an
  unstated inventory is a missing limit. Here it would be a missing
  safety check.

**Projects saved before #804: a load note, by name.**
- **Decided:** `AxisLimits::turns_part` becomes `Option<bool>`. A parsed
  rotary always states it (`Some(true)` for a table, `Some(false)` for a
  head tilt); linear axes leave it `None`. A rotary read back from a
  project with no `turns_part` key was saved before #804. Opening such a
  project adds a load note (the #342 channel, also logged) naming the
  document's or setup's machine and the rotary, and asking for the machine
  to be reloaded. Only rotaries on or possibly on the X line are named;
  `B`/`C` and stated off-X lines are not. A fresh save writes the key, so
  the note goes away once the machine is reloaded and the project re-saved.
- **Why:** with a plain `bool` a head tilt saved after #804 and a table A
  saved before it serialize the same (the key skipped on `false`), so the
  load couldn't tell them apart. With `Option`, "never stated" is its own
  value. This needs no project version step: the key stays additive.
- **Dropped:**
  - Bumping `PROJECT_VERSION` and noting every v58 rotary: that changes
    every project file's header and conflicts with every open PR that
    bumps it, for the same signal.
  - Refusing the post for an unknown kind: a pre-#804 head tilt (the
    shipped tilt-head) would refuse every op until reloaded. The note
    names it, and a reload clears it.
  - Re-deriving the kind at load: the embedded machine keeps no
    kinematics tree, so it would be a guess.

**Tester kit README: keep the G30 Z at the top.**
- **Decided:** step 4 now says to keep the stored G30 Z at the top of
  travel. The check, like the operator's own check in step 3, assumes Z
  stays at the top while a `G30` turns A.
