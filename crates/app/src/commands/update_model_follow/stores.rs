//! Every pick an operation stores (#3915, decision D7), taken from the
//! project files that save picks:
//!
//! | Store | Saved by |
//! |---|---|
//! | `Operation::src`, `SrcRef::Selection` | `project/file.rs` (`SrcDto::Selection`) |
//! | `AreaControls::extra`, `BoundaryShape::Picks` | `project/area.rs` |
//! | `AreaControls::open`, `OpenBoundary::picks` | `project/area.rs` |
//! | `AreaControls::do_not_touch`, `DoNotTouch::picks` | `project/area.rs` |
//! | `OpParams::Deburr::exclude_edges` | `project/params.rs`, `params_impl.rs` |
//! | `OpParams::MultiAxisDeburr::exclude_edges` | `project/multiaxis_deburr_dto.rs` |
//!
//! The per-kind stores of the sinker, the probe and the ruled wire hold
//! coordinates (`PathPicks`, targets), not feature ids; heights read the
//! selection and hold no picks of their own. `Operation` and
//! `AreaControls` are taken apart field by field, with no `..`, so a new
//! field there fails to build here until someone says whether it holds
//! picks.
//! Provenance: our own walk over our own types.

use crate::operation::{
    AreaControls, BoundaryShape, DoNotTouch, FeatureSel, GeoSel, OpParams, OpenBoundary, SrcRef,
};
use crate::schema::Operation;

/// Visit every pick stored inside `op`.
pub(super) fn for_each_pick(op: &Operation, mut f: impl FnMut(&GeoSel)) {
    let Operation {
        tool: _,
        src,
        params,
        stamp: _,
        setup: _,
        area,
        coolant: _,
        after: _,
        suppressed: _,
    } = op;
    if let SrcRef::Selection(items) = src {
        items.iter().for_each(&mut f);
    }
    let AreaControls {
        containment: _,
        offset: _,
        outer_offset: _,
        corner: _,
        extension: _,
        extra,
        open,
        do_not_touch,
    } = area;
    for e in extra {
        if let BoundaryShape::Picks(items) = &e.shape {
            items.iter().for_each(&mut f);
        }
    }
    if let Some(OpenBoundary { picks, side: _ }) = open {
        picks.iter().for_each(&mut f);
    }
    let DoNotTouch {
        clearance: _,
        picks,
    } = do_not_touch;
    picks.iter().for_each(&mut f);
    if let OpParams::Deburr { exclude_edges, .. }
    | OpParams::MultiAxisDeburr { exclude_edges, .. } = params
    {
        exclude_edges.iter().for_each(&mut f);
    }
}

/// Visit every pick stored inside `op`, to change it in place.
pub(super) fn for_each_pick_mut(op: &mut Operation, mut f: impl FnMut(&mut GeoSel)) {
    let Operation {
        tool: _,
        src,
        params,
        stamp: _,
        setup: _,
        area,
        coolant: _,
        after: _,
        suppressed: _,
    } = op;
    if let SrcRef::Selection(items) = src {
        items.iter_mut().for_each(&mut f);
    }
    let AreaControls {
        containment: _,
        offset: _,
        outer_offset: _,
        corner: _,
        extension: _,
        extra,
        open,
        do_not_touch,
    } = area;
    for e in extra.iter_mut() {
        if let BoundaryShape::Picks(items) = &mut e.shape {
            items.iter_mut().for_each(&mut f);
        }
    }
    if let Some(OpenBoundary { picks, side: _ }) = open {
        picks.iter_mut().for_each(&mut f);
    }
    let DoNotTouch {
        clearance: _,
        picks,
    } = do_not_touch;
    picks.iter_mut().for_each(&mut f);
    if let OpParams::Deburr { exclude_edges, .. }
    | OpParams::MultiAxisDeburr { exclude_edges, .. } = params
    {
        exclude_edges.iter_mut().for_each(&mut f);
    }
}

/// `op` holds a `Face` or `Patch` pick on the model named `model`.
pub(super) fn holds_face_pick(op: &Operation, model: &str) -> bool {
    let mut found = false;
    for_each_pick(op, |g| {
        found |=
            g.entity == model && matches!(g.feature, FeatureSel::Face(_) | FeatureSel::Patch(_));
    });
    found
}

/// The kinds whose pick writes numbers into the parameters when it is
/// made (`SetOperationGeometry`: `probe_from_picks`, `slot_from_picks`).
/// Generation keeps those numbers: a Probe never reads its pick again,
/// and a Slot only checks that its typed slot still fits the picked room.
/// So a pick of theirs that followed would keep the old face's numbers
/// against the moved face. Their picks never follow; they are reported
/// lost, and a new pick seeds the numbers again.
pub(super) fn seeds_from_picks(op: &Operation) -> bool {
    matches!(op.params, OpParams::Probe { .. } | OpParams::Slot { .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::{
        BoundaryMode, Carried, ChainSide, ExtraBoundary, Height, KindInfo, OpType,
    };

    fn pick(id: u64) -> GeoSel {
        GeoSel {
            entity: "part".into(),
            feature: FeatureSel::Face(id),
        }
    }

    fn op_of(kind: OpType) -> Operation {
        let carried = Carried {
            feed: 100.0,
            plunge: 50.0,
            climb: true,
            clearance: Height::fixed(30.0),
            stepover: 2.0,
            stepdown: 2.0,
            top: Height::fixed(10.0),
            bottom: Height::fixed(0.0),
            retract: Height::fixed(25.0),
            tilt: None,
        };
        let params = KindInfo::ALL
            .get(kind.ordinal())
            .map(|k| (k.seed)(&carried))
            .expect("a catalog row");
        Operation {
            tool: "t".into(),
            src: SrcRef::Selection(vec![pick(1)]),
            params,
            stamp: None,
            setup: None,
            area: AreaControls {
                extra: vec![ExtraBoundary {
                    mode: BoundaryMode::Include,
                    shape: BoundaryShape::Picks(vec![pick(2)]),
                }],
                open: Some(OpenBoundary {
                    picks: vec![pick(3)],
                    side: ChainSide::Inside,
                }),
                do_not_touch: DoNotTouch {
                    clearance: 0.0,
                    picks: vec![pick(4)],
                },
                ..AreaControls::default()
            },
            coolant: Default::default(),
            after: Default::default(),
            suppressed: false,
        }
    }

    fn with_excluded(mut op: Operation, id: u64) -> Operation {
        if let OpParams::Deburr { exclude_edges, .. }
        | OpParams::MultiAxisDeburr { exclude_edges, .. } = &mut op.params
        {
            exclude_edges.push(pick(id));
        }
        op
    }

    fn ids(op: &Operation) -> Vec<u64> {
        let mut out = Vec::new();
        for_each_pick(op, |g| out.push(g.feature.id()));
        out
    }

    /// The walk reaches every store of the inventory: the four the area
    /// and the source hold on any kind, and each deburr's excluded edges.
    #[test]
    fn the_walk_covers_every_store_of_the_inventory() {
        assert_eq!(ids(&op_of(OpType::Pocket)), vec![1, 2, 3, 4]);
        assert_eq!(
            ids(&with_excluded(op_of(OpType::Deburr), 5)),
            vec![1, 2, 3, 4, 5]
        );
        assert_eq!(
            ids(&with_excluded(op_of(OpType::MultiAxisDeburr), 6)),
            vec![1, 2, 3, 4, 6]
        );
    }

    #[test]
    fn the_mutable_walk_changes_every_store_in_place() {
        for kind in [OpType::Pocket, OpType::Deburr, OpType::MultiAxisDeburr] {
            let mut op = with_excluded(op_of(kind), 5);
            let before = ids(&op);
            for_each_pick_mut(&mut op, |g| {
                g.feature = FeatureSel::Face(g.feature.id() + 100)
            });
            let after: Vec<u64> = before.iter().map(|i| i + 100).collect();
            assert_eq!(ids(&op), after, "{kind:?}");
        }
    }

    #[test]
    fn a_probe_and_a_slot_seed_from_their_picks_and_nothing_else_does() {
        assert!(seeds_from_picks(&op_of(OpType::Probe)));
        assert!(seeds_from_picks(&op_of(OpType::Slot)));
        for kind in [
            OpType::Pocket,
            OpType::Deburr,
            OpType::Face,
            OpType::Contour,
        ] {
            assert!(!seeds_from_picks(&op_of(kind)), "{kind:?}");
        }
    }

    #[test]
    fn only_a_face_or_patch_pick_on_the_model_counts() {
        let mut op = op_of(OpType::Pocket);
        assert!(holds_face_pick(&op, "part"));
        assert!(!holds_face_pick(&op, "other"));
        for_each_pick_mut(&mut op, |g| g.feature = FeatureSel::Chain(g.feature.id()));
        assert!(
            !holds_face_pick(&op, "part"),
            "chains do not follow in this change"
        );
    }
}
