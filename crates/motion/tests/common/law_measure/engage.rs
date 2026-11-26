//! One walk of a toolpath over the stock grid: the entries (L5) and the
//! engagement of each planar feed against the stock that is left (L6).
//! Provenance: the radial engagement angle (the contact angle of the
//! cutter rim with the uncut stock, Altintas, "Manufacturing Automation",
//! 2000, ch. 2), read on a pixel stock model as in Choy and Chan
//! (Computer-Aided Design 35, 2003), with the rim meter of
//! `common/mask_grid.rs` (`contact_width`); our own design record (#4024).

use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::geo2d::Point2;

use super::entries::{Entry, Open, descends};
use super::stock::{Stock, Z_EPS, points};
use super::{Reading, Spec};

/// One rim sample of the meter (72 samples over 180 degrees).
const RIM_STEP_DEG: f64 = 2.5;
/// A full-width cut reads all the samples of the forward half.
const FULL_DEG: f64 = 180.0 - 0.5 * RIM_STEP_DEG;

pub(super) struct Walk {
    pub entries: Vec<Entry>,
    pub over: Reading,
    pub worst_deg: f64,
    pub full_width: Reading,
}

/// The engagement angle in degrees from the radial depth that the rim
/// meter gives (`ae = r (1 - cos theta)`).
fn angle(ae: f64, r: f64) -> f64 {
    (1.0 - ae / r).clamp(-1.0, 1.0).acos().to_degrees()
}

pub(super) fn walk(path: &Toolpath, spec: &Spec, stock: &mut Stock) -> Walk {
    let mut out = Walk {
        entries: Vec::new(),
        over: Reading::default(),
        worst_deg: 0.0,
        full_width: Reading::default(),
    };
    let step = stock.grid.cell;
    let mut open: Option<Open> = None;
    for (i, m) in path.moves.iter().enumerate() {
        if descends(m, spec.top) {
            open = Some(Open::read(open.take(), i, m, stock));
        } else if let Some(o) = open.take() {
            out.entries.push(o.close(stock));
        }
        if m.from.z.min(m.to.z) >= spec.top {
            continue;
        }
        let cuts = m.kind != MoveKind::Rapid
            && (m.from.z - m.to.z).abs() < 1e-9
            && m.to.z < spec.top - Z_EPS;
        if !cuts && m.arc.is_none() {
            stock.sweep(m.from, m.to);
            continue;
        }
        let pts = points(m, step);
        let mut worst: Option<(f64, Point2)> = None;
        for w in pts.windows(2) {
            let [a, b] = [w[0], w[1]];
            stock.sweep(a, b);
            if !cuts {
                continue;
            }
            let at = Point2::new(b.x, b.y);
            let ae = stock.grid.contact_width(
                at,
                (b.x - a.x, b.y - a.y),
                b.z,
                stock.r,
                &stock.cuttable,
                &stock.top,
            );
            let deg = angle(ae, stock.r);
            if worst.is_none_or(|(d, _)| deg > d) {
                worst = Some((deg, at));
            }
        }
        if let Some((deg, at)) = worst {
            out.worst_deg = out.worst_deg.max(deg);
            if deg > spec.limit_deg + RIM_STEP_DEG {
                out.over.add(at);
            }
            if deg >= FULL_DEG {
                out.full_width.add(at);
            }
        }
    }
    if let Some(o) = open.take() {
        out.entries.push(o.close(stock));
    }
    out
}
