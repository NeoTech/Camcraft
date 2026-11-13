//! #1543: the depth law a Circular takes per source, unit by unit - a
//! whole-model (finished part) source is judged (`Section`), a picked
//! Circular keeps its earlier law, and a source that is not the part
//! (the Stock, `part_solid` false) is never judged. Our own fixtures.

use super::generate::{authored_law, depth_law_of, DepthLaw};
use super::{seed_params, FeatureSel, GeoSel, OpType, SrcRef};

fn circular() -> super::OpParams {
    seed_params(OpType::Circular.ordinal())
}

fn picked() -> SrcRef {
    SrcRef::Selection(vec![GeoSel { entity: "part".into(), feature: FeatureSel::Chain(7) }])
}

#[test]
fn a_whole_model_circular_is_judged_against_the_part() {
    assert!(matches!(circular(), super::OpParams::Circular { .. }), "the seed is a Circular");
    assert_eq!(depth_law_of(&circular(), &SrcRef::Entity("part".into()), true), DepthLaw::Section);
}

#[test]
fn a_picked_circular_keeps_its_law() {
    assert_eq!(depth_law_of(&circular(), &picked(), true), DepthLaw::None);
}

#[test]
fn a_source_that_is_not_the_part_is_never_judged() {
    assert_eq!(depth_law_of(&circular(), &SrcRef::Stock, false), DepthLaw::None);
    assert_eq!(depth_law_of(&circular(), &SrcRef::Entity("stock".into()), false), DepthLaw::None);
}

#[test]
fn the_shared_authored_law_reads_the_source_alone() {
    // The one law #1537's Slot shares: only a whole model is judged.
    assert_eq!(authored_law(&SrcRef::Entity("part".into())), DepthLaw::Section);
    assert_eq!(authored_law(&picked()), DepthLaw::None);
    assert_eq!(authored_law(&SrcRef::Stock), DepthLaw::None);
}
