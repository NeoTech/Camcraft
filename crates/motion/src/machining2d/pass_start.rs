//! Where the passes of one level start (#3316): the order of a ring
//! family. One shared type for every kind with a pass order (#1448 adds
//! the start point beside it). The law is ours, from general machining
//! practice (cut from the open side so each pass engages one stepover);
//! no vendor material consulted.

/// Which side of the cutter-center area a level starts on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PassStart {
    /// The order each kind always cut: for the ring planner the core
    /// first, then outward. Every file before the option, byte for byte.
    #[default]
    Auto,
    /// Start at the area's holes (an island wall, the part wall of an
    /// Outside band) and walk out to its outer boundary.
    Inside,
    /// Start at the area's outer boundary (a pocket wall, the stock edge
    /// of an Outside band) and walk in to its holes.
    Outside,
}

impl PassStart {
    /// Every value, in the order the page lists them.
    pub const ALL: [PassStart; 3] = [PassStart::Auto, PassStart::Inside, PassStart::Outside];

    /// The name the page and the notes use.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            PassStart::Auto => "Auto",
            PassStart::Inside => "Inside",
            PassStart::Outside => "Outside",
        }
    }
}

/// The note for a level that must start on a wall (#3316): Outside on an
/// area whose outer boundary stands in the stock (a closed pocket), or
/// Inside on an area with a hole (a part wall or an island). The first
/// ring of such a level has uncut stock on both sides - a full-width cut
/// along the finish wall. No order can avoid it; the note says so.
/// `stock` is the stock's XY outline when the caller knows it; without it
/// an Outside start is judged a wall (the safe reading).
#[must_use]
pub fn start_wall_note(region: &crate::geo2d::Region, stock: Option<&crate::geo2d::Region>, start: PassStart) -> Option<String> {
    let says = |what: &str| {
        Some(format!(
            "Pass starts at {}: the first pass of each level runs on the {what} with uncut stock on both sides (a full-width cut)",
            start.label()
        ))
    };
    match start {
        PassStart::Auto => None,
        PassStart::Inside => {
            if region.loops().iter().any(|l| !l.is_ccw()) {
                says("part or island wall")
            } else {
                None
            }
        }
        PassStart::Outside => {
            // The outer boundary is air when no vertex of it stands
            // inside the stock outline.
            let wall = region.loops().iter().filter(|l| l.is_ccw()).any(|l| match stock {
                Some(s) => l.points().iter().any(|&p| s.contains(p)),
                None => true,
            });
            if wall { says("outer wall") } else { None }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PassStart, start_wall_note};
    use crate::geo2d::{Loop, Point2, Region};

    fn sq(c: f64, h: f64) -> Loop {
        let p = |x: f64, y: f64| Point2::new(c + x, c + y);
        let Ok(l) = Loop::new(vec![p(-h, -h), p(h, -h), p(h, h), p(-h, h)]) else { panic!("loop") };
        l
    }

    #[test]
    fn a_start_on_a_wall_is_named_and_a_start_in_air_is_not() {
        assert_eq!(PassStart::default(), PassStart::Auto);
        let Ok(band) = Region::new(vec![sq(0.0, 30.0), sq(0.0, 10.0)]) else { panic!("band") };
        let Ok(stock) = Region::new(vec![sq(0.0, 25.0)]) else { panic!("stock") };
        let Ok(pocket) = Region::new(vec![sq(0.0, 20.0)]) else { panic!("pocket") };
        // A band whose outer boundary is past the stock: Outside starts in air.
        assert_eq!(start_wall_note(&band, Some(&stock), PassStart::Outside), None);
        // Inside on the band starts on the part wall.
        assert!(start_wall_note(&band, Some(&stock), PassStart::Inside).is_some_and(|n| n.contains("part or island wall")));
        // Outside on a closed pocket starts on its wall.
        assert!(start_wall_note(&pocket, Some(&stock), PassStart::Outside).is_some_and(|n| n.contains("outer wall")));
        assert!(start_wall_note(&band, None, PassStart::Outside).is_some());
        assert_eq!(start_wall_note(&pocket, Some(&stock), PassStart::Inside), None);
        assert_eq!(start_wall_note(&band, Some(&stock), PassStart::Auto), None);
    }
}
