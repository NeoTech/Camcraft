//! #1543 / #1550: the depth law the kinds that cut geometry typed by
//! hand (Circular, Slot) take per source, unit by unit - a finished-part
//! source, whole model or pick, is judged (`Section`, the one
//! `authored_law` both kinds call), and a source that is not the part
//! (the Stock, `part_solid` false) is never judged. Our own fixtures.

use super::generate::{authored_law, depth_law_of, DepthLaw};
use super::{seed_params, FeatureSel, GeoSel, OpType, SrcRef};

fn circular() -> super::OpParams {
    seed_params(OpType::Circular.ordinal())
}

fn slot() -> super::OpParams {
    seed_params(OpType::Slot.ordinal())
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
fn a_picked_circular_is_judged_against_the_part() {
    // #1550: a ring typed below the picked floor gouges the part it
    // was picked off.
    assert_eq!(depth_law_of(&circular(), &picked(), true), DepthLaw::Section);
}

#[test]
fn a_source_that_is_not_the_part_is_never_judged() {
    for p in [circular(), slot()] {
        assert_eq!(depth_law_of(&p, &SrcRef::Stock, false), DepthLaw::None);
        assert_eq!(depth_law_of(&p, &SrcRef::Entity("stock".into()), false), DepthLaw::None);
        assert_eq!(depth_law_of(&p, &picked(), false), DepthLaw::None);
    }
}

#[test]
fn the_slot_answers_to_the_same_authored_law() {
    // #1504 / #1537: picked or whole model, the Slot keeps the part.
    assert!(matches!(slot(), super::OpParams::Slot { .. }), "the seed is a Slot");
    for src in [SrcRef::Entity("part".into()), picked()] {
        assert_eq!(depth_law_of(&slot(), &src, true), authored_law(&src));
        assert_eq!(depth_law_of(&circular(), &src, true), authored_law(&src));
    }
}

#[test]
fn the_shared_authored_law_reads_the_source_alone() {
    // The one law Circular and Slot share: the finished part is judged,
    // picked or whole; the Stock never is.
    assert_eq!(authored_law(&SrcRef::Entity("part".into())), DepthLaw::Section);
    assert_eq!(authored_law(&picked()), DepthLaw::Section);
    assert_eq!(authored_law(&SrcRef::Stock), DepthLaw::None);
}
