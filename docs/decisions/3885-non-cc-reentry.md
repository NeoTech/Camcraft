## #3885 - A 3D Pocket pass restart with a tool that is not centre-cutting: the core law decides a refusal

**Decided.** The full-disc test of #3670 stays the first test for a pass
restart in a 3D Pocket level, for each tool. For a tool that is not
centre-cutting, the M11.3 core law (`plunge_law::into_solid`) decides if
a refusal is possible:

1. The full disc is proved clear: a straight plunge, as before.
2. The core is not cleared: the M11.3 fallback (helix, then the ramp
   along the pass, else the named refusal). This is the law before #3857.
3. The core is cleared, but the outer disc is not proved clear: the M11.3
   fallback runs on a copy of the path. The copy is kept if it posts a
   helix or a ramp, has no refusal, and is clean (no plunge into uncut
   stock, no crossing of a posted move). Otherwise the restart is the
   straight plunge that the core law permits, with a note that names it.

**Why.** #3857 applied the full-disc test to all tools. For a tool that is
not centre-cutting, a restart over a cleared core then went to the M11.3
fallback. Where no helix or ramp fits, that fallback refuses the
operation. Before #3857 the same restart was a legal plunge: the core law
proves that the uncut stub of the tool sits over removed material, and
the outer end edges of the tool cut. So #3857 refused a legal program
(P3), and its statement that "the core law stays unchanged" was not true.
The full-disc preference is still correct: a helix or a ramp into the
outer stock is better than a plunge, and a centre-cutting tool gets the
same order (entry, ramp, named plunge). Now the two tool kinds agree, and
only an uncut core can refuse.

**Dropped.**

- Return to the core-only law for this tool (plunge whenever the core is
  cleared). This is the smallest change, but a tool that is not
  centre-cutting then plunges into outer stock where a helix or a ramp
  fits. A centre-cutting tool in the same place takes the entry.
- Keep the refusal. A refusal of a program that the M11.3 law permits is
  wrong behaviour, and the user has no setting that avoids it.

**Cost to change later.** Small. The policy is one branch in
`crates/motion/src/machining3d/reentry.rs::reenter`. It is pinned by three
unit tests in that file (`a_non_centre_cutting_restart_*`).
