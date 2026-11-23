## #3477 - A script without the `uprightReturn` declaration gets no upright section after a tilt

**What was decided.** A new manifest flag, `uprightReturn = true`, says
that the script retracts to a safe height and commands the rotaries to
zero at the start of each section that follows a tilted section. A new
engine pass, `post::upright_after_tilt`, reads the stream before any hook
runs. When the script does not declare the flag, the pass finds the first
section that:

1. comes after a section that left A, B or C away from zero (the modal
   angles of the last move that stated them), and
2. has a first move that states no A/B/C.

The engine then adds one counted warning that names the two operations
and the flag. A warning blocks the post in the app, as every engine
refusal by warning does. The shipped posts that write tilted moves
(fanuc, doosan-dnm, mach4, tormach) declare the flag. Each one has a
posting test of the real order in `crates/post/tests/upright_after_tilt.rs`.
The Mach4 AC trunnion fixture moved to `crates/post/tests/ac_trunnion_support/`
so that this file can pin Mach4 too.

**Why.** The angle words are modal. `post::dump` writes no angle on a
move without a leaning axis, so only the script can command the return.
The shipped posts do it (#3466), but no law makes a script of the user
do it. A refusal by name is the Prime Directive's answer: no silent cut
at the old lean.

**Considered and dropped.**

- *The name `uprightAfterTilt`* (the issue's example). PR #3466 already
  defines a script function `uprightAfterTilt()` in fanuc.cps and
  doosan-dnm.cps. A top-level `uprightAfterTilt = true` replaces that
  function, and the post fails with "not a callable function". The flag
  is `uprightReturn` instead.
- *A hard `PostError::Refused` before the hooks.* That changes the first
  refusal of programs that other laws already refuse. The warning is
  added after the other warnings, so their order stays the same, and the
  post is still blocked.
- *Refuse each section after a tilted section* (also a tilted one). A
  section whose first move states its angles commands the pose itself.
  The order of that turn against the first X/Y traverse is the
  first-traverse law (#3100, #3300), not this one.
- *A C angle left by C-axis live-tool work.* The live-tool law (#33) is
  section-scoped: the script disengages C at the section's end. A
  section with `PARAM LiveTool` does not leave C as a tilt.
- *A text check of the shipped posts' return in the engine.* The engine
  cannot read what a script writes before it runs. The posting tests pin
  the real order for each shipped post that declares the flag, as for
  `rotariesBeforeTraverse`.
