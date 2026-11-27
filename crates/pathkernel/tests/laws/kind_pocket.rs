//! The 2D Pocket row of the K7 registry (#3752): the real stage planners
//! with the choices of the 2D Pocket recipe of `camcraft-motion`
//! (`machining2d::kernel_pocket`, Waterline: rings of the boundary, no load
//! control, climb, stay down where proved, a helix entry), on a closed
//! square pocket and on a pocket with a round island, both made by the
//! kernel's own region stage. The registry is in the kernel crate and
//! cannot import motion; the whole-program proof (the old planner against
//! the kernel, the verifier, the post) is in motion (`kernel_laws/k6_pocket.rs`)
//! and app (`pocket_pass_start/kernel_route.rs`).
//!
//! L7 reads the kernel's own first run as its record: #3752 changes the
//! old 2D Pocket output on purpose (the rings link at depth, the lifts are
//! named, no dead-end ramp), so the old cut sequence is not the record. The
//! maintainer reads the changed programs at the switch-over (proof item 3).
//! Provenance: our own design record (RECOMMENDED.md "#4024 - Path kernel
//! K0", "Map of the present operations"); offset pocketing as in Held (1991).

use camcraft_pathkernel::entry::{ArcChoice, EntryChoice, EntryKind, EntryStage, Heights, MaterialChoice};
use camcraft_pathkernel::guide::{GuideChoice, Guides, RingRef, RingsChoice};
use camcraft_pathkernel::link::{LinkPolicy, LinkStage};
use camcraft_pathkernel::load::{LoadControl, LoadStage};
use camcraft_pathkernel::order::{OrderChoice, OrderStage, StartSide};
use camcraft_pathkernel::{AxisControl, Bounds, KernelRegion, MaterialSide, Point2, Recipe, Refusal, Region, Selection, ToolShape, WorkPlane, region};

use super::kit::{Case, Planners, plan_cuts, run};
use super::registry::KindRecipe;
use super::standins::ShapeReader;

/// The tool radius of the cases (mm).
const R: f64 = 2.0;
/// The ring stepover of the cases (mm).
const STEP: f64 = 1.0;

fn square(s: f64) -> Vec<Point2> {
    let h = 0.5 * s;
    vec![Point2::new(-h, -h), Point2::new(h, -h), Point2::new(h, h), Point2::new(-h, h)]
}

fn island() -> Vec<Point2> {
    (0..64)
        .map(|k| {
            let a = -f64::from(k) * std::f64::consts::TAU / 64.0;
            Point2::new(4.0 * a.cos(), 4.0 * a.sin())
        })
        .collect()
}

fn pocket(m: MaterialSide, holes: Vec<Vec<Point2>>) -> Result<KernelRegion, Refusal> {
    let sel = Selection::face(square(24.0), holes)?;
    let bounds = Bounds::new(None, Region::empty(), 0.0, -2.0)?;
    region(&sel, m, &bounds, WorkPlane::setup_xy(0.0)?, ToolShape::disc(R, 0.0, false, 20.0)?, 0.0)
}

fn square_pocket(m: MaterialSide) -> Result<KernelRegion, Refusal> {
    pocket(m, vec![])
}

fn island_pocket(m: MaterialSide) -> Result<KernelRegion, Refusal> {
    pocket(m, vec![island()])
}

/// The planners of the 2D Pocket recipe: the five stages of the kernel.
fn planners() -> Result<Planners, Refusal> {
    let entry = EntryStage::new(Heights::new(2.0, 1.0, 10.0)?, ArcChoice::new(R, 90.0)?, 0.01)?;
    Ok(Planners {
        guide: Box::new(Guides),
        order: Box::new(OrderStage),
        load: Box::new(LoadStage::new(STEP / (2.0 * R))?),
        link: Box::new(LinkStage),
        entry: Box::new(entry),
    })
}

/// The Waterline recipe of the 2D Pocket.
fn recipe() -> Result<Recipe, Refusal> {
    let guide = GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 0.0, STEP, None, None)?);
    let entry = EntryChoice::new(EntryKind::Helix, 2.0, vec![])?.with_material(MaterialChoice::new(17.0, Some(1.0))?);
    Recipe::new(WorkPlane::setup_xy(0.0)?, AxisControl::Normal, guide, LoadControl::None, OrderChoice::default(), LinkPolicy::StayWhereProved, entry)
}

fn case(name: &'static str, region: fn(MaterialSide) -> Result<KernelRegion, Refusal>, holes: Vec<Vec<Point2>>, no_split: bool) -> Result<Case, Refusal> {
    let loops = std::iter::once(square(24.0)).chain(holes).map(camcraft_pathkernel::Loop::new).collect::<Result<Vec<_>, _>>();
    let stock = loops.and_then(Region::new).map_err(|_| Refusal::new(camcraft_pathkernel::Stage::Region, "the pocket case geometry failed"))?;
    let reader = Box::new(ShapeReader(stock.clone()));
    Ok(Case { name, region, material: MaterialSide::Inside, recipe: recipe()?, stock, reader, no_split, recorded: None })
}

/// The registered 2D Pocket: its planners and its two cases, each with
/// the kernel's own first run as the L7 record.
pub fn pocket_kind() -> KindRecipe {
    let mut cases = vec![
        case("2D Pocket, square", square_pocket, vec![], true).expect("fixture"),
        case("2D Pocket, round island", island_pocket, vec![island()], false).expect("fixture"),
    ];
    let p = planners().expect("planners");
    for c in &mut cases {
        let r = run(&p, c, StartSide::Auto).unwrap_or_else(|e| panic!("{}: the kernel plans the pocket: {e}", c.name));
        c.recorded = Some(plan_cuts(&r.plan).into_iter().cloned().collect());
    }
    KindRecipe { planners: p, cases }
}
