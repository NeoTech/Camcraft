//! #1543 / #1550 / #1593: the depth law the kinds that cut geometry
//! typed by hand (Circular, Slot, Trochoidal slot) take per source, unit by unit - a finished-part
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

fn trochoid() -> super::OpParams {
    seed_params(OpType::TrochoidalSlot.ordinal())
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
    for p in [circular(), slot(), trochoid()] {
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

#[test]
fn the_trochoidal_slot_answers_to_the_same_authored_law() {
    // #1593: its endpoints and width are typed like the Slot's, so a
    // channel through the wall or under the floor gouges the part,
    // picked or whole model.
    assert!(matches!(trochoid(), super::OpParams::TrochoidalSlot { .. }), "the seed is a Trochoidal slot");
    for src in [SrcRef::Entity("part".into()), picked()] {
        assert_eq!(depth_law_of(&trochoid(), &src, true), DepthLaw::Section);
    }
}

#[test]
fn the_kinds_that_enter_or_remove_the_part_by_contract_stay_unjudged() {
    // #1593 audit: the 2D kinds still at `None` on a finished-part
    // source, each by contract (see `depth_law_of`'s doc): Trace,
    // Engrave and 2D Chamfer enter the part by design, and a whole-model
    // pocket family source is the volume to remove. Moving one of them
    // is a decision, not a drift - this table makes it a visible one
    // (#1595 owes Engrave and Trace a law of their own).
    let whole = SrcRef::Entity("part".into());
    for t in [OpType::Trace, OpType::Engrave, OpType::Chamfer2D] {
        let p = seed_params(t.ordinal());
        assert_eq!(depth_law_of(&p, &whole, true), DepthLaw::None, "{t:?} whole");
        assert_eq!(depth_law_of(&p, &picked(), true), DepthLaw::None, "{t:?} picked");
    }
    for t in [OpType::Pocket, OpType::Adaptive2D, OpType::Offset2D] {
        let p = seed_params(t.ordinal());
        assert_eq!(depth_law_of(&p, &whole, true), DepthLaw::None, "{t:?} whole");
        assert_eq!(depth_law_of(&p, &picked(), true), DepthLaw::Section, "{t:?} picked");
    }
}
