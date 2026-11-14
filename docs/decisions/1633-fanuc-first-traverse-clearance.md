## #1633 - the Fanuc-family mill posts state the clearance Z before the first X/Y traverse

**Decided.** `fanuc.cps`, `haas.cps`, `mazak.cps`, `doosan-dnm.cps`,
`sinumerik.cps` and `heidenhain.cps` take the #1614 (PR #1618) / #1616
(PR #1632) rule: at the program's start and after every tool change, the
first motion is preceded by a Z-only rapid to the section's
`ClearancePlane`, else its `RetractPlane`, never below the move's own Z,
then X/Y alone at that height, then the move as the stream states it. A
stream that states neither plane is refused by name with the #1618/#1632
wording ("first X/Y traverse with no clearance height stated (no
ClearancePlane or RetractPlane) - the post will not rapid X/Y at an unknown
height"), prefixed by the operation name where the post knows it. Each
dialect writes its own rapid: `G00 Z..` (Fanuc, Mazak, Doosan), `G0 Z..`
(Haas, 840D), and on the TNC `G00 G40 G90 Z+..*` - the manual's
positioning form, because the section's first motion block carries G40
G90 there.

The first motion of every kind is covered:
- a rapid: Z, then X/Y (left out when the rapid is already at that
  height - it is then the traverse itself), then the rapid;
- a feed, an arc, a probing move: Z, X/Y at the stream's start, then a
  rapid down to the start Z when it is lower, then the move;
- a canned cycle (the TNC's cycle call too): Z, X/Y over the hole, then
  the cycle (which would otherwise position X/Y at whatever height it was
  read).

**The frame (G43.4 and rotary paths): correct in the frame, not refused.**
The rapids are written after the record's own `tilt()`:
- Fanuc and Doosan with a tilted first record arm G43.4 first (that is
  where `tilt()` writes it), so the Z word is the tip's Z in the work
  offset - the frame the stream states its planes and its words in. The
  X/Y traverse follows under the same arm, and the rotaries turn only on
  the record itself, at its target, as before. No rotary word rides the
  inserted blocks.
- Doosan's indexing path (a 4th axis with no TCP) writes them in G43,
  the frame of the record's own words, with the table still clamped; the
  index (M11, the move, M10) is the record's, as before.
- Haas, Mazak, 840D and the TNC refuse tilted records (`tiltOk`), so the
  rapids are always in the upright frame.

**After the refusal, the move is not written (review of PR #1636).**
`xyFirst` returns false when it refuses, and every caller (`onRapid`,
`onLinear`, `onCircular`, `onCycle`, `onProbeMove`, `onProbeGuard`; the
TNC's four) then returns without writing its move. X/Y stay unknown, so
every later motion of that section is refused the same way (the warning is
counted once): the rest of the section is refused cleanly rather than
skipping one move and posting the next ones from a position the program
never reached. No combined `G00 X.. Y.. Z..` from an unknown height is in
the program text. Chosen over "skip only the first move" because the later
blocks would then start from a position no block put the tool at (a cycle
read unseated, a feed from nowhere). Blocking the post end to end on any
warning stays the callers' job: the app paths refuse to write on a warning
(`crates/app/src/posting.rs:588`, `posting_mounts.rs:248`,
`posting_per_op.rs:168`); the `post` CLI (`crates/post/src/bin/post.rs:163`)
writes the program and prints the warnings, as its recorded `.refusal`
cases document - which is why the post itself now leaves the move out.

**Taken on trust (not exercised on any control).**
- **The clearance plane over the tool-change X/Y.** The Z-only rapid is
  made at whatever X/Y the tool change left the spindle at, before the
  X/Y traverse. It trusts the stated clearance plane (else the retract
  plane) to clear everything under that X/Y - vise, fixture, tool rack,
  rotary table. The post cannot know what is there. Against main's one
  combined diagonal, this is safer when the tip stood below the part top
  (the #1614 case: the descent is shorter and vertical, the traverse is
  above the part), and it is not safer when something under the
  tool-change X/Y is taller than the stated clearance: the vertical
  descent meets it.
- **G43.4 with an unknown rotary pose (fanuc, doosan-dnm).** The Z and
  X/Y lines are written after the arm, with the rotaries still at the
  pose the previous section left (or wherever they stood at the program's
  start). Under TCP the Z word should be the tip's Z in the work frame;
  no control has run it, and the tool body at that pose is not judged by
  the inserted lines.
- **Doosan's index path.** Z is commanded in G43 (machine-Z length
  compensation) with the table clamped at its previous index; whether the
  stated clearance clears the part at that index is the clearance plane's
  promise, not checked by the post.

**Why.** The lift before the change is not proof of clearance: a long tool
on a high work offset can stand below the part top after `G43 H..` (the
#1614 PathPilot Hub reading). A combined `G00 X20 Y10 Z-3` from there is a
straight line through the part. Stating the height makes a setup that
cannot clear stop at the control's Z limit instead. The X/Y-alone line
(not only the Z line) keeps the traverse at the stated height: with the Z
line alone, the diagonal from clearance to a first target below the stock
top would still cross the part. Same shape and wording as #1618 and #1632,
so the posts read alike.

**Considered and dropped.**
- Inserting the Z line alone before the combined rapid (the smallest
  diff): dropped for the diagonal above.
- Refusing a tilted first record by name when the rotary pose is not
  known: dropped - every rapid of a real rotary program states its pose
  (`rotary_verify.rs`), so that would refuse every 5-axis program at its
  start. Under G43.4 the Z word is in the stream's own frame; that is the
  frame the clearance plane is stated in.
- Writing the Z rapid before the G43.4 arm (in G43 with the rotaries at
  an unknown pose): dropped - G43 Z is machine-Z length compensation, so
  with a tilted head or table the tip would not be at the stated height.
- Editing `assets/posts/AGENTS.md`: #1618 adds the "first traverse states
  its height" bullet; it is widened to these posts after #1618 merges.

**Test fixtures.** Hand-written test streams that stated no plane now
state one (as `post::dump` does for every real section);
`crates/app/tests/common/legacy_program.rs` carries #1618's identical hunk
(the legacy golden's START height as the clearance). The macro-B reader
of `inverted_probe` and the path reader of `inverted_equivalence` learned
that a Z-only move before X/Y are known is the stated height, not a CAM
motion.

**Cost to change.** One function per post (`xyFirst`/`traverseZ`), its six
call sites, the pinning suite
`crates/post/tests/first_traverse_clearance_fanuc_family.rs`, and the
goldens that gained the lines (each checked by script: only inserted
`Z`/`X Y`/`Z` rapids, at the first motion after a tool change).
