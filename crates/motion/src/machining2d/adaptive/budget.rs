//! The growth rings' planning budget (issue #13): the ring spiral
//! (`grow`) re-walks the whole cleared outline every revolution, so its
//! work grows with the square of how far the fronts must travel - a long
//! narrow channel (the band an Outside containment leaves round a part)
//! takes hundreds of revolutions, each over a ring hundreds of mm long,
//! and planned for minutes. Two laws keep it bounded and named:
//!
//! 1. **Up front**, the least work the spiral can need: the channel's
//!    length `L = area / (2 c)` (its center area over its width, `c` the
//!    clearance at the entry), its fronts each travelling `L / 2` at no
//!    more than the step cap a revolution, over rings averaging `L`
//!    round: `L^2 / (2 cap)`. Past [`LEAST_WORK_MAX`] tool radii the
//!    area is refused at once, by name.
//! 2. **While growing**, the length of every revolution tried is summed;
//!    past [`WORK_MAX`] tool radii the plan stops and is refused, whatever
//!    the shape (the Prime Directive's "never hang").
//! 3. **While growing** (issue #850), the polygon geometry every step
//!    tried runs through is counted too: the vertices of the cleared area
//!    it offsets and of the ring it opens, and of each revolution's
//!    margin area. Length does not bound time - an islanded area's rings
//!    carry the island's outline through every trial, and its plan spent
//!    ~2 ms a tool radius against ~1 ms in a plain channel, running about a
//!    minute before law 2 refused it. Past [`GEOM_MAX`] vertices the plan
//!    is refused the same way. The count is deterministic (never a wall
//!    clock): the same project plans, or is refused, on every machine.
//!
//! The refused area is cut by the Offset rings, named (`pocket_hsm`).
//! Ours: the bound is plain arithmetic on the ring family's geometry;
//! the limits are calibrated on our own fixtures (the 60 x 14 slot needs
//! ~110 r up front and ~5.8 k r grown; the user's 14 mm band round a
//! 60 x 50 part ~3.4 k r up front). No vendor material consulted.

use std::cell::Cell;

use crate::geo2d::Region;

/// The least growth work (tool radii of revolution length) an area may
/// need before it is refused up front.
pub(super) const LEAST_WORK_MAX: f64 = 2_000.0;

/// The growth work (tool radii of revolutions tried, every margin
/// attempt) a plan may spend before it is refused.
pub(super) const WORK_MAX: f64 = 30_000.0;

/// The growth geometry (polygon vertices the trial steps and the
/// revolutions' margin areas ran through, every margin attempt) a plan
/// may spend before it is refused. Calibrated on our fixtures
/// (`docs/decisions/850-adaptive-geometry-budget.md`): the 60 x 14 slot,
/// the largest that plans, spends ~252 k; the 60 x 40 pocket round a
/// 12 x 12 island ~865 k in its first margin attempt alone.
pub(super) const GEOM_MAX: u64 = 500_000;

/// The growth plan's running work: revolution length (mm, law 2) and
/// polygon vertices (law 3), summed over every margin attempt.
#[derive(Debug, Default)]
pub(super) struct Work {
    len: Cell<f64>,
    verts: Cell<u64>,
}

impl Work {
    /// A revolution `mm` long was tried.
    pub(super) fn add_len(&self, mm: f64) {
        if mm.is_finite() && mm > 0.0 {
            self.len.set(self.len.get() + mm);
        }
    }

    /// The step ran through `regions`' vertices.
    pub(super) fn add_geometry(&self, regions: &[&Region]) {
        let n = regions.iter().flat_map(|r| r.loops()).fold(0u64, |a, l| a.saturating_add(u64::try_from(l.points().len()).unwrap_or(u64::MAX)));
        self.verts.set(self.verts.get().saturating_add(n));
        #[cfg(test)]
        PEAK.with(|p| p.set(p.get().max(self.verts.get())));
    }

    /// The vertices counted so far.
    #[cfg(test)]
    pub(super) fn verts(&self) -> u64 {
        self.verts.get()
    }

    /// Why the plan is refused after `revs` revolutions, for a tool of
    /// radius `r`: past [`WORK_MAX`] or [`GEOM_MAX`]; `None` within both.
    pub(super) fn over(&self, r: f64, revs: usize) -> Option<String> {
        if self.len.get() > WORK_MAX * r {
            Some(over_work(revs))
        } else if self.verts.get() > GEOM_MAX {
            Some(over_geometry(revs, self.verts.get()))
        } else {
            None
        }
    }
}

#[cfg(test)]
thread_local! {
    /// The most geometry any growth plan on this thread spent (tests).
    pub(super) static PEAK: Cell<u64> = const { Cell::new(0) };
}

/// The channel an area reads as: its length (mm) from its center
/// `area` and the clearance `clear` at the entry.
pub(super) fn channel_len(area: f64, clear: f64) -> f64 {
    if clear > 1e-9 { area / (2.0 * clear) } else { f64::INFINITY }
}

/// The least growth work (tool radii) for a channel `len` long, fronts
/// stepping at most `cap` mm a revolution, for a tool of radius `r`;
/// with the least revolutions it takes.
pub(super) fn least_work(len: f64, cap: f64, r: f64) -> (f64, f64) {
    let revs = 0.5 * len / cap.max(1e-9);
    (revs * len / r.max(1e-9), revs)
}

/// The refusal for a channel `width` mm wide (cutter reach included)
/// and `len` long that would need `revs` revolutions at least.
pub(super) fn too_long(width: f64, len: f64, revs: f64) -> String {
    format!(
        "is refused: the area is a channel {width:.1} mm wide and ~{len:.0} mm long (the band round a part?) - one adaptive spiral would need at least {revs:.0} revolutions along it, each round a ring up to ~{:.0} mm long: too narrow and long to plan", 2.0 * len
    )
}

/// The refusal once the growth work ran past [`WORK_MAX`].
pub(super) fn over_work(revs: usize) -> String {
    format!("is refused: growing the cleared area ran past its planning budget after {revs} revolutions")
}

/// The refusal once the growth geometry ran past [`GEOM_MAX`].
pub(super) fn over_geometry(revs: usize, verts: u64) -> String {
    format!(
        "is refused: growing the cleared area ran past its planning budget after {revs} revolutions - its rings (~{} k outline vertices worked, over {} k) are too detailed to plan in seconds (an island?)",
        verts / 1000,
        GEOM_MAX / 1000
    )
}

#[cfg(test)]
#[path = "budget_fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_band_needs_far_more_work_than_a_short_slot() {
        // The 60 x 14 slot (center 50 x 4) and the 14 mm band round a
        // 60 x 50 part (center ~275 x 4), D10 at the default load.
        let cap = 1.5 * 2.0 * 5.0 * crate::load::DEFAULT_OPTIMAL_LOAD;
        let (slot, _) = least_work(channel_len(200.0, 2.0), cap, 5.0);
        let (band, revs) = least_work(channel_len(1100.0, 2.0), cap, 5.0);
        assert!(slot < 0.1 * LEAST_WORK_MAX && band > 1.5 * LEAST_WORK_MAX, "slot {slot}, band {band}");
        assert!(too_long(14.0, 275.0, revs).contains("14.0 mm wide"));
        assert_eq!(channel_len(10.0, 0.0), f64::INFINITY);
    }

    #[test]
    fn the_geometry_budget_refuses_by_name_and_saturates() {
        let Ok(sq) = crate::geo2d::Loop::new(vec![
            crate::geo2d::Point2::new(0.0, 0.0),
            crate::geo2d::Point2::new(1.0, 0.0),
            crate::geo2d::Point2::new(1.0, 1.0),
            crate::geo2d::Point2::new(0.0, 1.0),
        ]) else {
            panic!("square")
        };
        let Ok(reg) = Region::new(vec![sq]) else { panic!("region") };
        let w = Work::default();
        w.add_len(f64::NAN);
        w.add_geometry(&[&reg, &reg]);
        assert_eq!(w.verts(), 8);
        assert!(w.over(5.0, 3).is_none());
        w.add_len(WORK_MAX * 5.0 + 1.0);
        assert!(w.over(5.0, 3).is_some_and(|m| m.contains("planning budget after 3 revolutions")));
        let w = Work::default();
        w.verts.set(GEOM_MAX);
        assert!(w.over(5.0, 9).is_none());
        w.add_geometry(&[&reg]);
        let Some(m) = w.over(5.0, 9) else { panic!("over the geometry budget") };
        assert!(m.contains("planning budget after 9 revolutions") && m.contains("too detailed"), "{m}");
        w.verts.set(u64::MAX);
        w.add_geometry(&[&reg]);
        assert_eq!(w.verts(), u64::MAX);
    }
}
