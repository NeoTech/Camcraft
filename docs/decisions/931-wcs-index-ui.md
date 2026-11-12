## #931 - work offsets as an index, the post supplies the word (slice 3: UI and MCP)

**Decided.** The Setups page combo (also drawn on the WCS page) lists
offsets `1..=max` of the setup's OWN post, read from
`setup::work_offset_names` (the cached table from slice 2), each worded
"Work offset n (NAME)", or "Work offset n" when the post gives that
offset no name (the triage wording note). The wording lives in app
(`setup_offsets::entry_label`) so the UI and any later caller share one
spelling. The default post lists six entries; Tormach PathPilot lists
nine (G59.1-G59.3).

**A stored pick beyond the post's range stays visible as itself.** When a
setup holds offset 9 and is switched to a six-offset post, the combo
shows "Work offset 9 - not on this post" as the selected entry and lists
it after 1-6 (and only it - offsets 7 and 8 are not offered, since the
command would refuse them). The post gate from slice 2 refuses it by
setup name. Considered and dropped: hiding it (the combo would read as
offset 1 - the silent fallback triage point 2 forbids) and clamping it
(a write the user did not ask for).

**Other UI words.** Slice 2 already moved the browser tree, the heights
page row, the viewport chips (`interact.rs`) and the status line
(`state.rs`) to `setup::wcs_name`. Left here: the combo's hover text and
the heights page hint, which named a fixed "G54-G59" range; both now say
the post names the offsets. The probe page's "Set G54".."Set G59" results
are unchanged on purpose: the probe register arithmetic reaches offsets
1-6 only (triage point 4), so a probe never sets an extended offset.

**MCP.** `set_work_offset` takes `index` (1-based; a number, or the words
G54-G59 as aliases of 1-6) and keeps `word` as the older spelling of the
same argument. Both may be given only if they agree. Parsing
(`setup_offsets::parse_index`) checks only the shape and the core sanity
bound (1-255); whether the setup's post can express the offset is the
command's check (`SetSetupWorkOffset::new` -> `setup_offsets::check`), so
the MCP refusal is the GUI's refusal, by setup name. The reply, `set_wcs`
and `get_job`'s `setups[].wcs` carry `index` and `name`, with `word` kept
as an alias of `name`. Considered and dropped: accepting a post's own
name (e.g. "G59.1") as input - names are display only and differ by
post, so the number is the one stable input.

**PROCESS_VERSION** is bumped to 21 (the camcraft-mcp skill doc changed):
one above the highest open bump (20, PR #1033). It renumbers at merge if
another process PR lands first.

**Cost to change later.** Small: the wording is one function in app, the
MCP aliases one parser.
