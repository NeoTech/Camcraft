//! The entry in a cleared column: the tool goes down where the stock is
//! cleared to the depth of the cut under its whole disc, then moves in the
//! plane onto the start of the cut. The caller reads `cleared` from its
//! stock reader. The cut start serves when it is in the column; else the
//! nearest corner of the column whose straight lead stays in the legal room.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 6: Entry and exit"; the cutter-centre erosion as in Held (1991).

use super::proof::{Area, Entry, Proof};
use super::room::{Ctx, ROOM_TOL, geo};
use super::EntryKind;
use crate::{Point2, in_area};

/// The corners of the column tried for a lead, nearest first.
const CORNER_TRIES: usize = 64;

/// The cleared-column entry at the start of the first cut, if one fits.
pub(super) fn enter(ctx: &mut Ctx<'_>) -> Result<Option<Entry>, crate::Refusal> {
    if ctx.cleared.is_empty() {
        return Ok(None);
    }
    let shrink = ctx.r() - ROOM_TOL;
    let col = if shrink > 0.0 { geo(ctx.cleared.offset(-shrink))? } else { ctx.cleared.clone() };
    let col = geo(col.intersect(&ctx.legal))?;
    let Ok(area) = Area::new(col) else {
        return Ok(None);
    };
    let s = ctx.start.at();
    let mut spots: Vec<Point2> = if in_area(area.region(), s) {
        vec![s]
    } else {
        area.region().loops().iter().flat_map(|l| l.points().iter().copied()).collect()
    };
    let dist = |c: &Point2| (c.x - s.x).hypot(c.y - s.y);
    spots.sort_by(|a, b| dist(a).total_cmp(&dist(b)));
    for at in spots.into_iter().take(CORNER_TRIES) {
        let Ok(lead) = ctx.straight_lead(at) else { continue };
        let (approach, descent) = (vec![ctx.above(at)?], vec![ctx.at_depth(at)?]);
        let proof = Proof::InColumn(area.clone());
        if let Ok(e) = Entry::led(EntryKind::ClearedColumn, approach, descent, lead, proof, ctx.choice, ctx.tool()) {
            return Ok(Some(e));
        }
    }
    Ok(None)
}
