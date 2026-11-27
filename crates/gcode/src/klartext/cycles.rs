//! Klartext cycles: `CYCL DEF 247` (preset), the drilling cycles `CYCL DEF 200` / `203` / `205` and their call (`CYCL CALL`, `M99`).
//! Provenance: HEIDENHAIN TNC cycle programming manuals (cycle 200 DRILLING, 203 UNIVERSAL DRILLING, 205 UNIVERSAL PECKING, 247 DATUM SETTING; Q lists in docs/research/heidenhain/differences.md section B3 and cycles.md), general machining knowledge for the Q meanings; expansion follows the hole chain of `crate::cycle_expand`.

use camcraft_cl::cl::{Cycle, CycleKind, MoveKind, lattice};
use camcraft_geometry::geo2d::Point2;
use camcraft_geometry::geometry::Point3;

use super::exec::Machine;
use super::source::{Block, q_index};
use super::words::value;
use crate::cycle_expand::peck_count;
use crate::domain::{check_dwell, check_feed, check_len};
use crate::types::{GcodeError, WorkOffset};

/// A defined drilling cycle, all levels in mm in the work frame.
#[derive(Clone, Debug, PartialEq)]
pub struct DrillDef {
    /// Set-up clearance level: surface + Q200.
    pub r_z: f64,
    /// Hole bottom: surface + Q201 (Q201 < 0).
    pub bottom_z: f64,
    /// Workpiece surface Q203.
    pub surface_z: f64,
    /// End level: the higher of the set-up clearance and surface + Q204.
    pub clear_z: f64,
    /// Plunging depth Q202 (> 0).
    pub peck: f64,
    /// Plunge feed Q206 (mm/min).
    pub feed: f64,
    /// Dwell at depth Q211 (s).
    pub dwell: f64,
}

/// Q numbers each drilling cycle reads, and the ones that must be written.
fn q_set(cycle: u32) -> (&'static [u32], &'static [u32]) {
    const REQUIRED: &[u32] = &[200, 201, 206, 202, 203, 204];
    match cycle {
        200 => (REQUIRED, &[200, 201, 206, 202, 210, 203, 204, 211, 395]),
        203 => (
            REQUIRED,
            &[
                200, 201, 206, 202, 210, 203, 204, 212, 213, 205, 211, 208, 256, 395,
            ],
        ),
        _ => (
            REQUIRED,
            &[
                200, 201, 206, 202, 203, 204, 212, 205, 258, 259, 257, 256, 211, 379, 253, 208, 395,
            ],
        ),
    }
}

impl Machine<'_> {
    pub fn cycle_block(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        match toks.first().map(String::as_str) {
            Some("DEF") => self.cycle_def(b, &toks[1..]),
            Some("CALL") => {
                let w = super::words::decode(b, &toks[1..], &self.q).map_err(|e| {
                    match &toks.get(1) {
                        Some(t) if ["PAT", "POS"].contains(&t.as_str()) => b.unsupported(
                            &format!("CYCL CALL {t}"),
                            "only CYCL CALL at the tool position is read",
                        ),
                        _ => e,
                    }
                })?;
                if w.has_coords() || w.feed.is_some() || w.r0 || w.mb.is_some() {
                    return Err(b.syntax("CYCL CALL takes only M functions"));
                }
                self.pre_m(b, &w.m)?;
                self.cycle_call(b)?;
                self.post_m(b, &w.m, None)
            }
            _ => Err(b.syntax("expected CYCL DEF or CYCL CALL")),
        }
    }

    fn cycle_def(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let Some(num) = toks.first() else {
            return Err(b.syntax("CYCL DEF needs a cycle number"));
        };
        if num.contains('.') {
            return Err(b.unsupported(
                &format!("CYCL DEF {num}"),
                "old-style cycles (n.0 / n.1 blocks) are not read",
            ));
        }
        let cycle: u32 = num
            .parse()
            .map_err(|_| b.syntax(&format!("cycle number '{num}' is not a number")))?;
        let params = self.cycle_params(b, &toks[1..])?;
        match cycle {
            247 => self.preset(b, &params),
            200 | 203 | 205 => {
                self.cycle = Some(drill_def(b, cycle, &params)?);
                Ok(())
            }
            _ => Err(b.unsupported(
                &format!("CYCL DEF {cycle}"),
                "this cycle is not read (read: 200, 203, 205, 247)",
            )),
        }
    }

    /// Name words, then `Qn=value` entries (also `Qn = value`).
    fn cycle_params(&self, b: &Block, toks: &[String]) -> Result<Vec<(u32, f64)>, GcodeError> {
        let joined: String = toks.join(" ").replace(" =", "=").replace("= ", "=");
        let mut out: Vec<(u32, f64)> = Vec::new();
        let mut in_params = false;
        for t in joined.split_whitespace() {
            let Some((lhs, rhs)) = t.split_once('=') else {
                if in_params {
                    return Err(b.syntax(&format!("'{t}' after the Q parameters")));
                }
                continue; // the cycle name
            };
            in_params = true;
            let idx =
                q_index(lhs).ok_or_else(|| b.syntax(&format!("'{lhs}' is not a Q parameter")))?;
            if out.iter().any(|(q, _)| *q == idx) {
                return Err(b.syntax(&format!("Q{idx} appears twice")));
            }
            out.push((idx, value(b, &format!("Q{idx}"), rhs, &self.q)?));
        }
        Ok(out)
    }

    /// `CYCL DEF 247 Q339=n`: activate preset n, read as work offset n (1..6).
    fn preset(&mut self, b: &Block, params: &[(u32, f64)]) -> Result<(), GcodeError> {
        let [(339, n)] = params else {
            return Err(b.invalid_cycle("CYCL DEF 247 needs exactly Q339"));
        };
        const ROWS: [WorkOffset; 6] = [
            WorkOffset::G54,
            WorkOffset::G55,
            WorkOffset::G56,
            WorkOffset::G57,
            WorkOffset::G58,
            WorkOffset::G59,
        ];
        let row = ROWS.iter().copied().find(|w| f64::from(w.number()) == *n);
        let Some(offset) = row else {
            return Err(b.unsupported(
                "CYCL DEF 247",
                &format!("preset Q339={n} is not read (read: 1 to 6)"),
            ));
        };
        self.modal.work_offset = offset;
        self.sync_section();
        Ok(())
    }

    /// Run the defined drilling cycle at the current XY position.
    pub fn cycle_call(&mut self, b: &Block) -> Result<(), GcodeError> {
        let d = self.cycle.clone().ok_or_else(|| {
            b.invalid_cycle("cycle call without a drilling cycle definition (CYCL DEF 200/203/205)")
        })?;
        let pecks = peck_count(d.surface_z, d.bottom_z, d.peck).ok_or_else(|| {
            b.invalid_cycle(&format!(
                "plunging depth Q202 ({}) needs too many pecks",
                d.peck
            ))
        })?;
        let (hx, hy) = (self.pos.x, self.pos.y);
        let at = |z: f64| Point3::new(hx, hy, z);
        // The tool goes in the tool axis at FMAX to the set-up clearance.
        self.push_line(b, MoveKind::Rapid, at(d.r_z), None);
        let first_move = self.current.builder.moves_len();
        let mut prev = d.r_z;
        for k in 1..=pecks {
            // Each infeed goes Q202 deeper from the surface; the last one ends at the bottom.
            let t = if k == pecks {
                d.bottom_z
            } else {
                (d.surface_z - d.peck * k as f64).max(d.bottom_z)
            };
            if t >= prev {
                return Err(b.invalid_cycle(&format!(
                    "plunging depth Q202 ({}) makes no progress",
                    d.peck
                )));
            }
            prev = t;
            self.push_line(b, MoveKind::Plunge, at(t), Some(d.feed));
            if t <= d.bottom_z {
                break;
            }
            self.push_line(b, MoveKind::Rapid, at(d.r_z), None);
        }
        self.push_line(b, MoveKind::Rapid, at(d.clear_z), None);
        let kind = if d.peck < d.surface_z - d.bottom_z {
            CycleKind::Deep
        } else {
            CycleKind::Drill
        };
        self.current.builder.push_cycle(Cycle {
            kind,
            xy: Point2::new(lattice(hx), lattice(hy)),
            r_z: lattice(d.r_z),
            bottom_z: lattice(d.bottom_z),
            entry_z: lattice(d.r_z),
            peck: lattice(d.peck),
            dwell: d.dwell,
            first_move,
        });
        Ok(())
    }
}

fn drill_def(b: &Block, cycle: u32, params: &[(u32, f64)]) -> Result<DrillDef, GcodeError> {
    let (required, allowed) = q_set(cycle);
    if let Some((q, _)) = params.iter().find(|(q, _)| !allowed.contains(q)) {
        return Err(b.invalid_cycle(&format!("CYCL DEF {cycle} has no parameter Q{q}")));
    }
    let get = |q: u32| params.iter().find(|(n, _)| *n == q).map(|(_, v)| *v);
    if let Some(q) = required.iter().find(|q| get(**q).is_none()) {
        return Err(b.invalid_cycle(&format!("CYCL DEF {cycle} needs Q{q}")));
    }
    // Options that change the motion and that this reader does not model.
    for (q, what) in [
        (210, "dwell at the top"),
        (212, "decrement"),
        (213, "chip breaks"),
        (257, "chip-break depth"),
        (379, "deepened start point"),
        (395, "depth reference"),
    ] {
        if get(q).is_some_and(|v| v != 0.0) {
            return Err(b.unsupported(
                &format!("CYCL DEF {cycle} Q{q}"),
                &format!("a non-zero {what} is not read"),
            ));
        }
    }
    let v = |q: u32| get(q).unwrap_or(0.0);
    let (setup, depth, feed, peck, surface, second) =
        (v(200), v(201), v(206), v(202), v(203), v(204));
    if setup < 0.0 || second < 0.0 {
        return Err(b.invalid_cycle("set-up clearances Q200 and Q204 must not be negative"));
    }
    if depth >= 0.0 {
        return Err(b.invalid_cycle(&format!(
            "depth Q201 ({depth}) must be negative (into the material)"
        )));
    }
    if peck <= 0.0 {
        return Err(b.invalid_cycle(&format!("plunging depth Q202 ({peck}) must be positive")));
    }
    let surface_z = check_len(b.line, "Q203", surface)?;
    let r_z = check_len(b.line, "set-up clearance", surface + setup)?;
    Ok(DrillDef {
        r_z,
        bottom_z: check_len(b.line, "hole bottom", surface + depth)?,
        surface_z,
        clear_z: check_len(b.line, "2nd set-up clearance", surface + second)?.max(r_z),
        peck: check_len(b.line, "Q202", peck)?,
        feed: check_feed(b.line, feed)?,
        dwell: check_dwell(b.line, v(211))?,
    })
}
