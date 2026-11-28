//! The fixture law of `camcraft-sim check` (#2586): the flute, shank
//! and holder bands of each move, and the tool body of each rapid,
//! against each `--fixture` box, by the region-free laws of `verify`.
//! Provenance: the fixture laws of `camcraft_sim::verify` (M3.2 stage B).

use std::collections::BTreeMap;

use camcraft_sim::cl::Toolpath;
use camcraft_sim::fixture::FixtureBox;
use camcraft_sim::geometry::Point3;
use camcraft_sim::tools::ToolGeom;
use camcraft_sim::verify::{bands_vs_fixtures, rapids_vs_fixtures, FixturePart, Warning};

use super::args::ToolSpec;
use super::frames::{shift, shifted, span};
use super::program::Program;
use super::report::Report;

const CHECK: &str = "fixture law";

/// The tool stack of a T map entry, or `None` where it states no flute
/// length and holder.
fn geom(t: &ToolSpec) -> Option<ToolGeom> {
    let (flute, holder) = (t.flute?, t.holder?);
    let (shank_d, shank_len) = t.shank.unwrap_or((t.diameter, 0.0));
    Some(ToolGeom {
        radius: 0.5 * t.diameter,
        nose_radius: 0.0,
        flute_len: flute,
        shank_radius: 0.5 * shank_d,
        shank_len,
        holder_radius: 0.5 * holder.0,
        holder_len: holder.1,
    })
}

/// Judge each section against the boxes (in the stock's frame).
pub fn check(program: &Program, boxes: &[[f64; 6]], tools: &BTreeMap<u32, ToolSpec>, report: &mut Report) {
    if boxes.is_empty() {
        return report.pass(CHECK, "no fixture is stated (--fixture): nothing stands in the work area to strike");
    }
    let boxes: Vec<FixtureBox> = boxes
        .iter()
        .map(|b| FixtureBox { min: Point3::new(b[0], b[1], b[2]), max: Point3::new(b[3], b[4], b[5]) })
        .collect();
    let base = program.sections.first().map(|s| (s.offset, s.origin));
    let (mut judged, mut found) = (0usize, 0usize);
    for s in &program.sections {
        let Some((first, last)) = span(&s.lines, &s.skip) else { continue };
        let Some(d) = base.and_then(|b| shift(b, (s.offset, s.origin))) else {
            report.not_judged(CHECK, &format!("lines {first}-{last}: the machine positions of G{} and the stock's offset are not both given (--wcs)", 53 + s.offset));
            continue;
        };
        let Some(g) = s.tool.and_then(|n| tools.get(&n)).and_then(geom) else {
            let t = s.tool.map_or_else(|| "no tool".to_owned(), |n| format!("T{n}"));
            report.not_judged(CHECK, &format!("lines {first}-{last}: {t} has no flute length and holder in --tools"));
            continue;
        };
        // Keep only the moves that the run knows, with their NC lines.
        let (moves, lines): (Vec<_>, Vec<usize>) = s
            .moves
            .iter()
            .zip(&s.lines)
            .zip(&s.skip)
            .filter(|(_, k)| !**k)
            .map(|((m, l), _)| (shifted(m, d), *l))
            .unzip();
        let path = Toolpath::new(moves);
        let mut hits: Vec<Warning> = bands_vs_fixtures(&path, &g, &boxes, None, 0.01);
        hits.extend(rapids_vs_fixtures(&path, &g, &boxes));
        let mut seen: Vec<usize> = Vec::new();
        for w in hits {
            if let Warning::FixtureHit { at, fixture_part } = w
                && !seen.contains(&at)
            {
                seen.push(at);
                found += 1;
                let line = lines.get(at).copied().unwrap_or_default();
                let part = match fixture_part {
                    FixturePart::Shank => "shank",
                    FixturePart::Holder => "holder",
                    FixturePart::Spindle => "spindle housing",
                    FixturePart::Rapid => "tool on a rapid",
                    FixturePart::Flute => "flute",
                };
                let t = s.tool.unwrap_or_default();
                report.fail(CHECK, &format!("line {line}: the {part} of T{t} strikes a fixture box"));
            }
        }
        judged += lines.len();
    }
    if judged > 0 && found == 0 {
        report.pass(CHECK, &format!("{judged} moves clear of {} fixture boxes", boxes.len()));
    }
}
