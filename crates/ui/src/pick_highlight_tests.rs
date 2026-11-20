//! Pins for the input key of `pick_highlight` (#2201): a still frame
//! builds nothing, and each changed input (pose, facets, display-off,
//! the pick itself) builds again. crates/ui/AGENTS.md, "Frame cost".
//! Provenance: our own unit tests; no vendor material consulted.

use std::sync::Arc;

use camcraft_app::geometry::{Point3, cube};
use camcraft_app::model;
use camcraft_app::operation::{FeatureSel, GeoSel};
use camcraft_app::schema::Location;

use super::{hover_builds, resolved_hover, resolved_row_hover, row_hover_builds};
use crate::state::State;

/// A state with one cube model "Part" and its first face row-hovered.
fn hovered_cube() -> State {
    let mut st = State::new();
    let part = model::spawn(&st.world, "Part");
    model::set_facets(&st.world, &part, cube(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 10.0, 10.0)).facets);
    let features = model::features_of(&st.world, &part).expect("cube features");
    let fid = features.faces.first().expect("a face").id;
    st.geo_hover_drawn = Some(GeoSel { entity: "Part".into(), feature: FeatureSel::Face(fid) });
    st
}

fn max_x(r: &super::ResolvedHighlights) -> f64 {
    r.polylines.iter().flat_map(|p| p.pts.iter()).map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max)
}

#[test]
fn highlight_still_frames_build_the_outline_once() {
    let st = hovered_cube();
    let first = resolved_row_hover(&st);
    assert!(!first.polylines.is_empty(), "the hovered face draws an outline");
    for _ in 0..16 {
        let again = resolved_row_hover(&st);
        assert!(Arc::ptr_eq(&first, &again), "a still frame reads the cached outline");
    }
    assert_eq!(row_hover_builds(&st), 1, "a still frame builds nothing");
    assert_eq!(hover_builds(&st), 0, "the row hover does not count as a viewport hover");
}

/// The viewport hover slot (`state.hover`) has its own pin: a still
/// frame builds once, a new hovered feature builds again, and the
/// row-hover count does not move.
#[test]
fn viewport_hover_still_frames_build_once_and_a_new_feature_builds_again() {
    let mut st = hovered_cube();
    st.geo_hover_drawn = None;
    let part = model::find_by_name(&st.world, "Part").expect("Part");
    let features = model::features_of(&st.world, &part).expect("cube features");
    let first = features.faces.first().expect("a face").id;
    let second = features.faces.get(1).expect("a second face").id;
    st.hover = Some(crate::state::HoverHit { entity: "Part".into(), facet: 0, feature: Some(FeatureSel::Face(first)) });
    let a = resolved_hover(&st);
    assert!(!a.polylines.is_empty(), "the hovered face draws an outline");
    for _ in 0..16 {
        assert!(Arc::ptr_eq(&a, &resolved_hover(&st)), "a still frame reads the cached outline");
    }
    assert_eq!(hover_builds(&st), 1, "a still frame builds nothing");
    st.hover = Some(crate::state::HoverHit { entity: "Part".into(), facet: 1, feature: Some(FeatureSel::Face(second)) });
    let b = resolved_hover(&st);
    assert_eq!(hover_builds(&st), 2, "a new hovered feature builds again");
    assert!(!Arc::ptr_eq(&a, &b));
    assert_eq!(row_hover_builds(&st), 0, "the viewport hover does not count as a row hover");
}

#[test]
fn highlight_rebuilds_when_the_pose_moves() {
    let st = hovered_cube();
    let before = resolved_row_hover(&st);
    let part = model::find_by_name(&st.world, "Part").expect("Part");
    let loc = Location { pos: Point3::new(50.0, 0.0, 0.0), ..Location::default() };
    model::set_transform(&st.world, &part, loc);
    let after = resolved_row_hover(&st);
    assert_eq!(row_hover_builds(&st), 2, "a moved body builds again");
    assert!((max_x(&after) - max_x(&before) - 50.0).abs() < 1e-9, "the outline moves with the body");
}

#[test]
fn highlight_rebuilds_when_the_facets_are_replaced() {
    let st = hovered_cube();
    let before = resolved_row_hover(&st);
    assert!(!before.polylines.is_empty());
    let part = model::find_by_name(&st.world, "Part").expect("Part");
    model::set_facets(&st.world, &part, cube(Point3::new(0.0, 0.0, 0.0), Point3::new(30.0, 7.0, 3.0)).facets);
    let after = resolved_row_hover(&st);
    assert_eq!(row_hover_builds(&st), 2, "a re-meshed body builds again");
    assert!(!Arc::ptr_eq(&before, &after), "the old outline is never served for the new mesh");
}

#[test]
fn highlight_rebuilds_when_the_body_is_hidden_or_the_pick_changes() {
    let mut st = hovered_cube();
    assert!(!resolved_row_hover(&st).polylines.is_empty());
    st.hidden_models.push("Part".into());
    assert!(resolved_row_hover(&st).polylines.is_empty(), "a hidden body draws nothing");
    assert_eq!(row_hover_builds(&st), 2);
    st.hidden_models.clear();
    let part = model::find_by_name(&st.world, "Part").expect("Part");
    let features = model::features_of(&st.world, &part).expect("cube features");
    let other = features.faces.get(1).expect("a second face").id;
    st.geo_hover_drawn = Some(GeoSel { entity: "Part".into(), feature: FeatureSel::Face(other) });
    assert!(!resolved_row_hover(&st).polylines.is_empty());
    assert_eq!(row_hover_builds(&st), 3, "a new pick builds again");
}
