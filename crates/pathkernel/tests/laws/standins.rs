//! The stand-in stage planners of the K7 suite (#4027). No kind plans on
//! the kernel yet, so these small planners prove the law checks: the ones
//! here obey every law; each law file adds the planners that break its law
//! in one named way. They are test code, not a design of the K2-K5 stages.
//! Provenance: our own design record; sharp offset rings of a pocket as in
//! Held (1991).

use camcraft_pathkernel::entry::{Area, Entry, EntryChoice, EntryKind, EntryPlanner, Exit, Proof};
use camcraft_pathkernel::guide::{Curve, Family, GuideChoice, GuidePlanner, GuideSet, RingRef, RingsChoice};
use camcraft_pathkernel::link::{Hop, LinkPolicy, LinkProof, LinkTally, USER_RETRACT};
use camcraft_pathkernel::load::{Controlled, LoadControl, LoadPlanner, OverLimit, PLAN_TOL, Reach, Reading, StockRead};
use camcraft_pathkernel::order::{Direction, OrderChoice, OrderPlanner, OrderedSet, StartSide};
use camcraft_pathkernel::plan::Cut;
use camcraft_pathkernel::region::{KernelRegion, MaterialSide, PlanarRegion};
use camcraft_pathkernel::{AxisControl, Loop, PlanePoint, Point2, Positive, Recipe, Refusal, Region, Stage, ToolShape, Verdict, WorkPlane};

use super::grid::samples;
use super::kit::{Case, Planners, dist, outside, planar, poly, poly_dist};

/// The tool radius of the fixtures (mm).
pub const R: f64 = 2.0;
/// The ring stepover of the fixtures (mm): ae/D 0.2.
pub const STEP: f64 = 0.8;
/// The engagement limit of the fixtures (ae/D).
pub const LIMIT: f64 = 0.5;
/// The most rings that a stand-in plans.
const MAX_RINGS: u32 = 200;

pub const GEO_FAILED: &str = "the stand-in geometry failed";
pub const RINGS_ONLY: &str = "the stand-in guide plans rings only";
pub const LEAVES_AREA: &str = "the straight hop leaves the cutter-centre area";
pub const FRESH: &str = "a full-width cut where no ring cut before";

fn geo<E>(_: E) -> Refusal {
    Refusal::new(Stage::Region, GEO_FAILED)
}

pub fn sq(x0: f64, y0: f64, s: f64) -> Result<Loop, Refusal> {
    let p = Point2::new;
    Loop::new(vec![p(x0, y0), p(x0 + s, y0), p(x0 + s, y0 + s), p(x0, y0 + s)]).map_err(geo)
}

pub fn square_area(c: Point2, s: f64) -> Result<Region, Refusal> {
    Region::new(vec![sq(c.x - s / 2.0, c.y - s / 2.0, s)?]).map_err(geo)
}

/// The region of a pocket `part` (the area to clear). Inside: the
/// cutter-centre area is `part` less the tool radius. Outside: a band
/// around `part`, so that the material side changes the centre.
fn pocket_region(part: Region, m: MaterialSide) -> Result<KernelRegion, Refusal> {
    let tool = ToolShape::disc(R, 0.0, false, 20.0)?;
    let center = match m {
        MaterialSide::Inside => part.offset_sharp(-R).map_err(geo)?,
        MaterialSide::Outside => part.offset_sharp(R + 6.0).map_err(geo)?.difference(&part.offset_sharp(R).map_err(geo)?).map_err(geo)?,
    };
    // The part walls are the boundary loops of the cutter-centre area (the
    // partition check of #4025), not the outline of `part`.
    let walls = center.loops().to_vec();
    PlanarRegion::new(WorkPlane::setup_xy(0.0)?, center, m, walls, vec![], Region::empty(), Region::empty(), tool)
        .map(KernelRegion::Planar)
}

/// A 20 mm square pocket: concentric rings, no split ring.
pub fn square_region(m: MaterialSide) -> Result<KernelRegion, Refusal> {
    pocket_region(Region::new(vec![sq(0.0, 0.0, 20.0)?]).map_err(geo)?, m)
}

/// A 30 mm square pocket with a 10 mm square island in its middle.
pub fn island_region(m: MaterialSide) -> Result<KernelRegion, Refusal> {
    pocket_region(Region::new(vec![sq(0.0, 0.0, 30.0)?, sq(10.0, 10.0, 10.0)?]).map_err(geo)?, m)
}

pub fn rings_choice() -> Result<GuideChoice, Refusal> {
    Ok(GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 0.0, STEP, None, None)?))
}

pub fn recipe(guide: GuideChoice, entry: EntryKind, link: LinkPolicy) -> Result<Recipe, Refusal> {
    let load = LoadControl::constant(LIMIT, Reach::Stock)?;
    let entry = EntryChoice::new(entry, 2.0, vec![])?;
    Recipe::new(WorkPlane::setup_xy(0.0)?, AxisControl::Normal, guide, load, OrderChoice::default(), link, entry)
}

/// The planner's own stock reader: the stock area, read point by point.
pub struct ShapeReader(pub Region);

impl StockRead for ShapeReader {
    fn material_at(&self, p: Point2) -> Option<bool> {
        Some(self.0.contains(p))
    }
}

fn case(name: &'static str, region: fn(MaterialSide) -> Result<KernelRegion, Refusal>, side: f64, no_split: bool) -> Result<Case, Refusal> {
    let stock = Region::new(vec![sq(0.0, 0.0, side)?]).map_err(geo)?;
    let recipe = recipe(rings_choice()?, EntryKind::Helix, LinkPolicy::StayWhereProved)?;
    let reader = Box::new(ShapeReader(stock.clone()));
    Ok(Case { name, region, material: MaterialSide::Inside, recipe, stock, reader, no_split, recorded: None })
}

pub fn square_case() -> Result<Case, Refusal> {
    case("square pocket", square_region, 20.0, true)
}

pub fn island_case() -> Result<Case, Refusal> {
    case("pocket with an island", island_region, 30.0, false)
}

/// The planner set that obeys every law.
pub fn good() -> Planners {
    Planners {
        guide: Box::new(RingGuide { extra: 0.0 }),
        order: Box::new(SideOrder),
        load: Box::new(ReadLoad { blind: false }),
        link: Box::new(StraightLink),
        entry: Box::new(ProvedEntry),
    }
}

pub fn loop_curve(l: &Loop, k: u32) -> Result<Curve, Refusal> {
    let pts = l.points().iter().map(|p| PlanePoint::flat(*p)).collect::<Result<Vec<_>, _>>()?;
    Curve::new(pts, None, true, k, Family::Ring)
}

/// The sharp offset rings of `center`, from `first` inward each `step`.
pub fn rings(center: &Region, first: f64, step: f64, count: Option<u32>) -> Result<Vec<Curve>, Refusal> {
    let mut out = Vec::new();
    for k in 0..count.unwrap_or(MAX_RINGS) {
        let ring = center.offset_sharp(-(first + f64::from(k) * step)).map_err(geo)?;
        if ring.is_empty() {
            break;
        }
        for l in ring.loops() {
            out.push(loop_curve(l, k)?);
        }
    }
    Ok(out)
}

/// Rings of the centre area; `extra` adds to the stepover (0 obeys).
pub struct RingGuide {
    pub extra: f64,
}

impl GuidePlanner for RingGuide {
    fn guide(&self, r: &KernelRegion, g: &GuideChoice, _tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
        let GuideChoice::Rings(rc) = g else {
            return Err(Refusal::new(Stage::Guide, RINGS_ONLY));
        };
        let curves = rings(planar(r)?.center(), rc.first_offset(), rc.stepover() + self.extra, rc.count())?;
        Ok(Verdict::done(GuideSet::new(curves)))
    }
}

/// `c` in the other direction, from the same start point.
pub fn reversed(c: &Curve) -> Result<Curve, Refusal> {
    let mut p = c.pts().to_vec();
    if c.closed() {
        p[1..].reverse();
    } else {
        p.reverse();
    }
    Curve::new(p, c.arc(), c.closed(), c.k(), c.family())
}

/// Inside: the highest ring index first; Outside: the lowest first. Auto
/// resolves to Inside. Conventional reverses each curve.
pub fn order_curves(input: &GuideSet, mut curves: Vec<Curve>, side: StartSide, dir: Direction) -> Result<OrderedSet, Refusal> {
    curves.sort_by_key(Curve::k);
    if side == StartSide::Inside {
        curves.reverse();
    }
    if dir == Direction::Conventional {
        curves = curves.iter().map(reversed).collect::<Result<_, _>>()?;
    }
    let start = curves.first().and_then(|c| c.pts().first()).map_or(Point2::new(0.0, 0.0), |p| p.at());
    OrderedSet::new(input, curves, start, side)
}

pub fn resolve(s: StartSide) -> StartSide {
    if s == StartSide::Outside { StartSide::Outside } else { StartSide::Inside }
}

pub struct SideOrder;

impl OrderPlanner for SideOrder {
    fn order(&self, g: GuideSet, _r: &KernelRegion, o: &OrderChoice, _stock: Option<&Region>) -> Result<Verdict<OrderedSet>, Refusal> {
        Ok(Verdict::done(order_curves(&g, g.curves().to_vec(), resolve(o.start), o.direction)?))
    }
}

/// Reads each cut every half tool radius: a full-width slot where no
/// earlier cut is within a diameter and within one diameter of a corner,
/// else the width left by the nearest earlier cut. `blind` reads every cut
/// as a light side cut (an L6 breaker).
pub struct ReadLoad {
    pub blind: bool,
}

impl LoadPlanner for ReadLoad {
    fn control(
        &self,
        s: OrderedSet,
        _r: &KernelRegion,
        stock: &dyn StockRead,
        lc: &LoadControl,
        tool_r: f64,
    ) -> Result<Verdict<Controlled>, Refusal> {
        let limit = match lc {
            LoadControl::ConstantEngagement(c) => c.load(),
            _ => 1.0,
        };
        let cap = limit * (1.0 + PLAN_TOL);
        let lines: Vec<Vec<Point2>> = s.curves().iter().map(poly).collect();
        let (mut readings, mut over) = (Vec::new(), Vec::new());
        for (i, line) in lines.iter().enumerate() {
            let mut pts = samples(line, tool_r / 2.0);
            // A closed curve ends at its start: one reading for one place.
            if s.curves()[i].closed() {
                pts.pop();
            }
            for (p, _) in pts {
                let near = lines[..i].iter().map(|e| poly_dist(p, e)).fold(f64::INFINITY, f64::min);
                // The engagement climbs at a ring corner: read it as a slot there.
                let corner = line.iter().any(|v| dist(*v, p) <= 2.0 * tool_r);
                let load = match (self.blind, stock.material_at(p)) {
                    (true, _) => 0.2,
                    (false, Some(true)) if corner => 1.0,
                    (false, Some(true)) => (near / (2.0 * tool_r)).min(1.0),
                    (false, _) => 0.0,
                };
                readings.push(Reading::new(p, load)?);
                if load > cap {
                    over.push(OverLimit::new(p, load, FRESH)?);
                }
            }
        }
        Ok(Verdict::done(Controlled::new(s, readings, over, limit)?))
    }
}

pub fn start_of(c: &Cut) -> Point2 {
    c.curve().pts().first().map_or(Point2::new(0.0, 0.0), |p| p.at())
}

pub fn end_of(c: &Cut) -> Point2 {
    poly(c.curve()).last().copied().unwrap_or(Point2::new(0.0, 0.0))
}

/// Stays on the straight hop where it stays in the centre area, else lifts.
pub struct StraightLink;

impl LinkProof for StraightLink {
    fn link(&self, from: &Cut, to: &Cut, r: &KernelRegion, _planned: &[Cut], policy: LinkPolicy, tally: &mut LinkTally) -> Hop {
        let (a, b) = (end_of(from), start_of(to));
        let inside = |p: &PlanarRegion| samples(&[a, b], 0.05).iter().all(|(q, _)| outside(p.center(), *q) <= 1e-3);
        let hop = match (policy, planar(r)) {
            (LinkPolicy::StayWhereProved, Ok(p)) if inside(p) => Hop::stay(vec![a, b]),
            (LinkPolicy::StayWhereProved, _) => Hop::lift(LEAVES_AREA),
            _ => Hop::lift(USER_RETRACT),
        };
        tally.record(hop.or_else(|_| Hop::lift(LEAVES_AREA)).expect("a named lift"))
    }
}

/// A helix where the tool enters material (the area that the plan has not
/// cleared and that is not air), with its proof.
pub struct ProvedEntry;

pub fn helix(at: Point2, r: &KernelRegion, e: &EntryChoice, kind: EntryKind) -> Result<Entry, Refusal> {
    let tool = planar(r)?.tool();
    let area = Area::new(square_area(at, 4.0 * tool.radius())?)?;
    let proof = Proof::IntoMaterial { area, angle_or_pitch: Positive::new(Stage::Entry, 1.0)? };
    Entry::new(kind, vec![PlanePoint::new(at, 5.0)?], vec![PlanePoint::flat(at)?], proof, e, tool)
}

impl EntryPlanner for ProvedEntry {
    fn entry(&self, first: &Cut, r: &KernelRegion, cleared: &Region, e: &EntryChoice) -> Result<Verdict<Entry>, Refusal> {
        proved_entry(first, r, cleared, e).map(Verdict::done)
    }
    fn exit(&self, _last: &Cut, _r: &KernelRegion, _cleared: &Region) -> Result<Exit, Refusal> {
        Exit::new(None, 5.0)
    }
}

fn proved_entry(first: &Cut, r: &KernelRegion, cleared: &Region, e: &EntryChoice) -> Result<Entry, Refusal> {
    let at = start_of(first);
    let tool = planar(r)?.tool();
    let free = planar(r)?.air().union(cleared).map_err(geo)?;
    if free.contains(at) {
        let area = Area::new(square_area(at, tool.radius())?)?;
        return Entry::new(EntryKind::Air, vec![], vec![PlanePoint::flat(at)?], Proof::InAir(area), e, tool);
    }
    helix(at, r, e, EntryKind::Helix)
}
