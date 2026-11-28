//! #3915: a `.sc` model's source face record rides the project file under
//! the key `source_faces`, with no version step: a reopened project still
//! follows picks by name at the next re-import, a record that does not
//! read or does not tile its facets is dropped and the file opens as it
//! would without it, and a model with no record saves no key.
//! Provenance: our own fixtures (`source_names`).

use serde_json::{Value, json};

use camcraft_app::commands::History;
use camcraft_app::world::new_world;
use camcraft_app::{model, project, source_faces};

use super::area_support::{AreaControls, op, pocket, sel};
use super::source_names::*;

const OP: &str = "op1";

fn saved(world: &flecs_ecs::prelude::World, tag: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "camcraft-3915-{tag}-{}.camproj",
        std::process::id()
    ));
    project::save(world, &path).expect("save");
    let text = std::fs::read_to_string(&path).expect("read back");
    let _ = std::fs::remove_file(&path);
    text
}

fn part_row(doc: &mut Value) -> &mut Value {
    doc["entities"]
        .as_array_mut()
        .and_then(|rows| rows.iter_mut().find(|r| r["name"] == PART))
        .expect("the part's row")
}

fn opened(text: &str) -> flecs_ecs::prelude::World {
    let world = new_world();
    project::open(&world, text).expect("the project opens");
    world
}

fn has_record(world: &flecs_ecs::prelude::World) -> bool {
    let e = model::find_by_name(world, PART).expect("the part");
    source_faces::of(world, &e).is_some()
}

#[test]
fn the_record_rides_the_project_file_and_picks_follow_after_it_opens() {
    let world = part_world(Some(plain(&block(40.0))));
    land_op(
        &world,
        OP,
        op(
            sel(PART, face(&world, FRONT)),
            pocket(10.0, 5.0),
            AreaControls::default(),
        ),
    );
    let text = saved(&world, "rides");
    let mut doc: Value = serde_json::from_str(&text).expect("json");
    let row = part_row(&mut doc);
    assert_eq!(
        row["source_faces"]["names"].as_array().map(Vec::len),
        Some(6)
    );
    assert_eq!(
        row["source_faces"]["names"][FRONT],
        json!({ "name": "front", "verified": true })
    );
    assert_eq!(row["source_faces"]["ranges"][0], json!([0, 2, 0]));

    let fresh = opened(&text);
    let e = model::find_by_name(&fresh, PART).expect("the part");
    let rec = source_faces::of(&fresh, &e).expect("the record came back");
    // The alignment holds after the load: every facet answers to the name
    // of the cube face it lies on.
    let facets = model::facets_of(&fresh, &e).expect("facets");
    assert_eq!(facets.len(), 12);
    for t in 0..facets.len() {
        let r = rec.face_of(t).expect("every facet is named");
        assert_eq!(rec.names[r as usize].name, CUBE[t / 2], "facet {t}");
    }

    let mut h = History::default();
    let (report, _) = reimport(
        &fresh,
        &mut h,
        block(50.0),
        Some(reversed(&block(50.0), None)),
    );
    assert_eq!(report.followed.len(), 1, "{report:?}");
    assert_eq!(src_of(&fresh, OP), vec![on_part(face(&fresh, FRONT))]);
}

#[test]
fn a_record_that_does_not_read_or_tile_is_dropped_and_the_file_opens() {
    let world = part_world(Some(plain(&block(40.0))));
    let text = saved(&world, "dropped");
    let doc: Value = serde_json::from_str(&text).expect("json");
    let six = json!([{ "name": "a" }, { "name": "b" }, { "name": "c" }, { "name": "d" }, { "name": "e" }, { "name": "f" }]);
    let cases = [
        ("short", json!({ "names": six, "ranges": [[0, 11, 0]] })),
        (
            "past the names",
            json!({ "names": [], "ranges": [[0, 12, 0]] }),
        ),
        (
            "face past u32",
            json!({ "names": six, "ranges": [[0, 12, 5_000_000_000_u64]] }),
        ),
        ("not a record", json!("names")),
        ("unknown shape", json!({ "faces": [] })),
        (
            "long name",
            json!({ "names": [{ "name": "x".repeat(20_000) }], "ranges": [[0, 12, 0]] }),
        ),
    ];
    for (case, bad) in cases {
        let mut d = doc.clone();
        part_row(&mut d)["source_faces"] = bad;
        let fresh = opened(&serde_json::to_string(&d).expect("json"));
        assert!(!has_record(&fresh), "{case}: dropped, never trusted");
        let e = model::find_by_name(&fresh, PART).expect("the part");
        assert_eq!(
            model::facets_of(&fresh, &e).map(|f| f.len()),
            Some(12),
            "{case}: the model loads as before"
        );
    }
}

#[test]
fn a_model_without_a_record_saves_no_key() {
    let world = part_world(None);
    let mut doc: Value = serde_json::from_str(&saved(&world, "none")).expect("json");
    assert!(
        part_row(&mut doc).get("source_faces").is_none(),
        "no record, no key: the file is as before"
    );

    // An update from a file with no names drops the record, and the key.
    let world = part_world(Some(plain(&block(40.0))));
    let mut h = History::default();
    reimport(&world, &mut h, block(50.0), None);
    let mut doc: Value = serde_json::from_str(&saved(&world, "dropped-by-update")).expect("json");
    assert!(part_row(&mut doc).get("source_faces").is_none());
}

#[test]
fn hostile_names_leave_a_review_line_the_project_file_accepts() {
    let long = "r".repeat(300);
    let list = [
        "bottom",
        "top",
        "front\nside",
        "back",
        long.as_str(),
        "left",
    ];
    let world = part_world(Some(record(&block(40.0), names(&list), [0, 1, 2, 3, 4, 5])));
    let picks = vec![on_part(face(&world, FRONT)), on_part(face(&world, RIGHT))];
    land_op(
        &world,
        OP,
        op(
            camcraft_app::operation::SrcRef::Selection(picks),
            pocket(10.0, 5.0),
            AreaControls::default(),
        ),
    );
    let mut h = History::default();
    let (report, _) = reimport(
        &world,
        &mut h,
        block(50.0),
        Some(record(&block(50.0), names(&list), [0, 1, 2, 3, 4, 5])),
    );
    assert_eq!(report.followed.len(), 2, "{report:?}");
    let review = review_of(&world, OP);
    assert_eq!(review.len(), 1);
    assert!(
        review[0].len() <= 240 && !review[0].chars().any(char::is_control),
        "{:?}",
        review[0]
    );

    let fresh = opened(&saved(&world, "hostile"));
    assert_eq!(
        review_of(&fresh, OP),
        review,
        "the line survives a save and a load"
    );
}
