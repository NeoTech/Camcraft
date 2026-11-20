## #2017 - Tool page: one side preview, the 3D view only while a pick is armed

**Decided.** The side preview is the one preview of the tool page. It draws
the bands from `tool_body::checked_bands`: the checked profile (a machine
default is floored by `tool_body::checked_profile`, the same call as
`judged`) through `tool_body::stack`, the stack the collision check sweeps.
The 3D mesh view no longer shows at rest. It shows only while a Pick button
(tip, axis, radial, spindle) is armed.

**Why.** The 3D view is the only surface where a click picks the tip or
axis. Removing it fully would remove that function and no other control
replaces it. Showing it only while a pick is armed meets "one preview" at
rest and keeps the pick.

**Dropped.** (1) Delete the 3D view and its picks: loses placement by click.
(2) A second derivation of the holder shape in the UI: the issue forbids it.

**Cost to change.** Delete `placement_view` and `preview_3d` in
`tool_body_page.rs` and drop the Pick buttons when numeric placement is
enough.
