//! #3915: face picks follow their faces by SolidCraft name when a `.sc`
//! model imports again - in the update's own undo step, with a review
//! line on each operation that a follow touched, and each pick that could
//! not follow named in the re-import report and refused at generate as
//! before. The matching rules themselves are pinned in
//! `camcraft_geometry::source_follow`'s tests.
//! Provenance: our own fixtures (`source_names`).

use camcraft_app::commands::{CreateOperation, GenerateOperation, History, MarkOperationReviewed};
use camcraft_app::operation::{BoundaryMode, ExtraBoundary};
use camcraft_app::posting;

use super::area_support::{
    AreaControls, BoundaryShape, ChainSide, OpType, OpenBoundary, TOOL, op, operation, pocket, run,
    sel,
};
use super::source_names::*;

const OP: &str = "op1";

/// The block with its record, and one pocket whose source picks cube
/// face `k`.
fn picked(k: usize) -> (flecs_ecs::prelude::World, History) {
    let world = part_world(Some(plain(&block(40.0))));
    land_op(
        &world,
        OP,
        op(
            sel(PART, face(&world, k)),
            pocket(10.0, 5.0),
            AreaControls::default(),
        ),
    );
    (world, History::default())
}

#[test]
fn a_face_pick_follows_its_name_in_the_same_undo_step() {
    let (world, mut h) = picked(FRONT);
    let before = face(&world, FRONT);
    let (report, summary) = reimport(
        &world,
        &mut h,
        block(50.0),
        Some(reversed(&block(50.0), None)),
    );
    let after = face(&world, FRONT);
    assert_ne!(before, after, "the width change moved the front face's id");
    assert_eq!(src_of(&world, OP), vec![on_part(after.clone())]);
    assert_eq!(
        review_of(&world, OP),
        vec!["face 'front' of 'part' followed by its SolidCraft name after the model was imported again".to_owned()]
    );
    assert_eq!(report.followed.len(), 1);
    assert_eq!(report.followed[0].names, vec!["front".to_owned()]);
    assert!(report.lost.is_empty(), "{:?}", report.lost);
    assert!(
        summary.contains("1 pick followed by SolidCraft name"),
        "{summary}"
    );

    assert!(h.undo(&world));
    assert_eq!(
        src_of(&world, OP),
        vec![on_part(before)],
        "undo puts the old id back"
    );
    assert!(
        review_of(&world, OP).is_empty(),
        "and takes the review line away"
    );
    assert!(h.redo(&world));
    assert_eq!(
        src_of(&world, OP),
        vec![on_part(after)],
        "redo follows again"
    );
    assert_eq!(review_of(&world, OP).len(), 1);
}

#[test]
fn every_pick_store_of_an_operation_follows() {
    let world = part_world(Some(plain(&block(40.0))));
    let area = AreaControls {
        extra: vec![ExtraBoundary {
            mode: BoundaryMode::Exclude,
            shape: BoundaryShape::Picks(vec![on_part(face(&world, BACK))]),
        }],
        open: Some(OpenBoundary {
            picks: vec![on_part(face(&world, RIGHT))],
            side: ChainSide::Inside,
        }),
        do_not_touch: camcraft_app::operation::DoNotTouch {
            clearance: 0.0,
            picks: vec![on_part(face(&world, FRONT))],
        },
        ..AreaControls::default()
    };
    land_op(
        &world,
        OP,
        op(sel(PART, face(&world, TOP)), pocket(10.0, 5.0), area),
    );
    let mut h = History::default();
    let (report, _) = reimport(
        &world,
        &mut h,
        block(50.0),
        Some(reversed(&block(50.0), None)),
    );
    assert_eq!(report.followed.len(), 4, "{report:?}");
    let o = op_of(&world, OP);
    assert_eq!(src_of(&world, OP), vec![on_part(face(&world, TOP))]);
    assert!(
        matches!(&o.area.extra[0].shape, BoundaryShape::Picks(p) if *p == vec![on_part(face(&world, BACK))])
    );
    assert_eq!(
        o.area.open.as_ref().map(|b| b.picks.clone()),
        Some(vec![on_part(face(&world, RIGHT))])
    );
    assert_eq!(
        o.area.do_not_touch.picks,
        vec![on_part(face(&world, FRONT))]
    );
    let review = review_of(&world, OP);
    assert_eq!(review.len(), 1, "one line for the operation: {review:?}");
    assert!(
        review[0].starts_with("4 picks of 'part' followed by their SolidCraft names"),
        "{}",
        review[0]
    );
    for n in ["top", "back", "right", "front"] {
        assert!(
            review[0].contains(&format!("face '{n}'")),
            "{n}: {}",
            review[0]
        );
    }
}

#[test]
fn a_pick_that_cannot_follow_stays_and_is_named_in_the_report() {
    let mut unverified = names(&CUBE);
    unverified[TOP].verified = false;
    let wide = block(50.0);
    let cases = [
        (
            "renamed",
            plain(&block(40.0)),
            reversed(&wide, Some((TOP, name("lid")))),
            "no face has these names now",
        ),
        (
            "unverified",
            record(&block(40.0), unverified, [0, 1, 2, 3, 4, 5]),
            reversed(&wide, None),
            "did not verify",
        ),
        (
            "two candidates",
            plain(&block(40.0)),
            record(
                &wide,
                names(&["top", "front", "back", "right", "left"]),
                [0, 0, 1, 2, 3, 4],
            ),
            "2 faces have these names now",
        ),
    ];
    for (case, old, new, why) in cases {
        let world = part_world(Some(old));
        let before = face(&world, TOP);
        land_op(
            &world,
            OP,
            op(
                sel(PART, before.clone()),
                pocket(10.0, 5.0),
                AreaControls::default(),
            ),
        );
        let mut h = History::default();
        let (report, summary) = reimport(&world, &mut h, block(50.0), Some(new));
        assert_eq!(
            src_of(&world, OP),
            vec![on_part(before)],
            "{case}: the pick stays as it was"
        );
        assert!(
            review_of(&world, OP).is_empty(),
            "{case}: nothing followed, nothing to review"
        );
        assert!(report.followed.is_empty(), "{case}");
        assert_eq!(report.lost.len(), 1, "{case}");
        assert_eq!(report.lost[0].names, vec!["top".to_owned()], "{case}");
        assert_eq!(report.lost[0].operation, OP, "{case}");
        assert!(
            report.lost[0]
                .why
                .as_deref()
                .is_some_and(|w| w.contains(why)),
            "{case}: {:?}",
            report.lost[0].why
        );
        assert!(
            summary.contains("1 pick could not follow: face 'top' in 'op1'"),
            "{case}: {summary}"
        );
        let j = report.to_json().expect("the import answer's picks");
        assert_eq!(j["lost"][0]["operation"], OP, "{case}");
        assert_eq!(j["lost"][0]["names"][0], "top", "{case}");
    }
}

#[test]
fn a_carried_pick_a_model_without_a_record_and_a_plain_update_behave_as_today() {
    // A carried face: the x = -20 wall keeps its id, and its pick.
    let (world, mut h) = picked(LEFT);
    let left = face(&world, LEFT);
    let (report, _) = reimport(
        &world,
        &mut h,
        block(50.0),
        Some(reversed(&block(50.0), None)),
    );
    assert_eq!(src_of(&world, OP), vec![on_part(left)]);
    assert!(report.is_empty() && review_of(&world, OP).is_empty());

    // No record on the old model: nothing follows, the phrase is today's.
    let world = part_world(None);
    let front = face(&world, FRONT);
    land_op(
        &world,
        OP,
        op(
            sel(PART, front.clone()),
            pocket(10.0, 5.0),
            AreaControls::default(),
        ),
    );
    let mut h = History::default();
    let (report, summary) = reimport(
        &world,
        &mut h,
        block(50.0),
        Some(reversed(&block(50.0), None)),
    );
    assert_eq!(src_of(&world, OP), vec![on_part(front)]);
    assert!(report.is_empty() && review_of(&world, OP).is_empty());
    assert!(!summary.contains("pick"), "{summary}");

    // An update with no record (an STL, a STEP): nothing follows.
    let (world, mut h) = picked(FRONT);
    let front = face(&world, FRONT);
    let (report, _) = reimport(&world, &mut h, block(50.0), None);
    assert_eq!(src_of(&world, OP), vec![on_part(front)]);
    assert!(report.is_empty() && review_of(&world, OP).is_empty());
}

#[test]
fn a_probe_or_slot_pick_set_its_numbers_and_does_not_follow() {
    for kind in [OpType::Probe, OpType::Slot] {
        let world = part_world(Some(plain(&block(40.0))));
        let top = face(&world, TOP);
        let params = operation::retype_params(&pocket(10.0, 5.0), kind as usize);
        land_op(
            &world,
            OP,
            op(sel(PART, top.clone()), params, AreaControls::default()),
        );
        let mut h = History::default();
        let (report, _) = reimport(
            &world,
            &mut h,
            block(50.0),
            Some(reversed(&block(50.0), None)),
        );
        assert_eq!(
            src_of(&world, OP),
            vec![on_part(top)],
            "{kind:?}: the pick stays"
        );
        assert!(review_of(&world, OP).is_empty(), "{kind:?}");
        assert!(report.followed.is_empty(), "{kind:?}");
        assert_eq!(report.lost.len(), 1, "{kind:?}");
        let why = report.lost[0].why.as_deref().unwrap_or("");
        assert!(
            why.contains("set the operation's numbers"),
            "{kind:?}: {why}"
        );
    }
}

#[test]
fn a_do_not_touch_pick_that_cannot_follow_is_refused_at_generate() {
    let world = part_world(Some(plain(&block(40.0))));
    let area = AreaControls {
        do_not_touch: camcraft_app::operation::DoNotTouch {
            clearance: 0.0,
            picks: vec![on_part(face(&world, BACK))],
        },
        ..AreaControls::default()
    };
    let back = face(&world, BACK);
    land_op(
        &world,
        OP,
        op(sel(PART, face(&world, TOP)), pocket(10.0, 5.0), area),
    );
    let mut h = History::default();
    let (report, _) = reimport(
        &world,
        &mut h,
        block(50.0),
        Some(reversed(&block(50.0), Some((BACK, name("back2"))))),
    );
    assert_eq!(
        src_of(&world, OP),
        vec![on_part(face(&world, TOP))],
        "the source pick followed"
    );
    let o = op_of(&world, OP);
    assert_eq!(
        o.area.do_not_touch.picks,
        vec![on_part(back)],
        "the protected pick did not"
    );
    assert_eq!(report.lost.len(), 1);
    assert_eq!(report.lost[0].names, vec!["back".to_owned()]);
    let why = run(&world, &o).expect_err("a protected face that is gone refuses");
    assert!(
        why.contains("Do not touch") && why.contains("no longer in 'part'"),
        "{why}"
    );
}

#[test]
fn a_followed_pick_holds_the_post_until_the_operation_is_marked_reviewed() {
    let world = part_world(Some(plain(&block(40.0))));
    let mut h = History::default();
    // A facing pass on the top face at its own height: the area suite's
    // law for a top-face pick ("the facing stays on the picked face").
    // A 2D Pocket would read the picked face as its floor.
    let create = CreateOperation::new(
        &world,
        OP,
        TOOL,
        sel(PART, face(&world, TOP)),
        super::area_support::face(10.0),
    )
    .expect("a facing pass on the top");
    h.execute(&world, create);
    let needs_review = |w: &flecs_ecs::prelude::World| {
        posting::verification_gate(w)
            .into_iter()
            .any(|p| p.contains(OP) && p.contains("needs review before posting"))
    };
    assert!(!needs_review(&world), "a fresh operation needs no review");
    let (report, _) = reimport(
        &world,
        &mut h,
        block(50.0),
        Some(reversed(&block(50.0), None)),
    );
    assert_eq!(report.followed.len(), 1);
    h.execute(
        &world,
        GenerateOperation::new(&world, OP).expect("the followed pick generates"),
    );
    assert!(
        needs_review(&world),
        "the follow is made visible: the post waits for a person"
    );
    h.execute(
        &world,
        MarkOperationReviewed::new(&world, OP).expect("mark reviewed"),
    );
    assert!(
        !needs_review(&world),
        "marked reviewed, the review clause is gone"
    );
    assert!(h.undo(&world));
    assert!(
        needs_review(&world),
        "undo of the mark brings the line back"
    );
}
