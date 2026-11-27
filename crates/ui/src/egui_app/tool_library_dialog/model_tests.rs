//! #4452: the preview model's finer revolve and the part of each facet
//! (flute, shank, holder) that the 3D view inks; and that each fluted
//! kind's preview model carries a flute mesh.
//! Provenance: our own tests over the shipped catalog tools.

use super::*;
use camcraft_app::tools::catalog;

fn shipped(name: &str) -> Tool {
    match catalog().into_iter().find(|(n, _)| *n == name) {
        Some((_, t)) => t,
        None => panic!("{name} is a shipped tool"),
    }
}

fn model(t: &Tool) -> ToolModel {
    build(&Key { tool: t.clone(), doc: None, cutter: None, body: None, holder: None })
}

/// The distinct angles of the vertices on the ring at height `z`.
fn ring_angles(mesh: &[Triangle], z: f64) -> usize {
    let mut a: Vec<f64> = mesh
        .iter()
        .flat_map(|t| [t.a, t.b, t.c])
        .filter(|p| (p.z - z).abs() < 1e-9 && p.x.hypot(p.y) > 1e-9)
        .map(|p| p.y.atan2(p.x).rem_euclid(std::f64::consts::TAU))
        .collect();
    a.sort_by(f64::total_cmp);
    a.dedup_by(|x, y| (*x - *y).abs() < 1e-9 || (std::f64::consts::TAU - (*x - *y).abs()).abs() < 1e-9);
    a.len()
}

#[test]
fn the_preview_revolve_is_finer_than_before() {
    // The revolve took 32 facets around before #4452.
    assert!(RING >= 96, "the preview ring has {RING} facets");
    assert_eq!(RING, super::super::flutes::SEGS, "as many as the fluted steel");
    let t = shipped("EM-1000-4FL");
    let m = model(&t);
    let shank_z = m.bare.iter().flat_map(|q| [q.a.z, q.b.z, q.c.z]).find(|v| (v - t.flute_len).abs() < 1e-9);
    assert!(shank_z.is_some(), "the steel has a ring at the flute length");
    assert_eq!(ring_angles(&m.bare, t.flute_len), RING, "{RING} vertices around the ring");
    assert!(m.bare.len() >= 2 * RING, "the bare steel has {} facets", m.bare.len());
    assert!(m.mesh.len() <= crate::egui_app::tool_model_view::MAX_PAINTED, "inside the painted budget");
}

#[test]
fn each_facet_carries_its_part() {
    let t = shipped("EM-1000-4FL");
    let m = model(&t);
    assert_eq!(m.parts.len(), m.mesh.len(), "one part per facet");
    let holder = m.mesh.len() - m.bare.len();
    assert!(holder > 0, "the end mill has its parameter holder");
    assert!(m.parts[m.bare.len()..].iter().all(|p| *p == Part::Holder), "the facets after the steel are the holder");
    for (t3, p) in m.bare.iter().zip(m.parts.iter()) {
        let c = (t3.a.z + t3.b.z + t3.c.z) / 3.0;
        let want = if c < t.flute_len { Part::Flute } else { Part::Shank };
        assert_eq!(*p, want, "a facet at z {c}");
    }
    for want in [Part::Flute, Part::Shank, Part::Holder] {
        assert!(m.parts.contains(&want), "the end mill has a {want:?} part");
    }
    let (Some(f), Some(fp)) = (&m.fluted, &m.fluted_parts) else { panic!("a 4-flute end mill has flutes") };
    assert_eq!(f.len(), fp.len(), "one part per fluted facet");
    let (mesh, parts, _) = m.shown(false, true);
    assert!(Arc::ptr_eq(parts, fp) && mesh.len() <= parts.len(), "the bare fluted mesh reads the fluted parts");
    let (mesh, parts, _) = m.shown(true, false);
    assert!(Arc::ptr_eq(mesh, &m.mesh) && Arc::ptr_eq(parts, &m.parts), "the plain mesh reads the plain parts");
}

/// #4452: each kind with a flute length has a flute mesh in the preview
/// model, painted with the cutting part's ink.
#[test]
fn each_fluted_kind_has_a_flute_mesh_in_the_preview() {
    let names = [
        "EM-1000-4FL", "BN-0600", "BB-1000-R1", "DR-0600", "CM-0600-90", "TAP-M6", "TM-0600-P125",
        "TMS-0600-P050-150", "TS-2000-0600", "KS-1600-0300", "DT-2000-60", "LP-1000", "FM-CV-R6",
    ];
    for name in names {
        let t = shipped(name);
        let m = model(&t);
        let Some(f) = &m.fluted else { panic!("{name} ({:?}) has a flute mesh", t.kind) };
        let parts = m.fluted_parts.as_ref().map_or(0, |p| p.iter().filter(|p| **p == Part::Flute).count());
        assert!(parts > 0 && !f.is_empty(), "{name}: the flutes carry the cutting part");
    }
    let plain = model(&shipped("PRB-0400"));
    assert!(plain.fluted.is_none() && plain.fluted_parts.is_none(), "a probe draws no flutes");
}

#[test]
fn a_part_ink_is_shaded_by_the_light() {
    use crate::egui_app::tool_model_view::{PartInks, SHADE_MIN, shade};
    let t = camcraft_ui_kit::theme::Tokens::DARK;
    let inks = PartInks { flute: t.tool_model_flute, shank: t.tool_model_shank, holder: t.tool_model_holder };
    assert_eq!(inks.of(Part::Flute), t.tool_model_flute);
    assert_eq!(inks.of(Part::Shank), t.tool_model_shank);
    assert_eq!(inks.of(Part::Holder), t.tool_model_holder);
    assert_eq!(shade(t.tool_model_flute, 1.0), t.tool_model_flute, "a facet that faces the eye has the ink");
    let dim = shade(t.tool_model_flute, 0.0);
    let want = |c: u8| (f32::from(c) * SHADE_MIN).round() as u8;
    assert_eq!(dim.r(), want(t.tool_model_flute.r()), "an edge-on facet is darker");
    assert_eq!(shade(t.tool_model_holder, f64::NAN), shade(t.tool_model_holder, 0.3), "a NaN light reads as the plain view's");
}
