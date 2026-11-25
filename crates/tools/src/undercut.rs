//! Undercut cutters (#1444): the T-slot, keyseat (Woodruff), dovetail and lollipop heads, each wider than the neck that carries it - head shapes from the cutter side views of public machine-tool handbooks (Machinery's Handbook: T-slot cutters, Woodruff keyseat cutters, dovetail cutters); the record law is our own. No vendor material consulted.
//!
//! One reading for every consumer: the generator takes the contact
//! radius from [`Undercut::contact`], the verifier and the stock
//! simulation take the head from [`Undercut::rho`] and the neck from
//! `neck_r`. The solid is the head over `[0, head_top]` united with the
//! neck cylinder from `height` up.

use crate::spun::SpunPoint;
use crate::tools::{Tool, ToolGeom, ToolKind, add_pt, dome_radius};

/// The head's shape above the tip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Head {
    /// A flat disc: T-slot and keyseat cutters.
    Disc,
    /// A dovetail head widest at the corner radius equator (`corner_r`
    /// above the tip; at the tip when `corner_r` is 0), with flank slope
    /// `cot` = 1 / tan of the flank angle off the bottom face.
    Dove { cot: f64, corner_r: f64 },
    /// A whole ball of the head radius, centre one radius above the tip.
    Ball,
}

/// Dovetail flank geometry from tip radius `r`, flank slope `cot`
/// (`= tan(90 - tip_angle)`), and bottom corner radius `c`.
#[derive(Clone, Copy, Debug)]
struct DoveGeom {
    r: f64,
    c: f64,
    cot: f64,
    /// `hypot(1, cot) = 1 / cos(t)`, where `t = 90 - tip_angle`.
    h: f64,
    /// Height above the tip where the corner arc meets the flank.
    s_tan: f64,
    /// Radius where the corner arc meets the flank.
    r_tan: f64,
}

impl DoveGeom {
    fn new(r: f64, cot: f64, c: f64) -> Self {
        let h = 1.0_f64.hypot(cot);
        let s_tan = if c > 0.0 { c * (1.0 + cot / h) } else { 0.0 };
        let r_tan = if c > 0.0 { r - c + c / h } else { r };
        Self { r, c, cot, h, s_tan, r_tan }
    }

    fn rho(&self, s: f64) -> f64 {
        if self.c <= 0.0 {
            return (self.r - s * self.cot).max(0.0);
        }
        if s <= self.s_tan {
            let e = s - self.c;
            (self.r - self.c + (self.c * self.c - e * e).max(0.0).sqrt()).max(0.0)
        } else {
            self.flank_rho(s).max(0.0)
        }
    }

    fn flank_rho(&self, s: f64) -> f64 {
        if self.c <= 0.0 {
            self.r - s * self.cot
        } else {
            self.r - self.c + self.c * (self.h + self.cot) - s * self.cot
        }
    }
}

/// An undercut cutter as the laws read it, in mm above the tip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Undercut {
    /// The head's largest radius.
    pub r: f64,
    /// Where the neck starts (`flute_len`).
    pub height: f64,
    /// The neck's radius (`shank_d / 2`), under `r`.
    pub neck_r: f64,
    pub head: Head,
}

/// Is `kind` one of the undercut cutter kinds?
#[must_use]
pub fn is_kind(kind: ToolKind) -> bool {
    matches!(kind, ToolKind::TSlot | ToolKind::Keyseat | ToolKind::Dovetail | ToolKind::Lollipop)
}

impl Undercut {
    /// The undercut reading of `tool`: `None` for any other kind and for
    /// numbers [`refusal`] names (no law reads a head that is not one).
    #[must_use]
    pub fn of(tool: &Tool) -> Option<Self> {
        if !is_kind(tool.kind) || refusal(tool).is_some() {
            return None;
        }
        let head = match tool.kind {
            ToolKind::Dovetail => Head::Dove {
                cot: 1.0 / tool.tip_angle.to_radians().tan(),
                corner_r: tool.corner_r,
            },
            ToolKind::Lollipop => Head::Ball,
            _ => Head::Disc,
        };
        Some(Self { r: tool.diameter * 0.5, height: tool.flute_len, neck_r: tool.shank_d * 0.5, head })
    }

    /// The top of the head solid above the tip: the neck start, or the
    /// ball's pole.
    #[must_use]
    pub fn head_top(&self) -> f64 {
        match self.head {
            Head::Ball => 2.0 * self.r,
            Head::Disc | Head::Dove { .. } => self.height,
        }
    }

    /// The head's radius at height `s` above the tip; 0 outside the head.
    #[must_use]
    pub fn rho(&self, s: f64) -> f64 {
        if !(s >= 0.0 && s <= self.head_top()) {
            return 0.0;
        }
        match self.head {
            Head::Disc => self.r,
            Head::Dove { cot, corner_r } => DoveGeom::new(self.r, cot, corner_r).rho(s),
            Head::Ball => dome_radius(self.r, s),
        }
    }

    /// The point of the profile that cuts the wall of the undercut:
    /// `(radius, height above the tip)` of the widest point. The disc
    /// cuts on its whole rim (read at the tip), the dovetail at its
    /// corner radius equator (tip when `corner_r` is 0), the ball at
    /// its equator.
    #[must_use]
    pub fn contact(&self) -> (f64, f64) {
        match self.head {
            Head::Disc => (self.r, 0.0),
            Head::Dove { corner_r, .. } => (self.r, corner_r),
            Head::Ball => (self.r, self.r),
        }
    }

    /// How far the head stands out past the neck, mm.
    #[must_use]
    pub fn overhang(&self) -> f64 {
        self.r - self.neck_r
    }
}

/// Slack (mm) where a dovetail's flank top meets its neck: float dust
/// only (#3662). Inch sizes converted to mm put a flank that meets the
/// neck exactly one ulp under it; a neck wider by a real amount is
/// still refused.
const NECK_MEETS: f64 = 1e-9;

/// The first rule `tool` breaks as an undercut cutter, in plain words.
/// `None` for a sound cutter and for every other kind.
#[must_use]
pub fn refusal(tool: &Tool) -> Option<String> {
    if !is_kind(tool.kind) {
        return None;
    }
    let (kind, d, sd, fl) = (tool.kind, tool.diameter, tool.shank_d, tool.flute_len);
    let (r, sr) = (d * 0.5, sd * 0.5);
    if [d, sd, fl, tool.shank_len, tool.tip_angle, tool.corner_r].iter().any(|v| !v.is_finite()) {
        return Some(format!("a {kind}'s sizes must be finite numbers"));
    }
    if !(sd > 0.0 && sd < d) {
        return Some(format!("a {kind}'s neck (shank diameter) {sd} mm must be over 0 and under its {d} mm head diameter"));
    }
    if fl <= 0.0 {
        return Some(format!("a {kind}'s head height (flute length) {fl} mm must be over 0"));
    }
    if tool.shank_len <= 0.0 {
        return Some(format!("a {kind}'s neck length (shank length) {} mm must be over 0", tool.shank_len));
    }
    match kind {
        ToolKind::Dovetail => {
            let a = tool.tip_angle;
            if !(a > 0.0 && a < 90.0) {
                return Some(format!("a dovetail cutter's flank angle {a} degrees must be over 0 and under 90"));
            }
            let c = tool.corner_r;
            if c < 0.0 {
                return Some(format!("a dovetail cutter's corner radius {c} mm must not be negative"));
            }
            if c > r {
                return Some(format!("a dovetail cutter's corner radius {c} mm cannot exceed half its {d} mm head diameter"));
            }
            let g = DoveGeom::new(r, 1.0 / a.to_radians().tan(), c);
            if c > 0.0 && g.s_tan > fl + NECK_MEETS {
                return Some(format!(
                    "a dovetail cutter's {c} mm corner radius reaches {:.3} mm up its {a} degree flank, over its {fl} mm head height",
                    g.s_tan
                ));
            }
            let top = if fl >= g.s_tan { g.flank_rho(fl) } else { g.rho(fl) };
            if top <= 0.0 || top < sr - NECK_MEETS {
                return Some(format!(
                    "a dovetail cutter's {a} degree flank narrows its {d} mm head to {:.3} mm at {fl} mm height, under its {sd} mm neck",
                    2.0 * top
                ));
            }
            None
        }
        ToolKind::Lollipop => {
            let meet = r + (r * r - sr * sr).max(0.0).sqrt();
            if !(fl >= r && fl <= meet + 1e-9) {
                return Some(format!(
                    "a lollipop cutter's flute length {fl} mm must be from {r} mm (the ball's centre) to {meet:.3} mm (where the {sd} mm neck leaves the ball)"
                ));
            }
            // The stated cutter ends at flute + shank. It must contain
            // the whole ball, including the cap above the neck junction.
            if tool.shank_len < d - fl {
                return Some(format!(
                    "a lollipop cutter's neck length (shank length) {} mm must be at least {} mm so flute + shank reaches the {d} mm ball top",
                    tool.shank_len, d - fl
                ));
            }
            None
        }
        _ => None,
    }
}

/// An undercut cutter record of `kind` (#1444): head `d` mm across and
/// `head_h` mm high on a `neck_d` x `neck_len` mm neck; `angle` is the
/// dovetail's flank angle in degrees (inert on the other kinds). The
/// speeds are nominal starting values. A kind that is no undercut
/// cutter, or numbers [`refusal`] names, still build a record: the
/// tool sanity law judges it where it is stored.
#[must_use]
pub fn cutter(kind: ToolKind, d: f64, head_h: f64, neck_d: f64, neck_len: f64, angle: f64) -> Tool {
    Tool {
        kind,
        diameter: d,
        corner_r: 0.0,
        tip_angle: angle,
        pitch: 0.0,
        pitch_min: 0.0,
        thread_hand: Default::default(),
        nose_radius: 0.0,
        width: 0.0,
        flute_len: head_h,
        shank_d: neck_d,
        shank_len: neck_len,
        holder_d: 0.0,
        holder_len: 0.0,
        material_group: String::new(),
        rpm: 2000.0,
        feed: 150.0,
        tool_material: None,
        flutes: None,
        center_cutting: false,
        orientation: None,
        catalog: crate::catalog_fields::ToolCatalog::default(),
        cutting: Vec::new(),
        outline: Vec::new(),
        turning: None,
        tip_form: Default::default(),
        tip_diameter: 0.0,
        form: None,
    }
}

/// The tool's coaxial stack for the fixture laws: [`Tool::geom`], with
/// the flute band raised to the head's top. A lollipop's `flute_len` is
/// where the neck leaves the ball, but the ball goes on to its pole at
/// `2r`; the plain stack sweeps only the neck there and misses the
/// upper hemisphere. The raised band encloses the whole ball (and the
/// neck below the pole, which is narrower). The shank band shrinks by
/// the same amount, so the neck top and the holder stay where they
/// are; an accepted stack reaches at least the ball's pole. Every other
/// tool, and a refused head, reads [`Tool::geom`] unchanged.
#[must_use]
pub fn fixture_geom(tool: &Tool) -> ToolGeom {
    let g = tool.geom();
    let Some(u) = Undercut::of(tool) else { return g };
    let top = u.head_top();
    if top.partial_cmp(&g.flute_len) != Some(std::cmp::Ordering::Greater) {
        return g;
    }
    let neck_top = g.flute_len + g.shank_len;
    ToolGeom { flute_len: top, shank_len: (neck_top - top).max(0.0), ..g }
}

/// The head's side view for [`Tool::profile`], from the tip rim up; the
/// return is the height where the neck step starts. A dovetail ends at
/// the flute top. A lollipop draws the ball united with the neck: the
/// ball goes on above the neck start until its radius falls to the
/// neck's, so the upper ball is drawn as the laws read it (#1444).
/// Degenerate numbers draw a shorter picture, never a NaN.
pub(crate) fn head_outline(tool: &Tool, p: &mut Vec<(f64, f64)>) -> f64 {
    let r = tool.diameter.max(0.0) * 0.5;
    let zc = tool.flute_len.max(0.0);
    if tool.kind == ToolKind::Lollipop {
        let sr = (tool.shank_d.max(0.0) * 0.5).min(r);
        // Where the ball's radius falls to the neck's, above the equator.
        let meet = r + (r * r - sr * sr).max(0.0).sqrt();
        // Never past the neck top: the holder step starts there.
        let neck_top = zc + tool.shank_len.max(0.0);
        let low = zc.min(2.0 * r);
        let top = meet.min(neck_top).max(low);
        let steps = 24u32;
        let mut zs: Vec<f64> = (1..=steps).map(|i| top * f64::from(i) / f64::from(steps)).collect();
        if r < top {
            zs.push(r);
        }
        zs.sort_by(f64::total_cmp);
        zs.dedup();
        for z in zs {
            add_pt(p, z, dome_radius(r, z));
        }
        return top;
    }
    let cr = if tool.corner_r.is_finite() { tool.corner_r.clamp(0.0, r) } else { 0.0 };
    let tan = tool.tip_angle.to_radians().tan();
    if cr > 0.0 && tool.tip_angle.is_finite() && tool.tip_angle > 0.0 && tool.tip_angle < 90.0 && tan.is_finite() && tan > 0.0 {
        let g = DoveGeom::new(r, 1.0 / tan, cr);
        let arc_end = g.s_tan.min(zc);
        if cr.hypot(cr) >= crate::spun::MIN_SEGMENT {
            add_pt(p, 0.0, (r - cr).max(0.0));
        }
        if arc_end > 0.0 {
            let alpha_end = ((arc_end - cr) / cr).clamp(-1.0, 1.0).asin();
            let push_arc = |p: &mut Vec<(f64, f64)>, a0: f64, a1: f64| {
                let span = (a1 - a0).max(0.0);
                let steps = ((cr * span / 0.02).floor() as u32).clamp(1, 12);
                for i in 1..steps {
                    let phi = a0 + span * f64::from(i) / f64::from(steps);
                    let z = (cr * (1.0 + phi.sin())).min(zc);
                    let rho = (r - cr + cr * phi.cos()).clamp(0.0, r);
                    add_pt(p, z, rho);
                }
            };
            if arc_end > cr {
                push_arc(p, -std::f64::consts::FRAC_PI_2, 0.0);
                add_pt(p, cr, r);
                push_arc(p, 0.0, alpha_end);
                let end_r = if zc >= g.s_tan { g.r_tan } else { g.rho(arc_end) };
                add_pt(p, arc_end, end_r.clamp(0.0, r));
            } else {
                push_arc(p, -std::f64::consts::FRAC_PI_2, alpha_end);
                let end_r = if (arc_end - cr).abs() <= 1e-12 { r } else { g.rho(arc_end) };
                add_pt(p, arc_end, end_r.clamp(0.0, r));
            }
        }
        if zc > g.s_tan {
            add_pt(p, zc, g.flank_rho(zc).clamp(0.0, r));
        }
        return zc;
    }
    add_pt(p, 0.0, r);
    let top = if tan.is_finite() && tan > 0.0 { (r - zc / tan).clamp(0.0, r) } else { r };
    add_pt(p, zc, top);
    zc
}

/// The cutting meridian of a dovetail cutter (`Tool::meridian`): the
/// bottom face, the corner arc when `corner_r > 0`, and the flank to `fl`.
pub(crate) fn dovetail_meridian_points(tool: &Tool, r: f64, fl: f64) -> Vec<SpunPoint> {
    let cot = 1.0 / tool.tip_angle.to_radians().tan();
    let line = SpunPoint::line;
    let cr = tool.corner_r;
    if cr <= 0.0 {
        return vec![line(0.0, 0.0), line(0.0, r), line(fl, r - fl * cot)];
    }
    let g = DoveGeom::new(r, cot, cr);
    let mut p = vec![line(0.0, 0.0)];
    if r - cr >= crate::meridian::MIN_LEN {
        p.push(line(0.0, r - cr));
    }
    if let Some(last) = p.last_mut() {
        last.arc = cr;
    }
    if fl - g.s_tan >= crate::meridian::MIN_LEN {
        p.push(line(g.s_tan, g.r_tan));
        p.push(line(fl, g.flank_rho(fl).max(0.0)));
    } else {
        let top = if fl >= g.s_tan { g.flank_rho(fl) } else { g.rho(fl) };
        p.push(line(fl, top.max(0.0)));
    }
    p
}

#[cfg(test)]
#[path = "undercut_tests.rs"]
mod tests;
