// Provenance: original CamCraft 3D rest-machining catalog contract (#2867).
mod catalog_3d_rest {
    use super::*;

    #[test]
    fn regenerates() {
        run_case("3d", "rest", "regenerates", regenerate);
    }

    #[test]
    fn rebuilds() {
        run_case("3d", "rest", "rebuilds", rebuild);
    }

    /// The feed moves of `op` in the committed program, as end points
    /// with modal X, Y and Z.
    fn feed_points(op: &str) -> Vec<(f64, f64, f64)> {
        let nc = fs::read_to_string(repo("examples/catalog/3d/rest/rest.nc")).expect("read rest.nc");
        let (mut x, mut y, mut z) = (0.0, 0.0, f64::INFINITY);
        let mut out = Vec::new();
        for line in section(&nc, op) {
            x = word(line, 'X').unwrap_or(x);
            y = word(line, 'Y').unwrap_or(y);
            z = word(line, 'Z').unwrap_or(z);
            if matches!(line.split_whitespace().next(), Some("G01" | "G02" | "G03")) {
                out.push((x, y, z));
            }
        }
        out
    }

    /// #2867: the 12 mm pocket roughs down to the plate (Z4 + 0.2 mm stock
    /// to leave), and the 6 mm pocket after it cuts only the rest region:
    /// the 8 mm channel between the ribs (X -4..4, Y -15..15). The 6 mm
    /// tool enters at the two mouths through space that the 12 mm tool
    /// cleared, so its centre goes to |Y| <= 18, but it cuts only in the
    /// channel. Between the ribs the 6 mm centre keeps 3 mm + 0.2 mm off
    /// each wall (|X| <= 0.8).
    #[test]
    fn the_small_tool_cuts_only_the_rest_region() {
        let big = feed_points("3D Pocket D12");
        assert!(!big.is_empty(), "3D Pocket D12 section present");
        let big_low = big.iter().map(|p| p.2).fold(f64::INFINITY, f64::min);
        assert!((big_low - 4.2).abs() < 1e-6, "the 12 mm pocket roughs to Z4.2, lowest Z{big_low}");

        let rest = feed_points("3D Pocket D6 rest");
        assert!(!rest.is_empty(), "3D Pocket D6 rest section present");
        let low = rest.iter().map(|p| p.2).fold(f64::INFINITY, f64::min);
        assert!((low - 4.2).abs() < 1e-6, "the 6 mm pocket clears the channel to Z4.2, lowest Z{low}");
        let outside: Vec<_> = rest.iter().filter(|(x, y, _)| x.abs() > 2.5 || y.abs() > 18.0).collect();
        assert!(outside.is_empty(), "6 mm feed moves outside the rest region: {outside:?}");
        let on_walls: Vec<_> =
            rest.iter().filter(|(x, y, z)| *z < 14.0 && y.abs() < 15.0 && x.abs() > 0.8 + 1e-6).collect();
        assert!(on_walls.is_empty(), "6 mm feed moves closer than 0.2 mm to a rib: {on_walls:?}");
    }
}
