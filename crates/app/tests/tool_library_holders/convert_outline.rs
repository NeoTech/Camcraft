//! #4339: what a conversion must keep for the collision laws and the
//! post. On the `convert` shop: the posted program does not change when
//! the "same" row and both kinds of shank-added "different" row (the old
//! default stick-out, and a stick-out past the shank's end) convert by
//! name; and for each row kind that converts, the converted holder body
//! is the library holder's own outline bit for bit, or the row lists the
//! outline as a difference and the new outline covers the old one
//! everywhere - a converted tool never gets a smaller holder.
//! Provenance: our own tests over our own commands.

use camcraft_app::commands::{ConvertToComponents, GenerateOperation, History};
use camcraft_app::model;
use camcraft_app::operation::{OpStatus, status_of};
use camcraft_app::posting;
use camcraft_app::schema::ToolBody;
use camcraft_app::tool_body::body_of;
use camcraft_app::tool_convert::{outline_difference, plan, Verdict};
use flecs_ecs::prelude::World;

use super::convert::{shop, DEEP, LEGACY, LONG, OPS, SIX, SIX_B};

fn post(w: &World, tag: &str) -> String {
    let out = std::env::temp_dir().join(format!("camcraft-4339-{tag}-{}.nc", std::process::id()));
    let _ = std::fs::remove_file(&out);
    posting::post_setup(w, &out).expect("posted");
    let text = std::fs::read_to_string(&out).expect("program");
    let _ = std::fs::remove_file(&out);
    text
}

fn body(w: &World, name: &str) -> ToolBody {
    body_of(w, &model::find_by_name(w, name).expect("tool")).expect("a body")
}

fn tool(w: &World, name: &str) -> camcraft_app::tools::Tool {
    model::tool_of(w, &model::find_by_name(w, name).expect("tool")).expect("a tool")
}

/// Generate again each operation the conversion left stale.
fn regenerate(w: &World, h: &mut History) {
    for (op, _) in OPS {
        let e = model::find_by_name(w, op).expect("op");
        if status_of(w, &e) == OpStatus::Stale {
            h.execute(w, GenerateOperation::new(w, op).expect("generate again"));
        }
    }
}

#[test]
fn the_posted_program_of_each_converted_row_kind_does_not_change() {
    let (w, mut h) = shop();
    let before = post(&w, "posted-before");
    assert_eq!(plan(&w, &[SIX.to_owned()]).expect("plan")[0].verdict, Verdict::Same);
    for name in [LEGACY, DEEP] {
        let row = plan(&w, &[name.to_owned()]).expect("plan").remove(0);
        assert!(matches!(row.verdict, Verdict::Different(_)), "{name}: {:?}", row.verdict);
    }
    let names = [SIX.to_owned(), LEGACY.to_owned(), DEEP.to_owned()];
    h.execute(&w, ConvertToComponents::new(&w, &names).expect("convert by name"));
    regenerate(&w, &mut h);
    let after = post(&w, "posted-after");
    assert_eq!(after, before, "the same program after the conversion: same T, H and length words");
}

#[test]
fn the_converted_holder_outline_is_the_library_outline_or_a_listed_wider_difference() {
    for name in [SIX, SIX_B, LONG, LEGACY, DEEP] {
        let (w, mut h) = shop();
        let (old, now) = (body(&w, name), tool(&w, name));
        let row = plan(&w, &[name.to_owned()]).expect("plan").remove(0);
        let diffs = match &row.verdict {
            Verdict::Same => Vec::new(),
            Verdict::Different(d) => d.clone(),
            Verdict::NoMatch(why) => {
                assert!(why.contains("narrower than the library holder"), "{name}: {why}");
                assert!(ConvertToComponents::new(&w, &[name.to_owned()]).is_err(), "{name}: a narrower outline never converts");
                continue;
            }
        };
        h.execute(&w, ConvertToComponents::new(&w, &[name.to_owned()]).expect("convert"));
        let (new, next) = (body(&w, name), tool(&w, name));
        let same = old.spun == new.spun && old.profile() == new.profile();
        let listed = diffs.iter().any(|d| d.starts_with("holder outline"));
        assert_eq!(
            listed,
            !same,
            "{name}: an outline that is not bit for bit the library holder's is listed\nold {:?}\nnew {:?}",
            old.profile(),
            new.profile()
        );
        let judged = outline_difference(&old, &now, Some(&new), &next);
        assert!(judged.is_ok(), "{name}: never narrower: {judged:?}");
        if name == SIX || name == SIX_B {
            assert!(same, "{name}: the 'same' row keeps the library outline\nold {:?}\nnew {:?}", old.profile(), new.profile());
        }
    }
}
