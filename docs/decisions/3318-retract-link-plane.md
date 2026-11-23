## #3318 - Name the effective plane when the retract hop is off

**What was decided.** The shared 2D Offset and 2D Adaptive pane passes
`retract_z.is_some()` to `leads_tab_with_retract`. Without a retract hop,
the link choice reads `Clearance plane (hop off)`. With a configured hop,
it reads `Retract plane`. The Keep down fallback hint names the same plane.

The operation keeps its stored `Link::Retract` choice. Enabling the hop
therefore restores the retract label without another link edit. This is
a display change. It adds no command or MCP tool and changes no generated
motion. Other operation families retain their existing display behavior.

**Why.** The generator resolves an absent retract height to clearance.
The Heights tab and viewport already show that clearance plane. The Lead
tab must name the same plane instead of suggesting an unseen height.

**Considered and dropped.** Always adding a Retract Z value could change
the operation's motion and semantics. Adding only a hint would leave the
misleading combo value visible. Renaming the choice uses the behavior that
already exists.

The branch stacks on PR #3353 because both changes touch
`roughing2d_pane`. PR #3251 touches the same test file in a different
function. Whichever merges second must refresh.

**Cost to change later.** Low. Other callers can supply explicit retract
context after tests establish their link semantics. The stored parameter
and generator need no migration.
