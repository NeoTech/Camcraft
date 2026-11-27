//! #3915: the name follow reads its own facet order. The features caches
//! key a mesh by an order-free content digest, so a mesh with the same
//! triangles in another order, seen earlier on this thread, can give its
//! facet indexes to the new mesh. The record names facets by index, so a
//! follow that read those features would name the wrong faces.
//! Provenance: our own fixtures (`source_names`).

use camcraft_app::commands::History;
use camcraft_app::geometry::feature;
use camcraft_app::model;
use camcraft_app::operation::FeatureSel;

use super::area_support::{AreaControls, op, pocket, sel};
use super::source_names::*;

const OP: &str = "op1";

/// The id of cube face `k` of `width`, read from the facets themselves
/// (never through a cache, never through a record).
fn face_of(width: f64, k: usize) -> FeatureSel {
    let f = feature::features_of_facets(&block(width).facets);
    FeatureSel::Face(f.faces[f.facet_face[2 * k] as usize].id)
}

#[test]
fn a_cached_facet_order_of_the_same_triangles_does_not_move_a_pick() {
    let world = part_world(Some(plain(&block(40.0))));
    land_op(
        &world,
        OP,
        op(
            sel(PART, face(&world, FRONT)),
            pocket(10.0, 5.0),
            AreaControls::default(),
        ),
    );
    let mut h = History::default();

    // The new mesh's triangles in the reversed order, seen first: its
    // features enter the cache under the digest that the new mesh shares.
    let mut seen = block(50.0).facets.to_vec();
    seen.reverse();
    let _ = model::features_of_local(&seen);

    let (report, _) = reimport(
        &world,
        &mut h,
        block(50.0),
        Some(reversed(&block(50.0), None)),
    );
    let (front, back) = (face_of(50.0, FRONT), face_of(50.0, BACK));
    assert_ne!(front, back);
    assert_eq!(
        src_of(&world, OP),
        vec![on_part(front)],
        "the front pick follows the front face, not the face whose facets \
         the cached order puts at the front's indexes ({:?})",
        on_part(back)
    );
    assert_eq!(report.followed.len(), 1, "{:?}", report.lost);
    assert_eq!(report.followed[0].names, vec!["front".to_owned()]);
}
