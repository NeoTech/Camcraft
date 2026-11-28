## #4088 - The machine file states the last preset row

**What was decided.**

- A new optional key `controller.work_offset_max` states the highest work
  offset that the control holds. On a Heidenhain control this is the last
  row of the preset table. It must be a whole number from 1 to 255. The
  machine reader and the raw post engine refuse other values by name.
- The key is an installation fact, the same as `tooling.max_tool_number`
  (#4087). The mirror carries it.
- The post engine refuses a `PARAM WorkOffset` above the key, by setup
  name, before any script hook runs. This applies to each post, not only
  to the two Heidenhain posts.
- `fehlmann-picomax55.mch` states `"work_offset_max": 5`. The source is
  the 2008 delivery CD: `PRESET.PR` has rows 0..5
  (`docs/research/heidenhain/picomax-55-review.md`, defect 14). The machine
  description tells the shop to check the table at the control.
- A machine without the key keeps the post's own limit. A key above the
  post's own limit does not raise that limit.

**Why.**

- The row count is a fact of the installation, not of the dialect. The
  ISO Heidenhain post also serves other controls with other table sizes.
- The engine already refuses an offset above the script's own
  `workOffsetMax` by setup name. The machine limit uses the same place and
  the same message form.
- The refusal is a hard error, not a counted warning. The Klartext post
  throws its own error ("missing or unsupported preset") when the engine
  holds back the offset. That error hid the counted warning, so the
  operator did not see the machine limit.
- The Picomax value is 5 and not 6. A preset that does not exist stops
  the control with an alarm. A row that exists but is refused here has a
  workaround: raise the key in the machine file.

**Considered and dropped.**

- Refuse in each post script (`machine.workOffsetMax`), as #4087 does for
  the tool number. Each post would need its own check, and a new post
  could forget it. The engine check covers all posts.
- Leave the Picomax file without the key until the row count is checked at
  the control. Then the bug stays open on the one machine that has it.
  The issue's first "Done means" item (the check at the control) stays
  with the maintainer.
- Limit the setup's work offset list in the UI to the machine maximum.
  That is display work. The post refusal blocks the wrong program. A
  follow-up can do the display.

**Cost to change later.** The key is optional and is absent from all other
machine files. A rename touches the reader, the mirror, the engine check
and one test file.
