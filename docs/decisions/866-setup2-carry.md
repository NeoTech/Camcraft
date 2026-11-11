## #866 - Setup2 "fails" to reuse Setup1's stock (#872, #873, #874)

**What was decided.**
- The carry refusal the reporter hit is correct by design, not a frame
  bug: Setup2 held the part turned 90° about X *relative to Setup1*, a
  side-on turn the column lattice cannot re-index. The composition
  (`transform_between = A_to * A_from^-1`) already reads the relative
  turn; a test now pins that two setups turned alike carry. Lifting the
  limit is filed as #874 (effort:max, after the field engine #637).
- The "35 s freeze" was the user browsing in the modal file dialog,
  which runs inside the frame's "commands" phase. The dialog seam now
  times every modal dialog and the frame watch takes that time off the
  frame (#872). Making the dialogs asynchronous was not taken: the modal
  wait is ordinary platform behaviour and the misreport was the bug.
- Wording (#873): "furniture" leaves the Settings page; the carry
  refusal names the turn between the setups and the way out. The named
  constant `CARRY_TURN_REFUSAL` stays (tests and readers match on it);
  only its words changed.

**Why.** The Prime Directive keeps refusing a re-sample rather than
guessing; a refusal a user cannot act on, and a log that blames the app
for the user's own think time, both cost trust without adding safety.

**Considered and dropped.** Defaulting an unpicked, refused carry to
"Its own stock" silently - dropped: the stock source is a user's choice
and a silent switch would hide that the earlier setup's cuts are not
simulated. Excusing whole frames that held a dialog - dropped: work done
in the same frame beyond the wait must still be reported.

**Cost to change later.** Small: the wording lives in two constants and
one format string; the excuse is one call in `frame.rs`.
