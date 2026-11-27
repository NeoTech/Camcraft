//! 2D Pocket on the path kernel (#3752, the first recipe of K6, #4023):
//! the op's parameters become one choice for each stage, each Z level is
//! planned by the stages with that level's heights and stock state
//! (`kernel_level`), and the converter (`kernel_emit`) makes the plans of
//! the levels into one toolpath. Only the planner switch calls it; with
//! the switch at its default the old pocket planner runs, unchanged.
//!
//! The recipe (record "Map of the present operations"):
//! - Waterline: rings of the boundary at the stepover, no load control
//!   (each cut above the stated load is named).
//! - Offset rings: rings at the load step, constant engagement on the
//!   rings (passes inserted where a ring cuts above the load).
//! - Zigzag: refused by name (#3753): the rows of the kernel and their
//!   closing ring cross (the verifier reads 30 crossings on a square pocket).
//! - Auto: Waterline or Zigzag by the outline, the rule of the old planner.
//! - Adaptive: refused (the 2D Pocket retired it; the stock reach of the
//!   load stage is the HSM planner's, not this recipe's).
//!
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! ("The strategy is a recipe", the map row of 2D Pocket) and
//! `docs/decisions/3752-kernel-pocket-recipe.md`; offset pocketing as in
//! Held (1991). No vendor material consulted.

use crate::cl::Toolpath;
use crate::entry::Entry3D;
use crate::geo2d::{Point2, Region};
use crate::geometry::Point3;
use crate::kernel_emit::{EmitFeeds, EmitHeights, LevelPlan, emit_levels};
use crate::links::Link;
use camcraft_pathkernel::entry::{ArcChoice, EntryChoice, EntryKind, EntryStage, Heights, Hole, MaterialChoice};
use camcraft_pathkernel::guide::{GuideChoice, RingRef, RingsChoice};
use camcraft_pathkernel::link::LinkPolicy;
use camcraft_pathkernel::plan::{Plan, Step};
use camcraft_pathkernel::load::{LoadControl, LoadStage, Reach, StockRead};
use camcraft_pathkernel::order::{Direction, OrderChoice, StartSide as KernelSide};
use camcraft_pathkernel::{AxisControl, Recipe, Refusal, ToolShape, WorkPlane};

use super::arcs::ARC_FIT_TOL;
use super::equal_steps::cut_levels_for;
use super::face::GenError;
use super::kernel_level::{LevelStages, PlannedLevel, plan_level};
use super::kernel_region::{bounds_of, pocket_region};
use super::pass_start::{PassStart, StartSide};
use super::pocket::{PocketParams, PocketStrategy};
use super::predrilled::PreHole;

/// The status line note of each 2D Pocket that the path kernel planned.
pub const KERNEL_POCKET_NOTE: &str = "planner: the path kernel planned this 2D Pocket (CAMCRAFT_PLANNER=new)";
/// Refusal: the Adaptive strategy on the kernel route.
pub const KERNEL_POCKET_ADAPTIVE: &str = "the path kernel does not plan the 2D Pocket Adaptive strategy: the 2D Pocket retired it - use the 2D Adaptive operation";
/// Refusal: the Zigzag strategy on the kernel route (#3753 plans the rows).
pub const KERNEL_POCKET_ZIGZAG: &str = "the new path planner does not plan the 2D Pocket Zigzag strategy yet (#3753): its rows and their closing ring cross - select Waterline or Offset rings, or unset CAMCRAFT_PLANNER";
/// Refusal: a level that the stages planned only in part.
pub const KERNEL_POCKET_INCOMPLETE: &str = "the path kernel planned this level only in part: an area stays uncut";
/// Note: the corner blends are not planned by the kernel yet.
pub const KERNEL_NO_BLEND: &str =
    "kernel recipe: sharp corners instead of the blended turns - the path kernel does not blend corners yet";
/// The chord tolerance (mm) of the entry stage: the arc leads and helices are chords.
const ENTRY_CHORD_TOL: f64 = 0.01;
/// The height (mm) above the material where a descent starts, when the op sets none.
const DEFAULT_CLEAR: f64 = 1.0;

/// The stock that stands at the depth of a level: the pocket area. A 2D
/// pocket has vertical walls, so each level reads the same area; the cut
/// of the level above stops at its own depth and clears nothing below it.
struct AreaStock<'a>(&'a Region);

impl StockRead for AreaStock<'_> {
    fn material_at(&self, p: Point2) -> Option<bool> {
        Some(self.0.contains(p))
    }
}

fn refusal(e: &Refusal, z: f64) -> GenError {
    let msg = format!("2D Pocket on the path kernel: {e}");
    match e.place() {
        Some(p) => GenError::invalid_at(msg, Point3::new(p.x, p.y, z)),
        None => GenError::Invalid(msg),
    }
}

/// The strategy that the pocket cuts: Auto takes the rule of the old
/// planner (the outermost loop decides, holes and islands never do).
#[must_use]
pub fn resolved_strategy(region: &Region, p: &PocketParams) -> PocketStrategy {
    match p.strategy {
        PocketStrategy::Auto => {
            let outer = region
                .loops()
                .iter()
                .max_by(|a, b| a.area().abs().total_cmp(&b.area().abs()))
                .map(|l| l.points())
                .unwrap_or(&[]);
            p.strategy.resolve(outer, p.stepover, p.tool_diameter, ARC_FIT_TOL)
        }
        s => s,
    }
}

/// The guide, the load choice and the stated load limit (ae/D) of a strategy.
fn guide_and_load(strategy: PocketStrategy, p: &PocketParams) -> Result<(GuideChoice, LoadControl, f64), GenError> {
    let k = |e: Refusal| refusal(&e, p.top_z);
    let per_d = |s: f64| (s / p.tool_diameter).clamp(1e-6, 1.0);
    match strategy {
        PocketStrategy::Waterline | PocketStrategy::Auto => {
            let g = RingsChoice::new(RingRef::Boundary, 0.0, p.stepover, None, None).map_err(k)?;
            Ok((GuideChoice::Rings(g), LoadControl::None, per_d(p.stepover)))
        }
        PocketStrategy::OffsetRings => {
            let step = p.tiling_step();
            let g = RingsChoice::new(RingRef::Boundary, 0.0, step, None, None).map_err(k)?;
            let load = LoadControl::constant(p.optimal_load.clamp(1e-6, 1.0), Reach::Rings).map_err(k)?;
            Ok((GuideChoice::Rings(g), load, per_d(step)))
        }
        PocketStrategy::Zigzag if p.strategy == PocketStrategy::Auto => Err(GenError::Invalid(format!("{KERNEL_POCKET_ZIGZAG} (Auto chose Zigzag for this outline)"))),
        PocketStrategy::Zigzag => Err(GenError::Invalid(KERNEL_POCKET_ZIGZAG.into())),
        PocketStrategy::Adaptive => Err(GenError::Invalid(KERNEL_POCKET_ADAPTIVE.into())),
    }
}

/// The kernel's start side of a pass start (the start side reaches the
/// curves only through the order stage, law 3).
#[must_use]
pub fn kernel_side(start: PassStart) -> KernelSide {
    match start {
        PassStart::Auto => KernelSide::Auto,
        PassStart::Inside => KernelSide::Inside,
        PassStart::Outside => KernelSide::Outside,
    }
}

/// The link policy of the op: the rings stay at depth where a link is
/// proved, and lift with a named reason elsewhere. Only a pass link to the
/// clearance height is the user's own retract between cuts.
#[must_use]
pub fn link_policy(pass: Link) -> LinkPolicy {
    match pass {
        Link::Clearance => LinkPolicy::Retract,
        Link::Retract | Link::KeepDown => LinkPolicy::StayWhereProved,
    }
}

/// The descent angle (degrees) of a helix entry: the op's angle, else the
/// angle of its pitch at the helix radius, else the old law of one turn
/// per helix diameter of drop (atan(1 / pi), 17.7 degrees).
fn helix_angle(p: &PocketParams) -> f64 {
    let a = &p.links.approach;
    let deg = if a.helix_angle > 0.0 {
        a.helix_angle
    } else if a.helix_pitch > 0.0 {
        (a.helix_pitch / (std::f64::consts::TAU * p.helix_radius)).atan().to_degrees()
    } else {
        std::f64::consts::FRAC_1_PI.atan().to_degrees()
    };
    deg.clamp(0.1, 89.0)
}

/// The descent angle (degrees) of a ramp entry.
fn ramp_angle(p: &PocketParams) -> f64 {
    let a = &p.links.approach;
    let deg = if a.ramp_angle > 0.0 { a.ramp_angle } else { p.links.ramp_angle };
    if deg > 0.0 { deg.clamp(0.1, 89.0) } else { camcraft_pathkernel::entry::DEFAULT_DESCENT_ANGLE }
}

/// The entry choice of a level for the op's entry `e` and the holes open
/// down to the depth `z`. A plunge into material is not offered: the op
/// asks for an entry outside material (air, a hole, a cleared column),
/// and a closed pocket without one is refused by name. A helix or a ramp
/// that the user selected is used where the tool must enter material.
fn entry_choice(e: Entry3D, p: &PocketParams, holes: &[PreHole], z: f64) -> Result<EntryChoice, Refusal> {
    let r = 0.5 * p.tool_diameter;
    let open: Vec<Hole> = holes
        .iter()
        .filter(|h| h.at.x.is_finite() && h.at.y.is_finite() && h.floor.is_finite() && h.floor <= z + 1e-6)
        .map(|h| Hole::new(h.at, r))
        .collect::<Result<_, _>>()?;
    let (asked, material) = match e {
        Entry3D::Helix => (EntryKind::Helix, MaterialChoice::new(helix_angle(p), Some(p.helix_radius))?),
        Entry3D::Ramp => (EntryKind::Ramp, MaterialChoice::new(ramp_angle(p), None)?),
        Entry3D::Arc => (EntryKind::ArcLead, MaterialChoice::default()),
        Entry3D::Plunge => (EntryKind::Air, MaterialChoice::default()),
    };
    Ok(EntryChoice::new(asked, p.links.approach.approach_distance.max(0.0), open)?.with_material(material))
}

/// The area that the tool disc of a kernel pocket may sweep, for the
/// verifier: the pocket area, and round a part (`around_part`, the stock
/// known) also the air round the stock where the entry stage plans its air
/// entries, arc leads and exits. The part stays out of it, so each part
/// wall is still judged; the stock edge is no wall (no material stands
/// beyond it).
pub fn kernel_pocket_reach(region: &Region, p: &PocketParams, side: &StartSide) -> Result<Region, GenError> {
    let Some(stock) = side.stock.as_ref().filter(|_| side.around_part) else {
        return Ok(region.clone());
    };
    let a = &p.links.approach;
    let arc_r = if a.arc_radius > 0.0 { a.arc_radius } else { 0.5 * p.tool_diameter };
    let margin = a.approach_distance.max(0.0) + arc_r + 2.0 * p.tool_diameter;
    let air = stock.offset(margin)?.difference(stock)?;
    Ok(region.union(&air)?)
}

/// The tolerance (mm) of an approach point on a cut of the level above.
const ON_CUT_TOL: f64 = 1e-3;

/// True when the approach of each entry of `plan` stands over a cut of
/// each level above (`uppers`: the tool disc cut that column down to the
/// floor of each of them), or in the air round the stock (outside
/// `air_out`, the stock grown by the tool radius). Then the entries of the
/// level can start over the floor of the level above.
fn approach_proved(plan: &Plan, uppers: &[&Plan], air_out: Option<&Region>) -> bool {
    let polylines = |above: &Plan| -> Vec<Vec<Point2>> {
        above
            .steps()
            .iter()
            .filter_map(|s| match s {
                Step::Cut(c) => {
                    let mut pts: Vec<Point2> = c.curve().pts().iter().map(|q| q.at()).collect();
                    if c.curve().closed() {
                        pts.extend(pts.first().copied());
                    }
                    Some(pts)
                }
                _ => None,
            })
            .collect()
    };
    let levels: Vec<Vec<Vec<Point2>>> = uppers.iter().map(|u| polylines(u)).collect();
    let on_cut = |cuts: &[Vec<Point2>], q: Point2| cuts.iter().any(|c| c.windows(2).any(|w| seg_dist(q, w[0], w[1]) <= ON_CUT_TOL));
    let cleared = |q: Point2| !levels.is_empty() && levels.iter().all(|cuts| on_cut(cuts, q));
    let in_air = |q: Point2| air_out.is_some_and(|a| !a.contains(q));
    plan.steps().iter().all(|s| match s {
        Step::Entry(e) => e.approach().iter().all(|q| cleared(q.at()) || in_air(q.at())),
        _ => true,
    })
}

fn seg_dist(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    (p.x - (a.x + t * dx)).hypot(p.y - (a.y + t * dy))
}

/// The plans of a 2D Pocket on the path kernel: one for each Z level with
/// its work plane and its proved material top (`None`: the stock top), and
/// the status line notes of the recipe and the stages.
#[derive(Clone, Debug)]
pub struct KernelPocketPlan {
    pub levels: Vec<(PlannedLevel, WorkPlane, Option<f64>)>,
    pub notes: Vec<String>,
    pub strategy: PocketStrategy,
    heights: EmitHeights,
    feeds: EmitFeeds,
}

/// The 2D Pocket of `region` (the material to remove; its holes are
/// islands) on the path kernel, with the pass start and the stock of `side`
/// and the holes that the caller proved open (`holes`, read only when the
/// op asks for pre-drilled entries).
pub fn pocket_on_kernel(region: &Region, p: &PocketParams, holes: &[PreHole], side: &StartSide) -> Result<Toolpath, GenError> {
    let plan = pocket_kernel_plans(region, p, holes, side)?;
    let levels: Vec<LevelPlan<'_>> = plan.levels.iter().map(|(l, plane, top_z)| LevelPlan { plan: &l.plan, plane: *plane, top_z: *top_z }).collect();
    let mut path = emit_levels(&levels, plan.heights, plan.feeds)
        .map_err(|e| GenError::Invalid(format!("2D Pocket on the path kernel: {e}")))?
        .path;
    for n in plan.notes {
        if !path.notes.contains(&n) {
            path.notes.push(n);
        }
    }
    Ok(path)
}

/// The plans of [`pocket_on_kernel`], level by level, before the converter.
pub fn pocket_kernel_plans(region: &Region, p: &PocketParams, holes: &[PreHole], side: &StartSide) -> Result<KernelPocketPlan, GenError> {
    p.validate()?;
    if region.is_empty() {
        return Err(GenError::Empty("empty pocket region".into()));
    }
    let strategy = resolved_strategy(region, p);
    let (guide, load, limit) = guide_and_load(strategy, p)?;
    let a = &p.links.approach;
    let top = [Some(p.top_z), a.stock_top, a.material_top].into_iter().flatten().fold(f64::NEG_INFINITY, f64::max);
    let k = |e: Refusal| refusal(&e, top);
    let r = 0.5 * p.tool_diameter;
    let tool = ToolShape::disc(r, 0.0, a.center_cutting, (top - p.bottom_z).max(0.0) + r).map_err(k)?;
    let bounds = bounds_of(side, Region::empty(), top, p.bottom_z).map_err(k)?;
    let order = OrderChoice { start: kernel_side(side.start), direction: if p.climb { Direction::Climb } else { Direction::Conventional } };
    let stage_load = LoadStage::new(limit).map_err(k)?;
    let arc_r = if a.arc_radius > 0.0 { a.arc_radius } else { r };
    let arc = ArcChoice::new(arc_r, if a.arc_sweep > 0.0 { a.arc_sweep.min(180.0) } else { 90.0 }).map_err(k)?;
    let clear = if a.plunge_height > 0.0 { a.plunge_height } else { DEFAULT_CLEAR };
    let retract = p.retract_z.unwrap_or(p.clearance_z).max(top);
    let stock = AreaStock(region);
    let cleared = Region::empty();
    let zs = cut_levels_for(p.top_z, a, p.bottom_z, p.stepdown)?;
    // The air round the stock (no stock within one tool radius), when the
    // stock is known round a part.
    let air_out = match side.stock.as_ref().filter(|_| side.around_part) {
        Some(st) => Some(st.offset(r)?),
        None => None,
    };
    let mut planned: Vec<(PlannedLevel, WorkPlane, Option<f64>)> = Vec::with_capacity(zs.len());
    for (i, &z) in zs.iter().enumerate() {
        let kz = |e: Refusal| refusal(&e, z);
        let plane = WorkPlane::setup_xy(z).map_err(kz)?;
        let kr = pocket_region(region, side, &bounds, plane, tool, p.stock_to_leave).map_err(kz)?;
        let asked = if i == 0 { p.entry } else { p.links.transition.unwrap_or(p.entry) };
        let entry = entry_choice(asked, p, holes, z).map_err(kz)?;
        let recipe = Recipe::new(plane, AxisControl::Normal, guide, load, order, link_policy(p.links.pass), entry).map_err(kz)?;
        let plan_at = |material_top: f64| -> Result<PlannedLevel, GenError> {
            let heights = Heights::new(material_top - z, clear, retract - z).map_err(kz)?;
            let es = EntryStage::new(heights, arc, ENTRY_CHORD_TOL).map_err(kz)?;
            plan_level(&kr, &recipe, &LevelStages { entry: &es, load: &stage_load, stock: &stock, cleared: &cleared }).map_err(kz)
        };
        // A lower level enters over the floor of the level above where that
        // level cut the column of each entry; else from over the stock top.
        let uppers: Vec<&Plan> = planned.iter().map(|(l, _, _)| &l.plan).collect();
        let floor = match i.checked_sub(1).and_then(|j| zs.get(j)).copied() {
            Some(zu) if zu < top => {
                let level = plan_at(zu)?;
                approach_proved(&level.plan, &uppers, air_out.as_ref()).then_some((level, zu))
            }
            _ => None,
        };
        let (level, level_top) = match floor {
            Some((level, zu)) => (level, Some(zu)),
            None => (plan_at(top)?, None),
        };
        if let Some(g) = level.gaps.iter().max_by(|a, b| a.area_mm2().total_cmp(&b.area_mm2())) {
            let msg = format!(
                "{KERNEL_POCKET_INCOMPLETE} ({} gap(s), the largest {:.3} mm2) at Z {z:.3}",
                level.gaps.len(),
                g.area_mm2()
            );
            return Err(GenError::invalid_at(msg, Point3::new(g.at().x, g.at().y, z)));
        }
        planned.push((level, plane, level_top));
    }
    let heights = EmitHeights::new(p.clearance_z, top).map_err(|e| GenError::Invalid(e.to_string()))?;
    let feeds = EmitFeeds::new(p.feed, p.plunge_feed).map_err(|e| GenError::Invalid(e.to_string()))?;
    let mut notes = vec![KERNEL_POCKET_NOTE.to_owned()];
    if p.blend_turns {
        notes.push(KERNEL_NO_BLEND.to_owned());
    }
    if p.links.max_stay_down > 0.0 {
        notes.push(format!(
            "kernel link: a keep-down link of any proved length instead of the limit {:.3} mm - the link stage has no length limit yet",
            p.links.max_stay_down
        ));
    }
    for n in planned.iter().flat_map(|(l, _, _)| l.notes.iter()) {
        if !notes.contains(n) {
            notes.push(n.clone());
        }
    }
    Ok(KernelPocketPlan { levels: planned, notes, strategy, heights, feeds })
}
