## #339 slice 5 - links routed over the setup's fixtures

**Decided:** an upright mill's links are planned against the setup's
workholding at generation (`operation::fixture_route::plan_links`, one
hook in `generate.rs` before the verify pass). A link is the maximal run
of straight rapids between two non-rapid moves (or a path end). A run
the gate's own fixture laws (`workholding_warnings`: c2/d2 plus #476's
mesh acquittal) AND the whole-stack law below both find clear is left
byte for byte. A struck run is re-laid as retract, traverse, plunge
(G0 up at its start XY, G0 across, G0 down to its end) at

`H = max(the run's own top, highest steel under the traverse + 5 mm)`

where "under the traverse" is every fixture box the widest band of the
stack (flute, shank, holder bands, spindle housing when stated) reaches
along the run's XY segment. The tip above all of it puts every band
(they all stand above the tip) above the steel. The re-laid run is then
judged again by both laws; only a clean one replaces the original. The
path's notes say how many links rose, from where, over which fixture
and to what Z.

**Provenance:** the clearance-plane / retract-height link of published
CAM practice and machining handbooks; RS-274 / ISO 6983 G0. No vendor
material consulted.

**Refusals and what stays report-only:**
- *Steel taller than the machine reaches:* with a stated Z stroke the
  height is capped at its top (WCS origin + the program-frame stroke
  top, the travel check's own law), keeping at least 0.5 mm over the
  steel. Steel whose top is less than 0.5 mm under the cap refuses
  generation by name: "the link at moves i..j, (x, y) to (x, y), cannot
  clear fixture 'Clamp1': its steel stands to Z ..., and the machine's Z
  stroke tops out at Z ...". No stroke stated: no cap (the unstated-limit
  law, as everywhere else); the note states the height used.
- *Steel over the link's own end:* if the re-laid link is still struck,
  the steel stands over or beside the column the cut ends (or starts) in
  - an overhanging arm above a cut point. No link height helps; the
  link is left as laid, its `FixtureHit` stands (the post is blocked by
  name, today's verdict) and a path note says why. Refusing generation
  there was considered and dropped: it hides the path the user needs to
  see to move the fixture, and the post is blocked either way.
- *Keep-down FEED links are not re-laid.* In the CL stream a keep-down
  link is a feed indistinguishable from a cut, and re-laying a cut would
  silently leave material. And a keep-down link is only taken inside the
  certified cut area, so its stack sweeps steel the neighbouring passes'
  stacks (stepover <= diameter) sweep too: if it hits, the cuts hit and
  the op is refused at the post by name anyway. A keep-down RAPID (a run
  that stays at depth) is a link and does become retract / traverse /
  plunge. Tagging links in the CL stream (so feed links could be lifted
  individually) was considered and dropped for this slice: it changes
  the `Toolpath` shape, cache and digest for no case we could build.
- *Posed / rotary / turned:* tilted frames, a rotary axis, a pose off
  vertical, a turned C, lathe tools and machines keep today's
  report-only verdict. So do paths carrying canned cycles (holes post as
  the cycle, not as these moves - inserting a traverse there could post
  a drill), thread or probe records, and any run inside a compensated
  span (spans outside re-index).

**The whole-stack law used for detection and re-judging:** every band
over each move's true z span (low end + its foot to high end + its top)
against every box (strict-dip z rule), a box's own closed mesh
acquitting what it flagged (#476: no facet within tolerance and not
inside the solid). This is the #424/#425 rule (PR #464), which this
branch does not yet carry: the current upright-rapid law reads only the
flute disc. Judging with both laws means a routed link passes the gate
today and after #464 merges.

**Stamp:** fixtures and the machine stay off the stamp (M3.2 law), so a
routed path does not stale when a clamp moves; the post gate re-checks
the steel every time, and regenerating re-routes.

**Byte law:** no fixtures (an empty collision set) returns the path
untouched before any work; links that clear are never rewritten. Only
documents whose links strike steel post different G-code: the raised
G0 moves.

**Considered and dropped:**
- *Route at the op's clearance plane:* the clearance plane is under the
  steel in exactly the cases that need routing; a height derived from
  the steel is the only one that answers.
- *Detour around the steel in XY:* shorter in some layouts, but a
  planar path search over arbitrary fixtures with the stack's radius is
  a much larger law; rising is always available below the stroke.
- *Per-mesh height (lower over a vise's gap):* the box top is
  conservative and simple; the mesh still acquits the original link,
  so a link already clear inside a vise's box is never re-laid.
- *5 mm gap:* the conventional retract margin; any positive margin is
  safe (the tip clears the box top). Named `ROUTE_GAP`.
