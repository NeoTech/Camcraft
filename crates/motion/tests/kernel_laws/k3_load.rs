//! #4026 (K3 of the path kernel): the kernel plan of K4 round the disc of
//! #3934 and the dog bone of #3741, Inside and Outside, with the kernel's
//! load stage at a constant engagement of 25 % (the planned 60 degrees of
//! the old tables), read with the law measures of #4154. The readings are
//! pinned as measured, against the kernel plan without load control and
//! the old planners' rows (`laws_pockets.rs`). The path is the cuts of
//! the plan without the entry leads (`cuts_path_with`, #4269).

use super::common::law_measure::measure;
use super::fixture::{R, STEP, parts, planned_deg, spec_round, stock};
use super::k4_order_link::cuts_path_with;
use camcraft_motion::geo2d::{Point2, Region};
use camcraft_pathkernel::load::{LoadControl, LoadStage, Reach, StockRead};
use camcraft_pathkernel::order::StartSide;

/// The material that the tool can cut: the stock box less the part.
struct Left(Region, Region);

impl StockRead for Left {
    fn material_at(&self, p: Point2) -> Option<bool> {
        Some(self.0.contains(p) && !self.1.contains(p))
    }
}

/// For each part and side, without load control (`None`: the K4 plan,
/// each cut above the limit named) and with constant engagement:
/// [feed moves above the limit, the worst engagement in degrees, the
/// moves above the limit with no named place within one tool radius].
/// Measured; the old 2D Pocket rows are over 20 worst 178 (disc) and over
/// 32 worst 180 (dog bone). The worst Inside is the first ring round the
/// part: a full-width cut with no pass beside it, named (`FRESH_CUT`). The
/// unnamed moves of Outside are on the stock edge line, where a piece
/// starts: there the kernel's read and the meter of #4154 differ
/// (follow-up issue named in the decision file). Since #4254 a link comes
/// in to such a start from the air, at depth: the approach is a stretch of
/// a link, and the load stage reads the cuts only, so its moves above the
/// limit are not named yet (#4272; before #4254 the Outside rows were
/// [402, 145, 13], [334, 118, 74] on the disc and [724, 167, 3],
/// [678, 143, 66] on the dog bone; with #4253 the disc row with constant
/// engagement read [337, 118, 77], #4269). The path is the cuts-only path
/// of the plan (`cuts_path_with`): the approach stretches are its feeds.
const READ: [(&str, &str, [u32; 3], [u32; 3]); 4] = [
    ("disc", "Inside", [3507, 173, 0], [833, 173, 1]),
    ("disc", "Outside", [415, 145, 19], [344, 122, 81]),
    ("dog bone", "Inside", [2587, 173, 0], [581, 173, 1]),
    ("dog bone", "Outside", [739, 180, 13], [683, 143, 69]),
];

#[test]
fn kernel_load_control_round_the_disc_and_the_dog_bone() {
    let load = STEP / (2.0 * R);
    let stage = LoadStage::new(load).unwrap();
    let ce = LoadControl::constant(load, Reach::Rings).unwrap();
    let mut got = Vec::new();
    for (pname, part) in parts() {
        let reader = Left(stock(), part.clone());
        for (sname, side) in [("Inside", StartSide::Inside), ("Outside", StartSide::Outside)] {
            let spec = spec_round(&part, planned_deg(STEP, R));
            let mut rows = Vec::new();
            for lc in [LoadControl::None, ce] {
                let (path, _, v) = cuts_path_with(&part, side, Some((&stage, lc, &reader)));
                let r = measure(&path, &spec).unwrap();
                let ctl = v.as_ref().and_then(|v| v.out()).unwrap();
                eprintln!("{pname} {sname} {lc:?}: {}", r.row());
                eprintln!("  {} curves, {} named places; notes {:?}", ctl.set().curves().len(), ctl.over().len(), path.notes);
                assert_eq!(r.crossings.count, 0, "{pname} {sname}: no feed crosses another");
                assert_eq!(r.leaves.count, 0, "{pname} {sname}: each feed stays in the reach");
                assert_eq!(r.named_lifts, r.lifts.count, "{pname} {sname}: each lift has a named reason");
                let named = |q: &&Point2| ctl.over().iter().any(|o| (o.at().x - q.x).hypot(o.at().y - q.y) <= R);
                let unnamed: Vec<&Point2> = r.over.places.iter().filter(|q| !named(q)).collect();
                eprintln!("  unnamed over moves at {:?}", unnamed.iter().take(6).collect::<Vec<_>>());
                rows.push([r.over.count as u32, r.worst_deg.round() as u32, unnamed.len() as u32]);
            }
            got.push((pname, sname, rows[0], rows[1]));
        }
    }
    assert_eq!(got, READ.to_vec(), "the load readings of the kernel");
}
