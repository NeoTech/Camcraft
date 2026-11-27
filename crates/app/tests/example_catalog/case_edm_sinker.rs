// Provenance: clean-room checks of the catalog case edm/sinker (#2872),
// over CamCraft's own posted program and the invented EDM codes of its
// build script. Pulled into example_catalog.rs with `include!` so that each
// test keeps its name (the case contract test lists
// `catalog_edm_sinker::regenerates` and `catalog_edm_sinker::rebuilds`).

mod catalog_edm_sinker {
    use super::*;
    #[test]
    fn regenerates() {
        run_case("edm", "sinker", "regenerates", regenerate);
    }
    #[test]
    fn rebuilds() {
        run_case("edm", "sinker", "rebuilds", rebuild);
    }
}

/// #2872: the case edm/sinker posts what its README says. The rough Sinker
/// Burn leaves the floor the same allowance as the walls (undersize less
/// the rough gap), so its servo stops at the cavity floor (Z17) plus the
/// undersize. The Sinker Side Burn sinks to the floor plus the finish gap,
/// then moves the electrode out toward +X by the side amount and back.
/// That lateral pair is the side burn; a straight burn has none.
#[test]
fn sinker_case_posts_the_burn_and_the_side_burn() {
    const FLOOR: f64 = 17.0;
    const UNDERSIZE: f64 = 0.25;
    const FINISH_GAP: f64 = 0.03;
    const CENTRE: (f64, f64) = (5.0, 0.0);
    const AMOUNT: f64 = 0.22;
    let near = |a: Option<f64>, b: f64| a.is_some_and(|a| (a - b).abs() < 1e-9);

    let nc_path = repo("examples/catalog/edm/sinker/sinker.nc");
    let nc = fs::read_to_string(&nc_path)
        .unwrap_or_else(|e| panic!("case edm/sinker must hold {}: {e}", nc_path.display()));
    let ops: Vec<&str> = nc.lines().filter(|l| l.starts_with("(OPERATION: ")).collect();
    assert_eq!(
        ops,
        ["(OPERATION: Rough burn - SINKER BURN)", "(OPERATION: Side burn +X - SINKER SIDE BURN)"],
        "the two burns, in order"
    );

    let burn = section(&nc, "Rough burn");
    let conditions: Vec<&&str> = burn.iter().filter(|l| l.starts_with('E')).collect();
    assert_eq!(conditions, [&"ER1"], "the burn selects the rough condition");
    let depth: Vec<&&str> = burn.iter().filter(|l| l.starts_with("M101 ")).collect();
    assert_eq!(depth.len(), 1, "one servo burn to depth: {burn:?}");
    assert!(near(word(depth[0], 'Z'), FLOOR + UNDERSIZE), "rough depth: {}", depth[0]);
    assert!(!burn.iter().any(|l| l.starts_with("M102 ")), "a straight burn has no lateral move: {burn:?}");

    let side = section(&nc, "Side burn +X");
    let conditions: Vec<&&str> = side.iter().filter(|l| l.starts_with('E')).collect();
    assert_eq!(conditions, [&"EF1"], "the side burn selects the finish condition");
    let depth: Vec<&&str> = side.iter().filter(|l| l.starts_with("M101 ")).collect();
    assert_eq!(depth.len(), 1, "one servo burn to depth: {side:?}");
    assert!(near(word(depth[0], 'Z'), FLOOR + FINISH_GAP), "finish depth: {}", depth[0]);
    let lateral: Vec<&&str> = side.iter().filter(|l| l.starts_with("M102 ")).collect();
    assert_eq!(lateral.len(), 2, "out to the wall and back: {side:?}");
    for (block, x) in lateral.iter().zip([CENTRE.0 + AMOUNT, CENTRE.0]) {
        assert!(near(word(block, 'X'), x), "lateral X: {block}");
        assert!(near(word(block, 'Y'), CENTRE.1), "the move keeps Y: {block}");
        assert!(near(word(block, 'Z'), FLOOR + FINISH_GAP), "the move keeps the burn depth: {block}");
    }
}
