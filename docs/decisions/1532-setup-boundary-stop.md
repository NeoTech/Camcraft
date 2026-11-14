## #1532 - A program stops between setups by default

**Decided.** A new document-level post option, *Between setups*
(`PostOptions.between_setups`, project key `post_options.between_setups`,
written only when not the default), with four choices:

| Word | Label | What posts |
|---|---|---|
| `pause` (default) | Pause (M0) | a program stop after each setup's last operation, with a comment naming the next setup |
| `optional-stop` | Optional stop (M1) | the same, as an optional stop |
| `continue` | Run straight on | the old program: no stop at a setup change |
| `file-per-setup` | One file per setup | one whole program per setup (the multi-file post), no stop inside a file |

- The stop folds into the setup's last operation's after-action, so each
  dialect writes it where it writes an operator's own stop (spindle and
  coolant already off). A `PARAM SetupChange "<next setup>"` rides beside
  it for the comment. Fanuc lifts Z first (`G91 G28 Z0`, `G90`) and writes
  `(SETUP CHANGE - NEXT: <name>)`; Tormach PathPilot lifts with its
  machine-Z retract (`G53 G0 Z0`) and writes `(MSG, SETUP CHANGE - NEXT:
  <name>)` so the operator sees it on the screen.
- It is independent of *Skip pauses and homes*: that switch mutes the
  operations' own stops for a proven program; a re-clamp is not one of
  them. Running on is its own explicit choice.
- An operation whose own after-action is Go home at a setup change is
  refused by name: the post writes one after-action per gap, and a trip
  home is not a pause. Its own Pause (M0) there merges into the one stop.
- A boundary is per mount (per machine program): the last operation of
  one machine's program gets no stop because another machine runs a
  different setup next.
- The choice is program-level: setups owning their own post options that
  differ in it are refused by the existing "other post options" check.
- A post script that does not declare `afterModes` cannot stop: the
  engine refuses at the setup change by name (once; the generic
  after-operation refusal is not repeated). Tormach PathPilot now declares
  `afterModes` (it reads stop and optional stop; a Go home is refused
  there, since the G30 spot is the operator's preset).
- The one-file post with *One file per setup* and more than one setup is
  refused by name; *Post per setup* (`post_mounts`) writes the files.
- MCP: `post` takes an optional `between_setups` word; it is applied
  through the undoable `SetPostOptions` (or `SetSetupPostOptions` when the
  active setup owns its options) before posting.

**Why.** The issue's own law: with no choice made, the program must never
run from one setup into the next without a stop - the machine would cut
Setup 2's paths into a part still clamped for Setup 1. Pause is the safe
default; M1 is offered because some shops run the optional-stop switch on
for first parts. Lifting Z first keeps the tool clear of the part while
the operator flips it.

**Considered and dropped.**
- Defaulting to one file per setup: safer still, but changes every
  multi-setup user's output files at once and breaks the one-file habit;
  the pause keeps the one file and is just as safe.
- Making the stop obey *Skip pauses and homes*: a proven program still
  needs the part turned over, so muting it there is a gouge by switch.
- A per-setup choice: the gap belongs to two setups at once, so it is a
  program-level choice; per-setup options that disagree are refused.

**Cost to change later.** The word is a project-file key with a default;
adding a choice is additive. Dialects other than Fanuc and Tormach that
declare `afterModes` write the stop without the Z lift and the comment
(follow-up issue filed).
