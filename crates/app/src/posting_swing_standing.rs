//! The standing-steel half of the index-swing clause (#1107): as a 4th
//! axis that turns the work about X indexes, the WORK (stock, parts and
//! the workholding that turns with them) sweeps a cylinder about the A
//! centerline - and that cylinder must clear what stands still under
//! and beside it: the face the rotary is bolted to, and every fixed
//! fixture over the work's X span. A strike refuses the post by name;
//! standing steel the clause cannot read is named in the gate's
//! not-judged line, never skipped in silence.
//!
//! Provenance: our own design (issue #1107). The geometry is plain: the
//! work turning about a line along X reaches every point within its
//! swept radius of that line over its X span; a body whose YZ box keeps
//! a distance d from the line is clear when d exceeds that radius. The
//! microARC 4's 3.15 in centre height is Tormach's 4th-axis comparison
//! sheet 02/24 - no vendor implementation inspected.
//!
//! The laws:
//! - **Who is judged.** The ops the tool half judges ([`work_a`]): a
//!   work-turning A about X. An unlined rotary is the tool half's
//!   refusal already.
//! - **The face the rotary stands on.** The machine file's
//!   `axis_to_table` - the rotary's centre height over its own mounting
//!   face. Whatever stands under the rotary (the table, a fixture plate,
//!   a subplate) only lifts the line with it, so the nearest standing
//!   face under the rotary is exactly this far down. Unstated: named
//!   not judged.
//! - **Fixed fixtures.** Every fixed fixture of the setup whose X span
//!   overlaps the work's: its box's distance from the A line in YZ. A
//!   box that reaches across the line (a tailstock, the rotary's own
//!   body) or a fixture with no facets cannot be judged by a radius and
//!   is named not judged. Steel wholly beside the work in X is clear.
//! - **The margin.** [`MARGIN_MM`], the same as the tool's.

use super::index_swing::{MARGIN_MM, Swept, WorkA, index_angle, swept, work_a};
use crate::{fixture, model};
use flecs_ecs::prelude::World;

/// The prefix of the clause's not-judged line, said once.
pub const SWING_NOT_JUDGED: &str = "index swing did not judge the work against the standing steel";

/// Overlap slack, mm: steel touching the work's end face is beside it.
const TOUCH_MM: f64 = 1e-6;

/// The clause's output: blocking problems (one per strike) and the
/// names of what it could not judge (deduplicated, program order).
#[derive(Debug, Default)]
pub(super) struct Standing {
    pub problems: Vec<String>,
    pub not_judged: Vec<String>,
}

/// The distance from the origin to the box `[y0, y1] x [z0, z1]`, or
/// None when the box holds the origin (it reaches across the line).
fn rect_distance(y: (f64, f64), z: (f64, f64)) -> Option<f64> {
    let gap = |(lo, hi): (f64, f64)| if lo > 0.0 { lo } else if hi < 0.0 { -hi } else { 0.0 };
    let (dy, dz) = (gap(y), gap(z));
    (dy > 0.0 || dz > 0.0).then(|| dy.hypot(dz))
}

/// One fixed fixture against the swept work: Ok(None) clear or beside
/// the work, Ok(Some(d)) its distance from the line when the work
/// reaches it, Err(why) when it cannot be judged.
fn fixed_strike(world: &World, e: &flecs_ecs::prelude::Entity, s: &Swept, origin: [f64; 3]) -> Result<Option<f64>, String> {
    let tris = model::world_facets(world, e).unwrap_or_default();
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for p in tris.iter().flat_map(|t| [t.a, t.b, t.c]) {
        for (i, v) in [p.x, p.y, p.z].into_iter().enumerate() {
            if !v.is_finite() {
                return Err("its facets are not finite".into());
            }
            lo[i] = lo[i].min(v);
            hi[i] = hi[i].max(v);
        }
    }
    if tris.is_empty() {
        return Err("it has no facets".into());
    }
    let Some((wx0, wx1)) = s.x else { return Ok(None) };
    if hi[0] <= wx0 + TOUCH_MM || lo[0] >= wx1 - TOUCH_MM {
        return Ok(None); // beside the work along X: the swing never reaches it
    }
    let y = (lo[1] - origin[1], hi[1] - origin[1]);
    let z = (lo[2] - origin[2], hi[2] - origin[2]);
    let Some(d) = rect_distance(y, z) else {
        return Err("it reaches across the A centerline over the work's length".into());
    };
    Ok((s.r + MARGIN_MM > d).then_some(d))
}

/// Name `line` as not judged, once.
fn note(out: &mut Standing, line: String) {
    if !out.not_judged.contains(&line) {
        out.not_judged.push(line);
    }
}

/// The clause, in program order.
pub(super) fn standing_problems(world: &World) -> Standing {
    let mut out = Standing::default();
    let Ok(ops) = super::program_ops(world) else {
        return out; // the post refuses the program by name first
    };
    for (name, e) in ops {
        let Some(op) = model::operation_of(world, &e) else { continue };
        let Some(m) = crate::machine::of_setup(world, op.setup.as_deref()).1 else { continue };
        let Some(WorkA::Judged(letter, _)) = work_a(&m) else { continue };
        let a = model::toolpath_of(world, &e).map_or(0.0, |c| index_angle(&c.path));
        let (_, origin) = super::wcs_context(world, op.setup.as_deref());
        let s = swept(world, op.setup.as_deref(), origin);
        let face = m.axes.iter().find(|x| x.name == letter).and_then(|x| x.axis_to_table);
        match face {
            None => note(
                &mut out,
                format!("{} states no axis_to_table for {letter}: the work is not judged against the face the rotary stands on", m.model),
            ),
            Some(h) if !(h.is_finite() && h - s.r >= MARGIN_MM) => out.problems.push(format!(
                "{name}: the {letter}{a:.3} index swings the work into the face the rotary stands on - the stock, \
                 part and turning fixtures sweep {:.3} mm about the {letter} centerline, which stands {h:.3} mm \
                 above that face (table, plate or subplate), a clearance of {:.3} mm against the {MARGIN_MM:.1} mm \
                 needed",
                s.r,
                h - s.r
            )),
            Some(_) => {}
        }
        for (fname, fe) in fixture::tagged(world, op.setup.as_deref()) {
            if !fixture::def_of(world, &fe).is_some_and(|d| d.fixed) {
                continue; // it turns with the work: part of the sweep
            }
            match fixed_strike(world, &fe, &s, origin) {
                Ok(None) => {}
                Ok(Some(d)) => out.problems.push(format!(
                    "{name}: the {letter}{a:.3} index swings the work into standing fixture {fname} - the work \
                     sweeps {:.3} mm about the {letter} centerline and {fname} stands {d:.3} mm from it, a \
                     clearance of {:.3} mm against the {MARGIN_MM:.1} mm needed",
                    s.r,
                    d - s.r
                )),
                Err(why) => note(&mut out, format!("standing fixture {fname}: {why}")),
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::rect_distance;

    #[test]
    fn a_box_across_the_line_has_no_distance_and_one_under_it_has_its_gap() {
        assert_eq!(rect_distance((-5.0, 5.0), (-5.0, 5.0)), None);
        assert_eq!(rect_distance((-5.0, 5.0), (-90.0, -80.0)), Some(80.0));
        let d = rect_distance((30.0, 50.0), (-90.0, -40.0)).unwrap_or(f64::NAN);
        assert!((d - 50.0).abs() < 1e-12, "{d}");
    }
}
