//! Face names that do not fit the body (#3915): each misfit of a format-10
//! names record gives no names and one note that says why, and the import
//! goes on as before. Never a refusal. The fixture is the one of `names`:
//! the sample marked as format 10, its final body a block of our own.
//! Provenance: our own fixture, built by the test. The names layout is
//! SolidCraft's published crates/doc/src/model_cache.rs (v0.50.0).

use camcraft_kernel::solidcraft::{MAX_NAME_BYTES, NAMES_FORMAT};
use serde_json::{Value, json};

use super::import;
use super::names::{FINAL, block, design_with, info, names_of, names_record};

/// A manifest edit, named in the misfit cases.
type Edit = Box<dyn Fn(&mut Value)>;

/// The edit that puts `v` at place `at` of the list `list` of the final
/// body's names record.
fn put(list: &'static str, at: usize, v: Value) -> Edit {
    Box::new(move |m: &mut Value| {
        if let Some(slot) = m["model"]["names"][FINAL][list].get_mut(at) {
            *slot = v.clone();
        }
    })
}

/// The edit that takes `key` out of the final body's entry of the model
/// list `list` (`names` or `model_bodies`).
fn take(list: &'static str, key: &'static str) -> Edit {
    Box::new(move |m: &mut Value| {
        if let Some(n) = m["model"][list][FINAL].as_object_mut() {
            n.remove(key);
        }
    })
}

/// Any other edit, boxed.
fn edit(f: impl Fn(&mut Value) + 'static) -> Edit {
    Box::new(f)
}

/// The edit that drops the last item of the list `list` of the final
/// body's names record.
fn pop(list: &'static str) -> Edit {
    Box::new(move |m: &mut Value| {
        if let Some(l) = m["model"]["names"][FINAL][list].as_array_mut() {
            l.pop();
        }
    })
}

#[test]
fn names_that_do_not_fit_give_a_note_never_a_refusal() {
    let record = block();
    let names = names_of(&record);
    let good = names_record(&names, |_| info(true, None));
    let long = "n".repeat(MAX_NAME_BYTES + 1);
    let index = |v: Value| edit(move |m| m["model"]["model_bodies"][FINAL]["names"] = v.clone());
    let cases: Vec<(&str, Edit, &str)> = vec![
        ("one name short", pop("faces"), "5 names and 6 info items"),
        (
            "one info item short",
            pop("info"),
            "6 names and 5 info items",
        ),
        ("a name not a text", put("faces", 2, json!(7)), "name 2"),
        ("an empty name", put("faces", 2, json!("")), "name 2"),
        ("a name too long", put("faces", 2, json!(long)), "name 2"),
        (
            "info of three items",
            put("info", 3, json!([[], [], true])),
            "info 3",
        ),
        (
            "info of six items",
            put("info", 3, json!([[], [], true, [], "a", "b"])),
            "info 3",
        ),
        (
            "verified not a flag",
            put("info", 3, json!([[], [], "yes", []])),
            "info 3",
        ),
        (
            "aliases not a list",
            put("info", 3, json!(["a", [], true, []])),
            "info 3",
        ),
        (
            "former not a text",
            put("info", 3, json!([[], [], true, [], 5])),
            "info 3",
        ),
        ("no face list", take("names", "faces"), "no face list"),
        ("no info list", take("names", "info"), "no info list"),
        (
            "record not an object",
            edit(|m| m["model"]["names"][FINAL] = json!([1, 2])),
            "no face list",
        ),
        ("index past the list", index(json!(99)), "past the"),
        ("index a text", index(json!("13")), "no names record"),
        ("index negative", index(json!(-1)), "no names record"),
        ("no index", take("model_bodies", "names"), "no names record"),
        (
            "no names list",
            edit(|m| m["model"]["names"] = json!({})),
            "no names list",
        ),
    ];
    for (case, change, want) in cases {
        let bytes = design_with(record.clone(), |m| {
            m["format"] = json!(NAMES_FORMAT);
            m["model"]["model_bodies"][FINAL]["names"] = json!(FINAL);
            m["model"]["names"][FINAL] = good.clone();
            change(m);
        });
        let i = match import(&bytes) {
            Ok(i) => i,
            Err(e) => panic!("{case}: names refused the import: {e}"),
        };
        assert!(i.names.is_none(), "{case}: names kept");
        let note = i.names_note.unwrap_or_default();
        assert!(
            note.contains("face names were not read"),
            "{case}: note '{note}'"
        );
        assert!(
            note.contains(want),
            "{case}: note '{note}' does not say '{want}'"
        );
    }
}
