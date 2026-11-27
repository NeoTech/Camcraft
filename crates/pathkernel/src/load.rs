//! Stage 3, Load control (K3, #4026): the load choice, the readings and the
//! over-limit record, and the stock reader. The `Controlled` constructor
//! checks law 6: each cut read above the limit is named, and each name
//! agrees with a reading. The planner is [`LoadStage`] in `load/` (#4026):
//! the engagement law (`law.rs`), constant engagement on rings
//! (`rings.rs`) and trochoidal loops (`trochoid.rs`).
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`)
//! "Stage 3: Load control"; radial engagement as in Stori & Wright (2000)
//! and Held & Spielberger (2009).

mod law;
mod rings;
mod stage;
mod trochoid;

pub use law::NO_RASTER;
pub use rings::{AT_FLOOR, LOAD_GEOMETRY, LOAD_GEOMETRY_INVALID, NOT_A_RING, NOT_FIXED, TOO_MANY_PASSES};
pub use stage::{DEFAULT_LIMIT, FRESH_CUT, LOAD_NOT_PLANAR, LoadStage, NO_CONTROL, SET_NOT_KEPT, STOCK_REACH_ELSEWHERE};
pub use trochoid::{LOOP_LEAVES, LOOPS_NOT_A_CURVE, LOOPS_OVER, NO_ROOM, NOT_MARKED, SHAPE_KEPT};

use crate::frame::{Positive, Spot};
use crate::order::OrderedSet;
use crate::region::KernelRegion;
use crate::verdict::{NO_REASON, Refusal, Stage, Verdict};
use crate::{KERNEL_TOL, Point2};

/// The relative tolerance of the load limit: a cut above
/// `load x (1 + PLAN_TOL)` is over the limit.
pub const PLAN_TOL: f64 = 0.05;

/// Refusal reason: an engagement out of its domain.
pub const BAD_LOAD: &str = "an engagement ae/D must be a finite number from 0 to 1 (a limit: above 0)";
/// Refusal reason: a trochoid whose pitch is not below its width.
pub const BAD_TROCHOID: &str = "a trochoid needs a positive width and a positive pitch below the width";

/// A radial engagement ae/D from 0 to 1 (1 is a full-width slot).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Engagement(f64);

impl Engagement {
    pub fn new(stage: Stage, v: f64) -> Result<Self, Refusal> {
        if v.is_finite() && (0.0..=1.0).contains(&v) { Ok(Self(v)) } else { Err(Refusal::new(stage, BAD_LOAD)) }
    }
    /// A limit: an engagement above 0.
    pub fn limit(stage: Stage, v: f64) -> Result<Self, Refusal> {
        let e = Self::new(stage, v)?;
        if e.0 > 0.0 { Ok(e) } else { Err(Refusal::new(stage, BAD_LOAD)) }
    }
    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}

/// What the constant-engagement control reads the cut material from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Reach {
    /// The rings before (Offset rings).
    Rings,
    /// The in-process stock (Adaptive).
    Stock,
}

/// Trochoidal loops along a curve: a positive width and a positive pitch below it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trochoid {
    width: Positive,
    pitch: Positive,
}

impl Trochoid {
    pub fn new(width: f64, pitch: f64) -> Result<Self, Refusal> {
        let (w, p) = (Positive::new(Stage::Recipe, width), Positive::new(Stage::Recipe, pitch));
        match (w, p) {
            (Ok(w), Ok(p)) if p < w => Ok(Self { width: w, pitch: p }),
            _ => Err(Refusal::new(Stage::Recipe, BAD_TROCHOID)),
        }
    }
    #[must_use]
    pub fn width(&self) -> f64 {
        self.width.get()
    }
    #[must_use]
    pub fn pitch(&self) -> f64 {
        self.pitch.get()
    }
}

/// How the load on the tool is held. Each variant with data holds a
/// checked value.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum LoadControl {
    /// The identity: the curves stay; the load is still read and named.
    #[default]
    None,
    Trochoidal(Trochoid),
    ConstantEngagement(ConstantLoad),
}

/// A constant engagement: a limit above 0 and at most 1, read from `Reach`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConstantLoad {
    load: Engagement,
    reach: Reach,
}

impl ConstantLoad {
    pub fn new(load: f64, reach: Reach) -> Result<Self, Refusal> {
        Ok(Self { load: Engagement::limit(Stage::Recipe, load)?, reach })
    }
    #[must_use]
    pub fn load(&self) -> f64 {
        self.load.get()
    }
    #[must_use]
    pub fn reach(&self) -> Reach {
        self.reach
    }
}

impl LoadControl {
    /// A constant engagement of `load` (above 0, at most 1).
    pub fn constant(load: f64, reach: Reach) -> Result<Self, Refusal> {
        ConstantLoad::new(load, reach).map(Self::ConstantEngagement)
    }
}

/// One load reading: the engagement at a finite place of a cut.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    at: Spot,
    load: Engagement,
}

impl Reading {
    pub fn new(at: Point2, load: f64) -> Result<Self, Refusal> {
        Ok(Self { at: Spot::new(Stage::Load, at)?, load: Engagement::new(Stage::Load, load)? })
    }
    #[must_use]
    pub fn at(&self) -> Point2 {
        self.at.get()
    }
    #[must_use]
    pub fn load(&self) -> f64 {
        self.load.get()
    }
}

/// A cut read above the limit: its reading and a reason that is not blank.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverLimit {
    reading: Reading,
    reason: &'static str,
}

impl OverLimit {
    pub fn new(at: Point2, load: f64, reason: &'static str) -> Result<Self, Refusal> {
        if reason.trim().is_empty() {
            return Err(Refusal::new(Stage::Load, NO_REASON));
        }
        Ok(Self { reading: Reading::new(at, load)?, reason })
    }
    #[must_use]
    pub fn at(&self) -> Point2 {
        self.reading.at()
    }
    #[must_use]
    pub fn load(&self) -> f64 {
        self.reading.load()
    }
    #[must_use]
    pub fn reason(&self) -> &'static str {
        self.reason
    }
}

/// The ordered set after load control, with its readings and every cut
/// above the limit.
#[derive(Clone, Debug, PartialEq)]
pub struct Controlled {
    set: OrderedSet,
    readings: Vec<Reading>,
    over: Vec<OverLimit>,
    limit: Engagement,
}

/// Refusal reason: a reading above the limit with no over-limit record (law 6).
pub const UNNAMED_OVERLOAD: &str = "a cut above the load limit must be named in the over-limit list";
/// Refusal reason: an over-limit record that names no reading above the limit.
pub const STRAY_OVERLOAD: &str = "an over-limit record must name a reading above the limit, at its place and with its load";
/// Refusal reason: two over-limit records for one reading.
pub const DOUBLE_OVERLOAD: &str = "a reading above the limit must be named once";

fn same(r: &Reading, o: &OverLimit) -> bool {
    crate::near(r.at(), o.at()) && (r.load() - o.load()).abs() <= KERNEL_TOL
}

impl Controlled {
    /// Every record is valid by its type. Refused when the limit is not in
    /// (0, 1], when a reading above `limit x (1 + PLAN_TOL)` is not named
    /// exactly once, or when an over-limit record does not agree with a
    /// reading above the limit (same place and load).
    pub fn new(set: OrderedSet, readings: Vec<Reading>, over: Vec<OverLimit>, limit: f64) -> Result<Self, Refusal> {
        let limit = Engagement::limit(Stage::Load, limit)?;
        let cap = limit.get() * (1.0 + PLAN_TOL);
        let hot = |r: &&Reading| r.load() > cap;
        for r in readings.iter().filter(hot) {
            match over.iter().filter(|o| same(r, o)).count() {
                0 => return Err(Refusal::new(Stage::Load, UNNAMED_OVERLOAD).at(r.at())),
                1 => {}
                _ => return Err(Refusal::new(Stage::Load, DOUBLE_OVERLOAD).at(r.at())),
            }
        }
        if let Some(o) = over.iter().find(|o| !readings.iter().filter(hot).any(|r| same(r, o))) {
            return Err(Refusal::new(Stage::Load, STRAY_OVERLOAD).at(o.at()));
        }
        Ok(Self { set, readings, over, limit })
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
    #[must_use]
    pub fn limit(&self) -> f64 {
        self.limit.get()
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
