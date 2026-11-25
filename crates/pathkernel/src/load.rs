//! Stage 3, Load control (K3, #4026): the load choice, the readings and the
//! over-limit record, and the stock reader. The `Controlled` constructor
//! checks law 6: each cut read above the limit is named. K3 adds
//! `control()` and the engagement law.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 3: Load control"; radial engagement as in Stori & Wright (2000)
//! and Held & Spielberger (2009).

use crate::order::OrderedSet;
use crate::region::KernelRegion;
use crate::verdict::{Refusal, Stage, Verdict};
use crate::{KERNEL_TOL, Point2, finite};

/// The relative tolerance of the load limit: a cut above
/// `load x (1 + PLAN_TOL)` is over the limit.
pub const PLAN_TOL: f64 = 0.05;

/// What the constant-engagement control reads the cut material from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Reach {
    /// The rings before (Offset rings).
    Rings,
    /// The in-process stock (Adaptive).
    Stock,
}

/// How the load on the tool is held.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum LoadControl {
    /// The identity: the curves stay; the load is still read and named.
    #[default]
    None,
    Trochoidal { width: f64, pitch: f64 },
    /// `load` is the radial engagement ae/D, from 0 (excluded) to 1.
    ConstantEngagement { load: f64, reach: Reach },
}

/// Refusal reason: a trochoid with no positive width or pitch, or a pitch
/// not below the width.
pub const BAD_TROCHOID: &str = "a trochoid needs a positive width and a positive pitch below the width";
/// Refusal reason: an engagement out of (0, 1].
pub const BAD_LOAD: &str = "the engagement ae/D must be above 0 and at most 1";

impl LoadControl {
    /// Checks the choice when the recipe is made (validate early).
    pub fn validate(&self) -> Result<(), Refusal> {
        match *self {
            Self::None => Ok(()),
            Self::Trochoidal { width, pitch } => {
                let ok = finite(&[width, pitch]) && width > 0.0 && pitch > 0.0 && pitch < width;
                if ok { Ok(()) } else { Err(Refusal::new(Stage::Recipe, BAD_TROCHOID)) }
            }
            Self::ConstantEngagement { load, .. } => {
                if load.is_finite() && load > 0.0 && load <= 1.0 {
                    Ok(())
                } else {
                    Err(Refusal::new(Stage::Recipe, BAD_LOAD))
                }
            }
        }
    }
}

/// One load reading: the engagement ae/D at a place of a cut.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    pub at: Point2,
    pub load: f64,
}

/// A cut read above the limit, with its place and its reason.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverLimit {
    pub at: Point2,
    pub load: f64,
    pub reason: &'static str,
}

/// The ordered set after load control, with its readings and every cut
/// above the limit.
#[derive(Clone, Debug, PartialEq)]
pub struct Controlled {
    set: OrderedSet,
    readings: Vec<Reading>,
    over: Vec<OverLimit>,
}

/// Refusal reason: a reading that is not finite or below zero.
pub const BAD_READING: &str = "a load reading must be a finite engagement of 0 or more at a finite place";
/// Refusal reason: a reading above the limit with no over-limit record (law 6).
pub const UNNAMED_OVERLOAD: &str = "a cut above the load limit must be named in the over-limit list";

impl Controlled {
    /// Refused when a reading is not finite, or when a reading above
    /// `limit x (1 + PLAN_TOL)` has no over-limit record at its place.
    pub fn new(set: OrderedSet, readings: Vec<Reading>, over: Vec<OverLimit>, limit: f64) -> Result<Self, Refusal> {
        let ok = |r: &Reading| finite(&[r.at.x, r.at.y, r.load]) && r.load >= 0.0;
        if !limit.is_finite() || !readings.iter().all(ok) {
            return Err(Refusal::new(Stage::Load, BAD_READING));
        }
        let cap = limit * (1.0 + PLAN_TOL);
        let named = |r: &Reading| {
            over.iter().any(|o| (o.at.x - r.at.x).abs() <= KERNEL_TOL && (o.at.y - r.at.y).abs() <= KERNEL_TOL && !o.reason.is_empty())
        };
        if let Some(r) = readings.iter().find(|r| r.load > cap && !named(r)) {
            return Err(Refusal::new(Stage::Load, UNNAMED_OVERLOAD).at(r.at));
        }
        Ok(Self { set, readings, over })
    }
    #[must_use]
    pub fn set(&self) -> &OrderedSet {
        &self.set
    }
    #[must_use]
    pub fn readings(&self) -> &[Reading] {
        &self.readings
    }
    #[must_use]
    pub fn over(&self) -> &[OverLimit] {
        &self.over
    }
}

/// The in-process stock as the stages read it. Motion implements it, so
/// the kernel has no edge to the simulator.
pub trait StockRead {
    /// True when material stands at `p` in the plan's plane; `None` when the
    /// reader does not know (the stage must then not claim a proof).
    fn material_at(&self, p: Point2) -> Option<bool>;
}

/// A load planner.
pub trait LoadPlanner {
    fn control(&self, s: OrderedSet, r: &KernelRegion, stock: &dyn StockRead, lc: &LoadControl, tool_r: f64)
    -> Result<Verdict<Controlled>, Refusal>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guide::GuideSet;
    use crate::order::StartSide;

    fn empty_set() -> OrderedSet {
        OrderedSet::new(&GuideSet::default(), vec![], Point2::new(0.0, 0.0), StartSide::Inside).unwrap()
    }

    #[test]
    fn a_load_choice_is_checked_when_the_recipe_is_made() {
        assert!(LoadControl::None.validate().is_ok());
        assert!(LoadControl::Trochoidal { width: 6.0, pitch: 1.0 }.validate().is_ok());
        assert_eq!(LoadControl::Trochoidal { width: 1.0, pitch: 1.0 }.validate().unwrap_err().reason, BAD_TROCHOID);
        assert!(LoadControl::Trochoidal { width: 0.0, pitch: 0.0 }.validate().is_err());
        assert!(LoadControl::ConstantEngagement { load: 0.3, reach: Reach::Stock }.validate().is_ok());
        assert_eq!(LoadControl::ConstantEngagement { load: 0.0, reach: Reach::Rings }.validate().unwrap_err().reason, BAD_LOAD);
        assert!(LoadControl::ConstantEngagement { load: 1.5, reach: Reach::Rings }.validate().is_err());
    }

    #[test]
    fn a_cut_over_the_limit_must_be_named() {
        let p = Point2::new(2.0, 3.0);
        let hot = vec![Reading { at: p, load: 1.0 }];
        let e = Controlled::new(empty_set(), hot.clone(), vec![], 0.5).unwrap_err();
        assert_eq!((e.reason, e.at), (UNNAMED_OVERLOAD, Some(p)));
        let over = vec![OverLimit { at: p, load: 1.0, reason: "full-width first ring" }];
        let c = Controlled::new(empty_set(), hot, over, 0.5).unwrap();
        assert_eq!(c.over().len(), 1);
        // Within the tolerance band is not over the limit.
        assert!(Controlled::new(empty_set(), vec![Reading { at: p, load: 0.52 }], vec![], 0.5).is_ok());
        assert_eq!(Controlled::new(empty_set(), vec![Reading { at: p, load: f64::NAN }], vec![], 0.5).unwrap_err().reason, BAD_READING);
    }
}
