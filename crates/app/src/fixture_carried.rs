//! Workholding in a carried setup (#321): a setup entered with the body
//! the earlier setups machined (#317) holds THAT body, not its declared
//! block - so its fixtures are read against the carried stock. A fixture
//! standing inside the material the program really carries in (a vise
//! jaw modelled into a wall setup 1 left standing, a clamp on a face that
//! is still there) is workholding that cannot be where it is modelled:
//! every fixture verdict of that setup judged steel the machine will not
//! hold there, and the post is refused by name.
//!
//! Provenance: our own design per #308's research decision (the carried
//! body is the incoming stock, read through each setup's transform);
//! plain re-clamp practice from machine-tool handbooks. No vendor
//! material consulted.
//!
//! ## The laws here
//!
//! - **The carried body, not the declared block.** Only the body the
//!   setup is ENTERED with is read (`rest::entered_body`, the carried-body
//!   cache's entry): a clamp seated in a pocket setup 1 cut, inside the
//!   declared block but clear of the real material, is no conflict.
//! - **Penetration, not contact.** A column counts when its centre lies
//!   strictly inside the fixture's footprint and its material
//!   (`floor..top`) overlaps the fixture's steel by more than the
//!   contact tolerance - jaws gripping a side face and a part seated on
//!   a plate or in a nest are contact by design. A footprint narrower
//!   than a cell (holding no centre) is judged by the column under its
//!   own centre, by the box. The column sampling resolves to the
//!   carried grid's pitch: an overlap narrower than half a cell beside a
//!   face can pass, and is stated so in the decision record.
//! - **The shape, not the box (#1530).** The box is the broad phase; a
//!   fixture it flags is then judged by its closed world mesh, column by
//!   column: the steel on the column's vertical line (and on four lines
//!   [`CARRIED_CONTACT_TOL`] beside it - a face at the column centre is
//!   contact) against the column's material. A vise is its jaws, not the
//!   box that spans the part they grip; a nest is its walls and floor.
//!   The mesh only acquits: an open, refused or coarse-fidelity mesh, or
//!   a line no nudge reads, keeps the box verdict.
//! - **Live, like every workholding law.** Read at the post gate against
//!   the steel as it stands now, never baked into a stamp (`fixture`'s
//!   live-gate law). The machine table is not read: the table is what the
//!   setup's stock stands on, not steel placed against it.
//! - **Unreadable is not clear.** A carry the gate cannot read leaves the
//!   ops of that setup refused by their own generation (#318/#319); this
//!   clause adds nothing for it, it never reads the declared block instead.

use flecs_ecs::prelude::{Entity, World};

use crate::fixture::{FIXTURE_HIT_TOL, FixtureBox};
use crate::geo2d::Point2;
use crate::model;
use crate::stock_sim::Body;
use camcraft_sim::fixture_mesh::FixtureMesh;

/// The phrase every carried-workholding refusal carries.
pub const FIXTURE_IN_CARRIED: &str = "stands inside the stock carried from";

/// How far a fixture's MESH may reach into the carried material and
/// still be contact, in millimetres (#1530): a jaw face on a side face,
/// a part seated on a nest floor. Ten microns - below any fit a jaw or
/// nest is modelled to, above the float dust of a carried body.
pub const CARRIED_CONTACT_TOL: f64 = 0.01;

/// Walk the columns `b`'s footprint holds (centre strictly inside), and
/// the column under the footprint's centre when it holds none, taking
/// the worst depth `judge(k, centre, inside, worst so far)` reports.
fn scan(
    body: &Body,
    b: &FixtureBox,
    mut judge: impl FnMut(usize, Point2, bool, Option<f64>) -> Option<f64>,
) -> Option<f64> {
    let dex = &body.dex;
    let (bx0, bx1) = (b.min.x.min(b.max.x), b.max.x.max(b.min.x));
    let (by0, by1) = (b.min.y.min(b.max.y), b.max.y.max(b.min.y));
    if !(bx0.is_finite() && bx1.is_finite() && by0.is_finite() && by1.is_finite() && b.min.z.is_finite() && b.max.z.is_finite())
        || !(dex.cell.is_finite() && dex.cell > 0.0)
    {
        return None;
    }
    // The cell index span the footprint covers, clamped to the grid
    // (float -> index through a clamp, never an unchecked cast).
    let span = |lo: f64, hi: f64, o: f64, n: usize| -> (usize, usize) {
        let top = n as f64;
        let i0 = ((lo - o) / dex.cell - 1.0).floor().clamp(0.0, top);
        let i1 = ((hi - o) / dex.cell + 1.0).ceil().clamp(0.0, top);
        (i0 as usize, i1 as usize)
    };
    let (ix0, ix1) = span(bx0, bx1, dex.x0, dex.nx);
    let (iy0, iy1) = span(by0, by1, dex.y0, dex.ny);
    let mut worst: Option<f64> = None;
    let mut any_inside = false;
    let take = |worst: &mut Option<f64>, d: Option<f64>| {
        if let Some(d) = d {
            *worst = Some(worst.map_or(d, |w: f64| w.max(d)));
        }
    };
    for iy in iy0..iy1 {
        for ix in ix0..ix1 {
            let c = dex.center(ix, iy);
            let inside = c.x > bx0 + FIXTURE_HIT_TOL
                && c.x < bx1 - FIXTURE_HIT_TOL
                && c.y > by0 + FIXTURE_HIT_TOL
                && c.y < by1 - FIXTURE_HIT_TOL;
            if inside {
                any_inside = true;
                let k = ix + iy * dex.nx;
                if !body.empty_at(k) {
                    let d = judge(k, c, true, worst);
                    take(&mut worst, d);
                }
            }
        }
    }
    // A footprint narrower than a cell holds no column centre: the column
    // under its own centre speaks for it, never nothing.
    let mid = Point2::new((bx0 + bx1) * 0.5, (by0 + by1) * 0.5);
    if !any_inside && let Some(k) = dex.col(mid) && !body.empty_at(k) {
        let d = judge(k, mid, false, worst);
        take(&mut worst, d);
    }
    worst
}

/// Column `k`'s material overlap with the box's Z span.
fn box_depth(body: &Body, b: &FixtureBox, k: usize) -> f64 {
    let (bz0, bz1) = (b.min.z.min(b.max.z), b.max.z.max(b.min.z));
    body.dex.top_at(k).min(bz1) - body.floor_at(k).max(bz0)
}

/// How deep `b` stands in `body`'s material: the largest Z overlap of a
/// column whose centre lies strictly inside the box's footprint, or
/// `None` when no column overlaps beyond [`FIXTURE_HIT_TOL`].
#[must_use]
pub fn penetration(body: &Body, b: &FixtureBox) -> Option<f64> {
    scan(body, b, |k, _, _, _| Some(box_depth(body, b, k)).filter(|d| d.is_finite() && *d > FIXTURE_HIT_TOL))
}

/// Material `lo..hi` overlapped by `spans`, in millimetres.
fn overlap(spans: &[(f64, f64)], lo: f64, hi: f64) -> f64 {
    spans.iter().map(|&(a, b)| (b.min(hi) - a.max(lo)).max(0.0)).sum()
}

/// The facets one fixture's mesh judgement may visit (#1530): past it
/// every column not yet read keeps its box depth. Bounds the gate on a
/// mesh every line meets many facets of (a fine fan, a dense nest): a
/// few seconds at most, and the answer stays on the safe side.
pub const MESH_VISIT_BUDGET: u64 = 50_000_000;

/// How deep the closed `mesh` (whose world box is `b`) stands in
/// `body`'s material (#1530): per column the box holds, the material its
/// steel overlaps on the column's line - the least of the centre line
/// and four lines [`CARRIED_CONTACT_TOL`] beside it, so a face at the
/// centre is contact. `None` when no column overlaps beyond
/// [`CARRIED_CONTACT_TOL`]. A line no nudge reads, a spent
/// [`MESH_VISIT_BUDGET`], and a footprint narrower than a cell keep the
/// box's depth. A column whose box depth cannot beat the worst depth
/// already found is not read: the verdict and its depth stand. `budget`
/// is the facet visits left ([`MESH_VISIT_BUDGET`] for one fixture).
#[must_use]
pub fn mesh_penetration(body: &Body, b: &FixtureBox, mesh: &FixtureMesh, budget: &mut u64) -> Option<f64> {
    let t = CARRIED_CONTACT_TOL;
    scan(body, b, |k, c, inside, worst| {
        let boxed = box_depth(body, b, k);
        if !(boxed.is_finite() && boxed > t) || worst.is_some_and(|w| boxed <= w) {
            return None; // the box acquits, or cannot deepen the verdict
        }
        if !inside {
            return Some(boxed);
        }
        let (lo, hi) = (body.floor_at(k), body.dex.top_at(k));
        let mut depth = f64::INFINITY;
        for (dx, dy) in [(0.0, 0.0), (t, 0.0), (-t, 0.0), (0.0, t), (0.0, -t)] {
            let d = mesh.steel_on_line(c.x + dx, c.y + dy, budget).map_or(boxed, |s| overlap(&s, lo, hi));
            depth = depth.min(d);
            if depth <= t {
                return None; // one line clear within the tolerance: contact
            }
        }
        Some(depth).filter(|d| d.is_finite())
    })
}

/// One fixture's depth in `body`: the box flags, the mesh judges what it
/// flagged (#1530). A coarse-fidelity fixture (#864) or a mesh that
/// refuses keeps the box verdict, logged.
fn fixture_depth(world: &World, fname: &str, fe: &Entity, body: &Body) -> Option<f64> {
    let a = model::entity_world_aabb(world, fe)?;
    let b = FixtureBox { min: a.min, max: a.max };
    let boxed = penetration(body, &b)?;
    if crate::model_fidelity::chord_of(world, fe).is_some() {
        return Some(boxed);
    }
    let Some(tris) = model::world_facets(world, fe) else { return Some(boxed) };
    match FixtureMesh::build(&tris) {
        Ok(mesh) => {
            let mut budget = MESH_VISIT_BUDGET;
            let d = mesh_penetration(body, &b, &mesh, &mut budget);
            if budget == 0 {
                tracing::info!(fixture = %fname, "carried-stock hold partly judged by the bounding box: the mesh visit budget ran out");
            }
            d
        }
        Err(why) => {
            tracing::info!(fixture = %fname, "carried-stock hold judged by the bounding box: {why}");
            Some(boxed)
        }
    }
}

/// The gate's carried-workholding clause: per setup entered with a
/// carried body, one problem per tagged fixture standing in it. The
/// setup's first op in program order reads the chain (every op of a
/// setup is entered with the same body).
#[must_use]
pub fn carried_fixture_problems(world: &World) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen: Vec<Option<String>> = Vec::new();
    for (name, e) in crate::setup::operation_order(world) {
        let Some(op) = model::operation_of(world, &e) else { continue };
        let setup = crate::setup::effective(world, op.setup.as_deref());
        if seen.contains(&setup) {
            continue;
        }
        seen.push(setup.clone());
        let fixtures = crate::fixture::tagged(world, setup.as_deref());
        if fixtures.is_empty() {
            continue;
        }
        let tool_r = model::find_by_name(world, &op.tool)
            .and_then(|t| model::tool_of(world, &t))
            .map_or(0.0, |t| t.diameter * 0.5);
        // A refused carry is refused by every op's own generation.
        let Ok(Some((origin, body))) = crate::rest::entered_body(world, &name, setup.as_deref(), tool_r) else {
            continue;
        };
        let to = setup.as_deref().unwrap_or_default();
        for (fname, fe) in fixtures {
            if let Some(d) = fixture_depth(world, &fname, &fe, &body) {
                out.push(format!(
                    "setup '{to}': fixture '{fname}' {FIXTURE_IN_CARRIED} setup '{origin}' ({d:.2} mm deep) - \
                     the part that setup leaves cannot be held as modelled; move the fixture or re-check the hold"
                ));
            }
        }
    }
    out
}
