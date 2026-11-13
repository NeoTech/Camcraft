## #539 - Wire EDM: tabs / bridges and the tab cut-off

The umbrella (#290 D6) decided that tabs are off by default and, when on, are stated with no invented default, and that an unsupported slug is refused unless a tab or a capture is set. #534 left that hold law to this issue. These are the points it left open.

**Decided:**

- **What is stated.** `tabs: { count, width, at[], cutoff_condition }` on the Wire Contour, absent = off. Every key is optional in the file so a missing one loads and is refused by name at generation ("the tab count is not stated", "the tab width is not stated", "no tab position is stated", "no tab cut-off condition is named"), never defaulted. The count must match the positions (1 to 64). The width is the bridge's material width along the profile; the wire path's gap is that width plus the kerf width.
- **Where a tab may stand.** A tab is placed at the nearest point of the wire path to its stated position (refused if it stands farther than the kerf offset plus the width from the path). Its whole gap must lie on one straight move of the path: a width at or past the side, a tab across a corner, or a tab on the facets of a round ("a tab on an arc too short") is refused by name with the side's length. Two tabs whose gaps overlap or leave under 1 um of cut between them are refused.
- **How the main cut leaves the tabs.** The ring between tabs is a run. Each run is entered from the one threading point by its own straight lead through the waste, cut both ways from its foot to the gaps, and left by retracing the wire's own kerf back to the threading point. The wire is never cut or rethreaded mid-program - it travels without cutting only in slot it already burnt. A lead whose kerf would come within the kerf offset of the product (a punch's far side, a non-convex die) is refused by name ("place the tabs so every stretch between them faces the start hole"); the generator's kerf certificate still runs over every move.
- **The cut-off.** Follows the main cut in the same path. For each tab the wire travels in the kerf from the threading point to the gap along the nearer run, cuts across the gap on the main path's own line, retraces, and returns. It posts as a second section ("<op> tab cut-off") with its own condition; the main section ends at a program stop (M00) so the operator secures the slug before the tabs are severed.
- **The cut-off condition must give the main cut's kerf offset** (within 1 um). The cut-off runs on the main path's line; another offset would leave a step at each tab end (a smaller gap) or burn the product at the step (a larger one).
- **Where the cut-off starts, without a CL field.** The path returns to the threading point exactly once per run and once per tab, so the cut-off starts after the count-th return (`wire_edm::cutoff_split`). The post and the verify gate both read it; a tabbed op whose path does not have that shape is refused ("regenerate it").
- **The hold law (#534's slug law, completed).** A tabbed cut: its main cut must free nothing ("the tabs do not hold it" otherwise), and its whole path, cut-off included, must free the stated piece (the existing wrong-slug and nothing-freed laws). The report says "holds its slug by tabs through the main cut; the tab cut-off frees ...". An untabbed cut that frees a piece is refused ("drops unsupported - state tabs ... or state that the slug is captured") unless `capture: true` is stated; the report then says "into the stated capture".
- **`capture` is a statement, not geometry.** A boolean on the op (absent = false), read live at the gate, not in the stamp - it moves no wire. Existing tests that posted an untabbed punch or die now state `capture: true`; their NC and CL goldens are byte-identical.

**Why:** Prime Directive - a slug that drops unheld can jam the wire, break it, or strike the work; a tab width, count or place guessed by the program is wrong output.

**Considered and dropped:**

- A `Toolpath` field (or a CL record) marking the cut-off: touches the cl crate, the CL text, the frame transforms, the cache DTO and a dozen literal constructions, for one index the path's own shape already states.
- Cutting and rethreading the wire at each tab: needs a rethreading hole at the tab, which no stated geometry provides.
- A separate "tab cut-off" operation kind: a new kind, its page and registry entry, and a link back to the main cut's tabs, for what is one cut's second pass.
- Defaulting the cut-off condition to the main one: no invented default (D6).
- Tab fields on the op page: the UI pages are #535's; the keys are reachable through the file and the MCP parameter patch (the DTO).

**Not exercised on a real EDM control (D10):** the golden `wire_contour_die_tabs.nc` posts against the synthetic fixture with an invented second condition (`C120`, same wire and gap). A user with a real machine dry-runs before any cut.

**Cost to change later:** a different cut-off offset needs a step move at each tab end and its own kerf law; tabs on arcs need arc-length gaps on arc moves (the path has none today). Both stand on `tabs.rs` alone.

**Merge with main (open runs #1109, no-core burnout #540), 2026-11-13:**

- Tabs stated on an open run or on a burnout are refused by name in the generator (`wire_contour` before `wire_open`, and `wire_burnout`), pinned in `open_tests` and `burnout_tests`. Chosen over silently ignoring the tabs (Prime Directive: never silent). Considered and dropped: tabbing an open run (it closes no slug to hold) and tabbing the burnout's outer ring (a burnout must free nothing).
- The hold law's "an untabbed freed piece must be captured" check skips a burnout (its own no-core law already refuses any freed piece) and never reaches an open run (`judge` returns before the slug law). The kerf and slug laws are unchanged: no tolerance was loosened.
- Main's existing fixtures that post a freed slug or punch (`wire_readers`, `posted_gates/wire.rs`) state `capture: true`, as the PR's own fixtures already did, so they stay judged clean; main's open-run and burnout fixtures state `capture: false`.
