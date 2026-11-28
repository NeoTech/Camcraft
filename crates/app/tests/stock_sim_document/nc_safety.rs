//! Review pins (#4470): the first motion, unsupported taper tips, and
//! document mesh fidelity must never disappear from an NC verdict.

use super::*;
use camcraft_app::commands::TagFixture;
use camcraft_app::{model, model_fidelity};
use camcraft_app::stock_sim::DEV_TOL;
use camcraft_tools::taper::TaperTip;

fn parsed_from(text: &str, start: Point3) -> camcraft_gcode::ParsedProgram {
    let mut parse = ParseOptions::default();
    parse.dialect.initial_position = start;
    parse_with(text, &parse).expect("parses from the stated position")
}

#[test]
fn nc_first_rapid_crossing_fixture_is_checked_with_clear_endpoints() {
    let world = world();
    let parsed = parsed_from("G21 G90 G54\nT1 M6\nG0 X10 Y0 Z10\nM30", Point3::new(-10.0, 0.0, 10.0));
    let clamp = FixtureBox { min: Point3::new(-1.0, -3.0, 0.0), max: Point3::new(1.0, 3.0, 15.0) };
    let opts = NcSimOptions { target: Some(Vec::new()), fixtures: Some(vec![clamp]), ..known_opts() };
    let report = simulate_nc(&world, &parsed, &opts).expect("known approach is judged");
    assert!(report.findings.iter().any(|f| f.line == 3 && f.kind == NcFindingKind::Fixture), "{:?}", report.findings);
    assert!(!report.is_clean());
}

#[test]
fn nc_first_feed_crossing_target_is_checked_with_clear_endpoints() {
    let world = world();
    let parsed = parsed_from("G21 G90 G54\nT1 M6\nG1 X10 Y0 Z10 F300\nM30", Point3::new(-10.0, 0.0, 10.0));
    let target = cube(Point3::new(-1.0, -3.0, 0.0), Point3::new(1.0, 3.0, 15.0));
    let opts = NcSimOptions { target: Some(target.facets.to_vec()), fixtures: Some(Vec::new()), ..known_opts() };
    let report = simulate_nc(&world, &parsed, &opts).expect("known approach is judged");
    assert!(report.findings.iter().any(|f| f.line == 3 && f.kind == NcFindingKind::Gouge), "{:?}", report.findings);
    assert!(!report.is_clean());
}

#[test]
fn nc_unknown_start_refuses_straights_and_arcs_in_simulation_and_playback() {
    let world = world();
    for first in ["G0 X10 Y0 Z10", "G1 X10 Y0 Z10 F300", "G3 X10 Y0 Z10 I10 J0 F300"] {
        let text = format!("G21 G17 G90 G54\nT1 M6\n{first}\nM30");
        let parsed = parsed_from(&text, Point3::new(-10.0, 0.0, 10.0));
        let opts = NcSimOptions::default();
        let err = simulate_nc(&world, &parsed, &opts).expect_err("unknown initial position is refused");
        assert!(err.to_string().contains("initial tool position"), "{err}");
        let err = match play_seed(&world, &parsed, &opts, 2_000.0) {
            Err(err) => err,
            Ok(_) => panic!("unknown initial position must not play"),
        };
        assert!(err.to_string().contains("initial tool position"), "{err}");
    }
}

#[test]
fn nc_flat_and_ball_taper_tips_are_refused_from_every_tool_source() {
    for form in [TaperTip::Flat, TaperTip::Ball] {
        for source in [NcToolSource::Explicit, NcToolSource::Document, NcToolSource::Library] {
            let world = world();
            let mut cutter = tool("CM-0600-90");
            cutter.tip_form = form;
            cutter.tip_diameter = if form == TaperTip::Flat { 2.0 } else { 0.0 };
            cutter.corner_r = if form == TaperTip::Ball { 1.0 } else { 0.0 };
            assert!(cutter.is_sane(), "the {form} tool must be valid geometry");
            let mut opts = known_opts();
            let number = match source {
                NcToolSource::Document => {
                    History::default().execute(&world, AddTool::new(&world, "taper", cutter).expect("sane taper"));
                    camcraft_app::tool_numbers::number_of(&world, "taper").expect("numbered document tool")
                }
                NcToolSource::Explicit | NcToolSource::Library => {
                    let mut row = LibraryTool::bare("taper", cutter);
                    if source == NcToolSource::Explicit {
                        opts.tool_map.push((9, "taper".into()));
                    } else {
                        row.number = Some(9);
                    }
                    opts.library.push(row);
                    9
                }
            };
            let text = format!("G21 G90 G54\nT{number} M6\nG0 X0 Y0 Z40\nM30");
            let parsed = parse_with(&text, &known_parse()).expect("parses");
            let map = unmapped_tools(&world, &parsed, &opts).expect("tool map");
            assert_eq!(map.unmapped, vec![number], "{form} from {source:?}");
            assert!(map.reasons.iter().any(|why| why.contains(&form.to_string()) && why.contains(&format!("T{number}"))), "{:?}", map.reasons);
            assert!(matches!(simulate_nc(&world, &parsed, &opts), Err(NcSimError::UnmappedTools { .. })));
            assert!(matches!(play_seed(&world, &parsed, &opts, 2_000.0), Err(NcSimError::UnmappedTools { .. })));
        }
    }
}

fn set_fidelity(world: &World, name: &str, chord: f64) {
    let entity = model::find_by_name(world, name).expect("named mesh");
    model_fidelity::set(world, &entity, Some(chord));
}

fn stock_from_model(world: &World, chord: f64) {
    let mut history = History::default();
    history.execute(world, ImportModel::from_mesh("blank", cube(Point3::new(-30.0, -30.0, 0.0), Point3::new(30.0, 30.0, 20.0))));
    set_fidelity(world, "blank", chord);
    history.execute(world, SetStock::new(world, StockShape::FromModel { source: "blank".into() }, Location::default()).expect("model stock"));
}

fn air_program() -> camcraft_gcode::ParsedProgram {
    parse_with("G21 G90 G54\nT1 M6\nG0 X0 Y0 Z40\nM30", &known_parse()).expect("air program")
}

fn assert_fidelity_refusal(world: &World, opts: &NcSimOptions, name: &str) {
    let parsed = air_program();
    let err = simulate_nc(world, &parsed, opts).expect_err("untrusted facets cannot give a verdict");
    assert!(err.to_string().contains(name), "{err}");
    let err = match play_seed(world, &parsed, opts, 2_000.0) {
        Err(err) => err,
        Ok(_) => panic!("untrusted facets cannot give playback verdicts"),
    };
    assert!(err.to_string().contains(name), "{err}");
}

#[test]
fn nc_reduced_target_tightens_simulation_and_playback_even_with_explicit_tolerance() {
    let world = world();
    let excess = DEV_TOL * 0.25;
    let chord = camcraft_app::geometry::import::step::CHORD_CAP + excess;
    set_fidelity(&world, "part", chord);
    for tol in [None, Some(DEV_TOL), Some(10.0)] {
        let opts = NcSimOptions { tol, ..known_opts() };
        let report = simulate_nc(&world, &air_program(), &opts).expect("reduced target is still verifiable");
        assert!((report.tol - (DEV_TOL - excess)).abs() < 1e-12, "{}", report.tol);
        let seed = play_seed(&world, &air_program(), &opts, 2_000.0).expect("seed");
        let play = FieldPlay::from_seed(seed).expect("playback");
        assert!((play.view().tol - report.tol).abs() < 1e-12);
    }
}

#[test]
fn nc_reduced_stock_tightens_simulation_and_playback_without_document_target() {
    let world = world();
    let excess = DEV_TOL * 0.25;
    stock_from_model(&world, camcraft_app::geometry::import::step::CHORD_CAP + excess);
    // An external target does not erase fidelity of the stock itself.
    let opts = NcSimOptions { target: Some(Vec::new()), tol: Some(10.0), ..known_opts() };
    let report = simulate_nc(&world, &air_program(), &opts).expect("reduced stock is verifiable");
    assert!((report.tol - (DEV_TOL - excess)).abs() < 1e-12, "{}", report.tol);
    let seed = play_seed(&world, &air_program(), &opts, 2_000.0).expect("seed");
    assert!((FieldPlay::from_seed(seed).expect("playback").view().tol - report.tol).abs() < 1e-12);
}

#[test]
fn nc_unknown_target_and_fixture_refuse_simulation_and_playback_with_explicit_tolerance() {
    for fixture in [false, true] {
        let world = world();
        let name = if fixture { "clamp" } else { "part" };
        if fixture {
            let mut history = History::default();
            history.execute(&world, ImportModel::from_mesh(name, cube(Point3::new(40.0, -5.0, 0.0), Point3::new(50.0, 5.0, 20.0))));
            history.execute(&world, TagFixture::new(&world, name, None).expect("fixture"));
        }
        set_fidelity(&world, name, model_fidelity::UNKNOWN_MM);
        for tol in [None, Some(10.0)] {
            assert_fidelity_refusal(&world, &NcSimOptions { tol, ..known_opts() }, name);
        }
    }
}

#[test]
fn nc_unknown_stock_refuses_even_when_target_and_fixtures_are_overridden() {
    let world = world();
    stock_from_model(&world, model_fidelity::UNKNOWN_MM);
    let opts = NcSimOptions { target: Some(Vec::new()), fixtures: Some(Vec::new()), tol: Some(10.0), ..known_opts() };
    assert_fidelity_refusal(&world, &opts, "stock");
}

#[test]
fn nc_unverifiable_reduced_target_cannot_be_bypassed_by_a_large_tolerance() {
    let world = world();
    set_fidelity(&world, "part", camcraft_app::geometry::import::step::CHORD_CAP + DEV_TOL * 2.0);
    assert_fidelity_refusal(&world, &NcSimOptions { tol: Some(10.0), ..known_opts() }, "part");
}

#[test]
fn nc_legacy_stock_with_a_missing_source_refuses_simulation_and_playback() {
    let world = world();
    stock_from_model(&world, camcraft_app::geometry::import::step::CHORD_CAP + DEV_TOL * 0.25);
    // Legacy copies have no own record. Once their source is gone there
    // is no evidence that their retained facets have any known fidelity.
    let stock = camcraft_app::setup::stock_entity_name(&world, None);
    let entity = model::find_by_name(&world, &stock).expect("stock copy");
    model_fidelity::set(&world, &entity, None);
    History::default().execute(&world, camcraft_app::commands::DeleteEntity::new(&world, "blank").expect("delete source"));
    let opts = NcSimOptions { target: Some(Vec::new()), fixtures: Some(Vec::new()), tol: Some(10.0), ..known_opts() };
    assert_fidelity_refusal(&world, &opts, "stock");
}
