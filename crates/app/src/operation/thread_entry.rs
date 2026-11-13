//! The thread mill's way in (#992): a thread mill is not centre-cutting,
//! and the thread ride feeds it down the hole's axis and out along its
//! lead-in arc. That is air only when an earlier operation opened the
//! hole, so each threaded hole is read in the stock the program ahead
//! leaves standing (the prior grid the drilling laws read,
//! [`super::drill_prior`]) and a hole not open to the thread's depth is
//! refused by name - never a thread mill fed into solid metal.
//!
//! Provenance: public machine-tool handbook thread milling (the tap-drill
//! hole is made first, to the minor diameter, deeper than the thread);
//! the prior-grid reading is our own design. No vendor material consulted.

use crate::cl::Toolpath;
use crate::drilling::Hole;
use crate::machining2d::GenError;
use crate::stock_sim::{Columns, Dexel};

/// Float slack on the grid's stored heights, mm: a column cut exactly to
/// the thread mill's lowest point is air there.
const Z_SLACK: f64 = 1e-6;

/// The lowest point the path takes the tool to inside `hole` (its moves
/// whose ends stand within the hole's minor circle - every thread move
/// does, the door keeps the mill inside it), or `None` when the path
/// never enters the hole.
fn lowest_in(path: &Toolpath, hole: &Hole) -> Option<f64> {
    let inside = |p: crate::geometry::Point3| (p.x - hole.xy.x).hypot(p.y - hole.xy.y) <= hole.radius + 1e-6;
    path.moves
        .iter()
        .flat_map(|m| [m.from, m.to])
        .filter(|p| inside(*p))
        .map(|p| p.z)
        .fold(None, |lo: Option<f64>, z| Some(lo.map_or(z, |l| l.min(z))))
}

/// Why the program ahead has not opened every threaded hole, by name, or
/// `Ok` when each is air under the thread mill. `prior` is the grid the
/// setup's earlier operations leave standing; `None` (nothing ahead cut,
/// or what is ahead cannot be read, or a posed frame no column grid
/// reads) is unknown, which is refused - an unjudged hole is never air.
///
/// The disk read is the hole's minor circle less one grid cell (the
/// lattice's own resolution: a drill of exactly the minor diameter cuts
/// every column whose centre falls inside it), never narrower than the
/// mill itself, from the stock top down to the mill's lowest point.
pub(super) fn judge(holes: &[Hole], path: &Toolpath, tool_r: f64, prior: Option<&Dexel>) -> Result<(), GenError> {
    for hole in holes {
        let Some(low) = lowest_in(path, hole) else {
            continue;
        };
        let (x, y, d) = (hole.xy.x, hole.xy.y, hole.radius * 2.0);
        let advice = format!(
            "a thread mill does not cut at its centre - add a drilling operation of at least the {d:.3} mm minor diameter, deep enough for its point, before the thread"
        );
        let Some(dex) = prior else {
            return Err(GenError::Invalid(format!(
                "the hole at ({x:.3}, {y:.3}) is not drilled yet: nothing ahead of the thread in this setup's program opens it, or the program ahead cannot be read - {advice}"
            )));
        };
        let r = tool_r.max(hole.radius - dex.cell);
        let top = if r.is_finite() && r >= 0.0 {
            crate::verify::prior_surface(&Columns::of_grid(dex), hole.xy, r)
        } else {
            None
        };
        match top {
            Some(top) if top <= low + Z_SLACK => {}
            Some(top) => {
                return Err(GenError::Invalid(format!(
                    "the hole at ({x:.3}, {y:.3}) is not drilled yet: the stock the program ahead leaves stands at z {top:.3} inside it, above the thread mill's lowest point at z {low:.3} - {advice}"
                )));
            }
            None => {
                return Err(GenError::Invalid(format!(
                    "the hole at ({x:.3}, {y:.3}) cannot be judged: the stock the program ahead leaves is unreadable there - {advice}"
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cl::{Move, MoveKind};
    use crate::geo2d::Point2;
    use crate::geometry::Point3;

    fn hole() -> Hole {
        Hole { xy: Point2::new(0.0, 0.0), radius: 3.3, entry_z: 20.0, floor_z: 8.0, through: false, stepped: false }
    }

    /// A plunge down the axis to z 8 and a step out to the wall.
    fn path() -> Toolpath {
        let mv = |from: Point3, to: Point3| Move::new(MoveKind::Feed, from, to, Some(100.0));
        let mut t = Toolpath::default();
        t.moves.push(mv(Point3::new(0.0, 0.0, 25.0), Point3::new(0.0, 0.0, 8.0)));
        t.moves.push(mv(Point3::new(0.0, 0.0, 8.0), Point3::new(0.3, 0.0, 8.0)));
        t
    }

    /// A 20 mm block over a 20 x 20 square, the hole's column cut to `cut_z` out to `cut_r`.
    fn block(cut_r: f64, cut_z: f64) -> Dexel {
        let mut d = Dexel::block(-10.0, -10.0, 10.0, 10.0, 20.0, 0.25).expect("block");
        d.cut_chord(Point3::new(0.0, 0.0, cut_z), Point3::new(0.0, 0.0, cut_z), cut_r);
        d
    }

    fn refusal(prior: Option<&Dexel>) -> String {
        judge(&[hole()], &path(), 3.0, prior).err().map(|e| e.to_string()).unwrap_or_default()
    }

    #[test]
    fn a_hole_drilled_to_the_minor_diameter_below_the_thread_is_air() {
        assert!(judge(&[hole()], &path(), 3.0, Some(&block(3.3, 5.0))).is_ok());
        // Cut exactly to the mill's lowest point is still air.
        assert!(judge(&[hole()], &path(), 3.0, Some(&block(3.3, 8.0))).is_ok());
    }

    #[test]
    fn solid_shallow_or_narrow_stock_under_the_mill_is_refused_by_name() {
        let solid = refusal(Some(&block(0.0, 20.0)));
        assert!(solid.contains("the hole at (0.000, 0.000) is not drilled yet") && solid.contains("6.600 mm minor diameter"), "{solid}");
        let shallow = refusal(Some(&block(3.3, 9.0)));
        assert!(shallow.contains("stands at z 9.000") && shallow.contains("lowest point at z 8.000"), "{shallow}");
        // Opened only to the mill's own width: the minor circle (less a cell) still holds stock.
        assert!(refusal(Some(&block(2.5, 5.0))).contains("not drilled yet"));
        assert!(refusal(Some(&block(3.0, 5.0))).contains("not drilled yet"));
    }

    #[test]
    fn an_unknown_prior_is_refused_never_read_as_air() {
        let none = refusal(None);
        assert!(none.contains("nothing ahead of the thread"), "{none}");
        // A hole off the grid has no reading at all.
        let mut far = hole();
        far.xy = Point2::new(50.0, 50.0);
        let mut p = path();
        for m in &mut p.moves {
            m.from.x += 50.0;
            m.from.y += 50.0;
            m.to.x += 50.0;
            m.to.y += 50.0;
        }
        let msg = judge(&[far], &p, 3.0, Some(&block(3.3, 5.0))).err().map(|e| e.to_string()).unwrap_or_default();
        assert!(msg.contains("cannot be judged"), "{msg}");
    }

    #[test]
    fn a_hole_the_path_never_enters_is_not_read() {
        let mut other = hole();
        other.xy = Point2::new(5.0, 5.0);
        other.radius = 1.0;
        assert!(judge(&[other], &path(), 0.5, None).is_ok());
    }
}
