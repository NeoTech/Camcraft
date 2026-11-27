//! #4269 (K6a of the path kernel): the verifier of `camcraft-sim` and the
//! entry measure on the kernel plans of the disc and the dog bone, made
//! into toolpaths by the real converter (`camcraft_motion::kernel_emit`).
//! Pins: no rapid in stock or in the part, no plunge into stock, no gouge
//! of the part, and each descent below the stock top is the descent of a
//! plan entry (no plunge without an entry), started in air or going into
//! material only by the helix or ramp that the entry stage proved.

use super::common::law_measure::{EntryKind, EntryStart, measure};
use super::fixture::{R, STEP, TOP, parts, planned_deg, spec_round, stock};
use super::k4_order_link::{CLEARANCE, convert, kernel_plan};
use camcraft_motion::verify::{Mode, ToolGeom, VerifyContext, Warning, verify};
use camcraft_pathkernel::order::StartSide;
use camcraft_pathkernel::plan::Step;

#[test]
fn the_verifier_finds_no_rapid_into_stock_and_no_plunge_without_an_entry() {
    let tool = ToolGeom { radius: R, nose_radius: 0.0, flute_len: 30.0, shank_radius: R, shank_len: 10.0, holder_radius: 15.0, holder_len: 30.0 };
    let st = stock();
    for (pname, part) in parts() {
        for (sname, side) in [("Inside", StartSide::Inside), ("Outside", StartSide::Outside)] {
            let (plan, _) = kernel_plan(&part, side);
            let path = convert(&plan).path;
            // The Outside start enters in the air round the stock, with its arc
            // lead up to 1.5 tool radii out: the verifier's area is the stock
            // grown by four tool radii, less the part, so a gouge is a gouge
            // of the part (with the band of `fixture::band`, 2 radii, the air
            // leads read as gouges 17 mm or more from the part).
            let area = st.offset(4.0 * R).unwrap().difference(&part).unwrap();
            let cx = VerifyContext {
                region: &area,
                stock: Some(&st),
                tool: &tool,
                mode: Mode::Area,
                stock_top_z: TOP,
                clearance_z: CLEARANCE,
                gouge_tol: 0.01,
                z_tol: 0.05,
                ..VerifyContext::default()
            };
            let warns = verify(&path, &cx);
            let gouges = warns.iter().filter(|w| matches!(w, Warning::Gouge { .. })).count();
            let other: Vec<&Warning> = warns.iter().filter(|w| !matches!(w, Warning::Gouge { .. })).collect();
            eprintln!("{pname} {sname}: {} moves, verifier: {gouges} gouge(s), other {other:?}", path.moves.len());
            let bad: Vec<&Warning> = warns
                .iter()
                .filter(|w| matches!(w, Warning::RapidInStock { .. } | Warning::RapidInPart { .. } | Warning::PlungeIntoStock { .. }))
                .collect();
            assert!(bad.is_empty(), "{pname} {sname}: the verifier finds a rapid in stock or a plunge into stock: {bad:?}");
            assert_eq!(gouges, 0, "{pname} {sname}: the verifier finds no gouge of the part");
            let r = measure(&path, &spec_round(&part, planned_deg(STEP, R))).unwrap();
            let planned = plan.steps().iter().filter(|s| matches!(s, Step::Entry(_))).count();
            eprintln!("  entries {:?}", r.entries);
            assert_eq!(r.entries.len(), planned, "{pname} {sname}: each descent is a plan entry");
            for e in &r.entries {
                let proved = e.start != EntryStart::Solid || matches!(e.kind, EntryKind::Helix | EntryKind::Ramp);
                assert!(proved, "{pname} {sname}: a plunge into solid stock without a proved entry: {e:?}");
            }
        }
    }
}
