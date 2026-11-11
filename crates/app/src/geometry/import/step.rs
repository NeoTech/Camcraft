//! STEP reader: B-rep shells tessellated to triangles with the `truck`
//! geometry crates (ricosjp, Apache-2.0), used as a library dependency.
//!
//! Provenance: file syntax per ISO 10303-21; surface tessellation is truck's
//! own published algorithm - nothing here from any CAM vendor.
//!
//! Hardening notes (Prime Directive):
//! - `truck` panics on shells that are not oriented/closed. Every truck call
//!   on the parse path runs inside `catch_unwind`; a kernel panic becomes a
//!   clean `ImportError::Step`, logged, never propagated.
//! - Tessellation is ADAPTIVE: each shell is meshed coarse first, then refined
//!   toward a quality target only while the projected facet count stays inside
//!   a memory budget. A model that cannot tessellate at target quality still
//!   imports - at reduced fidelity, logged - instead of spiking memory and
//!   being thrown away.
//! - Facet collection is capped per shell while extracting, as a last line of
//!   defense on top of the tolerance schedule.

use std::panic::AssertUnwindSafe;

use truck_meshalgo::prelude::*;
use truck_stepio::r#in::Table;

use super::{ImportError, MAX_FACETS};
use crate::geometry::{Mesh, Point3, Triangle};

/// Lower bound for tessellation tolerance; truck asserts tol >= 1e-7.
const MIN_TOL: f64 = 1e-6;
/// The coarsest chord tolerance a STEP mesh is made at (mm): half the
/// program sim's verdict tolerance, so the facets' sagitta on a curved
/// wall stays inside what the gouge verdict already forgives (#689).
const MAX_CHORD: f64 = crate::stock_sim::DEV_TOL * 0.5;

/// The finest chord tolerance the STEP importer promises (mm); a model
/// tessellated coarser than this is "reduced fidelity" (#864).
pub const CHORD_CAP: f64 = MAX_CHORD;

thread_local! {
    /// Test-only override of the facet budget (#864's pins force a small
    /// STEP fixture over budget instead of committing a huge model).
    static BUDGET_OVERRIDE: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}

/// Run `f` with the STEP facet budget lowered to `budget` on this thread.
/// Never raises it past [`MAX_FACETS`]. For tests; the override is
/// cleared on return.
#[doc(hidden)]
pub fn with_facet_budget<R>(budget: usize, f: impl FnOnce() -> R) -> R {
    BUDGET_OVERRIDE.with(|b| b.set(Some(budget)));
    let out = f();
    BUDGET_OVERRIDE.with(|b| b.set(None));
    out
}

fn facet_budget() -> usize {
    BUDGET_OVERRIDE
        .with(std::cell::Cell::get)
        .map_or(MAX_FACETS, |b| b.min(MAX_FACETS))
}

pub fn parse(data: &[u8]) -> Result<Mesh, ImportError> {
    parse_with_chord(data).map(|(mesh, _)| mesh)
}

/// [`parse`], and the chord error the kept mesh was actually made at, in
/// millimetres: the coarsest kept tolerance over all shells, scaled by the
/// file's length unit (#864). Never NaN, infinite or negative - such a
/// value is refused as a corrupt import, not carried into a verdict.
pub fn parse_with_chord(data: &[u8]) -> Result<(Mesh, f64), ImportError> {
    if data.is_empty() {
        return Err(ImportError::Empty);
    }
    let text = std::str::from_utf8(data)
        .map_err(|_| ImportError::Step("file is not valid UTF-8".into()))?;
    let exchange = ruststep::parser::parse(text)
        .map_err(|e| ImportError::Step(format!("syntax error: {e}")))?;
    // An exchange with zero DATA sections parses fine but would index out of
    // bounds inside truck's convenience helpers; guard the only raw index.
    let Some(section) = exchange.data.first() else {
        return Err(ImportError::Step("file has no DATA section".into()));
    };
    // #306: the file's own length unit, read before any geometry - a
    // part saved in inches or metres must not import with its numbers
    // read as millimetres. No stated unit keeps millimetres.
    let mm_per_unit = super::step_units::mm_per_unit(section)
        .map_err(|e| ImportError::Step(format!("length unit: {e}")))?;
    let table = Table::from_data_section(section);
    if table.shell.is_empty() {
        return Err(ImportError::Step("no shells found".into()));
    }

    let max_facets = facet_budget();
    let mut facets: Vec<Triangle> = Vec::new();
    let mut chord = 0.0f64;
    for shell in table.shell.values() {
        if facets.len() >= max_facets {
            return Err(ImportError::TooLarge);
        }
        let budget = max_facets - facets.len();
        // The whole per-shell kernel sequence runs inside one guarded call.
        // Everything it touches is shared-immutable or thread-local, so
        // AssertUnwindSafe holds for our purposes: on a panic we discard all
        // outputs and abort the import.
        let per_shell = std::panic::catch_unwind(AssertUnwindSafe(|| {
            shell_facets(&table, shell, budget)
        }));
        let (tris, tol) = match per_shell {
            Err(_) => {
                eprintln!("step: truck kernel panicked on a shell; import aborted");
                return Err(ImportError::Step(
                    "geometry kernel failed on a shell (degenerate or unclosed B-rep)".into(),
                ));
            }
            Ok(Err(err)) => return Err(err),
            Ok(Ok(kept)) => kept,
        };
        chord = chord.max(tol);
        facets.extend(tris);
    }
    if facets.is_empty() {
        return Err(ImportError::Empty);
    }
    // Scale after tessellating: the tolerance schedule is relative to
    // each shell's own extent, so the mesh's quality is the same in any
    // unit, and only the numbers change.
    if let Some(k) = mm_per_unit.filter(|k| *k != 1.0) {
        let mm = |p: Point3| Point3::new(p.x * k, p.y * k, p.z * k);
        for t in &mut facets {
            *t = Triangle::new(mm(t.a), mm(t.b), mm(t.c));
        }
        chord *= k.abs();
    }
    if !(chord.is_finite() && chord >= 0.0) {
        return Err(ImportError::Step(format!("tessellation reported a chord error of {chord} mm")));
    }
    Ok((Mesh::from_facets(facets), chord))
}

/// Tessellate one STEP shell into triangles, staying inside `budget` facets.
/// Runs inside `catch_unwind`; returns `Err` only for reasons we control.
fn shell_facets(
    table: &Table,
    shell: &truck_stepio::r#in::ShellHolder,
    budget: usize,
) -> Result<(Vec<Triangle>, f64), ImportError> {
    let cshell = table
        .to_compressed_shell(shell)
        .map_err(|e| ImportError::Step(format!("shell conversion failed: {e}")))?;
    let mut extent = 0.0f64;
    for p in cshell.vertices.iter() {
        for c in [p.x, p.y, p.z] {
            if !c.is_finite() {
                return Err(ImportError::Step("shell vertices are not finite".into()));
            }
            extent = extent.max(c.abs());
        }
    }
    // One tessellation pass: mesh the whole shell at `tol`, extract
    // triangles, stop with `TooLarge` as soon as `budget` is crossed. A
    // closure so the heavily-aliased CompressedShell type is inferred
    // instead of spelled out.
    let triangulate = |tol: f64| -> Result<Vec<Triangle>, ImportError> {
        let meshed = cshell.robust_triangulation(tol).to_polygon();
        let positions = meshed.positions();
        let mut facets: Vec<Triangle> = Vec::new();
        for face in meshed.face_iter() {
            if face.len() < 3 {
                continue;
            }
            let at = |i: usize| -> Option<Point3> {
                let v = face.get(i)?.pos;
                let p = positions.get(v)?;
                let pt = Point3::new(p.x, p.y, p.z);
                pt.is_finite().then_some(pt)
            };
            let Some(a) = at(0) else { continue };
            for i in 1..face.len() - 1 {
                let (Some(b), Some(c)) = (at(i), at(i + 1)) else {
                    continue;
                };
                let tri = Triangle::new(a, b, c);
                // Zero-area facets are tessellation slivers: drop them, they
                // carry no geometry and would poison the STL-style gates.
                let n = tri.normal();
                if n.x != 0.0 || n.y != 0.0 || n.z != 0.0 {
                    facets.push(tri);
                    if facets.len() > budget {
                        return Err(ImportError::TooLarge);
                    }
                }
            }
        }
        Ok(facets)
    };
    // Quality target: ~0.1% of the model extent, floored so tiny parts still
    // subdivide and truck's assert (tol >= 1e-7) holds - and never coarser
    // than half the program gate's verdict tolerance (#689). The sim judges
    // the program against these facets, and on a concave wall a chord
    // stands inside the true surface by its sagitta: a mesh coarser than
    // the verdict charged a bore at the hole's own size as a gouge the
    // wall's full height. Half the verdict leaves the other half for the
    // cut itself, so a nominal finish passes and 0.05 mm into the wall is
    // still a gouge. The facet budget below still bounds the mesh.
    // min/max, not clamp: a NaN extent then lands on a finite target
    // instead of carrying NaN into truck's tolerance assert.
    #[allow(clippy::manual_clamp)]
    let tol_target = (extent * 1e-3).min(MAX_CHORD).max(MIN_TOL);
    // Start 250x coarser than the target - the first pass then costs a few
    // dozen triangles per face no matter how big the model, so no shell can
    // silently allocate a monster mesh before we have measured it. The
    // schedule (coarse start, halving toward the target inside the budget)
    // lives in `step_refine`, which also keeps the tolerance of the mesh
    // it actually kept (#864).
    let kept = super::step_refine::refine(extent, tol_target, budget, triangulate)?;
    if kept.tol > tol_target {
        eprintln!(
            "step: tessellated at reduced fidelity (tol {:e}, target {tol_target:e}, {} facets) to bound memory",
            kept.tol,
            kept.facets.len()
        );
    }
    Ok((kept.facets, kept.tol))
}
