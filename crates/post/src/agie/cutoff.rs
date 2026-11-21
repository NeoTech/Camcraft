//! Straight open QN2 cut from the Agie Processor open-work format and CAMCraft's controller trials.
use super::{AgieJob, AgieSide, contour};

/// Air tails and perpendicular approach are the geometry measured on the qualified 20 mm cycle.
const TAIL: f64 = 4.0;
const APPROACH: f64 = 3.0;
const STROKES: [f64; 2] = [350.0, 250.0];

pub(super) struct Cutoff {
    pub line: [[f64; 2]; 2],
    pub entry: [f64; 2],
    pub exit: [f64; 2],
}

pub(super) fn prepare(job: &AgieJob) -> Result<Cutoff, String> {
    let rough_mm = f64::from(contour::stock_offset_um(job)?) / 1000.0;
    let margin = rough_mm + 0.125; // furthest wire centre plus wire radius
    if job.stock_min[2] != -20.0 || job.start.is_some() {
        return Err("Agie open cutoffs require 20 mm stock and two outside air endpoints; clear the drilled start hole".into());
    }
    let &[a, b] = job.contour.as_slice() else {
        return Err("Select one straight axis-aligned edge across the stock".into());
    };
    // SBL start points have three decimal places. Require the ISO and SBL
    // geometries to share that grid so rounding cannot shift the nominal cut.
    if [a, b]
        .into_iter()
        .flatten()
        .any(|v| !v.is_finite() || (v - (v * 1000.).round() / 1000.).abs() > 1e-9)
    {
        return Err("Agie open cutoff coordinates must be representable to 0.001 mm".into());
    }
    let along = if a[0] != b[0] && a[1] == b[1] {
        0
    } else if a[0] == b[0] && a[1] != b[1] {
        1
    } else {
        return Err("Agie open cutoffs require a straight edge parallel to X or Y".into());
    };
    let across = 1 - along;
    if a[along].min(b[along]) != job.stock_min[along]
        || a[along].max(b[along]) != job.stock_max[along]
    {
        return Err("Both ends of the selected edge must meet opposite stock faces".into());
    }
    let sign = (b[along] - a[along]).signum();
    let left = if along == 0 { sign } else { -sign };
    let waste = match job.side {
        AgieSide::Left => left,
        AgieSide::Right => -left,
        _ => return Err("An open Agie cut needs an explicit left or right kerf side".into()),
    };
    let mut line = [a, b];
    line[0][along] -= sign * TAIL;
    line[1][along] += sign * TAIL;
    let [mut entry, mut exit] = line;
    entry[across] += waste * APPROACH;
    exit[across] += waste * APPROACH;
    // Keep the cut away from either stock corner. Include the approach and
    // all smaller trim offsets. Each change of side happens at an air tail.
    for point in [a, entry] {
        if point[across] - margin < job.stock_min[across]
            || point[across] + margin > job.stock_max[across]
        {
            return Err(
                "The Agie open cut and its approach need clearance from the stock corners".into(),
            );
        }
    }
    // The operation explicitly confirms that stock centre is the installed
    // XY travel midpoint. Include the work origin used for non-cutting moves.
    for (i, span) in STROKES.iter().enumerate() {
        let mid = (job.stock_min[i] + job.stock_max[i]) / 2.;
        for p in [line[0], line[1], entry, exit, [0., 0.]] {
            if (p[i] - mid).abs() + margin > span / 2. {
                return Err(
                    "The Agie air tails, approach or work origin exceed the installed XY travel"
                        .into(),
                );
            }
        }
    }
    Ok(Cutoff { line, entry, exit })
}
