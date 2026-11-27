//! The stock check of `camcraft-sim check` (#2586): the program cuts a
//! stock box move by move, and before each move is cut it is judged
//! against the material that is left: a rapid through material, and
//! the holder law (the shank and holder
//! bands over the flute must not meet material).
//! Provenance: height-field and distance-field stock removal is general
//! CAM verification practice from published literature (the `stock_sim`
//! module states its sources); the band stack over the flute is the
//! holder law of the `verify` module. The move-by-move order is ours.

use std::collections::BTreeMap;

use camcraft_sim::cl::{Move, MoveKind};
use camcraft_sim::geometry::Point3;
use camcraft_sim::stock_sim::{auto_cell, Backend, SimOp, Stock, FIELD_BAND};

use super::args::ToolSpec;
use super::frames::{shifted, span};
use super::program::Program;
use super::report::Report;

const CHECK: &str = "stock";
const HOLDER: &str = "holder law";
/// The most sample points along one move.
const MAX_SAMPLES: usize = 20_000;

/// The sample points of the tool tip along `mv`, about `step` apart,
/// the two ends exact.
pub fn samples(mv: &Move, step: f64) -> Vec<Point3> {
    let mut out = vec![mv.from];
    for (a, b) in mv.segments3(step * 0.25) {
        let len = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2) + (b.z - a.z).powi(2)).sqrt();
        let n = (len / step).ceil().clamp(1.0, MAX_SAMPLES as f64) as usize;
        for k in 1..=n {
            let f = k as f64 / n as f64;
            out.push(Point3::new(a.x + (b.x - a.x) * f, a.y + (b.y - a.y) * f, a.z + (b.z - a.z) * f));
        }
        if out.len() > MAX_SAMPLES {
            break;
        }
    }
    out
}

/// Whether the disc of radius `r` about `(x, y)` at height `z` holds
/// material deeper than `thr`: its centre and eight points on its rim.
fn disc_in_material(stock: &Stock, p: Point3, r: f64, thr: f64) -> bool {
    let probe = |x: f64, y: f64| {
        let d = stock.distance(Point3::new(x, y, p.z));
        d.is_finite() && d < -thr
    };
    if probe(p.x, p.y) {
        return true;
    }
    r > 0.0
        && (0..8).any(|k| {
            let a = f64::from(k) * std::f64::consts::FRAC_PI_4;
            probe(p.x + r * a.cos(), p.y + r * a.sin())
        })
}

/// The bands over the flute: (name, radius, height of the band bottom
/// over the tip). `None` where the T map does not state the stack.
fn bands(t: &ToolSpec) -> Option<Vec<(&'static str, f64, f64)>> {
    let (flute, holder) = (t.flute?, t.holder?);
    let (shank_d, shank_len) = t.shank.unwrap_or((t.diameter, 0.0));
    let mut out = Vec::new();
    if shank_len > 0.0 {
        out.push(("shank", 0.5 * shank_d, flute));
    }
    out.push(("holder", 0.5 * holder.0, flute + shank_len));
    Some(out)
}

/// Cut the pending moves into the stock, and empty the list.
fn cut(stock: &mut Stock, pending: &mut Vec<Move>, r: f64, ball: bool) -> Result<(), String> {
    if pending.is_empty() {
        return Ok(());
    }
    let op = SimOp { moves: pending, radius: r, ball, cone: None, corner: None };
    stock.simulate(&[op]).map_err(|e| e.to_string())?;
    pending.clear();
    Ok(())
}

/// What one move meets in a stock: the first point of a rapid in
/// material, and the first band over the flute in material.
struct Probe {
    rapid: Option<Point3>,
    band: Option<(&'static str, Point3)>,
}

impl Probe {
    fn any(&self) -> bool {
        self.rapid.is_some() || self.band.is_some()
    }

    fn run(st: &Stock, pts: &[Point3], rapid: bool, r: f64, stack: Option<&[(&'static str, f64, f64)]>, cell: f64, thr: f64) -> Self {
        let rapid = if rapid { pts.iter().find(|p| disc_in_material(st, **p, (r - cell).max(0.0), thr)).copied() } else { None };
        let band = stack.and_then(|stack| {
            pts.iter().find_map(|p| {
                stack
                    .iter()
                    .find(|(_, br, off)| disc_in_material(st, Point3::new(p.x, p.y, p.z + off), (br - cell).max(0.0), thr))
                    .map(|(name, _, _)| (*name, *p))
            })
        });
        Self { rapid, band }
    }
}

/// Judge and cut each section in order. `tools` is the T map.
pub fn check(program: &Program, stock_box: [f64; 6], tools: &BTreeMap<u32, ToolSpec>, cell: Option<f64>, report: &mut Report) {
    let [x0, y0, _, x1, y1, z1] = stock_box;
    let r_min = tools.values().map(|t| 0.5 * t.diameter).fold(f64::INFINITY, f64::min);
    let cell = cell.unwrap_or_else(|| auto_cell(r_min, x1 - x0, y1 - y0));
    let mut stock = match Stock::block(Backend::Field { band: FIELD_BAND }, x0, y0, x1, y1, z1, cell) {
        Ok(s) => s,
        Err(e) => return report.not_judged(CHECK, &format!("the stock box does not build: {e}")),
    };
    let thr = cell.max(0.01);
    let base = program.sections.first().map(|s| (s.offset, s.origin));
    let (mut judged, mut rapids, mut found, mut holder_found) = (0usize, 0usize, 0usize, 0usize);
    let mut holder_unstated: Vec<u32> = Vec::new();
    for s in &program.sections {
        let Some((first, last)) = span(&s.lines, &s.skip) else { continue };
        let Some(shift) = base.and_then(|b| super::frames::shift(b, (s.offset, s.origin))) else {
            report.not_judged(
                CHECK,
                &format!("lines {first}-{last} (G{}): the machine positions of this work offset and the stock's are not both given (--wcs)", 53 + s.offset),
            );
            continue;
        };
        let tool = match s.tool.map(|n| (n, tools.get(&n))) {
            Some((_, Some(t))) => *t,
            Some((n, None)) => {
                report.not_judged(CHECK, &format!("lines {first}-{last}: T{n} is not in --tools"));
                continue;
            }
            None => {
                report.not_judged(CHECK, &format!("lines {first}-{last}: no tool is loaded (no T word before them)"));
                continue;
            }
        };
        let t_no = s.tool.unwrap_or_default();
        let r = 0.5 * tool.diameter;
        let stack = bands(&tool);
        if stack.is_none() && !holder_unstated.contains(&t_no) {
            holder_unstated.push(t_no);
        }
        let mut pending: Vec<Move> = Vec::new();
        for ((mv, line), skip) in s.moves.iter().zip(&s.lines).zip(&s.skip) {
            if *skip {
                continue;
            }
            let mv = shifted(mv, shift);
            let pts = samples(&mv, cell);
            let probe = |st: &Stock| Probe::run(st, &pts, mv.kind == MoveKind::Rapid, r, stack.as_deref(), cell, thr);
            let mut hit = probe(&stock);
            // Material only goes: a probe that is clean on the stock
            // before the pending moves is clean after them. A suspected
            // hit is read again on the exact stock.
            if hit.any() && !pending.is_empty() {
                if let Err(e) = cut(&mut stock, &mut pending, r, tool.ball) {
                    return report.not_judged(CHECK, &format!("line {line}: the stock engine refuses a move: {e}; later moves are not judged"));
                }
                hit = probe(&stock);
            }
            if mv.kind == MoveKind::Rapid {
                rapids += 1;
            }
            if let Some(p) = hit.rapid {
                found += 1;
                report.fail(CHECK, &format!("line {line}: the rapid runs T{t_no} through material at X{:.3} Y{:.3} Z{:.3}", p.x, p.y, p.z));
            }
            if let Some((name, p)) = hit.band {
                holder_found += 1;
                report.fail(HOLDER, &format!("line {line}: the {name} of T{t_no} meets material at X{:.3} Y{:.3} Z{:.3} (tip)", p.x, p.y, p.z));
            }
            pending.push(mv);
            judged += 1;
        }
        if let Err(e) = cut(&mut stock, &mut pending, r, tool.ball) {
            return report.not_judged(CHECK, &format!("lines {first}-{last}: the stock engine refuses a move: {e}; later moves are not judged"));
        }
    }
    if judged == 0 {
        report.not_judged(CHECK, "no move of the program was judged");
        return;
    }
    if found == 0 {
        report.pass(CHECK, &format!("{judged} moves cut, {rapids} rapids clear of material (cell {cell:.4} mm)"));
    }
    for t in &holder_unstated {
        report.not_judged(HOLDER, &format!("T{t}: the T map states no flute length and holder (flute=, holder=)"));
    }
    if holder_found == 0 && judged > 0 && holder_unstated.is_empty() {
        report.pass(HOLDER, "no shank or holder band meets material");
    }
}
