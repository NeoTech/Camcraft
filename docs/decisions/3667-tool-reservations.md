## #3667 - Tool reservations: the tool window's next number and the full-probe-pocket refusal

**What was decided**

1. The tool window compares a new tool's typed number with the number
   that `AddTool` gives. A new function, `tool_numbers::next_free_for`,
   calls the same allocation as `AddTool` (`tool_reservations::allocate`).
   It skips the reserved pockets of the active machine for a cutter, and
   it gives a probe a free reserved probe pocket. Before, the window used
   `next_free`, which ignores reservations. With T1 reserved for a probe,
   a typed 1 queued no `SetToolNumber`, so the new tool became T2 with
   no message. Now a typed 1 queues `SetToolNumber`, and that command
   refuses by name.
2. When each reserved probe pocket holds a tool, a new probe is still
   refused. The refusal now names the pockets and tells the user what to
   do:

       tool 'Probe 2': the probe pockets of <model> (T1) all hold a tool.
       Delete or renumber the tool in one of these pockets, or reserve
       one more probe pocket in the machine file

3. A refusal names a `tool_setter` reservation as "tool setter". This
   applies to the app refusals and to the post check of the tool stream.
   The word "probe" does not change, so the tests and agents that match
   on it still match.

**Why**

Item 1: the window and the command must agree on the number. Items 2
and 3: the issue asks for a refusal that says what to do, and for a
plain name in place of the file word.

**Considered and dropped**

- Put a second probe in a free ordinary pocket. Dropped: the machine
  file reserves probe pockets so that the control and its macros find
  the probe there. A probe in another pocket can break those macros.
  The refusal is the safe result; the user decides.
- Show the next free number as a hint in the window. Dropped: the
  window shows no such hint today. The issue is about the compare.

**Cost to change later**

Low. The allocation is in one function, and the words are in
`tool_reservations.rs` and `post/src/machine/reservations.rs`. The tests
`next_free_for_a_new_tool_matches_add_tool`,
`a_probe_with_every_probe_pocket_full_says_what_to_do`,
`a_tool_setter_pocket_is_named_in_plain_words` (app) and
`a_new_tools_typed_number_is_compared_with_the_reserved_aware_allocation`
(ui) pin them.
