## #4087 - The machine file states the highest tool number of the tool table

**What was decided.**

- A new optional key `tooling.max_tool_number` states the highest tool
  number that the control's tool table holds. It must be a whole number of
  1 or more. The machine reader and the raw post engine refuse other values
  by name.
- `fehlmann-picomax55.mch` states `"max_tool_number": 254`. The source is
  the machine parameter file (MP 7260 = 254) and the programming manual
  (tool numbers 0..254, p. 2-16), as the issue quotes them.
- The key is an installation fact, the same as `tooling.tool_change`. The
  mirror carries it, and scripts read it as `machine.toolNumberMax`.
- `heidenhain-klartext.cps` refuses a higher tool number with a script
  error. `heidenhain.cps` refuses it with a counted warning, which is how
  it refuses a spindle speed above the machine maximum.
- A machine that does not state the key keeps today's behaviour. Klartext
  keeps its own limit of 32767. The ISO post has no limit.

**Why.** A tool number above 254 passed CamCraft and stopped the Picomax
with an alarm. The limit belongs to the installation (the tool table size
is a machine parameter), not to the dialect. So the machine file states it,
and each post reads it.

**Considered and dropped.**

- Reuse `tooling.number_of_tools`. That key counts the distinct tools that
  a changer holds (`check_tool_count`). A tool table of 254 lines on a
  manual-change machine is a different fact. Using it would also refuse a
  program with more than 254 distinct tools for the wrong reason.
- Put 254 into the posts. The Klartext post already refuses other
  installations, but the ISO post serves other Heidenhain controls with
  other tool table sizes.
- Show the key on the machine sheet and in the machine editor. That is UI
  work. It is not needed to refuse the wrong program.

**Cost to change later.** The key is optional and absent from all other
machine files. A rename touches the reader, the mirror, the two posts and
one test file.
