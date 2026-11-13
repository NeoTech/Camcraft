//! The program stream of a Wire Contour (#533, part of #290): the
//! op's section as the wire EDM post reads it. A wire cut mounts no
//! tool (#532), so the section carries no TOOL, no spindle and no
//! coolant ask; it carries the machine's condition by its id (`PARAM
//! EdmCondition`, which only a script declaring `wireEdm` may post -
//! the engine refuses it by name for any other) and the product side
//! (`PARAM WireSide`, commentary). The moves are the op's cached
//! wire-centre path as generated - the kerf is already in them, so the
//! post states compensation off and never asks the control for a
//! second offset (#290 D5, #532's decision record).
//!
//! With skims (#538) each later pass is announced mid-section by its
//! own `PARAM EdmCondition` (`announce_skims`).
//!
//! Provenance: our own design over the stream contract
//! (docs/CNC-INTERMEDIATE.md) and #290's decisions D2, D5 and D10.
//!
//! And the verification gate's wire EDM clause (#534, part of #290 D7):
//! the 2.5D wire laws (`sim::verify::wire` - kerf, threading, slug) run
//! over every Wire Contour's cached path against the part and stock as
//! they stand now, made binding - every finding blocks the post, the
//! way a located gouge does - and the not-judged line that says the 3D
//! stock sim did not run for the wire, what judged it instead, and
//! which piece each cut frees.
//! Provenance: our own design, the plasma clause's (`posting_plasma`)
//! read onto the wire; the laws cite their public sources in their own
//! module.

use crate::model;
use crate::operation::wire_verify::wire_verdict;
use crate::operation::{self, OpParams, WireContourOp};
use crate::post::AfterOperation;
use crate::post::{self, cnc, Dump};
use crate::schema::Operation;
use crate::verify::wire::Slug;
use flecs_ecs::prelude::*;
use serde_json::json;

/// The clause's own words on the not-judged line, said once.
pub const WIRE_VERDICT: &str = "the 3D stock sim does not run for wire EDM";

/// The wire clause's two outputs: the blocking problems, and the
/// visible line naming the cuts judged and the pieces they free.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct WireClause {
    pub problems: Vec<String>,
    pub line: Option<String>,
}

fn slugs(s: &[Slug]) -> String {
    if s.is_empty() {
        return "frees nothing".into();
    }
    s.iter()
        .map(|g| {
            format!(
                "frees {:.2} mm² near X{:.3} Y{:.3} ({})",
                g.area,
                g.at.x,
                g.at.y,
                if g.product_area > crate::verify::wire::SLUG_AREA_TOL { "the part" } else { "waste" }
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Judge every Wire Contour of the program (document order, suppressed
/// ops left out - the same walk `program_ops` makes).
#[must_use]
pub fn wire_clause(world: &World) -> WireClause {
    let mut out = WireClause::default();
    let mut judged: Vec<String> = Vec::new();
    for (name, e) in crate::setup::operation_order(world) {
        let Some(op) = model::operation_of(world, &e) else { continue };
        if op.suppressed {
            continue;
        }
        let Some(verdict) = wire_verdict(world, &e) else { continue };
        match verdict {
            Err(why) => out.problems.push(format!("{name}: {why}")),
            Ok(v) => {
                out.problems.extend(v.findings.iter().map(|f| format!("{name}: wire {}", f.describe())));
                let (how, tail) = if v.held {
                    ("holds its slug by tabs through the main cut; the tab cut-off ", "")
                } else if v.captured {
                    ("", " into the stated capture")
                } else {
                    ("", "")
                };
                let what = match (v.skipped, &op.params) {
                    // #1109: an open run says which laws it skipped and why.
                    (Some(why), OpParams::WireContour(w)) => format!("open run ({} of travel): kerf judged; {why}", w.side.word()),
                    (Some(why), _) => why.to_owned(),
                    (None, _) => format!("{how}{}{tail}", slugs(&v.slugs)),
                };
                judged.push(format!("{name} {what}"));
            }
        }
    }
    if !judged.is_empty() {
        out.line = Some(format!(
            "{WIRE_VERDICT} - judged by the 2.5D wire laws (kerf, threading, slug): {}",
            judged.join("; ")
        ));
    }
    out
}

/// The section of one Wire Contour op `name` (entity `e`), from its
/// cached path. Refused by name when its heights do not resolve.
pub(super) fn stream(
    world: &World,
    name: &str,
    e: &Entity,
    op: &Operation,
    cache: &crate::schema::Toolpath,
    p: &WireContourOp,
    after: AfterOperation,
) -> Result<Vec<cnc::Record>, String> {
    let setup = op.setup.as_deref();
    let hs = operation::resolve_params(world, setup, Some(&op.src), &op.params).map_err(|err| format!("{name}: {err}"))?;
    let (wcs, origin) = super::wcs_context(world, setup);
    let (stock, stock_top_z) = match model::stock_world_aabb_for(world, setup) {
        Some(a) => ([a.min.x, a.max.x, a.min.y, a.max.y], a.max.z),
        None => ([0.0; 4], 0.0),
    };
    let display_name = model::operation_label(world, name, e);
    let op_kind = op.params.kind().to_string();
    let setup_name = crate::setup::effective(world, setup).unwrap_or_default();
    let d = Dump {
        op_name: &display_name,
        op_kind: &op_kind,
        setup_name: &setup_name,
        clearance_z: hs.clearance,
        retract_z: hs.clearance,
        top_z: stock_top_z,
        wcs,
        origin,
        stock,
        stock_top_z,
        after,
        warnings: cache
            .warnings
            .iter()
            .filter(|w| w.blocks_post())
            .map(|w| format!("{name}: {w}"))
            .collect(),
        ..Dump::default()
    };
    // #537: a 4-axis cut's moves lean as a taper's do.
    let four = p.four_axis.as_ref().map(|f| match f.pairing {
        operation::WirePairingRule::ArcLength => "arc length",
        operation::WirePairingRule::SyncPoints => "sync points",
    });
    // #536: a tapered cut's guide planes, read once from the machine.
    let taper = if p.taper_deg == 0.0 && four.is_none() {
        None
    } else {
        let machine = crate::machine::of_setup(world, setup)
            .1
            .ok_or_else(|| format!("{name}: a leaning wire cut needs its wire EDM machine mounted"))?;
        let guides = crate::machine::wire_edm::guide_planes(&machine, origin[2]).map_err(|e| format!("{name}: {e}"))?;
        Some((p.taper_deg, guides))
    };
    // #539: a tabbed cut posts as two sections - the main cut, which
    // leaves the tabs and ends at a program stop (the operator secures
    // the slug), then the tab cut-off with its own condition.
    let split = crate::operation::wire_verify::cutoff_from(p, &cache.path).map_err(|e| format!("{name}: {e}"))?;
    let (Some(m), Some(t)) = (split, p.tabs.as_ref()) else {
        let mut records = section(&cache.path, &d, name, &p.condition, p.side.word(), taper, four)?;
        if !p.skims.is_empty() {
            announce_skims(&mut records, cache, p, name)?;
        }
        return Ok(records);
    };
    // #538 x #539: skims are cut on an untabbed profile only (the
    // generator refuses the pair); a path that carries both is refused
    // here too, never posted with a skim's condition on the cut-off.
    if !p.skims.is_empty() {
        return Err(format!("{name}: skim passes are stated with tabs - skims are cut on an untabbed profile only; remove the tabs or the skims"));
    }
    let (head, tail) = cache.path.moves.split_at_checked(m).ok_or_else(|| format!("{name}: the tab cut-off's start is past the path"))?;
    let main = crate::cl::Toolpath { moves: head.to_vec(), ..crate::cl::Toolpath::default() };
    let cut = crate::cl::Toolpath { moves: tail.to_vec(), ..crate::cl::Toolpath::default() };
    let off_name = format!("{display_name} tab cut-off");
    let mut records = section(&main, &Dump { after: AfterOperation::ProgramStop, ..d.clone() }, name, &p.condition, p.side.word(), taper, four)?;
    records.extend(section(&cut, &Dump { op_name: &off_name, warnings: Vec::new(), ..d }, name, &t.cutoff_condition, p.side.word(), taper, four)?);
    Ok(records)
}

/// One wire section: the dump with the wire words before it opens;
/// `taper` (#536) is the angle and the guide planes of a leaning cut,
/// `four` (#537) the pairing rule of a 4-axis cut.
fn section(
    path: &crate::cl::Toolpath,
    d: &Dump,
    name: &str,
    condition: &str,
    side: &str,
    taper: Option<(f64, (f64, f64))>,
    four: Option<&str>,
) -> Result<Vec<cnc::Record>, String> {
    let mut records = post::dump(path, d);
    if let Some((_, guides)) = taper {
        records = guided(records, guides).map_err(|e| format!("{name}: {e}"))?;
    }
    // The wire words ride before the section opens, like every PARAM.
    let at = records
        .iter()
        .position(|r| matches!(r, cnc::Record::Section { begin: true }))
        .ok_or_else(|| format!("{name}: the wire section has no start"))?;
    let mut wire = vec![
        cnc::Record::Param { name: "EdmCondition".into(), value: json!(condition) },
        cnc::Record::Param { name: "WireSide".into(), value: json!(side) },
    ];
    if let Some((deg, _)) = taper.filter(|(deg, _)| *deg != 0.0) {
        wire.push(cnc::Record::Param { name: "WireTaper".into(), value: json!(deg) });
    }
    wire.extend(four.map(|rule| cnc::Record::Param { name: "WireFourAxis".into(), value: json!(rule) }));
    records.splice(at..at, wire);
    Ok(records)
}

/// #538: every pass starts and ends at the threading point (#532), so
/// each return to it closes a pass; the skim that follows is announced
/// by its own `PARAM EdmCondition` before its first move - the post
/// selects that condition there, with the wire still threaded. A path
/// that does not return once per pass is refused, never posted with
/// the wrong condition on a pass.
fn announce_skims(records: &mut Vec<cnc::Record>, cache: &crate::schema::Toolpath, p: &WireContourOp, name: &str) -> Result<(), String> {
    use crate::cl::MoveKind;
    let Some(start) = cache.path.moves.first().map(|m| m.from) else {
        return Err(format!("{name}: the wire path is empty - regenerate it"));
    };
    let at_start = |m: &cnc::MoveRec| (m.x - start.x).abs() < 1e-6 && (m.y - start.y).abs() < 1e-6;
    let mut out = Vec::with_capacity(records.len() + p.skims.len());
    let mut back = 0usize;
    let mut pending = false;
    for r in records.drain(..) {
        if let cnc::Record::Move(m) = &r
            && m.kind != MoveKind::Rapid
        {
            if pending {
                if let Some(s) = back.checked_sub(1).and_then(|i| p.skims.get(i)) {
                    out.push(cnc::Record::Param { name: "EdmCondition".into(), value: json!(s.condition) });
                }
                pending = false;
            }
            if at_start(m) {
                back = back.saturating_add(1);
                pending = true;
            }
        }
        out.push(r);
    }
    *records = out;
    if back != p.skims.len() + 1 {
        return Err(format!(
            "{name}: the wire path returns to its threading point {back} time(s), not once per pass ({} passes) - regenerate it",
            p.skims.len() + 1
        ));
    }
    Ok(())
}

/// A tapered section's moves as the guides take them (#536, #290 D2):
/// each leaning move's X Y become the lower guide's, and the params
/// before it state the upper guide's offset from the lower
/// (U V, as `PARAM WireU` and `WireV`) - computed here from the pose and the machine's guide planes,
/// so the post writes what the wire laws judged and invents nothing.
fn guided(records: Vec<cnc::Record>, guides: (f64, f64)) -> Result<Vec<cnc::Record>, String> {
    let mut out = Vec::with_capacity(records.len() * 2);
    for r in records {
        let cnc::Record::Move(mut m) = r else {
            out.push(r);
            continue;
        };
        if let Some(abc) = m.abc.take() {
            if m.center.is_some() {
                return Err("a tapered arc - the wire post writes straight tapered moves only".into());
            }
            let axis = crate::cl::angles_to_axis(abc).ok_or("a move's pose is not a direction")?;
            let at = crate::geometry::Point3::new(m.x, m.y, m.z);
            let (lower, uv) = crate::machine::wire_edm::guide_points(at, axis, guides).map_err(|e| format!("a move {e}"))?;
            m.x = lower[0];
            m.y = lower[1];
            // Two numbers, as the stream text carries a PARAM.
            out.push(cnc::Record::Param { name: "WireU".into(), value: json!(uv[0]) });
            out.push(cnc::Record::Param { name: "WireV".into(), value: json!(uv[1]) });
        }
        out.push(cnc::Record::Move(m));
    }
    Ok(out)
}
