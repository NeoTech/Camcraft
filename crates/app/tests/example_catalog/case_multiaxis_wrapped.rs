// Provenance: clean-room checks of the catalog case multiaxis/wrapped (#2869),
// over CamCraft's own project file and posted program. Pulled into
// example_catalog.rs with `include!` so that each test keeps its name (the
// case contract test lists `catalog_multiaxis_wrapped::regenerates` and
// `catalog_multiaxis_wrapped::rebuilds`).

mod catalog_multiaxis_wrapped {
    use super::*;

    /// The three rotary kinds are saved, and each posted section turns A.
    #[test]
    fn keeps_three_rotary_operations_and_posts_a_axis_moves() {
        use camcraft_app::operation::{self, OpParams};
        use camcraft_app::{model, project, setup, world};
        let dir = repo("examples/catalog/multiaxis/wrapped");
        let w = world::new_world();
        let notes = project::load_noted(&w, &dir.join("wrapped.camproj"))
            .expect("the wrapped catalog project must load");
        assert!(notes.is_empty(), "unexpected load notes: {notes:?}");
        let mut kinds = Vec::new();
        for (name, entity) in setup::operation_order(&w) {
            assert_eq!(operation::status_of(&w, &entity), operation::OpStatus::Ok, "{name} saved cache");
            let op = model::operation_of(&w, &entity).expect("saved operation");
            kinds.push(match op.params {
                OpParams::RotaryContour { .. } => "Contour",
                OpParams::RotaryParallel { .. } => "Parallel",
                OpParams::RotaryOffset { .. } => "Offset",
                other => panic!("unexpected operation in wrapped example: {other:?}"),
            });
        }
        kinds.sort_unstable();
        assert_eq!(kinds, ["Contour", "Offset", "Parallel"]);
        let part = model::find_by_name(&w, "wrapped").expect("embedded model");
        assert!(camcraft_app::model_source::of(&w, &part).is_none(), "portable model");

        let nc = fs::read_to_string(dir.join("wrapped.nc")).expect("read wrapped.nc");
        let sections: Vec<_> = nc.split("(OPERATION: ").skip(1).collect();
        assert_eq!(sections.len(), 3, "each rotary operation must appear in the program");
        for section in sections {
            // Motion blocks: G0, G1 and the inverse-time G93 G1 of a wrapped move.
            let mut angles: Vec<f64> = section
                .lines()
                .filter(|line| line.starts_with('G'))
                .flat_map(str::split_whitespace)
                .filter_map(|word| word.strip_prefix('A')?.parse::<f64>().ok())
                .collect();
            assert!(angles.iter().all(|a| a.is_finite()), "A angles must be finite");
            angles.sort_by(f64::total_cmp);
            angles.dedup();
            assert!(angles.len() > 1, "each rotary operation must move the A axis: {section}");
        }
    }

    // #2869: about 30 s each in debug alone, too close to the 60 s budget
    // beside the other cases. The heavy lane keeps both full checks; the
    // test above pins the committed program.
    #[test]
    #[ignore = "heavy lane (#2869 multiaxis/wrapped): ~30 s in debug alone, too close to its 60 s budget under the parallel suite - run with --release -- --ignored"]
    fn regenerates() {
        run_case("multiaxis", "wrapped", "regenerates", regenerate);
    }

    #[test]
    #[ignore = "heavy lane (#2869 multiaxis/wrapped): ~30 s in debug alone, too close to its 60 s budget under the parallel suite - run with --release -- --ignored"]
    fn rebuilds() {
        run_case("multiaxis", "wrapped", "rebuilds", rebuild);
    }
}
