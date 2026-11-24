//! #3894 (U3 of #3663): the to-scale side view of the tool library
//! window - the tool page's side drawing (`tool_preview::paint`, #2002)
//! at the size of the editor, with dimension lines: the diameter, the
//! flute length, the shank, the stick-out, the gauge length and the
//! holder's diameter and length. No flute helix is painted. Display only.
//! Provenance: our own drawing after general technical-drawing practice
//! (extension lines, a dimension line with end ticks, the value beside
//! it); no HSMWorks material involved.

use egui::{Align2, Color32, FontId, Painter, Pos2, Sense, Stroke, Ui, Vec2};

use camcraft_app::units::{self, Unit};

use super::model::ToolModel;
use crate::egui_app::tool_model_view::BG;
use crate::egui_app::tool_preview::{self, Scale};

const NOTE: Color32 = Color32::from_rgb(0x9E, 0xA8, 0xB8);
const DIM: Color32 = Color32::from_rgb(0x7F, 0xB4, 0xD8);
/// Room around the drawing for the dimension columns and their values.
const LEFT: f32 = 96.0;
const RIGHT: f32 = 120.0;
const TOP: f32 = 30.0;
const BOTTOM: f32 = 26.0;
/// The gap between the drawing and a dimension line, and between two lines.
const GAP: f32 = 12.0;

/// The side view of `m` in a `size` area.
pub(super) fn draw(ui: &mut Ui, m: &ToolModel, u: Unit, size: Vec2) {
    let (resp, painter) = ui.allocate_painter(size, Sense::hover());
    let rect = resp.rect;
    painter.rect_filled(rect, 4.0, BG);
    let inner = egui::Rect::from_min_max(rect.min + Vec2::new(LEFT, TOP), rect.max - Vec2::new(RIGHT, BOTTOM));
    if inner.width() < 20.0 || inner.height() < 20.0 {
        return;
    }
    let Some(s) = tool_preview::paint(&painter, inner, &m.tool, Some(&m.bands), Some(&m.steel)) else {
        return;
    };
    painter.text(rect.min + Vec2::new(6.0, 6.0), Align2::LEFT_TOP, format!("dimensions in {}", u.suffix()), font(), NOTE);
    dims(&painter, &s, m, u);
}

fn font() -> FontId {
    FontId::proportional(11.0)
}

fn dims(p: &Painter, s: &Scale, m: &ToolModel, u: Unit) {
    let d = &m.dims;
    let len = |v: f64| units::len(v, u, 2);
    let r_max = m.steel.iter().map(|q| q.1).chain(m.bands.iter().map(|b| b.2)).filter(|r| r.is_finite()).fold(0.0, f64::max);
    // The diameter under the tip.
    let r = d.diameter.max(0.0) * 0.5;
    let y = s.base_y + GAP;
    horizontal(p, s.at(0.0, -r), s.at(0.0, r), y, &format!("Ø {}", len(d.diameter)), Align2::CENTER_TOP);
    // Right: the flute length, then the shank above it; values right.
    let xr = s.at(0.0, r_max).x + GAP;
    let zs = d.flute_len + d.shank_len;
    for (z0, z1, text) in [
        (0.0, d.flute_len, format!("flute {}", len(d.flute_len))),
        (d.flute_len, zs, format!("shank Ø {} × {}", len(d.shank_d), len(d.shank_len))),
    ] {
        if let Some(mid) = vertical(p, s, xr, z0, z1, true) {
            p.text(Pos2::new(xr + 4.0, mid), Align2::LEFT_CENTER, text, font(), NOTE);
        }
    }
    // Left: the stick-out (tip to holder face) and, outside it, the gauge
    // length (tip to holder top); both values left of the two lines.
    let xl = s.at(0.0, -r_max).x - GAP;
    let xt = xl - GAP - 4.0;
    if let Some(mid) = vertical(p, s, xl, 0.0, d.stick_out, false) {
        p.text(Pos2::new(xt, mid), Align2::RIGHT_CENTER, format!("stick-out {}", len(d.stick_out)), font(), NOTE);
    }
    if let Some((hd, hl)) = d.holder {
        if vertical(p, s, xl - GAP, 0.0, d.gauge, false).is_some() {
            let y = s.at(d.gauge, 0.0).y + 8.0;
            p.text(Pos2::new(xt, y), Align2::RIGHT_CENTER, format!("gauge {}", len(d.gauge)), font(), NOTE);
        }
        let hr = hd.max(0.0) * 0.5;
        let y = s.at(d.gauge, 0.0).y - GAP * 0.5;
        let text = format!("holder Ø {} × {}", len(hd), len(hl));
        horizontal(p, s.at(d.gauge, -hr), s.at(d.gauge, hr), y, &text, Align2::CENTER_BOTTOM);
    }
}

/// A horizontal dimension between the x of `a` and `b` at height `y`,
/// with extension lines from the two points.
fn horizontal(p: &Painter, a: Pos2, b: Pos2, y: f32, text: &str, align: Align2) {
    if !(a.x.is_finite() && b.x.is_finite() && y.is_finite()) || (b.x - a.x).abs() < 1.0 {
        return;
    }
    let thin = Stroke::new(0.5, DIM);
    p.line_segment([a, Pos2::new(a.x, y)], thin);
    p.line_segment([b, Pos2::new(b.x, y)], thin);
    line_with_ticks(p, Pos2::new(a.x, y), Pos2::new(b.x, y));
    let dy = if align == Align2::CENTER_TOP { 2.0 } else { -2.0 };
    p.text(Pos2::new((a.x + b.x) * 0.5, y + dy), align, text, font(), NOTE);
}

/// A vertical dimension line from height `z0` to `z1` at screen `x`, with
/// extension lines toward the drawing (it stands right of the drawing when
/// `right`). Gives the screen height of its middle, for the value.
fn vertical(p: &Painter, s: &Scale, x: f32, z0: f64, z1: f64, right: bool) -> Option<f32> {
    if !(z0.is_finite() && z1.is_finite() && x.is_finite()) || z1 - z0 <= 0.0 {
        return None;
    }
    let (y0, y1) = (s.at(z0, 0.0).y, s.at(z1, 0.0).y);
    let thin = Stroke::new(0.5, DIM);
    let toward = if right { x - GAP * 0.75 } else { x + GAP * 0.75 };
    p.line_segment([Pos2::new(toward, y0), Pos2::new(x, y0)], thin);
    p.line_segment([Pos2::new(toward, y1), Pos2::new(x, y1)], thin);
    line_with_ticks(p, Pos2::new(x, y0), Pos2::new(x, y1));
    Some((y0 + y1) * 0.5)
}

/// The dimension line with a short slanted tick at each end.
fn line_with_ticks(p: &Painter, a: Pos2, b: Pos2) {
    let stroke = Stroke::new(1.0, DIM);
    p.line_segment([a, b], stroke);
    let t = Vec2::new(3.0, -3.0);
    for q in [a, b] {
        p.line_segment([q - t, q + t], stroke);
    }
}
