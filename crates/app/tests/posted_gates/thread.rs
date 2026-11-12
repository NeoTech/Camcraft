//! The thread-mill golden's gate (#894): `thread_mill_m8_multi_form`
//! regenerated through the real operation - the M8 x 1.25 thread in a
//! blind 96-flat round hole of a Ø36 disc, tap-drilled by a Drill
//! operation ahead (#992: a thread mill is not centre-cutting), the
//! catalog's Ø6 multi-form mill - and judged by every law its post runs:
//! the generator's own blocking warnings (the crest region, the root
//! solid law, #1205's round-wall test, #992's drilled-hole law) and the
//! post's verification gate over the whole program, drill included. The
//! thread is first proved byte-identical to its committed CL golden.
//!
//! Provenance: our own harness over our own golden (the shared fixture
//! of `thread_mill.rs`, `thread_support`); no vendor material consulted.

use crate::common::gen_to_cltext;
use crate::thread_support::{MULTI, R_M8, blocking, catalog_tool, make, thread, world_with};

use camcraft_app::cl::ThreadHand;
use camcraft_app::posting;

/// (golden, every blocking finding) for the thread golden.
pub fn thread_verdicts() -> Vec<(&'static str, Vec<String>)> {
    let (world, mut history) = world_with(R_M8, &[(MULTI, catalog_tool(MULTI))]);
    let path = make(&world, &mut history, MULTI, thread(R_M8, None, ThreadHand::Right, true));
    let name = "thread_mill_m8_multi_form.cl";
    let want = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden").join(name)).expect("golden");
    assert_eq!(gen_to_cltext(&path), want.replace("\r\n", "\n"), "{name}: the regenerated program is the golden");
    let mut w = blocking(&world);
    w.extend(posting::verification_gate(&world));
    vec![(name, w)]
}
