//! #4453: the Recipes tab's surface speed (Vc) and chip load (fz) as
//! inputs. Each table row keeps its stored spindle rpm and feed (the
//! project file does not change); a Vc edit sets the row's rpm, an fz
//! edit sets its feed, and the table shows the new number. The edits
//! write the window's draft only: the editor's one Apply lands it as
//! `ChangeTool`, one undo step, as for every other tool number. A missing
//! flute count, a zero rpm or a zero diameter refuses the input with a
//! reason; nothing divides by zero.
//! Provenance: the handbook relations n = Vc x 1000 / (pi x D) and
//! F = fz x z x n; no HSMWorks material involved.

use egui::{DragValue, RichText, Ui};

use camcraft_app::tool_cutting_rows::{CuttingRow, material_label};
use camcraft_app::tools::WorkMaterial;
use camcraft_app::units::Unit;

use super::super::fields::named;
use super::recipes_fill::{feed_per_tooth, surface_speed};
use crate::state::ToolUi;

/// The accessible name of a row's surface speed field: "Surface speed
/// (Brass)".
pub(crate) const VC: &str = "Surface speed";
/// The accessible name of a row's chip load field: "Chip load (Brass)".
pub(crate) const FZ: &str = "Chip load";

/// Why a surface speed cannot be typed for a tool of diameter `d`, mm.
#[must_use]
pub(crate) fn vc_refusal(d: f64) -> Option<&'static str> {
    (!(d > 0.0 && d.is_finite())).then_some("A surface speed needs a tool diameter above zero.")
}

/// Why a chip load cannot be typed for a row of `rpm` on a tool of
/// `flutes`.
#[must_use]
pub(crate) fn fz_refusal(rpm: f64, flutes: Option<u32>) -> Option<&'static str> {
    if !flutes.is_some_and(|z| z > 0) {
        Some("A chip load needs a stated flute count (the Flutes row below the table).")
    } else if !(rpm > 0.0 && rpm.is_finite()) {
        Some("A chip load needs a spindle rpm above zero (type the rpm or the surface speed first).")
    } else {
        None
    }
}

/// The spindle rpm for surface speed `vc` (m/min) on diameter `d` (mm):
/// n = Vc x 1000 / (pi x D).
///
/// # Errors
/// The reason, when the diameter is not above zero or `vc` is negative.
pub(crate) fn rpm_for(vc: f64, d: f64) -> Result<f64, &'static str> {
    if let Some(why) = vc_refusal(d) {
        return Err(why);
    }
    if !(vc >= 0.0 && vc.is_finite()) {
        return Err("A surface speed is zero or more.");
    }
    Ok(vc * 1000.0 / (std::f64::consts::PI * d))
}

/// The feed (mm/min) for chip load `fz` (mm) at `rpm` on `flutes`:
/// F = fz x z x n.
///
/// # Errors
/// The reason, when the flute count is not stated, the rpm is not above
/// zero or `fz` is negative.
pub(crate) fn feed_for(fz: f64, rpm: f64, flutes: Option<u32>) -> Result<f64, &'static str> {
    if let Some(why) = fz_refusal(rpm, flutes) {
        return Err(why);
    }
    let z = flutes.unwrap_or_default();
    if !(fz >= 0.0 && fz.is_finite()) {
        return Err("A chip load is zero or more.");
    }
    Ok(fz * f64::from(z) * rpm)
}

/// A drawn row's numbers: the tool's own (the default row the table
/// derives when the draft stores none), or a stored row.
#[derive(Clone, Copy)]
enum Target {
    Own,
    Row(usize),
}

/// The drawn rows, top down, in the order of the table and of the
/// derived lines (`recipes_tab::derived`).
fn targets(t: &ToolUi) -> Vec<Target> {
    let lead = (!t.cutting.iter().any(CuttingRow::is_unknown)).then_some(Target::Own);
    lead.into_iter()
        .chain(t.cutting.iter().enumerate().filter(|(_, r)| !r.is_recipe()).map(|(i, _)| Target::Row(i)))
        .collect()
}

fn numbers(t: &ToolUi, at: Target) -> Option<(Option<WorkMaterial>, f64, f64)> {
    match at {
        Target::Own => Some((None, t.rpm, t.feed)),
        Target::Row(i) => t.cutting.get(i).map(|r| (r.material, r.rpm, r.feed)),
    }
}

fn set_rpm(t: &mut ToolUi, at: Target, rpm: f64) {
    match at {
        Target::Own => t.rpm = rpm,
        Target::Row(i) => {
            if let Some(r) = t.cutting.get_mut(i) {
                r.rpm = rpm;
            }
        }
    }
}

fn set_feed(t: &mut ToolUi, at: Target, feed: f64) {
    match at {
        Target::Own => t.feed = feed,
        Target::Row(i) => {
            if let Some(r) = t.cutting.get_mut(i) {
                r.feed = feed;
            }
        }
    }
}

/// One line for each drawn row: its surface speed and chip load fields,
/// the row's material, and the derived line `lines[i]` as the hover text.
pub(super) fn draw(ui: &mut Ui, t: &mut ToolUi, u: Unit, lines: &[String]) {
    for (i, at) in targets(t).into_iter().enumerate() {
        let Some((material, rpm, feed)) = numbers(t, at) else { continue };
        let line = lines.get(i).map_or("", String::as_str);
        let label = material_label(material);
        ui.horizontal(|ui| {
            weak(ui, "=");
            let vc = surface_speed(t.diameter, rpm).unwrap_or_default();
            if let Some(v) = field(ui, vc_display(vc, u), vc_suffix(u), 0, 1.0, vc_refusal(t.diameter), &format!("{VC} ({label})"), line) {
                if let Ok(n) = rpm_for(vc_metric(v, u), t.diameter) {
                    set_rpm(t, at, n);
                }
            }
            weak(ui, "surface speed,");
            let fz = feed_per_tooth(feed, rpm, t.flutes).unwrap_or_default();
            let (prec, speed) = match u {
                Unit::Millimeters => (3, 0.001),
                Unit::Inches => (4, 0.0001),
            };
            if let Some(v) = field(ui, fz_display(fz, u), fz_suffix(u), prec, speed, fz_refusal(rpm, t.flutes), &format!("{FZ} ({label})"), line) {
                if let Ok(f) = feed_for(fz_metric(v, u), rpm, t.flutes) {
                    set_feed(t, at, f);
                }
            }
            weak(ui, &format!("per tooth ({label})"));
        });
    }
}

/// A value field; disabled with `refusal` as its hover text when the
/// value cannot be typed. Returns the new display value on a change.
#[allow(clippy::too_many_arguments)]
fn field(ui: &mut Ui, mut v: f64, suffix: &str, prec: usize, speed: f64, refusal: Option<&str>, name: &str, line: &str) -> Option<f64> {
    let drag = DragValue::new(&mut v)
        .speed(speed)
        .range(0.0..=f64::MAX)
        .custom_formatter(move |n, _| format!("{n:.prec$}"))
        .suffix(suffix);
    let r = named(ui.add_enabled(refusal.is_none(), drag), name);
    let r = match refusal {
        Some(why) => r.on_disabled_hover_text(why),
        None => r.on_hover_text(line),
    };
    (r.changed() && refusal.is_none()).then_some(v)
}

fn weak(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).weak());
}

/// m/min, or surface feet per minute in an inch document.
fn vc_display(vc: f64, u: Unit) -> f64 {
    match u {
        Unit::Millimeters => vc,
        Unit::Inches => vc * 1000.0 / 304.8,
    }
}

fn vc_metric(v: f64, u: Unit) -> f64 {
    match u {
        Unit::Millimeters => v,
        Unit::Inches => v * 304.8 / 1000.0,
    }
}

fn vc_suffix(u: Unit) -> &'static str {
    match u {
        Unit::Millimeters => " m/min",
        Unit::Inches => " sfm",
    }
}

/// mm, or inch in an inch document.
fn fz_display(fz: f64, u: Unit) -> f64 {
    match u {
        Unit::Millimeters => fz,
        Unit::Inches => fz / 25.4,
    }
}

fn fz_metric(v: f64, u: Unit) -> f64 {
    match u {
        Unit::Millimeters => v,
        Unit::Inches => v * 25.4,
    }
}

fn fz_suffix(u: Unit) -> &'static str {
    match u {
        Unit::Millimeters => " mm",
        Unit::Inches => " in",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_fz_edit_sets_the_feed_and_the_feed_gives_the_fz_back() {
        let f = feed_for(0.05, 12_000.0, Some(3)).unwrap_or_default();
        assert!((f - 1800.0).abs() < 1e-9, "{f}");
        let fz = feed_per_tooth(f, 12_000.0, Some(3)).unwrap_or_default();
        assert!((fz - 0.05).abs() < 1e-12, "{fz}");
    }

    #[test]
    fn a_vc_edit_sets_the_rpm_and_the_rpm_gives_the_vc_back() {
        let n = rpm_for(377.0, 10.0).unwrap_or_default();
        assert!((n - 377_000.0 / (std::f64::consts::PI * 10.0)).abs() < 1e-9, "{n}");
        let vc = surface_speed(10.0, n).unwrap_or_default();
        assert!((vc - 377.0).abs() < 1e-9, "{vc}");
    }

    #[test]
    fn a_missing_flute_count_or_a_zero_diameter_refuses_the_input() {
        assert!(feed_for(0.05, 12_000.0, None).is_err_and(|e| e.contains("flute count")));
        assert!(feed_for(0.05, 12_000.0, Some(0)).is_err_and(|e| e.contains("flute count")));
        assert!(feed_for(0.05, 0.0, Some(3)).is_err_and(|e| e.contains("spindle rpm")));
        assert!(rpm_for(300.0, 0.0).is_err_and(|e| e.contains("diameter")));
        assert!(rpm_for(300.0, f64::NAN).is_err());
        assert!(rpm_for(-1.0, 10.0).is_err());
        assert!(feed_for(f64::INFINITY, 1000.0, Some(2)).is_err());
    }

    #[test]
    fn the_inch_lens_converts_both_ways() {
        let sfm = vc_display(304.8, Unit::Inches);
        assert!((sfm - 1000.0).abs() < 1e-9);
        assert!((vc_metric(sfm, Unit::Inches) - 304.8).abs() < 1e-9);
        assert!((fz_metric(fz_display(0.1, Unit::Inches), Unit::Inches) - 0.1).abs() < 1e-12);
    }
}
