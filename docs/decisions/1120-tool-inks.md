## #1120 - Tool drawing inks become theme tokens (fourth family)

**What was decided.** Eighteen colour tokens join `Tokens`, each also a
theme-file key (`colors.<name>`):
- the tool, holder and profile previews: `tool_ground`, `tool_axis`,
  `tool_grid`, `tool_flute`, `tool_shank`, `tool_body`, `tool_edge`,
  `tool_holder`, `tool_note`, `tool_dim`;
- the library thumbnails: `thumb_flute` (their shank reads `tool_shank`);
- the refusal and warning inks of the tool pages: `bad_ink`, `warn_ink`;
- the tool body page: `mount_axis`, `mount_radial`, `mount_tip`,
  `mount_origin`, `mount_hint`.

Both tables hold the old values, so no pixel changes. The previews draw on
their own dark ground in the light theme too; that stays as it was.

The module colour consts (`NOTE`, `BAD`, `HOLDER`, `BG`, ...) become small
functions with the same visibility (`note_ink()`, `bad_ink()`,
`holder_ink()`, `tool_ground()`, ...), which read the live token. Their
callers change by name only.

`theme::with_alpha(colour, alpha)` gives a token's own colour at an alpha.
It reads the colour unmultiplied first, so a partly transparent token does
not darken twice (the defect that the #3993 review found in the facet
veil). The holder fills (alpha 90 in the profile, 70 in the preview) use
it, and `ground_ink::facet_veil` now calls it too.

**Why.** One key for each colour role, not one for each file: the same
note grey was a const in five files, the axis line in three. `bad_ink` and
`warn_ink` are general names, because the refusal red is also the text of a
refused value on the cutter pages; the status family can reuse them.

**Considered and dropped.**
- A light-column set for the previews. #1120 moves values; a light preview
  ground is a separate design question.
- Keeping the consts and adding the tokens beside them. Two sources for one
  colour is what #1120 removes.

**Cost to change later.** Small: eighteen fields and keys; the functions
keep the call sites stable.
