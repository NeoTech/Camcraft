// Provenance: original CamCraft five-axis catalog assertions for issue #2870.
mod catalog_multiaxis_five_axis {
    use super::*;

    #[test]
    fn posts_three_clean_five_axis_operations() {
        use camcraft_app::operation::OpParams;
        use camcraft_app::{model, operation, project, setup, world};

        let dir = repo("examples/catalog/multiaxis/five-axis");
        assert!(dir.is_dir(), "catalog case multiaxis/five-axis is missing");
        let metadata = fs::read_to_string(dir.join("case.toml")).expect("five-axis metadata");
        for kind in ["Swarf", "5-Axis Contour", "5-Axis Flow"] {
            assert!(metadata.contains(&format!("\"{kind}\"")), "metadata must name {kind}");
        }
        let program = fs::read_to_string(dir.join("five-axis.nc")).expect("five-axis NC case");
        let w = world::new_world();
        let notes = project::load_noted(&w, &dir.join("five-axis.camproj"))
            .expect("five-axis project");
        assert!(notes.is_empty(), "load notes: {notes:?}");
        let ops = setup::operation_order(&w);
        assert_eq!(ops.len(), 3, "Swarf, 5-Axis Contour, and 5-Axis Flow");
        for (index, (name, entity)) in ops.iter().enumerate() {
            assert_eq!(operation::status_of(&w, entity), operation::OpStatus::Ok, "{name} saved cache");
            let op = model::operation_of(&w, entity).expect("five-axis operation");
            assert!(
                matches!(
                    (index, &op.params),
                    (0, OpParams::Swarf { .. })
                        | (1, OpParams::Contour5D { .. })
                        | (2, OpParams::Flow5D { .. })
                ),
                "{name}: expected Swarf, 5-Axis Contour, and 5-Axis Flow in order"
            );
            let blocks = section(&program, name);
            assert!(blocks.iter().any(|line| line.starts_with("G01 ")), "{name} must cut");
            assert!(
                blocks.iter().any(|line| ['A', 'B'].iter().any(|axis| {
                    word(line, *axis).is_some_and(|value| value.abs() > 0.01)
                })),
                "{name} must post a tilted tool axis"
            );
        }
        let part = model::find_by_name(&w, "five-axis").expect("embedded five-axis model");
        assert!(camcraft_app::model_source::of(&w, &part).is_none(), "portable model");
        assert!(model::facets_of(&w, &part).is_some_and(|facets| !facets.is_empty()));
    }

    #[test]
    fn regenerates() {
        run_case("multiaxis", "five-axis", "regenerates", regenerate);
    }

    #[test]
    fn rebuilds() {
        run_case("multiaxis", "five-axis", "rebuilds", rebuild);
    }
}
