//! Posting's comparison with an explicitly acknowledged incoming stock
//! on the same upright, single-setup lattice. Unchanged deficits are
//! visible in the posting receipt. Any deepened column still goes to the full gate.
//! Provenance: a before/after height comparison on our stock simulator.

use flecs_ecs::prelude::World;

use super::{Deviation, Dexel, Planned, stock_grid};
use crate::geometry::Point3;

pub(super) fn report(
    world: &World,
    start: Option<&str>,
    planned: &[Planned],
    after: &Dexel,
    dev: &mut Deviation,
) -> Option<String> {
    // An undercut cutter's pass rides its own plan and lowers no column top
    // (#1444), so an unchanged top cannot say the program left that cell
    // alone: a program with one keeps the full gate.
    if dev.gouge_cells == 0 || planned.iter().any(|p| p.undercut.is_some()) {
        return None;
    }
    // The setup and lattice the walk started from (`carry::walk`; `start`
    // is where it stands, as it never spanned): the first cutting op's
    // setup or, for a program of touch probes only, which plans nothing,
    // the setup the probes run in, its uncut stock judged on the span-sized
    // lattice an infinite radius gives (#592). Only that stock's
    // acknowledgement counts: a probe in any other setup keeps the full gate.
    let setup = planned.first().map_or(start, |p| p.setup.as_deref());
    let judged = crate::setup::effective(world, setup);
    if unplanned_probe_setups(world)?.iter().any(|s| *s != judged) {
        return None;
    }
    let radius = planned.iter().map(|op| op.radius).fold(f64::INFINITY, f64::min);
    note(world, setup, radius, after, dev)
}

/// The located note for the acknowledged deficits of `setup`'s stock that
/// `after` leaves unchanged, taking them out of `dev`.
fn note(world: &World, setup: Option<&str>, radius: f64, after: &Dexel, dev: &mut Deviation) -> Option<String> {
    let source = crate::stock_source::incoming_stock_source(world, setup)?;
    let before = stock_grid(world, setup, radius).ok()?;
    let inherited = separate_unchanged(&before, after, dev)?;
    let units = crate::units::of(world);
    let depth = crate::units::len(inherited.worst, units, 2);
    // Located like the gouge line, so a person can find the worst cell.
    let at = inherited.at.map_or_else(String::new, |q| {
        format!(
            " at ({}, {}, z {})",
            crate::units::len(q.x, units, 2),
            crate::units::len(q.y, units, 2),
            crate::units::len(q.z, units, 2)
        )
    });
    Some(format!(
        "note: incoming stock '{source}' already has {} deficient cell(s), worst {depth} {}{at}. This program leaves those cells at their incoming heights",
        inherited.cells,
        units.suffix()
    ))
}

/// The effective setups of the program's touch probes, which the column
/// plan leaves out (#108); `None` keeps the full gate. The receipt's
/// "unchanged" reads only the planned cuts, and a torch or a wire removes
/// material the column never sees (#670), so a program with one gets no
/// note.
fn unplanned_probe_setups(world: &World) -> Option<std::collections::BTreeSet<Option<String>>> {
    use crate::tools::ToolKind;
    let mut setups = std::collections::BTreeSet::new();
    for (_, e) in crate::posting::program_ops(world).ok()? {
        let op = crate::model::operation_of(world, &e)?;
        let kind = crate::model::find_by_name(world, &op.tool).and_then(|t| crate::model::tool_of(world, &t))?.kind;
        match kind {
            ToolKind::Torch | ToolKind::Wire => return None,
            ToolKind::Probe => {
                setups.insert(crate::setup::effective(world, op.setup.as_deref()));
            }
            _ => {}
        }
    }
    Some(setups)
}

#[derive(Default)]
struct Gouges {
    cells: usize,
    volume: f64,
    worst: f64,
    at: Option<Point3>,
}

impl Gouges {
    fn add(&mut self, depth: f64, area: f64, at: Point3) {
        self.cells += 1;
        self.volume += depth * area;
        if depth > self.worst {
            self.worst = depth;
            self.at = Some(at);
        }
    }
}

fn separate_unchanged(before: &Dexel, after: &Dexel, dev: &mut Deviation) -> Option<Gouges> {
    let lattice = |d: &Dexel| (d.nx, d.ny, d.cell, d.x0, d.y0);
    if lattice(before) != lattice(after)
        || (dev.cells_x, dev.cells_y, dev.cell) != (after.nx, after.ny, after.cell)
        || dev.dev.len() != after.nx * after.ny
    {
        return None;
    }

    let mut inherited = Gouges::default();
    let mut machined = Gouges::default();
    let area = after.cell * after.cell;
    for (k, &deviation) in dev.dev.iter().enumerate() {
        if !deviation.is_finite() || deviation >= -dev.tol {
            continue;
        }
        let z = after.top_at(k);
        let xy = after.center(k % after.nx, k / after.nx);
        let found = if z.is_finite() && z == before.top_at(k) {
            &mut inherited
        } else {
            &mut machined
        };
        found.add(-deviation, area, Point3::new(xy.x, xy.y, z));
    }

    // Only adjust a verdict completely accounted for on this lattice.
    if inherited.cells == 0 || inherited.cells + machined.cells != dev.gouge_cells {
        return None;
    }
    dev.gouge_cells = machined.cells;
    dev.gouge_volume = machined.volume;
    dev.max_gouge = machined.worst;
    dev.max_gouge_at = machined.at;
    Some(inherited)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::cube;
    use crate::stock_sim::deviate;

    #[test]
    fn unchanged_deficits_do_not_hide_fresh_or_deepened_gouges() {
        let mut incoming = Dexel::block(0.0, 0.0, 4.0, 4.0, 10.0, 1.0).unwrap();
        incoming.lower(0, 5.0);
        incoming.lower(1, 8.0);
        let mut finished = incoming.clone();
        finished.lower(1, 7.0);
        finished.lower(2, 9.0);
        let part = cube(Point3::ZERO, Point3::new(4.0, 4.0, 10.0));
        let mut dev = deviate(&finished, &part.facets, 0.01).unwrap();
        assert_eq!(dev.gouge_cells, 3);
        let inherited = separate_unchanged(&incoming, &finished, &mut dev).unwrap();
        assert_eq!((inherited.cells, inherited.worst), (1, 5.0));
        assert_eq!((dev.gouge_cells, dev.gouge_volume, dev.max_gouge), (2, 4.0, 3.0));
        assert_eq!(dev.max_gouge_at, Some(Point3::new(1.5, 0.5, 7.0)));
    }

    #[test]
    fn a_different_lattice_keeps_the_full_verdict() {
        let mut incoming = Dexel::block(0.0, 0.0, 4.0, 4.0, 10.0, 1.0).unwrap();
        incoming.lower(0, 5.0);
        let mut shifted = incoming.clone();
        shifted.translate_origin(1.0, 0.0, 0.0);
        let part = cube(Point3::ZERO, Point3::new(5.0, 4.0, 10.0));
        let mut dev = deviate(&shifted, &part.facets, 0.01).unwrap();
        assert_eq!(dev.gouge_cells, 1);
        assert!(separate_unchanged(&incoming, &shifted, &mut dev).is_none());
        assert_eq!(dev.gouge_cells, 1);
    }
}
