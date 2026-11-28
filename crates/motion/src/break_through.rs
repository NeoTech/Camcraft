//! #1557: the through-hole break-through - how far past a through hole's
//! floor a Drill, Bore or Thread runs - said in the op's notes and judged
//! against what lies under the stock. Provenance: the shop rule that a
//! through drill exits the far face by about its point (machining
//! handbooks), and our own refusal law; no vendor material consulted.

use crate::cl::Toolpath;
use crate::geo2d::Point2;
use crate::geometry::Point3;
use crate::units::{self, Unit};

// Match the document generation tolerance at the stock boundary.
const Z_TOL: f64 = 1e-6;

/// The through holes' run and the deepest tip it takes them to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Broke {
    /// How far past each through floor the tip runs, mm.
    pub run: f64,
    /// The deepest tip of any through hole: its centre and Z.
    pub tip: Point3,
    /// The drill point length inside `run` (#2528): a Drill measured from
    /// its full diameter runs its tip the stated Break-through plus its
    /// point. Zero when the Break-through is measured from the tip.
    pub point: f64,
}

impl Broke {
    /// The same run, with `point` of it said as the drill point (#2528).
    #[must_use]
    pub fn with_point(self, point: f64) -> Self {
        Broke { point: if point.is_finite() { point.clamp(0.0, self.run.max(0.0)) } else { 0.0 }, ..self }
    }
}

/// The deepest through tip of `tips` (each through hole's centre and the
/// Z its tip reaches), run `run` past its floor; `None` when no hole runs
/// through (a blind list, or a Bottom stated by hand).
pub fn deepest(tips: impl IntoIterator<Item = (Point2, f64)>, run: f64) -> Option<Broke> {
    tips.into_iter()
        .filter(|(_, z)| z.is_finite())
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(xy, z)| Broke { run, tip: Point3::new(xy.x, xy.y, z), point: 0.0 })
}

/// The prefix of the note a break-through below the stock bottom carries.
pub const BELOW_STOCK_NOTE: &str = "Break-through below stock:";

/// The note every op with a through hole carries, and the warning when
/// the tip leaves the stock's bottom with nothing modelled under it.
/// `stock_bottom` is the stock's lowest Z (None: no stock, or a frame the
/// stock box does not read in); `steel` says workholding or a table is
/// modelled in the setup - then the fixture laws judge whatever stands
/// under the stock, refuse a tip that reaches it by name (a flute hit
/// holds the post), and leave air between parallels legal. With nothing
/// modelled, what the stock sits on is unknown (a table, a spoil board,
/// parallels): the program posts as it always has, and the note says the
/// tip leaves the stock and by how much.
pub fn judge(mut path: Toolpath, broke: Option<Broke>, stock_bottom: Option<f64>, steel: bool, unit: Unit) -> Toolpath {
    let Some(b) = broke else {
        return path;
    };
    let len = |v: f64| format!("{} {}", units::len(v, unit, 3), unit.suffix());
    // #2528: the note agrees with the Break-through field. A drill
    // measured from its full diameter names the field's value and the
    // drill point that its tip runs beyond it.
    let why = if b.point > Z_TOL {
        format!(
            "Break-through {} from the full diameter, plus the {} drill point",
            len(b.run - b.point),
            len(b.point)
        )
    } else {
        "Break-through".to_owned()
    };
    path.notes.push(format!(
        "through holes run {} past the part bottom ({why}), the deepest to Z{}",
        len(b.run),
        units::len(b.tip.z, unit, 3)
    ));
    if let Some(sb) = stock_bottom.filter(|z| z.is_finite())
        && !steel
        && b.tip.z < sb - Z_TOL
    {
        path.notes.push(format!(
            "{BELOW_STOCK_NOTE} the through hole at ({}, {}) runs to Z{}, {} under the stock bottom Z{} - \
             whatever the stock sits on is cut there; model the parallels, spoil board or fixture under it so the \
             fixture laws judge it, or lower the Break-through",
            units::len(b.tip.x, unit, 3),
            units::len(b.tip.y, unit, 3),
            units::len(b.tip.z, unit, 3),
            len(sb - b.tip.z),
            units::len(sb, unit, 3),
        ));
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tp() -> Toolpath {
        Toolpath::new(Vec::new())
    }

    #[test]
    fn the_deepest_tip_is_picked_and_said() {
        let b = deepest([(Point2::new(1.0, 2.0), -1.0), (Point2::new(3.0, 4.0), -2.5)], 1.0).expect("a through hole");
        assert_eq!(b.tip, Point3::new(3.0, 4.0, -2.5));
        let p = judge(tp(), Some(b), None, false, Unit::Millimeters);
        assert!(p.notes.iter().any(|n| n.contains("run 1.000 mm past the part bottom") && n.contains("Z-2.500")), "{:?}", p.notes);
        assert!(deepest(Vec::<(Point2, f64)>::new(), 1.0).is_none());
    }

    #[test]
    fn below_an_unmodelled_stock_bottom_is_said_and_modelled_steel_is_left_to_its_laws() {
        let below = |p: &Toolpath| p.notes.iter().any(|n| n.starts_with(BELOW_STOCK_NOTE));
        let b = Some(Broke { run: 1.0, tip: Point3::new(0.0, 0.0, -1.0), point: 0.0 });
        let p = judge(tp(), b, Some(0.0), false, Unit::Millimeters);
        assert!(below(&p) && p.notes.iter().any(|n| n.contains("1.000 mm under the stock bottom Z0.000")), "{:?}", p.notes);
        assert!(!below(&judge(tp(), b, Some(0.0), true, Unit::Millimeters)), "the fixture laws judge modelled steel");
        assert!(!below(&judge(tp(), b, Some(-1.0), false, Unit::Millimeters)), "on the stock bottom exactly");
    }

    #[test]
    fn an_inch_document_reads_inches() {
        let b = Some(Broke { run: 0.508, tip: Point3::new(0.0, 0.0, 5.0), point: 0.0 });
        let p = judge(tp(), b, Some(0.0), false, Unit::Inches);
        assert!(p.notes.iter().any(|n| n.contains("0.02000 in")), "{:?}", p.notes);
    }

    /// #2528: a 6 mm 118 deg drill with a 1 mm Break-through from the full
    /// diameter runs its tip 2.803 mm. The note names the field's 1 mm
    /// and the 1.803 mm point beside the total.
    #[test]
    fn a_full_diameter_drill_names_the_field_and_its_point() {
        let b = deepest([(Point2::new(0.0, 0.0), -2.803)], 2.803).map(|b| b.with_point(1.803));
        let p = judge(tp(), b, None, false, Unit::Millimeters);
        let want = "run 2.803 mm past the part bottom (Break-through 1.000 mm from the full diameter, plus the 1.803 mm drill point)";
        assert!(p.notes.iter().any(|n| n.contains(want)), "{:?}", p.notes);
        let plain = judge(tp(), deepest([(Point2::new(0.0, 0.0), -1.0)], 1.0), None, false, Unit::Millimeters);
        assert!(plain.notes.iter().any(|n| n.contains("(Break-through), the deepest")), "{:?}", plain.notes);
        let wild = deepest([(Point2::new(0.0, 0.0), -1.0)], 1.0).map(|b| b.with_point(f64::NAN));
        assert_eq!(wild.map(|b| b.point), Some(0.0), "a non-finite point is said as none");
    }
}
