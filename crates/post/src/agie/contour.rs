//! Quantize the nominal polygon and find an exterior perpendicular Agie punch entry (planar geometry).
use super::AgieJob;
use crate::agie_setup::PunchSetup20;

type Point = [i64; 2];
const UNITS: f64 = 100_000.0; // Five decimal places in the native ISO geometry file.
const MAX_VERTICES: usize = 4096;
// CCSEP20's 20 mm QN2 rough pass stands 0.245 mm outside each rectangular
// face. Include the 0.125 mm wire radius and round the guard out to 0.4 mm.
const STOCK_MARGIN: i64 = 40_000;

fn quantize(v: f64) -> Result<i64, String> {
    if !v.is_finite() || v.abs() > 9999.9999 {
        return Err("Agie coordinates must be finite and within ±9999.9999 mm".into());
    }
    Ok((v * UNITS).round() as i64)
}

fn cross(a: Point, b: Point, p: Point) -> i128 {
    i128::from(b[0] - a[0]) * i128::from(p[1] - a[1])
        - i128::from(b[1] - a[1]) * i128::from(p[0] - a[0])
}

fn between(a: Point, b: Point, p: Point) -> bool {
    (0..2).all(|d| (a[d].min(b[d])..=a[d].max(b[d])).contains(&p[d]))
}

fn intersects(a: Point, b: Point, c: Point, d: Point) -> bool {
    let (u, v, w, z) = (
        cross(a, b, c),
        cross(a, b, d),
        cross(c, d, a),
        cross(c, d, b),
    );
    (u.signum() * v.signum() == -1 && w.signum() * z.signum() == -1)
        || (u == 0 && between(a, b, c))
        || (v == 0 && between(a, b, d))
        || (w == 0 && between(c, d, a))
        || (z == 0 && between(c, d, b))
}

fn micrometres(value: f64) -> Result<i32, String> {
    // quantize rejects nonfinite or out-of-range geometry before conversion.
    let hundredth_microns = quantize(value)?;
    let microns = i32::try_from(hundredth_microns / 100)
        .map_err(|_| "Agie setup coordinate exceeds the micrometre range")?;
    if (value - f64::from(microns) / 1000.0).abs() > 1e-9 {
        return Err(
            "Agie stock and start-hole coordinates must be representable to 0.001 mm".into(),
        );
    }
    Ok(microns)
}

fn punch_setup<'a>(job: &AgieJob, program: &'a str) -> Result<PunchSetup20<'a>, String> {
    let [width, depth, height] = std::array::from_fn(|i| job.stock_max[i] - job.stock_min[i]);
    if !(width > 0.0 && width <= 350.0 && depth > 0.0 && depth <= 250.0) {
        return Err("Agie stock must fit the stated 350 × 250 mm working span".into());
    }
    if job.stock_max[2] != 0.0 || height != 20.0 {
        return Err(
            "The Agie Berco Fine Ra0.30 profile requires stock top Z0 and thickness 20 mm".into(),
        );
    }
    if !(0..2)
        .all(|i| job.start[i] - 1.0 >= job.stock_min[i] && job.start[i] + 1.0 <= job.stock_max[i])
    {
        return Err("The complete 2 mm Agie start hole must lie inside the stock".into());
    }
    Ok(PunchSetup20 {
        stem: program,
        stock_min_um: [
            micrometres(job.stock_min[0])?,
            micrometres(job.stock_min[1])?,
        ],
        stock_max_um: [
            micrometres(job.stock_max[0])?,
            micrometres(job.stock_max[1])?,
        ],
        start_hole_um: [micrometres(job.start[0])?, micrometres(job.start[1])?],
    })
}

pub(super) fn prepare<'a>(
    job: &AgieJob,
    program: &'a str,
) -> Result<(Vec<[f64; 2]>, PunchSetup20<'a>), String> {
    let setup = punch_setup(job, program)?;
    let input = &job.contour;
    if input.len() < 3 || input.len() > MAX_VERTICES + 1 {
        return Err(format!(
            "Agie needs one closed nominal polygon with 3–{MAX_VERTICES} vertices"
        ));
    }
    let mut ring = Vec::with_capacity(input.len());
    for &[x, y] in input {
        let p = [quantize(x)?, quantize(y)?];
        if !(0..2).all(|i| {
            let value = p[i] as f64 / UNITS;
            value >= job.stock_min[i] && value <= job.stock_max[i]
        }) {
            return Err("Agie nominal contour extends outside the stock".into());
        }
        ring.push(p);
    }
    if input.first() == input.last() {
        ring.pop();
    }
    let n = ring.len();
    if !(3..=MAX_VERTICES).contains(&n) {
        return Err("Agie contour vertex count exceeds the supported range".into());
    }
    // Validate quantized segments, including the closing edge, before choosing a start point.
    // Keep collinear intermediate vertices out of the search for the perpendicular entry edge.
    let mut outline = Vec::with_capacity(n);
    for i in 0..n {
        let a = ring[(i + n - 1) % n];
        let b = ring[i];
        let c = ring[(i + 1) % n];
        if a == b || (cross(a, b, c) == 0 && !between(a, c, b)) {
            return Err("Agie nominal contour contains a collapsed or reversed edge".into());
        }
        if cross(a, b, c) != 0 {
            outline.push(b);
        }
    }
    let n = outline.len();
    let mut area = 0_i128;
    for i in 0..n {
        let (a, b) = (outline[i], outline[(i + 1) % n]);
        area += cross([0, 0], a, b);
        for j in (i + 2)..n {
            if i == 0 && j + 1 == n {
                continue;
            }
            if intersects(a, b, outline[j], outline[(j + 1) % n]) {
                return Err("Agie nominal contour crosses or touches itself".into());
            }
        }
    }
    if area == 0 {
        return Err("Agie nominal contour has no enclosed area".into());
    }
    if n != 4
        || (0..n).any(|i| {
            let (a, b) = (outline[i], outline[(i + 1) % n]);
            a[0] != b[0] && a[1] != b[1]
        })
    {
        return Err(
            "The verified Agie rough-pass envelope requires an axis-aligned rectangular punch"
                .into(),
        );
    }
    // The hole (radius 1 mm) and the expanded rectangle both fit in the
    // convex stock. The straight perpendicular entry between them fits too.
    for i in 0..2 {
        let left = quantize(job.stock_min[i])? + STOCK_MARGIN;
        let right = quantize(job.stock_max[i])? - STOCK_MARGIN;
        if outline.iter().any(|p| !(left..=right).contains(&p[i])) {
            return Err("The Agie rough pass and wire need at least 0.4 mm of stock beyond every punch face".into());
        }
    }
    if area < 0 {
        outline.reverse();
    }
    let hole = [quantize(job.start[0])?, quantize(job.start[1])?];
    let (edge, foot) = entry_face(&outline, hole).ok_or(
        "Agie needs the start hole at least 3 mm outside a straight face, with at least 3 mm of straight face on both sides of the perpendicular entry for the lead and separation"
    )?;
    // Begin the nominal program at the middle of the chosen edge, facing along that edge. The
    // hole and the full three-millimetre lead share its exterior half-plane. Compensation is
    // deliberately left to the Agie technology. The closing move returns to the inserted point.
    outline.rotate_left((edge + 1) % n);
    let mut vertices = vec![foot];
    vertices.extend(outline);
    let vertices = vertices
        .into_iter()
        .map(|p| [p[0] as f64 / UNITS, p[1] as f64 / UNITS])
        .collect();
    Ok((vertices, setup))
}

fn entry_face(ring: &[Point], hole: Point) -> Option<(usize, Point)> {
    let mut best = None;
    let mut nearest = f64::INFINITY;
    for (i, &a) in ring.iter().enumerate() {
        let b = ring[(i + 1) % ring.len()];
        let side = cross(a, b, hole);
        if side >= 0 || ring.iter().any(|&p| cross(a, b, p) < 0) {
            continue;
        }
        let dx = (b[0] - a[0]) as f64;
        let dy = (b[1] - a[1]) as f64;
        let length = dx.hypot(dy);
        let distance = -(side as f64) / length;
        if distance < 3.0 * UNITS || distance >= nearest {
            continue;
        }
        let t =
            (((hole[0] - a[0]) as f64) * dx + ((hole[1] - a[1]) as f64) * dy) / (length * length);
        if !(0.0 < t && t < 1.0) {
            continue;
        }
        let foot = [
            (a[0] as f64 + t * dx).round() as i64,
            (a[1] as f64 + t * dy).round() as i64,
        ];
        // Room on each side for the control's 3 mm transition and 1 mm
        // separation, independent of which way the resident pass runs.
        if cross(a, b, foot) != 0
            || [a, b]
                .iter()
                .any(|p| (foot[0] - p[0]).abs().max((foot[1] - p[1]).abs()) < 300_000)
        {
            continue;
        }
        nearest = distance;
        best = Some((i, foot));
    }
    best
}
