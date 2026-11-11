## #692 - The one pick rule: the remaining pickers after #705 / #709

**Decided:** an audit of every picker on main (posted on #692) found one
bypass left: MCP `locate_part` read its mate ids with `parse_feature_id`
and never asked `pick::check_pickable`, so a vertex along a round rim or
inside a face was taken as a mate point. It now runs the same check the
other MCP pick tools run (`mcp::pick::pick_of`). The user guide gets a
"What a click picks" section (`site/content/guide/operations.md`).

**Why, choice by choice:**
- **The check sits at the MCP door, not in the `Locate` command.** Every
  other pick made by id is checked there (`picks_of`), and the GUI's mates
  can only come from the one pick rule's clicks. Moving the check into
  `pick::mate_datum` would change what the command accepts for every
  caller, project data included - a wider change than this
  fix needs. Considered and dropped for now; cheap to add later if a
  second by-id entrance appears.
- **No "pick through" mode in the viewport.** The issue allows it only as
  an explicit, off-by-default mode. The API carries it (`PickQuery::through`,
  pinned); the viewport keeps it off (`feature_click::query_at`), as #709
  decided. Offering it is a new UI control, not part of this fix.
- **Centre picks wait for #759.** `PickKinds::CENTRES` exists; no page asks
  for it until the WCS circle-centre PR lands, so nothing was wired here to
  avoid building on an unmerged PR.
- **Model-mode clicks stay on `pick::first_hit_f`.** They select a whole
  model, not a face, edge or point, and already use the app's
  front-most-surface law (models over stock); routing them through
  `pick_feature` would only add a feature-graph build per click.

**Cost to change later:** low - one call per entrance, and one doc section.
