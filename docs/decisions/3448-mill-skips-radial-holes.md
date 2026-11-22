## #3448 - Detect holes on a mill names radial holes in `skipped`

**What was decided.** On a machine of kind `Mill` (the same test that
Generate uses for its "no turret" refusal), Detect holes does not store
radial holes in the Drill op's `cross` list. It names each one in
`skipped`, with its size, origin and direction. The Z holes stay. If no Z
hole remains, Detect holes refuses and names each radial hole. On a lathe
or a mill-turn, the behaviour does not change.

**Why.** Before, detection stored the radial holes on a mill. Generate then
refused the op ("Z holes AND cross holes", or "no turret"), and
`edit_operation` cannot clear `cross`, because the field is not in the
params patch. The user could only delete the op. The issue asks for the Z
holes to generate and the other holes to be named.

**Dropped.**
- Make `cross` editable through `edit_operation`. The issue accepts either
  fix. On a mill no `cross` list is ever filled now, so this is not
  necessary for the bug. It stays possible as a separate enhancement.
- Keep the #2601 behaviour (store on a mill, refuse at Generate). It gives
  the dead end that this issue reports.
- Try to drill a radial hole along the setup's Z when its axis is
  vertical in that setup. That is reach logic that needs its own design.

**Cost to change.** Small. The rule is in one function,
`operation::cross_reach::keep_reachable`. The #2601 test now asserts the
recognizer output through `cross_holes_of` and the mill refusal.
