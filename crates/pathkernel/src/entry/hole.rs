//! The entry in a pre-drilled hole: the tool goes down at the centre of a
//! hole of the choice, then moves in the plane onto the start of the cut.
//! The nearest hole whose centre and lead are in the legal room serves. A
//! hole narrower than the tool needs a tool that can plunge: the tool then
//! cuts the ring between the hole and its own radius.
//! Provenance: pre-drilled entry holes as in public machining handbooks
//! (Machinery's Handbook, milling entry practice); our own design record
//! `docs/decisions/4024-path-kernel-design.md` "Stage 6: Entry and exit".

use super::proof::{Entry, Proof};
use super::room::Ctx;
use super::EntryKind;

/// The pre-drilled entry at the start of the first cut, if one fits.
pub(super) fn enter(ctx: &mut Ctx<'_>) -> Result<Option<Entry>, crate::Refusal> {
    let s = ctx.start.at();
    let dist = |c: crate::Point2| (c.x - s.x).hypot(c.y - s.y);
    let mut holes: Vec<_> = ctx.choice.holes().to_vec();
    holes.sort_by(|a, b| dist(a.center()).total_cmp(&dist(b.center())));
    let (r, plunges) = (ctx.r(), ctx.tool().can_plunge());
    for h in holes {
        let c = h.center();
        if (h.radius() < r && !plunges) || !ctx.legal.contains(c) {
            continue;
        }
        let Ok(lead) = ctx.straight_lead(c) else { continue };
        let (approach, descent) = (vec![ctx.above(c)?], vec![ctx.at_depth(c)?]);
        if let Ok(e) = Entry::led(EntryKind::Predrilled, approach, descent, lead, Proof::InHole(h), ctx.choice, ctx.tool()) {
            return Ok(Some(e));
        }
    }
    Ok(None)
}
