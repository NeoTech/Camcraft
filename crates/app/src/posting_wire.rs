//! The verification gate's wire EDM clause (#534, part of #290 D7):
//! the 2.5D wire laws (`sim::verify::wire` - kerf, threading, slug) run
//! over every Wire Contour's cached path against the part and stock as
//! they stand now, made binding - every finding blocks the post, the
//! way a located gouge does - and the not-judged line that says the 3D
//! stock sim did not run for the wire, what judged it instead, and
//! which piece each cut frees.
//! Provenance: our own design, the plasma clause's (`posting_plasma`)
//! read onto the wire; the laws cite their public sources in their own
//! module.

use crate::model;
use crate::operation::wire_verify::wire_verdict;
use crate::verify::wire::Slug;
use flecs_ecs::prelude::*;

/// The clause's own words on the not-judged line, said once.
pub const WIRE_VERDICT: &str = "the 3D stock sim does not run for wire EDM";

/// The wire clause's two outputs: the blocking problems, and the
/// visible line naming the cuts judged and the pieces they free.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct WireClause {
    pub problems: Vec<String>,
    pub line: Option<String>,
}

fn slugs(s: &[Slug]) -> String {
    if s.is_empty() {
        return "frees nothing".into();
    }
    s.iter()
        .map(|g| {
            format!(
                "frees {:.2} mm² near X{:.3} Y{:.3} ({})",
                g.area,
                g.at.x,
                g.at.y,
                if g.product_area > crate::verify::wire::SLUG_AREA_TOL { "the part" } else { "waste" }
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Judge every Wire Contour of the program (document order, suppressed
/// ops left out - the same walk `program_ops` makes).
#[must_use]
pub fn wire_clause(world: &World) -> WireClause {
    let mut out = WireClause::default();
    let mut judged: Vec<String> = Vec::new();
    for (name, e) in crate::setup::operation_order(world) {
        let Some(op) = model::operation_of(world, &e) else { continue };
        if op.suppressed {
            continue;
        }
        let Some(verdict) = wire_verdict(world, &e) else { continue };
        match verdict {
            Err(why) => out.problems.push(format!("{name}: {why}")),
            Ok(v) => {
                out.problems.extend(v.findings.iter().map(|f| format!("{name}: wire {}", f.describe())));
                judged.push(format!("{name} {}", slugs(&v.slugs)));
            }
        }
    }
    if !judged.is_empty() {
        out.line = Some(format!(
            "{WIRE_VERDICT} - judged by the 2.5D wire laws (kerf, threading, slug): {}",
            judged.join("; ")
        ));
    }
    out
}
