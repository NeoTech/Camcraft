## #2410 - Router machine kind: router options on the Milling tab

**Chosen:** A machine file states `"kind": "router"` (also "CNC router" or
"routing"). `Kind::from_word` reads the word as `Kind::Mill`, and the new
`machine::is_router` reads the word as a router. So a router keeps every
mill rule: axes, spindle, tools, posts, checks. A router only offers more.
One app function, `router_offer::router_options`, lists what the setup's
machine offers. The Setups page "Sheet job" group, the corner relief rows
(Pocket, 2D Adaptive, 2D Offset) and the MCP `get_job` field
`setups[].router_options` read that one function.

- A router machine offers all router options.
- A machine that is not a router offers none.
- A project with no machine offers all of them. Nothing says that the job
  is not a router job, and the old pages showed them.
- A stated option stays shown on a mill: a stated sheet job, or a corner
  relief that is not off. Nothing is hidden or deleted from a project.

The six shipped router files now state "router": Tormach 24R, xsTECH,
xsTECH Pro (#620) and Sienci AltMill 2x4, 4x4, 4x8 (#728). The generic
Grbl, grblHAL and TinyG files ("router / mill") and the Makera desktop
mills stay "mill".

**Why:** The escalation comment on #2410 found that a new `Kind::Router`
variant changes about 33 exhaustive matches in 5 crates. Each match is a
place where a router can stop acting as a mill, which can change posted
G-code or a check. A word that reads as a mill has no such risk: no match
changes, and no golden moves. The maintainer's decision says that a router
is a mill with more options on the Milling tab, which is the same shape.

**Dropped:**
- A `Kind::Router` enum variant (the risk above).
- A list of operations that a router hides. The router uses the mill's
  operation set now. No shipped operation was found that a router cannot
  run. The function is the place to add one.
- The tool-change macros of #2407. They are post-defined and have no page
  control, so there is nothing to show or hide.
- A "router" entry in the machine editor's Kind combo. The combo shows a
  router file as "mill" and writes the word only when the user changes the
  combo, so the file keeps "router" when the user does not touch it.
- An update of the `camcraft-mcp` skill text (a process file).

**Cost to change later:** Each open router sub-issue (#2398, #2399, #2401,
#2402, #2404, #2405, #2406, #2408, #2409, Nest #2163) adds one
`RouterOption` row and reads `router_offer::offers` in its page. An older
release that opens a project with a router machine file refuses the word
"router" by name; it does not read it as something else.
