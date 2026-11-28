//! #3646: a multi-seat indexable milling cutter - a face mill or an
//! indexable end mill: one cutter body (a tool item, #3642) whose seats
//! (`CSW1_1..CSW1_N`, or one seat port and a seat count) all take one
//! shared insert - resolved to the milling `Tool` record the toolpath,
//! gouge and collision laws read. Our own design over the public
//! ISO 13399 cutter properties (`DC`, the cutting edge angle `KAPR`,
//! `APMX`) and the ISO 1832 insert code; textbook plane geometry. No
//! vendor source or file consulted.
//!
//! The law: the tool is never smaller than the real cutter. Inside the
//! cutting depth its cylinder holds every cutting edge, and its shank
//! holds the rest of the inserts above it. Each refusal names the
//! cutter body: bad numbers, a seat that does not take the insert, a
//! corner that does not fit, a body that stands outside the cutter, a
//! seat that the tool does not hold, an assembly with other components.

use crate::component::{ComponentKind, ToolComponent};
use crate::component_fit::check_insert_fit;
use crate::holder_profile::MAX_BODY_RADIUS;
use crate::insert_codes::InsertShape;
use crate::insert_iso1832::{Corner, InsertSpec};
use crate::taper::TaperTip;
use crate::tools::{Tool, ToolKind};

#[path = "milling_cutter_assembly.rs"]
mod assembly;
#[path = "milling_cutter_error.rs"]
mod error;
pub use error::MillingError;

/// The most seats one cutter has: a tool record counts 1..=32 flutes.
pub const MAX_SEATS: usize = 32;
/// A cutting edge angle this close to 90 degrees is read as 90.
const SQUARE_TOL: f64 = 1e-9;
/// How far (mm) the seat geometry may put the cutting point off `DC`.
pub const SEAT_TOL: f64 = 1e-3;

/// The cutter body's own facts, as its catalog states them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MillingBody {
    /// `DC`: the diameter where the main cutting edges (carried past
    /// the corner) meet the tip plane. On a round-insert cutter, the
    /// largest diameter the round edges sweep.
    pub dc: f64,
    /// `KAPR`: the angle (degrees) of the main cutting edge from the
    /// tip plane - 90 for a shoulder mill, 45 for a common face mill.
    /// Not read on a round-insert cutter.
    pub kapr_deg: f64,
    /// `APMX`: the largest axial depth of cut (mm).
    pub apmx: f64,
    /// Can it plunge into solid material? False for a face or shell
    /// mill with a centre hole (the M11.3 plunge refusal).
    pub center_cutting: bool,
    /// A seat count for a body that states one seat port for all its
    /// seats; `None` counts the body's seat ports.
    pub seat_count: Option<u32>,
}

/// What a multi-seat cutter resolves to, in the tool frame (tip at the
/// origin, +Z toward the spindle).
#[derive(Clone, Debug, PartialEq)]
pub struct MillingCutter {
    /// The cutter body's name, for the refusals that follow.
    pub name: String,
    /// The stated `DC`.
    pub dc: f64,
    pub kind: ToolKind,
    /// The cutting diameter: `DC` at 90 degrees and on round inserts;
    /// below 90 degrees the larger diameter that the edges reach at the
    /// top of the cutting depth (a `DC` cylinder would be smaller than
    /// the cutter there).
    pub diameter: f64,
    pub corner_r: f64,
    /// One cutting edge for each seat.
    pub flutes: u32,
    /// `APMX`, no deeper than the insert edge reaches at `KAPR`.
    pub flute_len: f64,
    pub center_cutting: bool,
    /// The diameter that holds the inserts above the cutting depth.
    pub insert_d: f64,
    /// How high above the tip the inserts reach.
    pub insert_top: f64,
    /// The cutter body and the shared insert this cutter was derived
    /// from: an assembly must mount exactly these (#3646 review).
    body: ToolComponent,
    insert: ToolComponent,
}

/// Resolve the cutter `body` with its facts `spec` and the shared
/// `insert` (the module's law).
pub fn derive(body: &ToolComponent, spec: &MillingBody, insert: &ToolComponent) -> Result<MillingCutter, MillingError> {
    let name = body.name().to_owned();
    for (what, value) in [("DC", spec.dc), ("cutting edge angle", spec.kapr_deg), ("APMX", spec.apmx)] {
        if !value.is_finite() {
            return Err(MillingError::NonFinite { name, what, value });
        }
    }
    if !(spec.dc > 0.0 && spec.dc <= 500.0) {
        return Err(MillingError::BadDiameter { name, dc: spec.dc });
    }
    if spec.apmx <= 0.0 {
        return Err(MillingError::BadDepth { name, apmx: spec.apmx });
    }
    if !matches!(body.kind(), ComponentKind::ToolItem { .. }) {
        return Err(MillingError::NotABody { name });
    }
    let ports = body.ports();
    let flutes = seat_count(&name, spec.seat_count, ports.len())?;
    for port in &ports {
        check_insert_fit(body, port, insert)?;
    }
    let Some(ins) = insert.insert() else {
        // The fit checks refuse a component that is not an insert.
        return Err(MillingError::NotABody { name: insert.name().to_owned() });
    };
    let shape = if ins.shape == InsertShape::R { round(&name, spec, ins)? } else { edged(&name, spec, ins)? };
    let kind = if shape.corner_r <= 0.0 {
        ToolKind::EndMill
    } else if (shape.corner_r - 0.5 * shape.diameter).abs() <= 1e-9 {
        ToolKind::BallNose
    } else {
        ToolKind::BullNose
    };
    Ok(MillingCutter {
        name,
        dc: spec.dc,
        kind,
        diameter: shape.diameter,
        corner_r: shape.corner_r,
        flutes,
        flute_len: shape.flute_len,
        center_cutting: spec.center_cutting,
        insert_d: shape.insert_d,
        insert_top: shape.insert_top,
        body: body.clone(),
        insert: insert.clone(),
    })
}

/// The seat count: the stated one (the body states one seat port for
/// all, or a port for each), else the body's seat ports.
fn seat_count(name: &str, stated: Option<u32>, ports: usize) -> Result<u32, MillingError> {
    let n = match stated {
        None => ports,
        Some(0) => return Err(MillingError::NoSeats { name: name.to_owned() }),
        Some(n) => {
            let count = usize::try_from(n).unwrap_or(usize::MAX);
            if ports != 1 && ports != count {
                return Err(MillingError::SeatCount { name: name.to_owned(), stated: n, ports });
            }
            count
        }
    };
    if n == 0 {
        return Err(MillingError::NoSeats { name: name.to_owned() });
    }
    u32::try_from(n)
        .ok()
        .filter(|_| n <= MAX_SEATS)
        .ok_or_else(|| MillingError::TooManySeats { name: name.to_owned(), count: n })
}

/// The cutter's meridian shape before it becomes a tool record.
struct Shape {
    diameter: f64,
    corner_r: f64,
    flute_len: f64,
    insert_d: f64,
    insert_top: f64,
}

/// A round insert: the corner is half its diameter (the size code, cut
/// to whole mm, so the model corner is never rounder than the insert);
/// the edge cuts up to half the insert; the insert stands no higher
/// than its diameter (the code plus 1 mm) and inside `DC`.
fn round(name: &str, spec: &MillingBody, ins: &InsertSpec) -> Result<Shape, MillingError> {
    let ic = f64::from(ins.size);
    let corner_r = 0.5 * ic;
    if corner_r > 0.5 * spec.dc {
        let why = format!("the Ø{ic} mm round insert is larger than the Ø{} mm cutting diameter", spec.dc);
        return Err(MillingError::Corner { name: name.to_owned(), why });
    }
    let flute_len = spec.apmx.min(corner_r);
    Ok(Shape { diameter: spec.dc, corner_r, flute_len, insert_d: spec.dc, insert_top: ic + 1.0 })
}

/// An insert with straight edges at the cutting edge angle `KAPR`.
fn edged(name: &str, spec: &MillingBody, ins: &InsertSpec) -> Result<Shape, MillingError> {
    let k = spec.kapr_deg;
    if !(k > 0.0 && k <= 90.0) {
        return Err(MillingError::BadLeadAngle { name: name.to_owned(), deg: k });
    }
    let square = k >= 90.0 - SQUARE_TOL;
    let (sin_k, cos_k) = if square { (1.0, 0.0) } else { k.to_radians().sin_cos() };
    let cot_k = cos_k / sin_k;
    let size = f64::from(ins.size);
    // The size code cuts the edge to whole mm: the true edge is no
    // shorter than `edge_lo` (the depth bound) and no longer than
    // `edge_hi` (the envelope bound).
    let ic = ins.inscribed_circle_mm();
    let edge_hi = match (ic, ins.shape.edge_per_ic()) {
        (Some(ic), Some(per)) => (ic * per).max(size),
        _ => size + 1.0,
    };
    let edge_lo = size.min(edge_hi);
    let flute_len = spec.apmx.min(edge_lo * sin_k);
    let corner_r = match ins.corner {
        Corner::Facet { .. } => 0.0,
        c => c.nose_radius_mm().unwrap_or(0.0),
    };
    let diameter = spec.dc + 2.0 * flute_len * cot_k;
    let corner = |why: String| Err(MillingError::Corner { name: name.to_owned(), why });
    if corner_r > 0.5 * spec.dc {
        return corner(format!("its {corner_r} mm nose radius is more than half the {} mm DC", spec.dc));
    }
    // Below 90 degrees the model corner sits on the wider diameter: it
    // holds the real corner only while it is no larger than the depth
    // and the edge's run-out over that depth.
    if !square && (corner_r > flute_len || corner_r > flute_len * cot_k) {
        return corner(format!(
            "its {corner_r} mm nose radius is larger than the {flute_len:.3} mm cutting depth or the {:.3} mm the edge runs out over it",
            flute_len * cot_k
        ));
    }
    // Any point of the insert is within its longest chord of the tip
    // corner, so no higher than that.
    let chord = match (ic, ins.shape.corner_deg()) {
        (Some(ic), Some(a)) => ic / (0.5 * a.to_radians()).sin(),
        _ => 2.0 * (size + 1.0),
    };
    // Past the top of the main edge the next edge turns inward by
    // 180 minus the corner angle; it runs farther out only when that
    // corner is more than 90 + KAPR degrees. Then take the safe bound:
    // the tip corner's radius plus the longest chord.
    let insert_r = if square {
        0.5 * spec.dc
    } else if max_corner(ins.shape) <= 90.0 + k {
        0.5 * spec.dc + edge_hi * cos_k
    } else {
        0.5 * spec.dc + chord
    };
    let insert_d = (2.0 * insert_r).max(diameter);
    if !insert_d.is_finite() || 0.5 * insert_d > MAX_BODY_RADIUS {
        let why = format!("the inserts reach Ø{insert_d:.3} mm, past the {MAX_BODY_RADIUS} mm body radius cap");
        return corner(why);
    }
    Ok(Shape { diameter, corner_r, flute_len, insert_d, insert_top: chord.max(edge_hi * sin_k) })
}

/// The largest corner angle of an insert shape (degrees).
fn max_corner(shape: InsertShape) -> f64 {
    use InsertShape as S;
    let a = shape.corner_deg().unwrap_or(180.0);
    match shape {
        S::A | S::B | S::C | S::D | S::E | S::K | S::M | S::V => 180.0 - a,
        S::W => 160.0,
        _ => a,
    }
}

impl MillingCutter {
    /// The milling tool record: this cutter's geometry over `base`'s
    /// running data (speed, feed, materials, cutting rows, catalog
    /// facts, hand and holder stand-in). The fields of other kinds are
    /// reset to their inert values. The tool law (`why_not_sane`) has
    /// the last word.
    pub fn tool(&self, base: &Tool) -> Result<Tool, MillingError> {
        let t = Tool {
            kind: self.kind,
            diameter: self.diameter,
            corner_r: self.corner_r,
            tip_angle: 0.0,
            pitch: 0.0,
            pitch_min: 0.0,
            nose_radius: 0.0,
            width: 0.0,
            flute_len: self.flute_len,
            shank_d: self.insert_d,
            shank_len: (self.insert_top - self.flute_len).max(0.0),
            flutes: Some(self.flutes),
            center_cutting: self.center_cutting,
            orientation: None,
            outline: Vec::new(),
            turning: None,
            tip_form: TaperTip::Sharp,
            tip_diameter: 0.0,
            form: None,
            ..base.clone()
        };
        match t.why_not_sane() {
            Some(why) => Err(MillingError::Tool { name: self.name.clone(), why }),
            None => Ok(t),
        }
    }
}
