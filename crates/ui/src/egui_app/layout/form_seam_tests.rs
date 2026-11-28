//! #1122: form-seam guard tests — the migrated page families (holes,
//! turning, multi-axis, 2D, 3D, the common operation tabs) stay at zero
//! raw egui control calls. Unmigrated families are
//! outside the guard until their own PR adds them.
//! Provenance: our own source-level guard over `crates/ui/src/egui_app`.

const RAW_PATTERNS: &[&str] = &[
    "ui.button(",
    "ui.small_button(",
    "ui.checkbox(",
    "ui.label(",
    "ui.add(",
    "ui.add_enabled(",
    "ui.add_sized(",
    "ui.radio_value(",
];

/// The code lines of `src`, trimmed, with a line that starts a method
/// call (`.add_enabled(`) joined to the line before it, so a call that
/// rustfmt splits after `ui` still counts.
fn joined_code(src: &str) -> String {
    let mut out = String::new();
    for line in src.lines().map(str::trim).filter(|line| !line.starts_with("//")) {
        if !line.starts_with('.') {
            out.push('\n');
        }
        out.push_str(line);
    }
    out
}

fn count_raw_calls(src: &str) -> usize {
    let code = joined_code(src);
    RAW_PATTERNS.iter().map(|pat| code.matches(pat).count()).sum()
}

#[test]
fn the_guard_counts_a_raw_call_split_over_two_lines() {
    assert_eq!(count_raw_calls("if ui\n    .add_enabled(on, b)\n    .clicked() {}"), 1);
    assert_eq!(count_raw_calls("ui.label(a);\n// ui.button(b)\nlist_row(ui, c);"), 1);
}

struct Family<'a> {
    name: &'a str,
    max_raw: usize,
    sources: &'a [(&'a str, &'a str)],
}

const FAMILIES: &[Family<'_>] = &[
    Family {
        name: "holes",
        max_raw: 0,
        sources: &[
            ("holes_page.rs", include_str!("../holes_page.rs")),
            ("thread_page.rs", include_str!("../thread_page.rs")),
        ],
    },
    Family {
        name: "turning",
        max_raw: 0,
        sources: &[
            ("turning_page.rs", include_str!("../turning_page.rs")),
            ("turning_more_page.rs", include_str!("../turning_more_page.rs")),
            ("turning_start_exit_rows.rs", include_str!("../turning_start_exit_rows.rs")),
            ("thread_infeed_page.rs", include_str!("../thread_infeed_page.rs")),
        ],
    },
    Family {
        name: "multi-axis",
        max_raw: 0,
        sources: &[
            ("multiaxis_page.rs", include_str!("../multiaxis_page.rs")),
            ("rotary_page.rs", include_str!("../rotary_page.rs")),
            ("multiaxis_deburr_page.rs", include_str!("../multiaxis_deburr_page.rs")),
        ],
    },
    Family {
        name: "2D",
        max_raw: 0,
        sources: &[
            ("mill2d_page.rs", include_str!("../mill2d_page.rs")),
            ("mill2d_more_page.rs", include_str!("../mill2d_more_page.rs")),
            ("trochoid_page.rs", include_str!("../trochoid_page.rs")),
            ("vcarve_page.rs", include_str!("../vcarve_page.rs")),
            ("probe_page.rs", include_str!("../probe_page.rs")),
            ("plasma_page.rs", include_str!("../plasma_page.rs")),
            ("waterjet_page.rs", include_str!("../waterjet_page.rs")),
            ("wire_page.rs", include_str!("../wire_page.rs")),
            ("sinker_page.rs", include_str!("../sinker_page.rs")),
            ("sinker_pick_rows.rs", include_str!("../sinker_pick_rows.rs")),
            ("deburr_page.rs", include_str!("../deburr_page.rs")),
            ("pockets_page.rs", include_str!("../pockets_page.rs")),
            ("approach_page.rs", include_str!("../approach_page.rs")),
            ("predrilled_page.rs", include_str!("../predrilled_page.rs")),
            ("rest_pocket_page.rs", include_str!("../rest_pocket_page.rs")),
            ("start_exit_rows.rs", include_str!("../start_exit_rows.rs")),
            ("pass_start_row.rs", include_str!("../pass_start_row.rs")),
            ("load_field.rs", include_str!("../load_field.rs")),
            ("lead_page.rs", include_str!("../lead_page.rs")),
            ("kind_editors.rs", include_str!("../kind_editors.rs")),
        ],
    },
    Family {
        name: "3D",
        max_raw: 0,
        sources: &[
            ("mill3d_page.rs", include_str!("../mill3d_page.rs")),
            ("undercut_page.rs", include_str!("../undercut_page.rs")),
        ],
    },
    // The tabs and rows that every operation page shares: the page frame,
    // the geometry picks, the boundary, the heights and the feeds.
    Family {
        name: "operation tabs",
        max_raw: 0,
        sources: &[
            ("op_page.rs", include_str!("../op_page.rs")),
            ("geo_page.rs", include_str!("../geo_page.rs")),
            ("boundary.rs", include_str!("../boundary.rs")),
            ("heights_page.rs", include_str!("../heights_page.rs")),
            ("feeds_page.rs", include_str!("../feeds_page.rs")),
        ],
    },
];

#[test]
fn form_seam_guard_covers_only_the_migrated_families() {
    let names: Vec<_> = FAMILIES.iter().map(|f| f.name).collect();
    assert_eq!(names, ["holes", "turning", "multi-axis", "2D", "3D", "operation tabs"], "unmigrated families must not block a migration");
    assert!(FAMILIES.iter().all(|f| f.max_raw == 0), "a migrated family has no raw call");
    let files = |i: usize| FAMILIES[i].sources.iter().map(|(name, _)| *name).collect::<Vec<_>>();
    assert_eq!(files(0), ["holes_page.rs", "thread_page.rs"]);
    assert_eq!(files(1), ["turning_page.rs", "turning_more_page.rs", "turning_start_exit_rows.rs", "thread_infeed_page.rs"]);
    assert_eq!(files(2), ["multiaxis_page.rs", "rotary_page.rs", "multiaxis_deburr_page.rs"]);
    assert_eq!(files(3).len(), 20, "the 2D family: {:?}", files(3));
    assert_eq!(files(4), ["mill3d_page.rs", "undercut_page.rs"]);
    assert_eq!(files(5), ["op_page.rs", "geo_page.rs", "boundary.rs", "heights_page.rs", "feeds_page.rs"]);
}

#[test]
fn pages_do_not_add_raw_egui_calls_outside_the_form_seam() {
    let mut violations = Vec::new();
    for fam in FAMILIES {
        let mut total = 0;
        let mut per_file = Vec::new();
        for (name, src) in fam.sources {
            let n = count_raw_calls(src);
            if n > 0 {
                per_file.push(format!("{name}={n}"));
            }
            total += n;
        }
        if total > fam.max_raw {
            violations.push(format!(
                "family '{}' has {total} raw ui calls (limit {}): {}",
                fam.name,
                fam.max_raw,
                per_file.join(", ")
            ));
        }
    }
    assert!(violations.is_empty(), "raw egui calls bypass the form seam:\n{}", violations.join("\n"));
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Choice {
    First,
    Second,
}

struct SeamProbe {
    hits: usize,
    small_hits: usize,
    choice: Choice,
    count: u32,
}

impl Default for SeamProbe {
    fn default() -> Self {
        Self { hits: 0, small_hits: 0, choice: Choice::Second, count: 1 }
    }
}

#[test]
fn form_seam_controls_preserve_labels_clicks_and_accessible_names() {
    use egui::accesskit::Role;
    use egui_kittest::Harness;
    use egui_kittest::kittest::{NodeT, Queryable};

    let mut h = Harness::builder().with_size(egui::Vec2::new(480.0, 260.0)).build_ui_state(
        |ui, p: &mut SeamProbe| {
            super::list_row(ui, "1  X 10.000  Y 20.000");
            if super::small_button(ui, "Remove 1").clicked() {
                p.small_hits += 1;
            }
            if super::button(ui, "Detect holes").clicked() {
                p.hits += 1;
            }
            super::prop_row(ui, "Hand", |ui| {
                super::radio_pair(ui, &mut p.choice, (Choice::First, "Left hand"), (Choice::Second, "Right hand"));
            });
            super::prop_row(ui, "Spring passes", |ui| {
                super::super::fields::int_range_row(ui, &mut p.count, 0..=4);
            });
        },
        SeamProbe::default(),
    );
    h.run();

    assert!(h.query_by_label("1  X 10.000  Y 20.000").is_some());
    assert!(h.query_by_label("Left hand").is_some() && h.query_by_label("Right hand").is_some());
    let spin_named = h.root().children_recursive().any(|n| {
        let node = n.accesskit_node();
        node.role() == Role::SpinButton && node.labelled_by().next().is_some()
    });
    assert!(spin_named, "int_range_row inside prop_row must carry labelled_by");

    h.get_by_label("Detect holes").click();
    h.get_by_label("Remove 1").click();
    h.get_by_label("Left hand").click();
    h.run();

    assert_eq!(h.state().hits, 1);
    assert_eq!(h.state().small_hits, 1);
    assert_eq!(h.state().choice, Choice::First);
}

// #2295: a wide property pane must use its free space for field names.
fn assert_property_label_uses_spare_width(checkbox: bool) {
    use egui_kittest::Harness;

    const LABEL: &str = "Maximum spindle acceleration";
    let mut h = Harness::builder().with_size(egui::Vec2::new(560.0, 100.0)).build_ui_state(
        move |ui, value: &mut bool| {
            if checkbox {
                super::check_row(ui, LABEL, value);
            } else {
                super::prop_row(ui, LABEL, |ui| {
                    ui.add_sized(egui::Vec2::new(super::CTRL_W, 20.0), egui::Button::new("Value"));
                });
            }
        },
        false,
    );
    h.run();

    fn label_galleys(shape: &egui::Shape, out: &mut Vec<bool>) {
        match shape {
            egui::Shape::Text(text) if text.galley.text() == LABEL => out.push(text.galley.elided),
            egui::Shape::Vec(shapes) => shapes.iter().for_each(|shape| label_galleys(shape, out)),
            _ => {}
        }
    }
    let mut elided = Vec::new();
    for clipped in &h.output().shapes {
        label_galleys(&clipped.shape, &mut elided);
    }
    assert_eq!(elided.len(), 1, "the row must paint the property label");
    assert!(!elided[0], "the property label is cut off while the 560-point pane has free space");
}

#[test]
fn property_label_uses_spare_panel_width() {
    assert_property_label_uses_spare_width(false);
}

#[test]
fn checkbox_label_uses_spare_panel_width() {
    assert_property_label_uses_spare_width(true);
}

#[test]
fn property_controls_fit_a_narrow_panel_and_checkbox_caption_toggles() {
    use egui_kittest::kittest::Queryable;
    for width in [200.0, 260.0] {
        let mut h = egui_kittest::Harness::builder()
            .with_size(egui::Vec2::new(width, 100.0))
            .build_ui_state(
                |ui, checked: &mut bool| {
                    let pane = ui.max_rect();
                    let value = super::prop_row(ui, "Maximum spindle acceleration", |ui| {
                        ui.add_sized(egui::Vec2::new(super::CTRL_W, 20.0), egui::Button::new("Value"))
                    });
                    assert!(value.rect.width() >= super::CTRL_W, "the label must reserve the compact control width");
                    let checkbox = super::check_row(ui, "Synchronize spindle acceleration", checked);
                    for rect in [value.rect, checkbox.rect] {
                        assert!(pane.contains_rect(rect), "control {rect:?} extends outside pane {pane:?}");
                    }
                },
                false,
            );
        h.run();
        let label = h.get_by_role_and_label(egui::accesskit::Role::Label, "Maximum spindle acceleration").rect();
        let value = h.get_by_role_and_label(egui::accesskit::Role::Button, "Value").rect();
        assert!(label.right() <= value.left(), "label {label:?} overlaps control {value:?} at panel width {width}");
        h.get_by_role_and_label(egui::accesskit::Role::Label, "Synchronize spindle acceleration").click();
        h.run();
        assert!(*h.state(), "clicking the caption must still toggle the checkbox");
    }
}

#[test]
fn truncated_property_label_shows_full_name_on_hover() {
    use egui_kittest::kittest::Queryable;

    const LABEL: &str = "Maximum spindle acceleration";
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::Vec2::new(260.0, 100.0))
        .build_ui(|ui| {
            super::prop_row(ui, LABEL, |ui| {
                ui.add_sized(egui::Vec2::new(super::CTRL_W, 20.0), egui::Button::new("Value"));
            });
        });
    h.run();
    let center = h.get_by_role_and_label(egui::accesskit::Role::Label, LABEL).rect().center();
    h.ctx.style_mut_of(h.ctx.theme(), |style| style.interaction.tooltip_delay = 0.0);
    h.hover_at(center);
    h.run_steps(3);

    fn has_full_label(shape: &egui::Shape) -> bool {
        match shape {
            egui::Shape::Text(text) => text.galley.text() == LABEL && !text.galley.elided,
            egui::Shape::Vec(shapes) => shapes.iter().any(has_full_label),
            _ => false,
        }
    }
    assert!(
        h.output().shapes.iter().any(|clipped| has_full_label(&clipped.shape)),
        "hover must show the full name when the label must be cut off"
    );
}

#[derive(Default)]
struct MoreProbe {
    off_hits: usize,
    on_hits: usize,
    small_off_hits: usize,
    on: bool,
    count: u32,
    pct: f64,
}

/// #1122: the helpers the 2D family added keep what the raw calls did:
/// a greyed-out button takes no click, a check box toggles, a note and a
/// cut list row keep their text, and a number field in a row is named by
/// the row.
#[test]
fn the_2d_seam_helpers_keep_clicks_text_and_names() {
    use egui::accesskit::Role;
    use egui_kittest::Harness;
    use egui_kittest::kittest::{NodeT, Queryable};

    let mut h = Harness::builder().with_size(egui::Vec2::new(480.0, 300.0)).build_ui_state(
        |ui, p: &mut MoreProbe| {
            if super::button_enabled(ui, false, "Add skim").clicked() {
                p.off_hits += 1;
            }
            if super::button_enabled(ui, true, "Add sync pair").clicked() {
                p.on_hits += 1;
            }
            if super::small_button_enabled(ui, false, "Exclude the picked edges").clicked() {
                p.small_off_hits += 1;
            }
            super::checkbox(ui, &mut p.on, "Use the start point");
            super::note(ui, "Not taken - the kind has no exit");
            super::list_row_truncated(ui, "Edge 12 of body 3");
            super::prop_row(ui, "Tab count", |ui| super::drag_row(ui, egui::DragValue::new(&mut p.count)));
            super::prop_row(ui, "Optimal load", |ui| super::drag_sized_row(ui, egui::DragValue::new(&mut p.pct).suffix(" %")));
        },
        MoreProbe::default(),
    );
    h.run();

    for text in ["Not taken - the kind has no exit", "Edge 12 of body 3"] {
        assert!(h.query_by_label(text).is_some(), "{text} is on the page");
    }
    for row in ["Tab count", "Optimal load"] {
        let named = h.root().children_recursive().any(|n| {
            let node = n.accesskit_node();
            node.role() == Role::SpinButton && node.label().as_deref() == Some(row)
        });
        assert!(named, "the {row} field is named by its row");
    }

    h.get_by_label("Add skim").click();
    h.get_by_label("Add sync pair").click();
    h.get_by_label("Exclude the picked edges").click();
    h.get_by_label("Use the start point").click();
    h.run();

    assert_eq!(h.state().off_hits, 0, "a greyed-out button takes no click");
    assert_eq!(h.state().small_off_hits, 0, "a greyed-out small button takes no click");
    assert_eq!(h.state().on_hits, 1);
    assert!(h.state().on, "the check box toggles");
}
