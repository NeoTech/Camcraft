## #2758 - Simulate cylinder stock along X or Y

**Decision.** Use analytic vertical sections for a cylinder whose axis lies
along X or Y. Support both directions on each axis. `sim_frame` returns the
stock shape with its actual placement. `stock_grid` selects the analytic
builder.

Retain the initial top and the rounded floor in `Dexel` metadata. Count
removed volume from the initial top and limit it to the material above the
floor. `Body::on_floor` uses the same rounded floor when it builds a carried
body.

Keep the raw tool cut heights. The existing gouge verdict needs those heights
when a tool cuts through the stock. Do not clamp the cut heights to the floor.

**Reason.** A laid cylinder has a curved top and a curved underside. A top
surface alone invents material below the underside. Analytic sections retain
the round boundary without a mesh approximation. The simulation can count
real material and preserve its existing gouge checks.

**Dropped.** Do not add optional oblique stock support in this change. Keep
the named refusal for stock that stands off the machine axes. Keep the field
carry engine's cylinder refusal.

Do not build the cylinder from an inscribed polygon mesh. That mesh holds
less material at the round rim and can report removal that the tool did not
make. Do not clamp cut heights to the floor, because that can hide a gouge.

**Later cost.** More stock shapes need generic initial-top and floor metadata.
Arbitrary orientations need an oriented solid representation and checks for
its cutting, volume, carry and verdict laws.
