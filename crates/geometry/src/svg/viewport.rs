//! The root `<svg>` viewport: its physical size, the `viewBox` mapping and the Y flip.
//! Provenance: the W3C SVG 1.1 coordinate-systems chapter (`viewBox` and
//! `preserveAspectRatio`, user units as CSS px without a viewBox) and the CSS absolute-length
//! table (96 px to the inch); original implementation.

use super::number::{LenUnit, length, list};
use super::transform::Affine;
use super::xml::Node;
use super::{SvgError, invalid};

/// Where user space goes.
pub(super) struct Viewport {
    /// User space to output space: millimetres when `physical`, else viewport units. Y is up.
    pub map: Affine,
    /// The file states a physical size.
    pub physical: bool,
}

fn dimension(node: &Node, name: &str) -> Result<Option<(f64, LenUnit)>, SvgError> {
    let Some(text) = node.attr(name) else { return Ok(None) };
    let (v, unit) = length(text, &format!("<svg> {name}"))?;
    if unit == LenUnit::Relative {
        return Ok(None); // "100%": no size of its own
    }
    if v <= 0.0 {
        return Err(invalid(format!("<svg> {name} \"{text}\" is a zero or negative size")));
    }
    Ok(Some((v, unit)))
}

/// The `preserveAspectRatio` placement of a `vw` x `vh` viewBox at (`vx`, `vy`) in a
/// `w` x `h` viewport.
fn fit(node: &Node, [vx, vy, vw, vh]: [f64; 4], w: f64, h: f64) -> Affine {
    let par = node.attr("preserveAspectRatio").unwrap_or("xMidYMid meet");
    let mut words = par.split_whitespace().filter(|w| *w != "defer");
    let align = words.next().unwrap_or("xMidYMid");
    let slice = words.next() == Some("slice");
    let (sx, sy) = (w / vw, h / vh);
    if align == "none" {
        return Affine::scale(sx, sy).then(Affine::translate(-vx, -vy));
    }
    let s = if slice { sx.max(sy) } else { sx.min(sy) };
    let share = |axis: &str, free: f64| {
        if align.contains(&format!("{axis}Mid")) {
            free / 2.0
        } else if align.contains(&format!("{axis}Max")) {
            free
        } else {
            0.0
        }
    };
    let (tx, ty) = (share("x", w - vw * s), share("Y", h - vh * s));
    Affine::translate(tx, ty).then(Affine::scale(s, s)).then(Affine::translate(-vx, -vy))
}

/// Read the root element's size and viewBox.
pub(super) fn read(root: &Node) -> Result<Viewport, SvgError> {
    let view_box = match root.attr("viewBox") {
        None => None,
        Some(text) => {
            let v = list(text, "<svg> viewBox")?;
            match *v.as_slice() {
                [x, y, w, h] if w > 0.0 && h > 0.0 => Some([x, y, w, h]),
                [_, _, _, _] => return Err(invalid(format!("<svg> viewBox \"{text}\" has a zero or negative size"))),
                _ => return Err(invalid(format!("<svg> viewBox \"{text}\" must be four numbers"))),
            }
        }
    };
    let (width, height) = (dimension(root, "width")?, dimension(root, "height")?);
    let stated = |d: Option<(f64, LenUnit)>| d.is_some_and(|(_, u)| u != LenUnit::None);
    let physical = stated(width) || stated(height);
    // A unitless number beside a stated unit is CSS px; alone it is no physical size.
    let value = |d: Option<(f64, LenUnit)>| {
        d.map(|(v, u)| if physical { v * u.mm().unwrap_or(25.4 / 96.0) } else { v })
    };
    let (w, h) = match (value(width), value(height), view_box) {
        (Some(w), Some(h), _) => (Some(w), Some(h)),
        (Some(w), None, Some([.., vw, vh])) => (Some(w), Some(w * vh / vw)),
        (None, Some(h), Some([.., vw, vh])) => (Some(h * vw / vh), Some(h)),
        (Some(_), None, None) => return Err(invalid("<svg> states a width but no height and no viewBox")),
        (None, Some(_), None) => return Err(invalid("<svg> states a height but no width and no viewBox")),
        (None, None, _) => (None, None),
    };
    // User space to the viewport, Y still down.
    let (to_viewport, page_height) = match (view_box, w, h) {
        (Some(vb), Some(w), Some(h)) => (fit(root, vb, w, h), Some(h)),
        (Some([vx, vy, _, vh]), _, _) => (Affine::translate(-vx, -vy), Some(vh)),
        // No viewBox: a user unit is one CSS px.
        (None, _, h) if physical => (Affine::scale(25.4 / 96.0, 25.4 / 96.0), h),
        (None, _, h) => (Affine::IDENTITY, h),
    };
    // Y up, the page's bottom edge on y = 0 (or the user-space origin when the page has no
    // height).
    let flip = Affine { d: -1.0, f: page_height.unwrap_or(0.0), ..Affine::IDENTITY };
    Ok(Viewport { map: flip.then(to_viewport), physical })
}
