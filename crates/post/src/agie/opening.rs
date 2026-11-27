//! Native ISO lines and arcs (G01, G02, G03) for one selected reentrant DXF die opening on a
//! 20 mm plate (#3178). Provenance: our own implementation of classical line and circle geometry
//! on the integer ISO grid, RS-274 / ISO 6983 arc words with incremental I/J, and the offline
//! control checks of #2957 (c_die, counter-clockwise nominal ISO) and #3178.

mod walls;

use std::f64::consts::TAU;
use std::fmt::Write;
use std::path::Path;

pub(crate) use walls::Span;

use super::contour::{self, Point};
use super::round_profile::{HEIGHT_UM, LEAD, exact};
use super::{AgieJob, AgieSide, ExportedJob, publish, valid_name};
use crate::agie_setup::{InspectionSegment, PunchSetup20, Quality, qn2_offsets_um};

const UNITS: f64 = 100_000.0; // Five decimal places in the native ISO geometry file.
const HOLE_RADIUS: f64 = 100_000.0; // The predrilled 2 mm threading hole.
const MAX_SPANS: usize = 512;
/// The strict wire cover law: at most 0.002 mm of rest in an inside corner of the plate.
const REST: f64 = 0.002 * UNITS;
/// The arc radius range of the native jobs, 0.5 to 20 mm (#2022, #2983).
const RADIUS: std::ops::RangeInclusive<f64> = 50_000.0..=2_000_000.0;

fn mm(v: f64) -> f64 {
    v / UNITS
}

fn at(p: Point) -> String {
    format!("X{:.5} Y{:.5}", mm(p[0] as f64), mm(p[1] as f64))
}

fn cross(a: Point, b: Point) -> i128 {
    i128::from(a[0]) * i128::from(b[1]) - i128::from(a[1]) * i128::from(b[0])
}

fn dot(a: Point, b: Point) -> i128 {
    i128::from(a[0]) * i128::from(b[0]) + i128::from(a[1]) * i128::from(b[1])
}

/// One source span on the exact grid, its turn checked against the source sweep.
fn span(s: &InspectionSegment) -> Result<Span, String> {
    let point = |p: camcraft_geometry::geo2d::Point2| -> Result<Point, String> {
        Ok([exact(p.x)?, exact(p.y)?])
    };
    let (from, to) = (point(s.start)?, point(s.end)?);
    if from == to {
        return Err(format!("The native grid collapses a wall of the die opening at {}", at(from)));
    }
    let Some((center, sweep)) = s.arc else { return Ok(Span::line(from, to)) };
    let c = point(center)?;
    let wall = Span { from, to, arc: Some((c, sweep > 0.0)) };
    let (r0, r1) = (Span::radius2_at(c, from), Span::radius2_at(c, to));
    if r0 == 0 {
        return Err(format!("The arc from {} has its centre on its end point", at(from)));
    }
    if r0 != r1 {
        return Err(format!(
            "Native arc radius rule: the arc from {} to {} about {} has start radius {:.5} mm and end radius {:.5} mm; the die opening needs equal radii on the 0.00001 mm grid, and the control moves the centre of an unequal arc or plots no path",
            at(from),
            at(to),
            at(c),
            mm((r0 as f64).sqrt()),
            mm((r1 as f64).sqrt())
        ));
    }
    if !sweep.is_finite() || sweep == 0.0 || sweep.abs() >= TAU || (wall.sweep() - sweep.abs()).abs() * wall.radius() > 4.0 {
        return Err(format!("The arc from {} about {} cannot keep its sweep on the native grid", at(from), at(c)));
    }
    Ok(wall)
}

/// The signed area of the loop, positive when it turns counter-clockwise.
fn area(walls: &[Span]) -> f64 {
    walls
        .iter()
        .map(|w| {
            let chord = cross(w.from, w.to) as f64;
            let cap = w.arc.map_or(0.0, |(_, ccw)| {
                let t = w.sweep();
                w.radius().powi(2) * (t - t.sin()) * if ccw { 1.0 } else { -1.0 }
            });
            (chord + cap) / 2.0
        })
        .sum()
}

/// The die opening walls, counter-clockwise from the seam, and the zero-step die setup.
///
/// The retained plate is outside the opening; the control offsets each wall
/// toward the waste inside it (c_die). Every check below uses the largest
/// resident offset of the 20 mm QN2 row, the rough pass.
pub(super) fn prepare<'a>(
    job: &AgieJob,
    sources: &[InspectionSegment],
    program: &'a str,
) -> Result<(Vec<Span>, PunchSetup20<'a>), String> {
    if job.side != AgieSide::Die || !job.contour.is_empty() {
        return Err("The native DXF opening is a die only".into());
    }
    let setup = contour::punch_setup(job, program, Quality::Ccb030)?;
    if setup.height_um != HEIGHT_UM {
        return Err("The native DXF die opening requires the qualified 20 mm stock height".into());
    }
    let [rough_um, ..] = qn2_offsets_um(HEIGHT_UM)
        .ok_or("The QN2 recipe has no measured offset for this stock thickness")?;
    let k = f64::from(rough_um) * 100.0;
    if !(2..=MAX_SPANS).contains(&sources.len()) {
        return Err(format!("The die opening needs 2 to {MAX_SPANS} connected walls"));
    }
    let mut walls = sources.iter().map(span).collect::<Result<Vec<_>, _>>()?;
    let n = walls.len();
    if (0..n).any(|i| walls[i].to != walls[(i + 1) % n].from) {
        return Err("The die opening has a joint gap on the native grid; no bridge was added".into());
    }
    let signed = area(&walls);
    if !signed.is_finite() || signed == 0.0 {
        return Err("The die opening has no enclosed area".into());
    }
    if signed < 0.0 {
        walls = walls.into_iter().rev().map(Span::reversed).collect();
    }
    for w in &walls {
        let Some((c, ccw)) = w.arc else { continue };
        let r = w.radius();
        // A counter-clockwise arc of the opening bends around its centre in
        // the waste: the control offsets it to radius r - k.
        if ccw && r <= k {
            return Err(format!(
                "Native arc offset rule: the arc about {} has radius {:.5} mm, not larger than the rough wire-centre offset {:.3} mm; the control drops such an arc from each pass whose offset exceeds its radius and leaves a sharp corner",
                at(c),
                mm(r),
                mm(k)
            ));
        }
        if !RADIUS.contains(&r) {
            return Err(format!("The native DXF radius must be from 0.5 to 20 mm; the arc about {} has radius {:.5} mm", at(c), mm(r)));
        }
    }
    let reach = corners(&walls, k)?;
    spacing(&walls, (2.0 * (reach + k)).ceil())?;
    let margin = contour::stock_margin(rough_um);
    stock(&walls, job, margin)?;
    let start = job.start.ok_or("The closed contour requires a predrilled 2 mm start hole")?;
    let hole = [exact(start[0])?, exact(start[1])?];
    inside(&walls, hole)?;
    let (seam, foot) = entry(&walls, hole, margin as f64)?;
    let mut out = vec![Span { from: foot, ..walls[seam] }];
    out.extend(walls[seam + 1..].iter().chain(&walls[..seam]).copied());
    out.push(Span { to: foot, ..walls[seam] });
    Ok((out, setup))
}

/// Check every joint: an arc joins its neighbours tangentially, and two
/// straight walls meet at a corner that the rough pass can serve. Returns the
/// longest reach of the rough pass from a nominal corner, at least `k`.
fn corners(walls: &[Span], k: f64) -> Result<f64, String> {
    let n = walls.len();
    let mut reach = k;
    for (i, a) in walls.iter().enumerate() {
        let b = &walls[(i + 1) % n];
        let (u, v) = (a.tangent_at(a.to), b.tangent_at(b.from));
        let (turn, along) = (cross(u, v), dot(u, v));
        if turn == 0 && along > 0 {
            continue;
        }
        if turn == 0 || a.arc.is_some() || b.arc.is_some() {
            return Err(format!(
                "Native joint rule: the walls of the die opening meet at {} without a common tangent; an arc must join its neighbours tangentially, and only two straight walls may meet at a sharp corner",
                at(a.to)
            ));
        }
        let (lu, lv) = ((dot(u, u) as f64).sqrt(), (dot(v, v) as f64).sqrt());
        let cos = (along as f64) / (lu * lv);
        if 1.0 + cos <= 1e-12 {
            return Err(format!("The rough pass cannot be bounded at the nearly reversed corner at {}", at(a.to)));
        }
        let corner = k * (2.0 / (1.0 + cos)).sqrt();
        // A left turn of the counter-clockwise opening is an inside corner of
        // the retained plate: the wire cannot reach into it.
        if turn > 0 && corner - k > REST {
            return Err(format!(
                "Strict cover rule: the inside corner of the plate at {} turns {:.3} degrees; the rough offset {:.3} mm leaves {:.4} mm of rest, more than 0.002 mm",
                at(a.to),
                cos.clamp(-1.0, 1.0).acos().to_degrees(),
                mm(k),
                mm(corner - k)
            ));
        }
        reach = reach.max(corner);
    }
    Ok(reach)
}

/// Keep the rough passes of unrelated walls apart: each pair of walls that
/// are not neighbours stays more than `clearance` apart. Two straight walls
/// joined by a tangent arc of at most a half turn diverge from it and are
/// separated by its offset, so that pair is exempt.
fn spacing(walls: &[Span], clearance: f64) -> Result<(), String> {
    let n = walls.len();
    let sharp = |i: usize| {
        let (a, b) = (&walls[i % n], &walls[(i + 1) % n]);
        cross(a.tangent_at(a.to), b.tangent_at(b.from)) != 0
    };
    for (i, w) in walls.iter().enumerate() {
        if w.arc.is_none() && (sharp(i + n - 1) || sharp(i)) && w.length() <= clearance {
            return Err(format!(
                "The rough corner joins cannot be separated on the {:.5} mm wall from {}; the rough passes and wire need {:.5} mm",
                mm(w.length()),
                at(w.from),
                mm(clearance)
            ));
        }
    }
    let exempt = |i: usize, j: usize| {
        let m = (i + 1) % n;
        j == (i + 2) % n
            && walls[i].arc.is_none()
            && walls[j].arc.is_none()
            && walls[m].arc.is_some()
            && walls[m].sweep() <= std::f64::consts::PI + 1e-12
            && !sharp(i)
            && !sharp(m)
    };
    for i in 0..n {
        for j in i + 2..n {
            if (i == 0 && j + 1 == n) || exempt(i, j) || exempt(j, i) {
                continue;
            }
            let d = walls[i].distance(&walls[j]);
            if d <= clearance {
                return Err(format!(
                    "The walls from {} and from {} of the die opening are {:.5} mm apart; the rough passes and wire need more than {:.5} mm",
                    at(walls[i].from),
                    at(walls[j].from),
                    mm(d),
                    mm(clearance)
                ));
            }
        }
    }
    Ok(())
}

/// Each wall, with the swept rough pass and wire, stays `margin` inside the stock.
fn stock(walls: &[Span], job: &AgieJob, margin: i64) -> Result<(), String> {
    let lo = [contour::quantize(job.stock_min[0])?, contour::quantize(job.stock_min[1])?];
    let hi = [contour::quantize(job.stock_max[0])?, contour::quantize(job.stock_max[1])?];
    let m = margin as f64;
    for q in walls.iter().flat_map(Span::extrema) {
        if (0..2).any(|i| q[i] - m < lo[i] as f64 || q[i] + m > hi[i] as f64) {
            return Err("The die opening needs at least 0.4 mm of stock beyond its walls and arc extrema".into());
        }
    }
    Ok(())
}

/// The complete hole lies inside the opening, more than 1 mm from every wall.
fn inside(walls: &[Span], hole: Point) -> Result<(), String> {
    let p = [hole[0] as f64, hole[1] as f64];
    let clear = walls.iter().all(|w| w.point_distance(p) > HOLE_RADIUS);
    // A point more than 1 mm from every wall is on the same side of each
    // arc and of its chords within 1 grid unit (0.00001 mm): count crossings.
    let ring: Vec<_> = walls.iter().flat_map(|w| w.chords(1.0)).collect();
    let mut inside = false;
    for (a, b) in ring.iter().zip(ring.iter().cycle().skip(1)) {
        if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < a[0] + (p[1] - a[1]) * (b[0] - a[0]) / (b[1] - a[1]) {
            inside = !inside;
        }
    }
    if !clear || !inside {
        return Err("The complete 2 mm start hole must lie inside the die opening, more than 1 mm from every wall".into());
    }
    Ok(())
}

/// Enter one straight wall parallel to X or Y perpendicular to it, from the
/// hole inside the opening. The hole is at least 3 mm from the wall, the foot
/// has 3 mm of that wall on both sides, and the straight lead stays more than
/// the rough pass and wire `margin` from every other wall. The nearest such
/// wall wins.
fn entry(walls: &[Span], hole: Point, margin: f64) -> Result<(usize, Point), String> {
    let mut best: Option<(i64, usize, Point)> = None;
    for (i, w) in walls.iter().enumerate() {
        let d = [w.to[0] - w.from[0], w.to[1] - w.from[1]];
        if w.arc.is_some() || (d[0] != 0 && d[1] != 0) {
            continue;
        }
        let foot = if d[1] == 0 { [hole[0], w.from[1]] } else { [w.from[0], hole[1]] };
        let along = |p: Point, q: Point| (q[0] - p[0]) * d[0].signum() + (q[1] - p[1]) * d[1].signum();
        let depth = (hole[0] - foot[0]).abs() + (hole[1] - foot[1]).abs();
        if along(w.from, foot) < LEAD || along(foot, w.to) < LEAD || depth < LEAD || cross(d, [hole[0] - w.from[0], hole[1] - w.from[1]]) <= 0 {
            continue;
        }
        let lead = Span::line(hole, foot);
        let clear = walls.iter().enumerate().all(|(j, other)| j == i || other.distance(&lead) > margin);
        if clear && best.is_none_or(|(near, _, _)| depth < near) {
            best = Some((depth, i, foot));
        }
    }
    best.map(|(_, i, foot)| (i, foot)).ok_or_else(|| {
        "The die opening needs a straight wall parallel to X or Y with the threading hole at least 3 mm inside it, 3 mm of that wall on both sides of the perpendicular entry, and the lead 0.4 mm clear of every other wall".into()
    })
}

/// The nominal ISO: the seam, then each wall with G01, G02 or G03 and incremental I/J centres.
pub(super) fn iso(iso_name: &str, walls: &[Span]) -> String {
    let v = |u: i64| u as f64 / UNITS;
    let mut iso = format!("( {iso_name} )\r\n");
    let first = walls.first().map_or([0, 0], |w| w.from);
    let _ = write!(iso, "N0001 G00 X{:.5} Y{:.5} ;\r\n", v(first[0]), v(first[1]));
    for (i, w) in walls.iter().enumerate() {
        let n = i + 2;
        let [x, y] = w.to;
        let _ = match w.arc {
            None => write!(iso, "N{n:04} G01 X{:.5} Y{:.5} ;\r\n", v(x), v(y)),
            Some((c, ccw)) => write!(
                iso,
                "N{n:04} {} X{:.5} Y{:.5} I{:.5} J{:.5} ;\r\n",
                if ccw { "G03" } else { "G02" },
                v(x),
                v(y),
                v(c[0] - w.from[0]),
                v(c[1] - w.from[1])
            ),
        };
    }
    let _ = write!(iso, "N{:04} M02 ;\r\n", walls.len() + 2);
    iso
}

/// Publish one zero-step die whose nominal ISO has native lines and arcs: a
/// selected, possibly reentrant DXF opening on a 20 mm plate. The SBL and SBR
/// are those of the polygon die ([`super::export_with_review`]).
///
/// # Errors
/// Refuses unsupported shapes, placements and settings before it publishes.
pub fn export_opening_with_review(
    job: &AgieJob,
    spans: &[InspectionSegment],
    destination: &Path,
    program: &str,
    separation_length_mm: f64,
) -> Result<ExportedJob, String> {
    if separation_length_mm != 1.0 {
        return Err("Set the separation cut length (separationLengthMm) to 1.0 mm for this closed contour. A zero or unspecified separation cannot pass the control formal check".into());
    }
    if !valid_name(program) {
        return Err("The job program name must be a non-reserved 1–8 character DOS name starting with a letter and using uppercase letters or digits".into());
    }
    let (walls, setup) = prepare(job, spans, program)?;
    let setup = setup.encode_die().map_err(|error| error.to_string())?;
    let iso_name = format!("{program}.ISO");
    let files = [
        (iso_name.clone(), iso(&iso_name, &walls)),
        (format!("{program}.SBR"), setup.sbr),
        (format!("{program}.SBL"), setup.sbl),
    ];
    Ok(ExportedJob {
        files: publish(destination, &files)?,
        controller_review: setup.controller_review,
    })
}
