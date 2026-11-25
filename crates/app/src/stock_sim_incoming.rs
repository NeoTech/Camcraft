//! Posting's comparison with an explicitly acknowledged incoming stock
//! on the same upright, single-setup lattice. Unchanged deficits are
//! visible in the posting receipt. Any deepened column still goes to the full gate.
//! Provenance: a before/after height comparison on our stock simulator.

use flecs_ecs::prelude::World;

use super::{Deviation, Dexel, Planned, stock_grid};
use crate::geometry::Point3;

pub(super) fn report(
    world: &World,
    planned: &[Planned],
    after: &Dexel,
    dev: &mut Deviation,
) -> Option<String> {
    if dev.gouge_cells == 0 {
        return None;
    }
    let setup = planned.first()?.setup.as_deref();
    let source = crate::stock_source::incoming_stock_source(world, setup)?;
    let radius = planned.iter().map(|op| op.radius).fold(f64::INFINITY, f64::min);
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
        "note: incoming stock '{source}' already has {} deficient cells, worst {depth} {}{at}. This program leaves those cells at their incoming heights",
        inherited.cells,
        units.suffix()
    ))
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
