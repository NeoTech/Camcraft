//! The document's stock as the eye sees it (M7.2 B3): the program
//! walked through the FIELD arm, for the sphere-traced viewport.
//! Provenance: the M7.1 field engine fed the same plan the column
//! verdict walks (`stock_sim::plan_program`); the subdivision is plain
//! midpoint refinement. Nothing from vendor binaries.
//!
//! ## The laws here
//!
//! - **A display field, not a verdict.** The deviation status line and
//!   the post gate still read the column engine (`simulate_document`)
//!   until M7.3 proves the two verdict-identical; this field exists so
//!   the viewport can draw the machined volume. Its cell is chosen for
//!   the frame's texel budget, never finer than the column's own.
//! - **The true stock, not a bottomless one.** The verdict field's
//!   block reaches ten spans down to mimic a semi-infinite column; the
//!   eye builds the stock entity's own mesh, so the picture shows the
//!   block the machine holds.
//! - **Refusals by name, and a target that may be absent.** A rotated
//!   stock, a multi-setup program (the flip transform is not the eye's
//!   to guess), a program that will not plan: `Err`, and the viewport
//!   keeps its sheet. A model the field cannot sign (an open shell)
//!   leaves `target` empty with its reason - pixels then paint class 3,
//!   unjudged, never clean.

use crate::geometry::{transform_point, Point3, Triangle};
use crate::model;
use std::sync::Arc;

use crate::stock_sim::{auto_cell, part_facets, plan_program, SimOp, Stock};

// #2268: the view, its packed lattice and the display field build are
// plain data, so they live in camcraft-sim; the world walk stays here.
pub use camcraft_sim::field_view::{
    build_stock, build_target, subdivide, CutLayer, FieldView, Packed, PackedBase, DEFAULT_TEXELS,
};

/// Walk the document's program through the field arm, at the default
/// display budget.
pub fn field_view(world: &flecs_ecs::prelude::World) -> Result<FieldView, String> {
    field_view_with(world, DEFAULT_TEXELS)
}

/// The same walk with the stock's display cell sized for `texels`
/// cells of volume (a coarser picture is cheaper to cut, never less
/// honest: classes still come from the same gate).
pub fn field_view_with(world: &flecs_ecs::prelude::World, texels: f64) -> Result<FieldView, String> {
    let (mut stock, planned, cell) = display_parts(world, texels)?;
    let pose_tol = crate::stock_sim::rotary_pose_tol(cell);
    for p in &planned {
        // #1444: an undercut cutter removes its true head and neck.
        if let Some(u) = &p.undercut {
            stock
                .simulate_undercut(&u.moves, &u.cutter, u.neck_len)
                .map_err(|e| format!("the field view could not cut the program: {}: {e}", p.name))?;
            continue;
        }
        let op = SimOp { moves: &p.moves, radius: p.radius, ball: p.ball, cone: p.cone, corner: p.corner };
        // A rotary op turns the part: its moves are chorded about the
        // fitted axis (M13.1, `stock_sim::rotary`), never cut as the
        // un-chorded posed chords they are written as.
        // A turning op's moves revolve about its fitted spindle (#19,
        // `stock_sim::lathe`).
        let cut = match (&p.rotary, &p.lathe) {
            (_, Some(l)) => stock.simulate_lathe_profile(&p.moves, &l.profile, &l.axis),
            (Some(ax), None) => crate::stock_sim::simulate_rotary_slice(&mut stock, &op, &p.moves, 0, ax, pose_tol).map(|_| ()),
            (None, None) => {
                if p.drill {
                    stock.simulate_drill_display(std::slice::from_ref(&op))
                } else {
                    stock.simulate_display(std::slice::from_ref(&op))
                }
            }
        };
        cut.map_err(|e| format!("the field view could not cut the program: {e}"))?;
    }
    let (target, target_note, tol) = judged_target(world, cell);
    Ok(FieldView { stock, target: target.map(Arc::new), target_note, cell, tol, packed: None })
}

/// The uncut display stock, the plan it will be cut by, and its cell:
/// the walk `field_view_with` and the playback (`stock_play`) share.
pub(crate) fn display_parts(
    world: &flecs_ecs::prelude::World,
    texels: f64,
) -> Result<(Stock, Vec<crate::stock_sim::Planned>, f64), String> {
    let (facets, planned, cell) = display_inputs(world, texels)?;
    Ok((build_stock(&facets, cell)?, planned, cell))
}

/// `display_parts`' world half: the stock's world facets, the plan
/// and the cell - plain data a worker can build the field from.
pub(crate) fn display_inputs(
    world: &flecs_ecs::prelude::World,
    texels: f64,
) -> Result<(Vec<Triangle>, Vec<crate::stock_sim::Planned>, f64), String> {
    if !(texels.is_finite() && texels >= 1.0) {
        return Err(format!("the field view needs a positive texel budget, not {texels}"));
    }
    display_inputs_of(world, texels, plan_program(world)?, None)
}

/// `display_inputs` over a given plan (#2280: the playback's plan ends
/// before an op with no toolpath). `fallback_setup` names the setup
/// whose stock is drawn when the plan is empty.
pub(crate) fn display_inputs_of(
    world: &flecs_ecs::prelude::World,
    texels: f64,
    planned: Vec<crate::stock_sim::Planned>,
    fallback_setup: Option<String>,
) -> Result<(Vec<Triangle>, Vec<crate::stock_sim::Planned>, f64), String> {
    if !(texels.is_finite() && texels >= 1.0) {
        return Err(format!("the field view needs a positive texel budget, not {texels}"));
    }
    // #541: the field eye has no box atom - said, never cut as a disc.
    if let Some(p) = planned.iter().find(|p| p.electrode.is_some()) {
        return Err(format!("{}: {}", p.name, crate::stock_sim::ELECTRODE_FIELD_REFUSAL));
    }
    let first = planned.first().map_or(fallback_setup, |p| p.setup.clone());
    let eff = crate::setup::effective(world, first.as_deref());
    if planned.iter().any(|p| crate::setup::effective(world, p.setup.as_deref()) != eff) {
        return Err("the field view draws one setup's stock; a multi-setup program keeps the sheet (the flip transform is M7.3's)".into());
    }
    let stock_facets = stock_facets(world, first.as_deref())?;
    let (lo, hi) = bounds(&stock_facets).ok_or("the stock meshed empty - nothing to draw")?;
    let min_r = planned.iter().map(|p| p.radius).fold(f64::INFINITY, f64::min);
    let (sx, sy, sz) = (hi.x - lo.x, hi.y - lo.y, hi.z - lo.z);
    let budget = ((sx * sy * sz).max(0.0) / texels).cbrt();
    let cell = auto_cell(if min_r.is_finite() { min_r } else { 1.0 }, sx, sy).max(budget);
    if !(cell.is_finite() && cell > 0.0) {
        return Err(format!("the stock's extent gives no sane display cell ({cell})"));
    }
    Ok((stock_facets, planned, cell))
}

/// The model as a field at `cell`, or why it cannot be one.
pub(crate) fn display_target(world: &flecs_ecs::prelude::World, cell: f64) -> (Option<Stock>, Option<String>) {
    match part_facets(world) {
        Ok(part) => build_target(&part, cell),
        Err(why) => (None, Some(why)),
    }
}

/// [`display_target`] at the tolerance the program sim judges at
/// (#1056). When the gate refuses to judge the program on a model too
/// coarse for it, the picture is unjudged too, and says why - never
/// coloured off the mesh the refusal names.
pub(crate) fn judged_target(world: &flecs_ecs::prelude::World, cell: f64) -> (Option<Stock>, Option<String>, f64) {
    match crate::model_fidelity::program_tol(world) {
        Ok(tol) => {
            let (target, note) = display_target(world, cell);
            (target, note, tol)
        }
        Err(why) => (None, Some(why), crate::stock_sim::DEV_TOL),
    }
}

/// The setup's stock entity as world triangles (the mesh the viewport
/// already draws) in the sim's frame (`stock_follow::sim_frame`):
/// refusing a turn the axis-aligned field lattice cannot hold.
fn stock_facets(world: &flecs_ecs::prelude::World, setup: Option<&str>) -> Result<Vec<Triangle>, String> {
    let name = crate::setup::stock_entity_name(world, setup);
    let e = model::find_by_name(world, &name)
        .ok_or_else(|| format!("no stock '{name}' - the eye draws material, not air"))?;
    let placed = model::transform_of(world, &e).unwrap_or_default();
    // #2177: the sim's own law - a square-turned stock is drawn as its
    // unturned twin (the same solid), an oblique one refused by name.
    let (twin, loc) = match model::stock_def_of(world, &e) {
        Some(shape) => crate::stock_follow::sim_frame(&shape, placed).map(|(s, l)| (Some(s), l))?,
        None if placed.rot == crate::geometry::IDENTITY_Q => (None, placed),
        None => return Err(crate::stock_follow::OBLIQUE_STOCK.into()),
    };
    let tris: Arc<[Triangle]> = match twin.filter(|_| loc != placed) {
        None => model::facets_of(world, &e).ok_or("the stock entity carries no mesh")?,
        Some(s) => crate::stock::stock_mesh(&s).ok_or("the stock cannot be meshed")?.into(),
    };
    Ok(tris
        .iter()
        .map(|t| {
            Triangle::new(
                transform_point(t.a, loc.pos, loc.rot),
                transform_point(t.b, loc.pos, loc.rot),
                transform_point(t.c, loc.pos, loc.rot),
            )
        })
        .collect())
}

fn bounds(f: &[Triangle]) -> Option<(Point3, Point3)> {
    let mut lo = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
    let mut hi = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for t in f {
        for v in [t.a, t.b, t.c] {
            lo = Point3::new(lo.x.min(v.x), lo.y.min(v.y), lo.z.min(v.z));
            hi = Point3::new(hi.x.max(v.x), hi.y.max(v.y), hi.z.max(v.z));
        }
    }
    (lo.is_finite() && hi.is_finite()).then_some((lo, hi))
}
