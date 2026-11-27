## #4076 - Sodick LN2W post: control-side wire offset (G41 / G42 H, G40 on the lead-out)

**Decided.** The Sodick LN2W post (`assets/posts/sodick-ln2w.cps`) writes the
control's wire offset when the stream asks for it with a `COMP` record
("control" or "wear" compensation):

- `G41` / `G42 Hnnn` on its own line, after the pass's C word and before the
  lead-in move. This is the own-line form that the owner of an AQ600L (LN2W)
  called the easier one.
- H is the pass number from `H001`: the first pass is `H001`, the first skim
  `H002`, and so on.
- The first pass must be on `C001`: the owner states that `C001` always goes
  with `H001`. Another condition on the first pass is refused by name.
- A later pass writes the condition that the operation states (its
  `PARAM EdmCondition`). The post never computes `C00n` from the pass number.
- `G40` goes on the lead-out move, in the same block as its `G01`.
- `G29` is not written (the owner: optional, a restart mark).
- A comment line before each `G41` / `G42` names the register, its condition
  and the radius that the stream states, and tells the operator to set the
  register at the control.

The law that is not dialect-specific is a shared prelude helper,
`crates/post/src/post/wire_offset.rs` (`wireOffsetNew`, `wireOffsetRecord`,
`wireOffsetMove`, `wireOffsetEnd`), as the owner asked on #4076 ("keep the
offset logic in a shared helper that other wire dialects can call"):

- a lead-in shorter than the offset radius is refused by name;
- a lead-out shorter than the offset radius, or with no length, is refused by
  name. The owner stated the length rule for the lead-in only. The lead-out
  gets the same rule (review of PR #4139): on a `G40` block that is shorter
  than the radius, the control moves the wire centre onto the contour point
  and the kerf cuts into the part. If the owner permits a shorter lead-out,
  only the length test of the lead-out changes;
- the offset engages only on a straight lead-in and cancels only on a straight
  lead-out (an arc is refused by name);
- a change of side while the offset is on is refused;
- a section that ends with the offset on is refused.

**Why.** The owner's statements on #4076 (2026-11-25) are the design. The
refusals keep the Prime Directive: a register of a different condition, a
too-short lead-in or an offset left modal would cut in the wrong place.

**Considered and dropped.**

- Writing `C001` on the first pass whatever the operation states. Dropped:
  the post must not invent a condition. It refuses instead, and the operation
  names `C001`.
- Allowing the `C000` approach option with an offset. Dropped: `C000` would
  cut the lead-in with the register of `C001`. The two together are refused.
- Keeping the header line "WIRE-CENTRE PATH - NO WIRE OFFSET G41 G42 IS
  USED" and the section notes "KERF IN THE PATH" / "KERF GAP ... COMPARE WITH
  THE CONTROL ROW" in an offset program. Dropped after the review of PR
  #4139: the program stated two opposite things about the offset. The engine
  now publishes `programOffsets` before `onOpen` (one entry for each section:
  `"control"`, `"wear"` or `""`), as it does with `programBounds`. The post
  writes the header and the section notes from it. A program without a `COMP`
  record stays byte-identical. A wear section keeps its kerf notes, because
  its path is the wire-centre path.
- A per-program flag instead of a per-section list. Dropped: a wire-centre
  section in a program that also has an offset section must keep its "KERF IN
  THE PATH" note.
- App and generator work (a Wire Contour compensation mode, a part-contour
  path, and the wire laws judging it). Not in this change: the app's wire
  stream still states compensation off, so the app goldens of the square and
  the 2-axis part do not change. The post side is pinned on synthetic
  streams. The app side is a follow-up issue.

**Cost to change later.** Low. The words are in one post; the law is one
prelude constant that no other post calls yet. A dialect with a different
register format or G41 placement calls the same helper and writes its own
words.
