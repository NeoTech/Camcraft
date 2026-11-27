//! #3897 (U6), #4338 (U12 of #3663): the holder drawing of the tool
//! library window, shared by the read-only holder view and the holder
//! form: the outline to scale, nose down, with the spindle nose as a
//! dashed box above the gauge line and the shank range the holder clamps
//! as a dashed ghost below the nose. The dimension lines give the gauge
//! length, the largest and the nose diameter; the drawing keeps room for
//! their labels, so no label is cut off at the edge.
//! Frame cost (`crates/ui/AGENTS.md`): the shapes and labels are built
//! when the holder, the rect, the unit or the dimension toggle change; a
//! still frame builds nothing (`builds`).
//! Provenance: our own drawing over `holder_library::HolderEntry::points`;
//! the dimension style follows `side_view` (general drawing practice). No
//! HSMWorks material involved.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};

use camcraft_app::holder_library::HolderEntry;
use camcraft_app::units::{self, Unit};

use crate::egui_app::tool_model_view::BG;

const BODY: Color32 = Color32::from_rgb(86, 92, 104);
const EDGE: Color32 = Color32::from_rgb(214, 220, 228);
const DIM: Color32 = Color32::from_rgb(0x7F, 0xB4, 0xD8);
const NOTE: Color32 = Color32::from_rgb(0x9E, 0xA8, 0xB8);
/// The room (px) the labels keep at the right and the left of the outline.
const LABEL_R: f32 = 150.0;
const LABEL_L: f32 = 40.0;
/// The spindle nose box and the shank ghost, as shares of the gauge length.
const NOSE_BOX: f64 = 0.25;
const SHANK: f64 = 0.35;
/// The label of the spindle nose box.
pub(crate) const SPINDLE: &str = "spindle nose";

/// What the drawing was built from.
#[derive(Clone, PartialEq)]
struct Key {
    entry: HolderEntry,
    clamps: (f64, f64),
    rect: Rect,
    unit: Unit,
    dims: bool,
}

/// The kept drawing: its shapes and its labels.
struct Drawing {
    shapes: Vec<Shape>,
    labels: Vec<(Pos2, Align2, String)>,
}

thread_local! {
    static KEPT: RefCell<Option<(Key, Rc<Drawing>)>> = const { RefCell::new(None) };
    static BUILDS: Cell<u64> = const { Cell::new(0) };
}

/// The drawings built on this thread - a still frame adds none.
#[cfg(test)]
pub(crate) fn builds() -> u64 {
    BUILDS.with(Cell::get)
}

/// The largest diameter of the holder: the flange's or the body's.
pub(crate) fn largest_d(e: &HolderEntry) -> f64 {
    e.taper.map_or(0.0, |t| t.flange_d).max(e.body_r() * 2.0)
}

/// Holder `e` (shank range `clamps`, mm) to scale in a `size` area.
pub(crate) fn outline(ui: &mut Ui, e: &HolderEntry, clamps: (f64, f64), size: Vec2, u: Unit, dims: bool) {
    let (resp, painter) = ui.allocate_painter(size, Sense::hover());
    painter.rect_filled(resp.rect, 4.0, BG);
    let key = Key { entry: *e, clamps, rect: resp.rect, unit: u, dims };
    let d = KEPT.with(|k| {
        let mut k = k.borrow_mut();
        if let Some((kk, d)) = k.as_ref()
            && *kk == key
        {
            return Rc::clone(d);
        }
        BUILDS.with(|b| b.set(b.get().saturating_add(1)));
        let d = Rc::new(build(e, clamps, resp.rect, u, dims));
        *k = Some((key, Rc::clone(&d)));
        d
    });
    painter.extend(d.shapes.iter().cloned());
    for (at, align, text) in &d.labels {
        painter.text(*at, *align, text, FontId::proportional(11.0), NOTE);
    }
}

fn build(e: &HolderEntry, clamps: (f64, f64), rect: Rect, u: Unit, dims: bool) -> Drawing {
    let mut d = Drawing { shapes: Vec::new(), labels: Vec::new() };
    let Ok(points) = e.points(0.0, e.gauge_len) else {
        d.labels.push((rect.center(), Align2::CENTER_CENTER, "no outline".to_owned()));
        return d;
    };
    let pts: Vec<(f64, f64)> = points.iter().map(|p| (p.z, p.r)).filter(|(z, r)| z.is_finite() && r.is_finite()).collect();
    let r_max = pts.iter().map(|p| p.1).fold(0.0f64, f64::max);
    let z_max = pts.iter().map(|p| p.0).fold(0.0f64, f64::max);
    if r_max <= 0.0 || z_max <= 0.0 {
        return d;
    }
    // The spindle nose box is wider than the holder; the room left for
    // the labels on both sides is kept out of the scale.
    let r_box = r_max * 1.3;
    let (top, bottom) = (z_max * (1.0 + NOSE_BOX), -z_max * SHANK);
    let inner = Rect::from_min_max(
        Pos2::new(rect.left() + LABEL_L, rect.top() + 12.0),
        Pos2::new((rect.right() - LABEL_R).max(rect.left() + LABEL_L + 40.0), (rect.bottom() - 26.0).max(rect.top() + 52.0)),
    );
    let scale = (inner.width() / (2.0 * r_box as f32)).min(inner.height() / (top - bottom) as f32).max(f32::EPSILON);
    let axis = inner.center().x;
    let base = inner.bottom() + bottom as f32 * scale;
    let at = |z: f64, r: f64| Pos2::new(axis + r as f32 * scale, base - z as f32 * scale);
    d.shapes.push(Shape::line_segment(
        [Pos2::new(axis, rect.top() + 4.0), Pos2::new(axis, rect.bottom() - 4.0)],
        Stroke::new(1.0, Color32::from_rgb(70, 74, 84)),
    ));
    for w in pts.windows(2) {
        let ((z0, r0), (z1, r1)) = (w[0], w[1]);
        if z1 > z0 {
            d.shapes.push(Shape::convex_polygon(vec![at(z0, -r0), at(z0, r0), at(z1, r1), at(z1, -r1)], BODY, Stroke::NONE));
        }
    }
    let mut edge: Vec<Pos2> = pts.iter().map(|&(z, r)| at(z, r)).collect();
    edge.extend(pts.iter().filter(|p| p.1 > 0.0).rev().map(|&(z, r)| at(z, -r)));
    d.shapes.push(Shape::closed_line(edge, Stroke::new(1.0, EDGE)));
    let ghost = Stroke::new(1.0, NOTE);
    let dashed = |d: &mut Drawing, pts: &[Pos2]| d.shapes.extend(Shape::dashed_line(pts, ghost, 5.0, 4.0));
    // The spindle nose above the gauge line.
    let sp = [at(z_max, -r_box), at(top, -r_box), at(top, r_box), at(z_max, r_box), at(z_max, -r_box)];
    dashed(&mut d, &sp);
    d.labels.push((Pos2::new(at(z_max, r_box).x + 6.0, at((z_max + top) * 0.5, 0.0).y), Align2::LEFT_CENTER, SPINDLE.to_owned()));
    // The shank range below the nose: the largest shank, the smallest inside.
    let (from, to) = clamps;
    if to > 0.0 && to.is_finite() {
        let (rt, rf) = (to * 0.5, from.max(0.0) * 0.5);
        dashed(&mut d, &[at(0.0, -rt), at(bottom, -rt), at(bottom, rt), at(0.0, rt)]);
        if rf > 0.0 && (rt - rf) > 1e-9 {
            dashed(&mut d, &[at(0.0, -rf), at(bottom * 0.8, -rf), at(bottom * 0.8, rf), at(0.0, rf)]);
        }
        let len = |v: f64| units::len(v, u, 2);
        let text = if (to - from).abs() < 1e-9 { format!("shank Ø{}", len(to)) } else { format!("shank Ø{} to {}", len(from), len(to)) };
        d.labels.push((Pos2::new(at(0.0, rt).x + 6.0, at(bottom * 0.5, 0.0).y), Align2::LEFT_CENTER, text));
    }
    if dims {
        let len = |v: f64| units::len(v, u, 2);
        let stroke = Stroke::new(1.0, DIM);
        // The gauge length on the right, the largest diameter over the top,
        // the nose diameter under the nose.
        let x = at(0.0, r_max).x + 14.0;
        d.shapes.push(Shape::line_segment([Pos2::new(x, at(0.0, 0.0).y), Pos2::new(x, at(z_max, 0.0).y)], stroke));
        d.labels.push((Pos2::new(x + 4.0, at(z_max * 0.5, 0.0).y), Align2::LEFT_CENTER, format!("gauge length {}", len(e.gauge_len))));
        let y = at(z_max, 0.0).y - 8.0;
        d.shapes.push(Shape::line_segment([Pos2::new(at(0.0, -r_max).x, y), Pos2::new(at(0.0, r_max).x, y)], stroke));
        d.labels.push((Pos2::new(axis, y - 2.0), Align2::CENTER_BOTTOM, format!("Ø {}", len(largest_d(e)))));
        let rn = e.body_r();
        let y = at(0.0, 0.0).y + 8.0;
        d.shapes.push(Shape::line_segment([Pos2::new(at(0.0, -rn).x, y), Pos2::new(at(0.0, rn).x, y)], stroke));
        d.labels.push((Pos2::new(at(0.0, -rn).x - 4.0, y), Align2::RIGHT_CENTER, format!("nose Ø {}", len(rn * 2.0))));
    }
    d
}
