//! #4337 (U11 of #3663): the Recipes tab's derived line - the surface
//! speed and the chip load of the selected row, as text in the document's
//! units. Pure arithmetic over a `DerivedKey`; the tab keeps the line in
//! `RecipesView` and computes it again only when the key changes.
//! Provenance: the handbook relations Vc = pi x D x n / 1000 and
//! fz = F / (n x z) (`recipes_fill`); no HSMWorks material involved.

use camcraft_app::tool_cutting_rows::material_label;
use camcraft_app::units::Unit;

use crate::state::DerivedKey;

use super::recipes_fill::{feed_per_tooth, surface_speed};

/// "= 377 m/min surface speed, 0.050 mm per tooth (Aluminium 6061)"; a
/// value that cannot be derived names the input it needs.
#[must_use]
pub(crate) fn derived_line(k: &DerivedKey) -> String {
    let vc = surface_speed(k.diameter, k.rpm).map_or_else(
        || "no surface speed (it needs a diameter and a spindle rpm)".to_owned(),
        |vc| format!("{} surface speed", vc_text(vc, k.unit)),
    );
    let fz = feed_per_tooth(k.feed, k.rpm, k.flutes).map_or_else(
        || "no chip load (it needs a spindle rpm and a stated flute count)".to_owned(),
        |fz| format!("{} per tooth", fz_text(fz, k.unit)),
    );
    format!("= {vc}, {fz} ({})", material_label(k.material))
}

/// Surface speed: m/min, or surface feet per minute in an inch document.
fn vc_text(vc: f64, u: Unit) -> String {
    match u {
        Unit::Millimeters => format!("{vc:.0} m/min"),
        Unit::Inches => format!("{:.0} sfm", vc * 1000.0 / 304.8),
    }
}

/// Feed per tooth: mm, or inch in an inch document.
fn fz_text(fz: f64, u: Unit) -> String {
    match u {
        Unit::Millimeters => format!("{fz:.3} mm"),
        Unit::Inches => format!("{:.4} in", fz / 25.4),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camcraft_app::tools::WorkMaterial;

    fn key(rpm: f64, feed: f64) -> DerivedKey {
        DerivedKey {
            material: Some(WorkMaterial::Aluminium6061),
            diameter: 10.0,
            rpm,
            feed,
            flutes: Some(3),
            unit: Unit::Millimeters,
        }
    }

    #[test]
    fn the_derived_line_follows_rpm_and_feed() {
        let a = derived_line(&key(12_000.0, 1800.0));
        assert!(a.starts_with("= 377 m/min surface speed, 0.050 mm per tooth"), "{a}");
        let b = derived_line(&key(6000.0, 1800.0));
        assert!(b.starts_with("= 188 m/min surface speed, 0.100 mm per tooth"), "{b}");
        let c = derived_line(&key(6000.0, 900.0));
        assert!(c.starts_with("= 188 m/min surface speed, 0.050 mm per tooth"), "{c}");
        let inch = derived_line(&DerivedKey { unit: Unit::Inches, ..key(12_000.0, 1800.0) });
        assert!(inch.contains("sfm surface speed") && inch.contains(" in per tooth"), "{inch}");
        let none = derived_line(&DerivedKey { flutes: None, ..key(0.0, 1800.0) });
        assert!(none.contains("no surface speed") && none.contains("no chip load"), "{none}");
    }
}
