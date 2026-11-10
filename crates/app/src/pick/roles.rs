//! WCS-from-features roles (#701): every pick carries a chosen role -
//! Origin, X, Y or Z - over flat faces, straight edges, vertices and
//! curved walls. The first axis pick in Z, X, Y priority reads its axis
//! exactly, the second is projected perpendicular to it, and the third
//! axis follows right-handed - the third pick, if any, is overridden
//! wholesale (#187's rule generalized). A curved wall is new maths here:
//! its axis is the null space of the area-weighted facet-normal
//! covariance and its centre an algebraic (Kåsa) circle fit of the wall
//! vertices projected square to that axis, so a tilted bore reads true.
//! Pure: the UI reads the result and sends the existing `SetWcs`
//! command; nothing here writes the world. `align_321` is the old fixed
//! role table; this is the general one it grew out of.
//!
//! Provenance: datum-plane/line/point alignment as taught in coordinate
//! metrology; the covariance null-space axis fit and Kåsa's algebraic
//! circle fit (Kåsa, 1979) are standard; the Jacobi eigenvalue sweep is
//! the classic one for a real symmetric 3x3. Our own code, no vendor
//! material consulted.

use flecs_ecs::prelude::World;

use crate::geometry::{Triangle, feature, quat_to_euler, transform_point};
use crate::model;
use crate::operation::{FeatureSel, GeoSel};

use super::align::{
    Alignment, MIN_EDGE_TILT_DEG, cross, dot, gone, norm, outward_normal, quat_from_axes, scale,
    sub, straight_edge, vertex,
};
use super::face::face_origin;

/// The part of the frame a pick sets: the work origin, or one of the
/// three axes. At most one pick per role; the third axis (and any pick
/// standing where it would land) is derived, never read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Origin,
    X,
    Y,
    Z,
}

impl Role {
    fn label(self) -> &'static str {
        match self {
            Role::Origin => "Origin",
            Role::X => "X",
            Role::Y => "Y",
            Role::Z => "Z",
        }
    }
}

/// What one pick contributes, already placed in the viewer's frame: a
/// unit direction (axes only), an anchor point (origin, or the point an
/// axis stands at), and a datum plane ONLY when the pick is a flat face
/// - a line or a fitted wall is no plane, so it projects nothing.
struct Datum {
    dir: Option<[f64; 3]>,
    anchor: [f64; 3],
    plane: Option<([f64; 3], [f64; 3])>,
    /// A straight edge's two ends, so an origin pick can side the
    /// direction, exactly as #187 did.
    ends: Option<([f64; 3], [f64; 3])>,
}

/// The first two axis roles, in the order the frame is built: the first
/// present reads exactly, the second is projected. The third axis always
/// follows right-handed - its pick, if any, is never resolved.
const AXIS_PRIORITY: [Role; 3] = [Role::Z, Role::X, Role::Y];

/// A wall's facet normals must span at least this much to hold an axis
/// (a flat patch has none to fit). Relative to the mean squared norm.
const FLAT_FRACTION: f64 = 1e-8;

/// A wall's radii must agree within this fraction of the mean for the
/// circle fit to hold a centre (a crumpled wall is no bore).
const RING_BAND: f64 = 0.15;

/// Build a work offset from role-tagged picks: every pick says what it
/// sets - `Origin`, `X`, `Y` or `Z`. The first two axis roles present, in
/// `AXIS_PRIORITY` order, build the frame: the first reads its axis
/// exactly, the second is projected square to it; the third follows
/// right-handed and its pick, if any, is overridden wholesale (#187's
/// rule for any pair). The origin is the Origin pick, else the second
/// axis pick's anchor; a flat face's plane projects it (#187 again).
///
/// Refusals come in order: too few picks, a role set twice, too few
/// axes, then each datum by name, then how square the two axes stand.
pub fn align_roles(world: &World, picks: &[(GeoSel, Role)]) -> Result<Alignment, String> {
    if picks.len() < 2 {
        return Err("set the WCS from at least two picks - two axes, or an axis and an origin".into());
    }
    for r in [Role::Origin, Role::X, Role::Y, Role::Z] {
        if picks.iter().filter(|(_, role)| *role == r).count() > 1 {
            return Err(format!("only one pick may set {} - re-pick one of them", r.label()));
        }
    }
    // The two picks that carry the frame's axes, in priority order. A
    // third axis pick sits outside this list unread: that axis is
    // derived, so its pick is never resolved and never checked - even a
    // pick standing parallel to the first rides along.
    let frame: Vec<(usize, Role)> = AXIS_PRIORITY
        .iter()
        .filter_map(|r| picks.iter().position(|(_, role)| *role == *r).map(|i| (i, *r)))
        .collect();
    if frame.len() < 2 {
        return Err("set the WCS from two axes - a face, a straight edge or a wall each set one".into());
    }
    let (g1, g2) = (&picks[frame[0].0].0, &picks[frame[1].0].0);
    let mut d1 = datum_for(world, g1, frame[0].1)?;
    let mut d2 = datum_for(world, g2, frame[1].1)?;
    let origin = picks.iter().find(|(_, role)| *role == Role::Origin).map(|(g, _)| g.clone());
    let raw = match &origin {
        Some(g) => datum_for(world, g, Role::Origin)?.anchor,
        None => d2.anchor,
    };
    if origin.is_some() {
        // #187's siding rule, for either axis pick: one backed by a
        // straight edge runs from the end nearer the origin toward the
        // other, so the frame points away from the work, not at it.
        for d in [&mut d1, &mut d2] {
            if let (Some(dir), Some((a, b))) = (d.dir, d.ends) {
                if norm(sub(a, raw)) > norm(sub(b, raw)) {
                    d.dir = Some(scale(dir, -1.0));
                }
            }
        }
    }
    let a = d1.dir.ok_or_else(|| format!("{} of '{}' sets no axis - re-pick it", g1.feature, g1.entity))?;
    // The projection maths is #187's step for step, so a role table with
    // the old fixed roles agrees with `align_321` to the last digit.
    let tilt = MIN_EDGE_TILT_DEG.to_radians().sin();
    let (x, y, z) = match (frame[0].1, frame[1].1) {
        (Role::Z, Role::X) => {
            let x = square_to(g2, d2.dir, a, tilt)?;
            (x, cross(a, x), a)
        }
        (Role::Z, Role::Y) => {
            let x = square_to(g2, d2.dir, a, tilt)?;
            (x, a, cross(x, a))
        }
        (Role::X, Role::Z) => {
            let z = square_to(g2, d2.dir, a, tilt)?;
            (a, cross(z, a), z)
        }
        (Role::X, Role::Y) => {
            let y = square_to(g2, d2.dir, a, tilt)?;
            (a, y, cross(a, y))
        }
        (Role::Y, Role::Z) => {
            let z = square_to(g2, d2.dir, a, tilt)?;
            (cross(a, z), a, z)
        }
        (Role::Y, Role::X) => {
            let x = square_to(g2, d2.dir, a, tilt)?;
            (x, a, cross(x, a))
        }
        _ => return Err("two different axes set the frame - re-pick one of them".into()),
    };
    // #187's projection rule: the origin lands on the datum plane only
    // when the exact axis read it from a flat face - a line or a fitted
    // wall is no plane, so a bore's centre stays in its bore.
    let pos = match d1.plane {
        Some((p0, n)) => sub(raw, scale(n, dot(sub(raw, p0), n))),
        None => raw,
    };
    let q = quat_from_axes(x, y, z).ok_or("the picks do not make a usable frame - re-pick them")?;
    let (r, p, w) = quat_to_euler(q);
    let out = Alignment {
        pos,
        roll_deg: r.to_degrees(),
        pitch_deg: p.to_degrees(),
        yaw_deg: w.to_degrees(),
        x,
        y,
        z,
    };
    if out.pos.iter().chain(&[out.roll_deg, out.pitch_deg, out.yaw_deg]).all(|v| v.is_finite()) {
        Ok(out)
    } else {
        Err("the picks do not make a finite frame - re-pick them".into())
    }
}

/// The second axis pick, projected square to the first: #187's maths -
/// project, check the lean against `tilt` (already a sine), normalize.
/// Refused by name when the two picks stand too near parallel.
fn square_to(g: &GeoSel, d: Option<[f64; 3]>, axis: [f64; 3], tilt: f64) -> Result<[f64; 3], String> {
    let d = d.ok_or_else(|| format!("{} of '{}' sets no axis - re-pick it", g.feature, g.entity))?;
    let len = norm(d);
    let ex = sub(d, scale(axis, dot(d, axis)));
    let ex_len = norm(ex);
    if !len.is_finite() || len <= 0.0 || ex_len < len * tilt {
        return Err(format!(
            "{} of '{}' is too near parallel to the first axis - pick picks that lean apart",
            g.feature, g.entity
        ));
    }
    Ok(scale(ex, 1.0 / ex_len))
}

/// Read one pick into the datum it contributes, placed in the viewer's
/// frame (#326). An axis role reads a direction - a flat face its
/// outward normal, a straight edge its walk, a curved wall its fitted
/// axis; `Origin` reads a point - a vertex, a flat face's centroid or a
/// wall's centre. Only a flat face carries a plane to project onto. What
/// a pick cannot set for its role is refused by name, never guessed.
fn datum_for(world: &World, g: &GeoSel, role: Role) -> Result<Datum, String> {
    let pose = crate::setup_align::view_pose;
    match (g.feature, role) {
        (FeatureSel::Vertex(_), Role::Origin) => Ok(Datum {
            dir: None,
            anchor: vertex(world, g)?,
            plane: None,
            ends: None,
        }),
        (FeatureSel::Vertex(_), _) => Err(format!(
            "{} of '{}' sets no axis - pick a face, a straight edge or a wall for it",
            g.feature, g.entity
        )),
        (FeatureSel::Chain(_), Role::Origin) => Err(format!(
            "{} of '{}' sets an axis - pick a vertex or a wall for the origin",
            g.feature, g.entity
        )),
        (FeatureSel::Chain(_), _) => {
            let (a, b) = straight_edge(world, g, pose, &format!("for {}", role.label()))?;
            // The raw walk vector is the direction: `align_roles` squares
            // and normalizes it against the first axis, #187's maths, and
            // the ends are what sides it.
            Ok(Datum { dir: Some(sub(b, a)), anchor: a, plane: None, ends: Some((a, b)) })
        }
        (FeatureSel::Face(_), _) => match outward_normal(world, g, pose)? {
            Some(_) if role == Role::Origin => {
                let c = face_origin(world, &g.entity, g.feature).map_err(|why| why.to_owned())?;
                Ok(Datum { dir: None, anchor: c.centroid, plane: None, ends: None })
            }
            Some(n) => {
                let c = face_origin(world, &g.entity, g.feature).map_err(|why| why.to_owned())?;
                Ok(Datum { dir: Some(n), anchor: c.centroid, plane: Some((c.centroid, n)), ends: None })
            }
            // Not flat: a curved face is a wall, and a wall fits its own
            // axis and centre for either role.
            None => wall_datum(world, g, role),
        },
        (FeatureSel::Patch(_), _) => wall_datum(world, g, role),
    }
}

/// A curved wall's datum, for either role: an axis fitted through its
/// facets, anchored at the wall's centre. No plane - a wall is no flat
/// face, so the origin is never projected onto it.
fn wall_datum(world: &World, g: &GeoSel, role: Role) -> Result<Datum, String> {
    let e = model::find_by_name(world, &g.entity)
        .ok_or_else(|| gone(g))?;
    let feats = model::features_of(world, &e)
        .ok_or_else(|| gone(g))?;
    // Facet ids straight from the feature, and a patch's faces' facets:
    // one list either way, then placed where the part sits (#326).
    let sel = match g.feature {
        FeatureSel::Face(fid) => feats.faces.iter().find(|f| f.id == fid).map(|f| f.facets.clone()),
        FeatureSel::Patch(pid) => feature::patches(&feats)
            .into_iter()
            .find(|p| p.id == pid)
            .map(|p| {
                p.faces
                    .iter()
                    .flat_map(|fid| {
                        feats.faces.iter().find(|f| f.id == *fid).map(|f| f.facets.clone()).unwrap_or_default()
                    })
                    .collect()
            }),
        _ => None,
    };
    let ids = sel.ok_or_else(|| gone(g))?;
    let tris = model::facets_of(world, &e).ok_or_else(|| gone(g))?;
    let loc = crate::setup_align::view_pose(world, &e).unwrap_or_default();
    let wall: Vec<Triangle> = ids
        .iter()
        .filter_map(|i| tris.get(*i as usize))
        .map(|t| Triangle::new(
            transform_point(t.a, loc.pos, loc.rot),
            transform_point(t.b, loc.pos, loc.rot),
            transform_point(t.c, loc.pos, loc.rot),
        ))
        .collect();
    let (axis, centre) = fit_wall(&g.entity, &wall)?;
    Ok(Datum {
        dir: (role != Role::Origin).then_some(axis),
        anchor: centre,
        plane: None,
        ends: None,
    })
}

/// Fit a curved wall from its placed triangles: its axis from the
/// scatter of its facet normals, its centre from the circle its wall
/// vertices trace around that axis. Both are our own least-squares -
/// normal-covariance for the axis, Kåsa for the circle - so a bore cut
/// askew still finds its true axis, not the model's guess.
///
/// The checks are the refusal rules: too few facets, a scatter that is
/// not a ring, and a wall whose radii do not agree are all no axis.
fn fit_wall(name: &str, tris: &[Triangle]) -> Result<([f64; 3], [f64; 3]), String> {
    let n = tris.len();
    if n < 8 {
        return Err(format!("wall of '{}' is too small to fit an axis from", name));
    }
    // Axis: the direction the facet normals vary least. Winding does
    // not matter: a normal and its negation are the same wall normal.
    let mut cov = [[0.0f64; 3]; 3];
    for t in tris {
        let (a, b, c) = ([t.a.x, t.a.y, t.a.z], [t.b.x, t.b.y, t.b.z], [t.c.x, t.c.y, t.c.z]);
        let un = cross(sub(b, a), sub(c, a));
        let len = norm(un);
        if len <= 0.0 || !len.is_finite() {
            continue;
        }
        // Weight by area: twice the triangle is |n|, and the unit
        // outer product |n_hat><n_hat| is nn / |n|^2.
        let w = 0.5 * len;
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += w * un[i] * un[j] / (len * len);
            }
        }
    }
    let (eval, evec) = jacobi_sym3(cov);
    if eval[2] <= 0.0 || eval[1] <= FLAT_FRACTION * eval[2] {
        // Degenerate wall: the normals do not spread over a disc, so
        // their flat direction is not unique - a flat fan is no wall.
        return Err(format!("wall of '{}' is too flat to fit an axis from", name));
    }
    // The flat direction is the axis, and its sign is pinned - the
    // +Z-pointing root, then +Y, then +X - so two callers fitting the
    // same wall always see the same frame, whatever the facet winding.
    let mut axis = evec[0];
    if axis[2] < 0.0 || (axis[2] == 0.0 && (axis[1] < 0.0 || (axis[1] == 0.0 && axis[0] < 0.0))) {
        axis = [-axis[0], -axis[1], -axis[2]];
    }
    // Centre: project the wall's vertices into a plane square to the
    // axis and fit the circle they trace (Kåsa: solve the 2x2 normal
    // equations of x^2+y^2 = 2c.x + d).
    let (u, v) = perp_basis(axis);
    let mut sx = 0.0;
    let mut sy = 0.0;
    let mut st = 0.0;
    let mut pts: Vec<[f64; 2]> = Vec::with_capacity(n * 3);
    for t in tris {
        for q in [t.a, t.b, t.c] {
            let p = [q.x, q.y, q.z];
            let x = dot(p, u);
            let y = dot(p, v);
            sx += x;
            sy += y;
            st += dot(p, axis);
            pts.push([x, y]);
        }
    }
    let m = pts.len() as f64;
    let mx = sx / m;
    let my = sy / m;
    let tbar = st / m;
    let mut a11 = 0.0;
    let mut a12 = 0.0;
    let mut a22 = 0.0;
    let mut b1 = 0.0;
    let mut b2 = 0.0;
    for [x, y] in &pts {
        let dx = x - mx;
        let dy = y - my;
        a11 += dx * dx;
        a12 += dx * dy;
        a22 += dy * dy;
        b1 += dx * (dx * dx + dy * dy);
        b2 += dy * (dx * dx + dy * dy);
    }
    let det = a11 * a22 - a12 * a12;
    let scale_det = (a11 + a22) * (a11 + a22);
    if det.abs() <= 1e-12 * scale_det || !det.is_finite() {
        return Err(format!("wall of '{}' does not curve enough to fit an axis from", name));
    }
    let icx = b1 * a22 - b2 * a12;
    let icy = b2 * a11 - b1 * a12;
    let cx = icx / det;
    let cy = icy / det;
    let rads: Vec<f64> = pts.iter().map(|[x, y]| norm([x - mx - cx, y - my - cy, 0.0])).collect();
    let mean_r = rads.iter().sum::<f64>() / rads.len() as f64;
    if mean_r <= 0.0 || !mean_r.is_finite() {
        return Err(format!("wall of '{}' collapses to a point - no axis to fit", name));
    }
    let spread = rads
        .iter()
        .map(|r| (r - mean_r).abs())
        .fold(0.0f64, f64::max);
    if spread > RING_BAND * mean_r {
        return Err(format!("wall of '{}' is not round enough to fit an axis from", name));
    }
    // The circle's centre in the plane coordinates, walked back along
    // the basis, and set down at the wall's mean height along the axis.
    let centre = [
        (mx + cx) * u[0] + (my + cy) * v[0] + tbar * axis[0],
        (mx + cx) * u[1] + (my + cy) * v[1] + tbar * axis[1],
        (mx + cx) * u[2] + (my + cy) * v[2] + tbar * axis[2],
    ];
    Ok((axis, centre))
}

/// Two unit vectors square to `axis`, forming a right-handed basis with
/// it. Deterministic: built from the smallest component of the axis.
fn perp_basis(axis: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let i = if axis[0].abs() <= axis[1].abs() && axis[0].abs() <= axis[2].abs() {
        0
    } else if axis[1].abs() <= axis[2].abs() {
        1
    } else {
        2
    };
    let mut w = [0.0; 3];
    w[i] = 1.0;
    let u = {
        let t = cross(axis, w);
        scale(t, 1.0 / norm(t))
    };
    let v = cross(axis, u);
    (u, v)
}

/// Cyclic Jacobi for a real symmetric 3x3: eigenvalues ascending, with
/// matching eigenvectors as columns folded into `evec`. A handful of
/// sweeps always; the wall covariance is tiny.
fn jacobi_sym3(a: [[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
    let mut a = a;
    let mut v = [[0.0f64; 3]; 3];
    for i in 0..3 {
        v[i][i] = 1.0;
    }
    for _ in 0..32 {
        let off = norm([a[0][1], a[0][2], a[1][2]]);
        let diag = norm([a[0][0], a[1][1], a[2][2]]);
        if off <= 1e-15 * diag.max(1.0) {
            break;
        }
        for (p, q) in [(0, 1), (0, 2), (1, 2)] {
            let apq = a[p][q];
            if apq.abs() <= 1e-300 {
                continue;
            }
            let phi = 0.5 * (2.0 * apq).atan2(a[p][p] - a[q][q]);
            let (c, s) = (phi.cos(), phi.sin());
            // A' = J^T A J with J's (p,q) block [[c, -s], [s, c]].
            for k in 0..3 {
                let (akp, akq) = (a[k][p], a[k][q]);
                a[k][p] = c * akp + s * akq;
                a[k][q] = -s * akp + c * akq;
            }
            for k in 0..3 {
                let (akp, akq) = (a[p][k], a[q][k]);
                a[p][k] = c * akp + s * akq;
                a[q][k] = -s * akp + c * akq;
            }
            for k in 0..3 {
                let (vkp, vkq) = (v[k][p], v[k][q]);
                v[k][p] = c * vkp + s * vkq;
                v[k][q] = -s * vkp + c * vkq;
            }
        }
    }
    let mut idx = [0, 1, 2];
    idx.sort_by(|i, j| a[*i][*i].partial_cmp(&a[*j][*j]).unwrap_or(std::cmp::Ordering::Equal));
    let eval = [a[idx[0]][idx[0]], a[idx[1]][idx[1]], a[idx[2]][idx[2]]];
    // A_final = V^T A V with V built by right-multiplication, so
    // A V = V A_final: the eigenvectors are the COLUMNS of v, and
    // evec[i] belongs to eval[i] = the diagonal idx[i].
    let evec = core::array::from_fn(|i| [v[0][idx[i]], v[1][idx[i]], v[2][idx[i]]]);
    (eval, evec)
}
