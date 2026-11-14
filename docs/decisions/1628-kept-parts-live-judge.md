## #1628 - the parts beside an op's source are judged again live at the post, not stamped

**Decided (unattended, option (c) of the issue):** no stamp term. The parts
an operation's part laws read beside its source (`keep_law::parts_in_stock`
on a stock source, `keep_law::neighbour_parts` on a part source) stay out of
the op's stamp. Instead, when the program sim cannot judge (a turned stock,
posed moves - `posting_sim`'s not-judged arm), the post gate judges every
program op again whose cached verdict is clean and whose kept part set is
non-empty as it stands now (`operation::kept_rejudge::moved_part_problem`).
It re-runs, on the cached path, the very laws generation ran (the solid law
`generate::contain_into` with `DepthLaw::Keep`, or the mark law
`mark_law::mark_into` with its budget for a stock-source Engrave/Trace; the
same cutter, stock to leave, model excess and 3+2 frame; the tap's core
acquittal), once per kept part, and refuses by name:
`<op> now cuts part '<part>' (<finding>) - a part moved, added or no longer
workholding since it was generated; regenerate it`. On an aligned stock
nothing changes: the program sim already judges every part live.

**Why:** it closes the hole the issue names (a part moved, added, un-tagged
as a fixture or un-marked as raw stock only after a clean generation posted
into that part on a turned stock) without breaking the stamp contracts the
stamp-term draft on PR #1622 broke:
- `fixtures.rs::workholding_never_stales_a_fresh_toolpath` - un-tagging a
  vise still leaves the op Ok (pinned again by
  `kept_parts_live::a_fixture_untagged_then_moved_into_a_part_source_cut_on_a_turned_stock_is_refused_by_name`).
- `operations.rs::a_frommodel_stock_survives_its_source_being_deleted` and
  `a_model_update_does_not_touch_a_box_stock` - a stock-based path still
  never reads the model into its stamp.
- `fidelity_copies.rs` - deleting a stock's coarse source keeps the named
  fidelity refusal instead of a bare "regenerate".
- `stock_source_verdict.rs` - marking a casting raw stock only stales
  nothing.
All of these pass unchanged on this branch.

**Considered and dropped:**
- (a) the stamp term (`name`, pose, geometry signature of each kept model).
  Works, but restates four established contracts above and makes every
  neighbour edit churn every op of the setup to Stale - including edits that
  make the cut safer (moving a part away), which then refuse for no visible
  reason.
- (b) the live re-judge on every post, aligned stock included. The program
  sim already judges every part live there, so it would only add a second,
  slower judge with the same answer; (c) runs it only where nothing else
  judges the parts.
- Comparing a recorded digest of the kept set and refusing "regenerate" on
  any change: needs a new cached field (project file), and refuses safe
  edits too.
- Regenerating the op at post time and comparing: the path can legitimately
  differ (a moved vise re-routes links, `fixture_route`), so the fresh
  verdict would be about a different path than the one posted.

**Details:**
- An op whose cached warnings already block is skipped: it is refused by
  its own findings, by name (the existing suites assert that wording).
- An op whose generation job cannot be collected again is refused (no
  verdict is no pass). No fixture reaches that arm today: every input
  `collect_job` can fail on is a stamp input, so such an op reads Stale
  first.
- A 3+2 op's cached path is the world path; it is judged in the frame's
  local coordinates, every move as its 3-D chords at the arc-fit tolerance
  (a tilted path is already chords). Its finding is then named without the
  move number, which would not be the posted path's. Not exercised by a
  test: no clean tilted 2D op beside another part exists in the suites.
- Moving the part under a stock-source op already makes the op Stale (it
  moves another stamp input), so that edit is refused as "regenerate";
  adding a part, moving a neighbour and un-tagging a fixture leave it Ok and
  are now refused by name.

**Cost to change later:** the re-judge is one function over `collect_job`;
moving to (a) later is the stamp term on #1622's history (f4d86b41,
6abccc77) plus the contract restatements, and deleting `kept_moved`.
