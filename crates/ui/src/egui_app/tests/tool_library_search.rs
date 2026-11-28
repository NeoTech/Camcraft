//! #4335 (U9 of #3663): the tool list's one search box. In each source
//! (Document, My library, Catalog) it finds by one term, by two terms
//! (AND), by a diameter, shows the "no match" text for no match, and Esc
//! clears it; the count follows. The part number (#898) is matched too.
//! State keeps the rows and the hits: a still frame builds nothing, and a
//! changed tool or a changed text builds once. The search is view state:
//! it queues no command.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests; no HSMWorks material involved.

use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

use super::tool_library_editor::{H, library_window};
use super::*;
use crate::egui_app::tool_library_dialog::components;
use crate::egui_app::tool_library_dialog::list::{self, NO_MATCH};
use crate::egui_app::tool_library_dialog::list_header::{PLACEHOLDER, SEARCH};
use crate::state::ToolSource;

/// The window open on `source`, with no tool open in the editor.
fn open_on(source: ToolSource) -> H {
    let mut h = shell_harness();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Tools").click();
    h.run_steps(2);
    h.state_mut().state.close_tool_in_library();
    if let Some(v) = h.state_mut().state.tool_library.as_mut() {
        v.source = source;
    }
    h.run_steps(3);
    h
}

/// Type `text` into the search box in place of what it holds.
fn search(h: &mut H, text: &str) {
    library_window(h).get_by_role_and_label(Role::TextInput, SEARCH).click();
    h.run();
    h.input_mut().events.push(Event::Key { key: Key::A, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::COMMAND });
    h.run();
    library_window(h).get_by_role_and_label(Role::TextInput, SEARCH).type_text(text);
    h.run_steps(3);
}

/// Esc in the focused search box.
fn escape(h: &mut H) {
    library_window(h).get_by_role_and_label(Role::TextInput, SEARCH).click();
    h.run();
    h.input_mut().events.push(Event::Key { key: Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE });
    h.run_steps(3);
}

fn has(h: &H, label: &str) -> bool {
    library_window(h).query_all_by_label(label).next().is_some()
}

fn quiet(h: &H) {
    let st = &h.state().state;
    assert!(st.pending.is_empty(), "the search queues no command");
}

#[test]
fn the_search_box_has_its_name_and_placeholder_and_holds_between_sources() {
    let mut h = open_on(ToolSource::Catalog);
    let node = library_window(&h).get_by_role_and_label(Role::TextInput, SEARCH).accesskit_node().placeholder().map(str::to_owned);
    assert_eq!(node.as_deref(), Some(PLACEHOLDER));
    search(&mut h, "ball");
    super::tool_library_editor::click_window_button(&mut h, "Document");
    assert_eq!(h.state().state.tool_search, "ball", "one search for each source");
    quiet(&h);
}

#[test]
fn catalog_search_by_one_term_two_terms_a_diameter_no_match_and_esc() {
    let mut h = open_on(ToolSource::Catalog);
    let total = h.state_mut().state.tool_hits(ToolSource::Catalog, "").rows.len();
    assert!(has(&h, &list::count_text(total, total)), "the count shows all {total} tools");
    search(&mut h, "ball");
    assert!(has(&h, "BN-0600") && has(&h, "BN-1000") && !has(&h, "EM-0600-4FL"), "one term: the type");
    search(&mut h, "6 ball");
    assert!(has(&h, "BN-0600") && !has(&h, "BN-1000") && !has(&h, "EM-0600-4FL"), "two terms combine");
    search(&mut h, "Ø10");
    assert!(has(&h, "BN-1000") && !has(&h, "BN-0600"), "a diameter");
    search(&mut h, "no such tool");
    assert!(has(&h, NO_MATCH) && has(&h, &list::count_text(0, total)), "no match: the text and 0 of {total}");
    escape(&mut h);
    assert!(h.state().state.tool_search.is_empty(), "Esc clears the box");
    assert!(has(&h, "BN-0600") && has(&h, "EM-0600-4FL") && has(&h, &list::count_text(total, total)));
    quiet(&h);
}

#[test]
fn document_search_by_name_type_diameter_part_number_no_match_and_esc() {
    let mut h = open_on(ToolSource::Document);
    let Some(name) = h.state().state.tool_names().into_iter().next() else {
        panic!("the seeded document has a tool");
    };
    let mut tool = super::tool_library_editor::tool(&h, &name);
    // #898: a part number on the tool; the list reads it after the command.
    tool.catalog.part_number = Some("PN-4711-X".into());
    h.state_mut().state.pending.push_back(CommandKind::ChangeTool { name: name.clone(), tool: tool.clone() });
    h.run_steps(3);
    let u = h.state().state.units();
    let d = list::compact(tool.diameter, u);
    let kind = crate::egui_app::browser_tree::kind_name(tool.kind);
    search(&mut h, &name.to_lowercase());
    assert!(has(&h, &name), "one term: the name");
    search(&mut h, &format!("{d} {kind}"));
    assert!(has(&h, &name), "two terms: the diameter {d} and the type {kind}");
    search(&mut h, "pn-4711");
    assert!(has(&h, &name), "the part number");
    search(&mut h, &format!("{name} 987.5"));
    assert!(has(&h, NO_MATCH) && !has(&h, &name), "a second term that does not match");
    escape(&mut h);
    assert!(has(&h, &name) && h.state().state.tool_search.is_empty(), "Esc clears the box");
    quiet(&h);
}

#[test]
fn my_library_search_by_name_two_terms_no_match_and_esc() {
    let mut h = open_on(ToolSource::Library);
    let lib = h.state_mut().state.component_library();
    let Some((name, kind)) = lib.all.first().map(|c| (c.component.name().to_owned(), components::kind_text(&c.component).to_lowercase())) else {
        panic!("the built-in component library is not empty");
    };
    let total = lib.all.len();
    assert!(has(&h, &components::count_text(total, total)), "the count shows all {total} components");
    let word = name.split_whitespace().next().unwrap_or_default().to_lowercase();
    search(&mut h, &word);
    assert!(has(&h, &name), "one term: the name");
    search(&mut h, &format!("{word} {kind}"));
    assert!(has(&h, &name), "two terms: the name and the kind");
    search(&mut h, "no such component");
    assert!(has(&h, &components::count_text(0, total)) && has(&h, "No component matches the filters."), "no match: 0 of {total}");
    escape(&mut h);
    assert!(has(&h, &name) && has(&h, &components::count_text(total, total)), "Esc clears the box");
    quiet(&h);
}

#[test]
fn a_still_frame_builds_no_hits_and_a_change_builds_once() {
    let mut h = open_on(ToolSource::Document);
    h.run_steps(2);
    let b = h.state().state.tool_list.builds();
    h.run_steps(3);
    assert_eq!(h.state().state.tool_list.builds(), b, "a still frame reads and builds nothing");
    h.state_mut().state.tool_search = "zz".into();
    h.run_steps(3);
    let b1 = h.state().state.tool_list.builds();
    assert_eq!((b1.reads, b1.hits), (b.reads, b.hits + 1), "a new text builds the hits once, without a read");
    let Some(name) = h.state().state.tool_names().into_iter().next() else {
        panic!("the seeded document has a tool");
    };
    let mut tool = super::tool_library_editor::tool(&h, &name);
    tool.diameter += 1.0;
    h.state_mut().state.pending.push_back(CommandKind::ChangeTool { name, tool });
    h.run_steps(3);
    let b2 = h.state().state.tool_list.builds();
    assert!(b2.reads > b1.reads && b2.hits == b1.hits + 1, "a changed tool reads and builds once: {b1:?} -> {b2:?}");
    h.run_steps(3);
    assert_eq!(h.state().state.tool_list.builds(), b2, "and then holds");
}
