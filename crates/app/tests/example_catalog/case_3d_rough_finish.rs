// Provenance: original CamCraft 3D rough-and-finish catalog contract (#2866).
mod catalog_3d_rough_finish {
    use super::*;
    #[test]
    fn regenerates() {
        run_case("3d", "rough-finish", "regenerates", regenerate);
    }
    #[test]
    fn rebuilds() {
        run_case("3d", "rough-finish", "rebuilds", rebuild);
    }
}
