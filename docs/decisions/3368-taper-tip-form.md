## #3368 - a tapered mill has a tip form (sharp, flat, ball)

Plan step D1 of the tool library redesign (#3663). The tip math is the
research of #895.

**Decided: the tip lives in `crates/tools/src/taper.rs`.** `Tool` gets two
fields, `tip_form` (`TaperTip`: `Sharp`, `Flat`, `Ball`; default `Sharp`)
and `tip_diameter` (default 0). `corner_r` is the corner radius of a flat
tip and the ball radius of a ball tip. `Tool::profile` and
`Tool::meridian` call `taper::outline` and `taper::meridian_points`;
`tools.rs` grows by its field lines and one call. `Tool::geom` does not
change: its numbers (radius, flute, shank, holder) do not depend on the
tip. Why: `tools.rs` is at 740 lines, and the tip is one concern.

**Decided: a sharp taper keeps its corner radius inert.** The #895
research asks a sharp tip to state `corner_r == 0`. An older project can
hold a sharp taper with a corner radius, and a new refusal would stop it
from opening. So a sharp tip refuses only a tip diameter. Dropped: the
strict rule (it breaks old files).

**Decided: only a chamfer mill states a tip.** Another kind with a
non-sharp form or a tip diameter is refused by name in `why_not_sane`.
The editor clears the tip fields when the kind changes. Why: a stale tip
on an end mill would step the file versions for nothing.

**Decided: every operation refuses a flat or ball tip by name, at two
doors.** `tool_gate::tool_refusal` (creation, edit, re-tool) and
`tool_gate::tool_problem` (generation and the post). The plan named only
the first door. The second door is needed: a `ChangeTool` that gives an
existing operation's tool a flat tip does not pass the creation door, and
the generator would read the tip as a sharp cone (a smaller tool than the
real one, which can gouge). The fitting-tool list and the catalog hint
leave a flat or ball tip out. The tool stamp digests the tip form and tip
diameter only for a non-sharp tip, so no sharp taper's stamp moves.

**Decided (user, 24 Nov 2026): step the `.camproj` version for a non-sharp
tip only.** The issue text says "no version step", and the maintainer has
not answered the comment on #3368 yet. The user said: "Just start, we can
refactor that version step in or out later." A file with a flat or ball
tip says `TIP_FORM_PROJECT_VERSION` (79). A file without one says
`PROJECT_VERSION` (78), so an older build opens it as before. This build
reads 79 as 78 plus the tip keys. The step is isolated in
`project/file.rs`: `TIP_FORM_PROJECT_VERSION`, `NEWEST_PROJECT_VERSION`,
`written_project_version` and `tip_form_read_version`, with one call in
`wire.rs` (save) and one in `apply.rs` (load). A const assertion stops a
later unconditional step from taking 79 by accident. The tests that ask
for a refused future version use `NEWEST_PROJECT_VERSION + 1`.
To remove the step: delete those four items, write `PROJECT_VERSION` in
`wire.rs`, drop the `apply.rs` line, and point the future-version tests at
`PROJECT_VERSION + 1` again. Dropped: an unconditional step to 79 (every
saved file would become unreadable for older builds for a feature most
files do not use); no step (an older build reads a flat tip as sharp).

**Decided: the `.camtools` library steps to `TIP_FORM_VERSION` (6) for a
non-sharp tip only.** Version 5 is the drawn cutter of PR #3906
(`CUTTER_VERSION`). PR #3906 merged first, so the merge-time refresh of
this PR added 6 to the accepted list: the reader accepts 1 to 6, and a
library with both a cutter and a non-sharp tip says 6.

**Decided: three catalog tools.** `CM-0600-90-F10` (a 6 mm 90-degree
chamfer mill with a 1 mm flat tip), `TE-0250-03-F10` (a tapered flat end
mill, 1 mm tip, 3 degrees a side) and `TB-0160-01-R05` (a tapered ball
nose, R0.5, 1 degree a side). The numbers are common sizes of the public
tapered end mill tables, not one maker's part.

**Not done here:** the `camcraft-mcp` skill row for
`set_tool_cutter_params` (a process file) does not list the two new
params. The MCP tool's own description and `docs/agents/mcp-tools.md` do.
