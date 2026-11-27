//! #4339: the "Convert to components" preview of the tool library window.
//! Three standalone tools: a 6 mm end mill on a BT30 ER20 holder (its
//! stack gives the same tool), one with the old default stick-out (its
//! stack needs a longer shank: different) and an 8 mm one with no holder
//! (no match). Pinned: the header button opens the preview with one row
//! for each tool and its result; only the "same" row starts ticked and the
//! "no match" row cannot be ticked; "Convert selected" is one undo step
//! that gives the ticked tool its stack; the preview follows the document;
//! "Select every row that converts" ticks the different row; undo takes
//! each stack away; Close goes back to the tool list.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use camcraft_app::commands::{AddTool, SetToolBody};
use camcraft_app::tool_assembly as ta;
use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

use super::tool_library_components::{done, lib_dir, shell, shows, undo};
use super::tool_library_editor::{H, click_window_button, library_window};
use super::*;
use crate::egui_app::tool_library_dialog::{convert, list_header};

const SIX: &str = "Six";
const BARE: &str = "Bare";
const LEGACY: &str = "Legacy";

fn cat(name: &str) -> camcraft_app::tools::Tool {
    let Some((_, t)) = camcraft_app::tools::catalog().into_iter().find(|(n, _)| *n == name) else {
        panic!("the catalog has {name}");
    };
    t
}

fn shop(h: &mut H) {
    let st = &mut h.state_mut().state;
    for (name, c) in [(SIX, "EM-0600-4FL"), (LEGACY, "EM-0600-4FL"), (BARE, "EM-0800-4FL")] {
        let add = AddTool::new(&st.world, name, cat(c)).expect("tool");
        st.history.execute(&st.world, add);
    }
    // 10 mm of shank in the holder: the stack gives the same tool.
    let six = cat("EM-0600-4FL");
    let held = Some(six.flute_len + six.shank_len - 10.0);
    let holder = SetToolBody::library(&st.world, SIX, "BT30 ER20", None, held).expect("holder");
    st.history.execute(&st.world, holder);
    let old = SetToolBody::library(&st.world, LEGACY, "BT30 ER20", None, None).expect("default holder");
    st.history.execute(&st.world, old);
    st.doc_version += 1;
    h.run_steps(2);
}

fn has_stack(h: &H, name: &str) -> bool {
    let st = &h.state().state;
    model::find_by_name(&st.world, name).and_then(|e| ta::assembly_of(&st.world, &e)).is_some()
}

fn picked(h: &H) -> Vec<String> {
    h.state().state.tool_library.as_ref().and_then(|v| v.convert.as_ref()).map(|c| c.selection()).unwrap_or_default()
}

fn tick_disabled(h: &H, tool: &str) -> bool {
    library_window(h).get_by_label(&convert::tick_label(tool)).accesskit_node().is_disabled()
}

#[test]
fn the_preview_converts_the_ticked_rows_in_one_undo_step() {
    let dir = lib_dir("convert");
    let mut h = shell(&dir);
    shop(&mut h);
    h.get_by_role_and_label(Role::Button, "Tools").click();
    h.run_steps(2);
    assert!(shows(&h, list_header::NEW), "the tool list first");
    click_window_button(&mut h, convert::OPEN);
    assert!(shows(&h, convert::HEADING), "the preview stands in for the list");
    assert!(shows(&h, "BT30 ER20 > ER20 collet 6 mm > Six"), "the proposed stack of Six");
    assert!(shows(&h, "same"), "Six derives the same tool");
    assert!(shows(&h, "no match: no holder: the tool names no library holder"), "Bare names its reason");
    let six = cat("EM-0600-4FL");
    let diff = format!("different: shank length {:.3} -> {:.3} mm", six.shank_len, six.shank_len + six.shank_d);
    assert!(shows(&h, &diff), "Legacy names its difference: {diff}");
    assert!(!tick_disabled(&h, SIX) && !tick_disabled(&h, LEGACY) && tick_disabled(&h, BARE), "a no-match row cannot be ticked");
    assert_eq!(picked(&h), vec![SIX.to_owned()], "only the same row starts ticked");

    let before = done(&h);
    click_window_button(&mut h, convert::APPLY);
    assert_eq!(done(&h), before + 1, "Convert selected is one undo step");
    assert!(has_stack(&h, SIX) && !has_stack(&h, LEGACY) && !has_stack(&h, BARE), "only the ticked row converts");
    assert!(!shows(&h, "same"), "the preview follows the document: Six is no longer standalone");
    let apply = library_window(&h).get_by_role_and_label(Role::Button, convert::APPLY).accesskit_node().is_disabled();
    assert!(apply, "nothing is ticked");
    click_window_button(&mut h, convert::EVERY);
    assert_eq!(picked(&h), vec![LEGACY.to_owned()], "every row that converts");
    click_window_button(&mut h, convert::APPLY);
    assert!(has_stack(&h, LEGACY), "the ticked different row converts");
    assert_eq!(done(&h), before + 2);

    undo(&mut h);
    undo(&mut h);
    assert!(!has_stack(&h, SIX) && !has_stack(&h, LEGACY), "undo takes each stack away");
    click_window_button(&mut h, convert::CLOSE);
    assert!(!shows(&h, convert::HEADING) && shows(&h, list_header::NEW), "Close goes back to the tool list");
}
