//! The machine-part collision of `camcraft-sim check` (#2586): the part
//! tree of the machine file is mapped to the generated model of #2584,
//! the model is posed along each move at its machine position (work
//! position + origin), and the spindle and the parts that carry it must
//! not meet the table and the parts that carry the work. Each finding and
//! each pass says "judged on a generated model": its bodies are loose.
//! Provenance: ISO 841 axis nomenclature; the model is
//! `camcraft_sim::machine_model`; the body test is the separating axis
//! theorem (`sat`).

use camcraft_post::machine::{MovingSide, PartKind, TreePart};
use camcraft_sim::machine_model::{generate, Body, BodyKind, BodySource, JointKind, MachineSpec, PartSpec, SpindleSpec, ToolStationSpec};

use super::frames::span;
use super::inputs::MachineFile;
use super::program::Program;
use super::report::Report;
use super::sat::{boxes_overlap, overlap, Hull};
use super::stock::samples;

const CHECK: &str = "machine model";
/// The step between two poses along a move, mm.
const STEP: f64 = 2.0;
/// A touch closer than this is not a collision, mm.
const TOL: f64 = 0.01;

/// The letter (0 = X, 1 = Y, 2 = Z) that a linear part drives: its name, else its id.
fn letter(p: &TreePart) -> Option<usize> {
    [&p.name, &p.id].iter().find_map(|w| match w.to_ascii_uppercase().as_str() {
        "X" => Some(0),
        "Y" => Some(1),
        "Z" => Some(2),
        _ => None,
    })
}

/// The model input of the tree, and for each part its pose letter and
/// sign; a refusal names the part.
/// For each part: the program letter it drives and the sign of its value.
type Drives = Vec<Option<(usize, f64)>>;

fn spec(m: &MachineFile) -> Result<(MachineSpec, Drives), String> {
    let mut parts = Vec::with_capacity(m.tree.parts.len());
    let mut drives = Vec::with_capacity(m.tree.parts.len());
    for p in &m.tree.parts {
        let joint = match p.kind {
            PartKind::Table | PartKind::Head => JointKind::Fixed,
            PartKind::Linear => JointKind::Linear,
            PartKind::Rotary => JointKind::Rotary,
        };
        let mut s = PartSpec::new(&p.id, p.parent, joint);
        s.travel = p.travel;
        s.pivot = p.pivot;
        s.offset = p.offset.unwrap_or([0.0; 3]);
        s.axis = p.axis;
        let mut drive = None;
        if p.kind == PartKind::Linear {
            let k = letter(p).ok_or_else(|| format!("linear part {} names no X, Y or Z", p.id))?;
            let mut axis = p.axis.unwrap_or_else(|| {
                let mut a = [0.0; 3];
                a[k] = 1.0;
                a
            });
            // A part on the table side moves the work: it runs against the program axis.
            if p.moving_side == Some(MovingSide::Table) {
                axis = [-axis[0], -axis[1], -axis[2]];
            }
            s.axis = Some(axis);
            drive = Some((k, p.direction.sign()));
        }
        if p.kind == PartKind::Head {
            let b = p.spindle_body.unwrap_or_default();
            s.spindle = Some(SpindleSpec { diameter: b.diameter, length: b.length });
            s.tool_station = p.tool_station.map(|t| ToolStationSpec { max_tool_length: t.max_tool_length, max_tool_diameter: t.max_tool_diameter });
        }
        parts.push(s);
        drives.push(drive);
    }
    Ok((MachineSpec { parts }, drives))
}

/// For each part: whether it is on the spindle side - a head, or an axis
/// that moves the tool (#2583's moving side). A table is on the work side.
fn head_side(m: &MachineFile) -> Vec<bool> {
    m.tree.parts.iter().map(|p| p.kind == PartKind::Head || p.moving_side == Some(MovingSide::Tool)).collect()
}

/// The first pair (head side, work side) that overlaps.
fn first_hit<'a>(bodies: &'a [Body], head: &[bool]) -> Option<(&'a Body, &'a Body)> {
    let hulls: Vec<Option<Hull>> = bodies.iter().map(|b| Hull::new(&b.triangles)).collect();
    for (i, a) in bodies.iter().enumerate() {
        if !head.get(a.part).copied().unwrap_or(false) || a.kind == BodyKind::ToolEnvelope {
            continue;
        }
        for (j, b) in bodies.iter().enumerate() {
            if head.get(b.part).copied().unwrap_or(true) || b.kind != BodyKind::Block {
                continue;
            }
            if let (Some(ha), Some(hb)) = (&hulls[i], &hulls[j])
                && boxes_overlap(ha, hb, TOL)
                && overlap(ha, hb, TOL)
            {
                return Some((a, b));
            }
        }
    }
    None
}

/// Judge each section whose origin is given on the generated model of `m`.
pub fn check(program: &Program, m: Option<&MachineFile>, report: &mut Report) {
    let Some(m) = m else {
        return report.not_judged(CHECK, "no machine file is given (--machine)");
    };
    if m.tree.parts.is_empty() {
        return report.not_judged(CHECK, &format!("{} states no part tree (kinematics.default.parts): no model is generated", m.name));
    }
    let (spec, drives) = match spec(m) {
        Ok(s) => s,
        Err(e) => return report.not_judged(CHECK, &format!("{}: {e}", m.name)),
    };
    let model = match generate(&spec) {
        Ok(model) => model,
        Err(e) => return report.not_judged(CHECK, &e),
    };
    for n in &model.notes {
        report.note(n);
    }
    let head = head_side(m);
    let steel = |b: &Body, side: bool| head.get(b.part).copied() == Some(side) && b.kind != BodyKind::ToolEnvelope;
    if !model.bodies.iter().any(|b| steel(b, true)) || !model.bodies.iter().any(|b| steel(b, false)) {
        return report.not_judged(CHECK, &format!("{}: the model has no steel body on the spindle side or on the work side (see the notes)", m.name));
    }
    let ids: Vec<&str> = spec.parts.iter().map(|p| p.id.as_str()).collect();
    let (mut judged, mut found) = (0usize, 0usize);
    for s in &program.sections {
        let Some((first, last)) = span(&s.lines, &s.skip) else { continue };
        let Some(o) = s.origin else {
            report.not_judged(CHECK, &format!("lines {first}-{last}: no machine position is given for G{} (--wcs)", 53 + s.offset));
            continue;
        };
        for ((mv, line), skip) in s.moves.iter().zip(&s.lines).zip(&s.skip) {
            if *skip {
                continue;
            }
            judged += 1;
            for p in samples(mv, STEP) {
                let at = [p.x + o[0], p.y + o[1], p.z + o[2]];
                let values: Vec<(&str, f64)> = ids
                    .iter()
                    .zip(&drives)
                    .zip(&spec.parts)
                    .filter(|(_, part)| part.joint != JointKind::Fixed)
                    .map(|((id, d), _)| (*id, d.map_or(0.0, |(k, sign)| at[k] * sign)))
                    .collect();
                let posed = match model.pose(&values) {
                    Ok(b) => b,
                    Err(e) => return report.not_judged(CHECK, &format!("line {line}: {e}")),
                };
                if let Some((a, b)) = first_hit(&posed, &head) {
                    found += 1;
                    report.fail(
                        CHECK,
                        &format!(
                            "line {line}: the {} body of part {} meets part {} at machine X{:.3} Y{:.3} Z{:.3} ({})",
                            if a.kind == BodyKind::Spindle { "spindle" } else { "slide" },
                            a.part_id,
                            b.part_id,
                            at[0],
                            at[1],
                            at[2],
                            BodySource::Generated.judged_on()
                        ),
                    );
                    break;
                }
            }
        }
    }
    if judged > 0 && found == 0 {
        report.pass(CHECK, &format!("{judged} moves: the spindle side clears the work side ({})", BodySource::Generated.judged_on()));
    }
}
