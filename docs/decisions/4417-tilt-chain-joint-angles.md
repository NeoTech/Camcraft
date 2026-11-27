## #4417 - G43.4 posts write the joint angles of the machine's tilt chain

**Decided.** The nest of `tilthead.mch` means "A carries B", as
`assets/machines/AGENTS.md` says ("the nest chain is the kinematics"). The
Fanuc and Doosan DNM posts no longer write the stream's pose letters as
they come. A prelude function, `chainAngles(p)` (`post::tilt_chain`), gives
the angles that the machine's A and B rotaries hold for the stream's tool
axis, for the nest order of the machine file. The machine check
(`machine::judge::pose_problems`) judges the same angles through
`machine::tilt_chain::joint_angles`. The two copies of the rule have the
same refusal texts, and a test compares their angles.

The rule:

- No A rotary about X with a B rotary about Y, or a pose that turns at most
  one of the two: the letters are the joint angles (the two products are
  then the same rotation, so no program that turns one rotary changes).
- The outer factor of the tool-axis product is the parent rotary in a head
  chain, the rotary nearest the work in a table chain, and the table in a
  table-and-head machine. B outer: the letters. A outer: solved again.
- A outer: two solutions exist away from B = 90 deg. The first one that both
  travels hold is taken, else the principal one (B in -90..90), which the
  travel check then refuses by name.

**Refused by name** (post and check):

- A and B both turn and the file does not state which rotary carries the
  other (a static mirror with no `parent` and no `turnsPart`).
- A rotary of the pair stated along -X or -Y turns: these posts write the
  angle unnegated.
- C turns as well on an A-outer chain: that pose is not solved.

**Why.** The review of PR #4408 measured up to 27.3 deg between the
programmed tool axis and the axis that the machine reads, and a tool up to
0.37 mm into the model. A post must reproduce the CL axis or refuse.

**Dropped.**

- Refuse every pose that turns both A and B on an A-outer chain. Safe, but
  it refuses the shipped five-axis example on the shipped tilt head.
- Change the stream letters: the stream is machine-free and the sim and the
  dump read it. The joint angles belong to the machine.
- Re-read the nest as "B carries A": the machine file and its AGENTS.md
  say the opposite.

**Cost to change.** The rule is in two files (`machine/tilt_chain.rs` and
`post/tilt_chain.rs`). The solution choice (travel first, then principal)
is one line in each.
