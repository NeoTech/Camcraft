## #1473 - Natural start endpoints on open 2D Contours

**Chosen:** accept a start pick only when its XY equals the natural start of
the centerline run. Climb uses the first stored point. Conventional travel
uses the last stored point. The pick changes no point, lead, depth or move.
The existing parameter validation and open-chain exit refusal remain.

For the app, this slice supports one selected open DXF sketch chain with
Side Auto and no side override. The source chain is the cutter centerline,
so an exact endpoint match proves the source-to-cutter identity. The
existing climb XOR per-chain reverse setting determines its natural start.
A start pick with several selected sketch chains is refused by name.

**Why:** a natural endpoint needs no new projection or direction law.
Reversing a run to satisfy a pick can change the requested cut direction.
Accepting a nearby endpoint can mistake an interior source point for an
endpoint after the cutter offset swallows a terminal leg.

Part and stock source wrappers keep their named start-pick refusals.
The part wrapper also refuses picks in its no-material fallback. An open
machining boundary keeps its source-pick refusal, including an offset or
extension. A pick numerically equal to a cutter endpoint does not prove
source identity in these routes.

The fork at v0.27.0 rotates sketch geometry for WCS yaw but does not rotate
Contour picks. This slice therefore refuses an open sketch start pick
under nonzero WCS yaw before input rotation. No-pick and closed-only paths
retain their existing behavior. Importing a general pick-rotation change
from canonical main was dropped because it changes other operation kinds.

**Coverage:** the original regression runs the same natural-end assertion
on unchanged canonical main `6898b02ea3195e9124e6113afd6ab3cee38d0c50`.
It failed at the existing start refusal for `(0, 0)` with climb enabled.
The fixed test preserves that assertion and compares all automatic moves
for both travel directions. Additional pins cover leads at each depth,
wrong ends, interior and nearby points, the exit refusal, and unproved
part, stock and boundary picks. The short terminal-leg fixture retains its
clearance asserts and adds a source mapping refusal. App command tests
cover both climb settings, per-chain reversal, multiple-chain refusal and
the WCS yaw frame refusal. Independent verification supplies run results.
No golden or snapshot is changed.

Canonical main also has Do-not-touch open-run routes absent from this
fork cut. The inspected canonical source wrappers delegate an empty avoid
set to the public source wrappers. Their nonempty routes retain blanket
start and exit refusals. No avoid pipeline is imported by this slice.
The final independent precheck must inspect the actual target integration.

**Dropped and remaining:** interior starts, an endpoint contrary to the
requested travel, offset-source endpoint mapping and open-chain exit picks
stay refused. Tilted contours remain item 6 of #1473. This partial
contribution uses `Refs #1473` and does not close the issue.

**Cost of later change:** offset-source support needs explicit endpoint
correspondence through trimming and offsets, plus cut-direction and lead
proofs. Interior picks need a rule for the remaining run. WCS yaw support
needs the pick and chain in the same frame. None needs a file-format change
for this endpoint-only slice.
