//! #3646: a multi-seat cutter in its spindle assembly (#3643): the
//! assembly must mount the components the cutter was derived from,
//! every seat must be the mounted seat turned about the spindle axis,
//! and every judged body must stand inside the tool's real profile.
//! Our own design over rigid-frame algebra and the bull-nose meridian
//! (a plane circle and a line); no vendor source or file consulted.

use super::{MillingCutter, MillingError, SEAT_TOL};
use crate::frame::Frame;
use crate::holder_profile::{HolderBand, HolderProfile};
use crate::tool_assembly::{resolve, AssemblyNode, ResolvedAssembly};
use crate::tools::Tool;

/// How far (unit length) a seat's axes may stand from the mounted
/// seat's, each read in its own radial frame.
const TURN_TOL: f64 = 1e-6;

/// A seat's cutting point read in its own radial frame: the distance
/// from the axis, the height above the tip, and its Z and X axes on
/// (radial, tangential, axial).
struct SeatPose {
    r: f64,
    z: f64,
    axes: [f64; 6],
}

impl SeatPose {
    /// `None` for a cutting point on the spindle axis.
    fn of(f: &Frame) -> Option<Self> {
        let o = f.origin();
        let r = o[0].hypot(o[1]);
        if !r.is_finite() || r <= SEAT_TOL {
            return None;
        }
        let (er, et) = ([o[0] / r, o[1] / r], [-o[1] / r, o[0] / r]);
        let on = |v: [f64; 3]| [v[0] * er[0] + v[1] * er[1], v[0] * et[0] + v[1] * et[1], v[2]];
        let (z, x) = (on(f.z()), on(f.x()));
        Some(Self { r, z: o[2], axes: [z[0], z[1], z[2], x[0], x[1], x[2]] })
    }
}

impl MillingCutter {
    /// The tool in its spindle assembly `nodes` (the module's law). The
    /// assembly must mount this cutter's own body with its own insert;
    /// a body or insert with other data is refused, even on the same
    /// diameter. Each seat port of the body (one port also stands for
    /// the seats of a seat count, which are its copies turned about the
    /// axis) must put the cutting point on `DC`, in the tip plane, and
    /// turn the insert as the mounted seat does. The judged body is
    /// enclosed over the tool; a body inside the cutting depth must
    /// stand inside the tool's profile there - the corner arc
    /// included - and is refused by name when it does not.
    pub fn assembled(&self, base: &Tool, nodes: &[AssemblyNode]) -> Result<Tool, MillingError> {
        let r = resolve(nodes).map_err(|e| MillingError::Assembly { name: self.name.clone(), err: Box::new(e) })?;
        let cut = self.mounted(nodes)?;
        let seat_d = r.cutting.diameter;
        if !seat_d.is_finite() || (seat_d - self.dc).abs() > SEAT_TOL {
            return Err(MillingError::SeatDiameter { name: self.name.clone(), seat_d, dc: self.dc });
        }
        self.check_seats(nodes, &r, cut)?;
        let tool = self.tool(base)?;
        let Some(body) = &r.body else { return Ok(tool) };
        let fl = tool.flute_len;
        let mut kept: Vec<HolderBand> = Vec::new();
        for b in body.profile.bands() {
            let top = b.above + b.len;
            // The profile widens from the tip up the corner arc, so the
            // band's lowest point inside the cutting depth is where the
            // tool is narrowest under it.
            if b.above < fl - 1e-9 {
                let cut_r = profile_r(&tool, b.above);
                if b.r > cut_r + 1e-9 {
                    return Err(MillingError::BodyOutside { name: self.name.clone(), above: b.above, r: b.r, cut_r });
                }
            }
            if top > fl + 1e-9 {
                let lo = b.above.max(fl);
                kept.push(HolderBand { above: lo, len: top - lo, r: b.r });
            }
        }
        if kept.is_empty() {
            return Ok(tool);
        }
        let why = || MillingError::Tool { name: self.name.clone(), why: "its body could not be enclosed".to_owned() };
        HolderProfile::from_bands(kept).enclose(&tool).ok_or_else(why)
    }

    /// The index of the mounted insert, after the check that it and its
    /// parent are the very components this cutter was derived from.
    fn mounted(&self, nodes: &[AssemblyNode]) -> Result<usize, MillingError> {
        let no = |why: String| MillingError::Mismatch { name: self.name.clone(), why };
        let Some(cut) = nodes.iter().position(|n| n.component.insert().is_some()) else {
            return Err(no("the assembly cuts with no insert".to_owned()));
        };
        let n = &nodes[cut];
        if n.component != self.insert {
            let (got, want) = (n.component.name(), self.insert.name());
            return Err(no(format!("it mounts the insert '{got}', but the cutter was derived from '{want}'")));
        }
        let parent = n.mount.as_ref().and_then(|m| nodes.get(m.parent)).map(|p| &p.component);
        if parent != Some(&self.body) {
            let got = parent.map_or("nothing", |p| p.name());
            return Err(no(format!("the insert sits on '{got}', not on the cutter body it was derived from")));
        }
        Ok(cut)
    }

    /// Every seat port of the body against the mounted seat: on `DC`,
    /// in the tip plane, the insert turned the same way.
    fn check_seats(&self, nodes: &[AssemblyNode], r: &ResolvedAssembly, cut: usize) -> Result<(), MillingError> {
        let n = &nodes[cut];
        let Some(m) = &n.mount else { return Err(MillingError::NotABody { name: self.name.clone() }) };
        let cip = n.geometry.cip().copied().unwrap_or(Frame::IDENTITY);
        let (Some(bf), Some(cf)) = (r.frames.get(m.parent), r.frames.get(cut)) else {
            return Err(MillingError::NotABody { name: self.name.clone() });
        };
        let geometry = &nodes[m.parent].geometry;
        let off = |port: &str, what: &'static str, why: String| MillingError::SeatOff {
            name: self.name.clone(),
            port: port.to_owned(),
            what,
            why,
        };
        let mounted_port = m.port.as_deref().unwrap_or("");
        let Some(home) = SeatPose::of(&cf.then(&cip)) else {
            return Err(off(mounted_port, "radius", "its cutting point is on the spindle axis".to_owned()));
        };
        for port in self.body.ports() {
            let Some(local) = geometry.port(port) else {
                return Err(off(port, "frame", "the cutter body's geometry states no frame for it".to_owned()));
            };
            let turned = bf.then(local).turned(m.z_rot_deg);
            let Ok(seat) = turned.map(|f| f.then(&cip)) else {
                return Err(off(port, "frame", "its frame is not finite".to_owned()));
            };
            let Some(p) = SeatPose::of(&seat) else {
                return Err(off(port, "radius", "its cutting point is on the spindle axis".to_owned()));
            };
            if (p.r - 0.5 * self.dc).abs() > SEAT_TOL {
                let why = format!("its cutting point is on Ø{:.3} mm, but the cutter states DC {} mm", 2.0 * p.r, self.dc);
                return Err(off(port, "radius", why));
            }
            if (p.z - home.z).abs() > SEAT_TOL {
                let why = format!("its cutting point is {:.3} mm off the tip plane", p.z - home.z);
                return Err(off(port, "height", why));
            }
            if p.axes.iter().zip(&home.axes).any(|(a, b)| (a - b).abs() > TURN_TOL) {
                let why = format!("it turns the insert other than the mounted seat {mounted_port} does");
                return Err(off(port, "orientation", why));
            }
        }
        Ok(())
    }
}

/// How far the tool `t` reaches from the axis at `z` above the tip,
/// inside the cutting depth: the corner arc, then the cutting radius.
/// Nothing of the tool is below the tip.
fn profile_r(t: &Tool, z: f64) -> f64 {
    let (r, rc) = (0.5 * t.diameter, t.corner_r.max(0.0));
    if z < -1e-9 {
        0.0
    } else if z < rc {
        let dz = rc - z.max(0.0);
        r - rc + (rc * rc - dz * dz).max(0.0).sqrt()
    } else {
        r
    }
}
