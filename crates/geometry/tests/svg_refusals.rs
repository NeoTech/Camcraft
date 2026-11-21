//! #1880: malformed SVG input is refused by name and never panics or hangs (Prime Directive).
//!
//! Provenance: hand-built malformed SVG text and deterministic byte mutations.
//! Gate map (M4.4): covers camcraft_geometry::svg.

use camcraft_geometry::dxf::Limits;
use camcraft_geometry::svg;

fn page(body: &str) -> String {
    format!(r#"<svg width="100mm" height="100mm" viewBox="0 0 100 100">{body}</svg>"#)
}

fn refuses(text: &str, name: &str) {
    let error = svg::parse(text.as_bytes(), None).expect_err("bad SVG must be refused").to_string();
    assert!(error.to_lowercase().contains(&name.to_lowercase()), "{error:?} does not name {name:?}");
}

#[test]
fn svg_malformed_documents_are_named() {
    refuses("", "no <svg>");
    refuses("hello", "no <svg>");
    refuses("<html></html>", "no <svg>");
    refuses("<svg", "unterminated");
    refuses("<svg><g></svg>", "does not match");
    refuses("<svg><rect/>", "not closed");
    refuses("<svg><!-- open", "unterminated comment");
    refuses("<svg width=100mm></svg>", "quoted");
    let gzip: &[u8] = b"\x1f\x8b\x08\0";
    assert!(svg::parse(gzip, None).expect_err("svgz").to_string().contains("compressed"));
    let bad_utf8: &[u8] = b"<svg>\xff</svg>";
    assert!(svg::parse(bad_utf8, None).expect_err("not UTF-8").to_string().contains("UTF-8"));
}

#[test]
fn svg_malformed_paths_and_shapes_are_named() {
    refuses(&page(r#"<path d="L10 10"/>"#), "malformed path");
    refuses(&page(r#"<path d="M10 10 L20"/>"#), "malformed path");
    refuses(&page(r#"<path d="M10 10 X20 20"/>"#), "malformed path");
    refuses(&page(r#"<path d="M0 0 A0 5 0 0 1 10 10"/>"#), "degenerate arc from (0, 0) to (10, 10): a radius is zero");
    refuses(&page(r#"<path d="M0 0 A5 5 0 2 1 10 10"/>"#), "malformed path");
    refuses(&page(r#"<rect width="-5" height="5"/>"#), "negative");
    refuses(&page(r#"<circle r="-1"/>"#), "negative");
    refuses(&page(r#"<polyline points="0,0 10"/>"#), "points");
    refuses(&page(r#"<rect width="5" height="5" transform="rotate(1 2)"/>"#), "transform");
    refuses(&page(r#"<rect width="5" height="5" transform="scale(0)"/>"#), "transform");
    refuses(&page(r#"<rect width="50%" height="5"/>"#), "percent");
    refuses(&page(r#"<line x2="1e400"/>"#), "finite");
}

#[test]
fn svg_zero_or_negative_document_sizes_are_refused() {
    refuses(r#"<svg width="0mm" height="10mm"><rect width="1" height="1"/></svg>"#, "zero or negative");
    refuses(r#"<svg width="10mm" height="-1mm"><rect width="1" height="1"/></svg>"#, "zero or negative");
    refuses(r#"<svg viewBox="0 0 0 10"><rect width="1" height="1"/></svg>"#, "viewBox");
    refuses(r#"<svg viewBox="0 0 10"><rect width="1" height="1"/></svg>"#, "viewBox");
    refuses(r#"<svg width="10mm"><rect width="1" height="1"/></svg>"#, "no height");
}

#[test]
fn svg_import_budget_is_enforced() {
    let small = Limits { max_bytes: 64, max_entities: 4, max_vertices: 16 };
    let big = page(&"<line x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\"/>".repeat(4));
    let error = svg::parse_with_limits(big.as_bytes(), None, small).expect_err("bytes").to_string();
    assert!(error.contains("import budget"), "{error}");
    let roomy = Limits { max_bytes: 1 << 20, ..small };
    let many = page(&"<line x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\"/>".repeat(8));
    let error = svg::parse_with_limits(many.as_bytes(), None, roomy).expect_err("elements").to_string();
    assert!(error.contains("import budget"), "{error}");
    let curve = page(r#"<circle r="40" transform="scale(1 0.5)"/>"#);
    let error = svg::parse_with_limits(curve.as_bytes(), None, roomy).expect_err("vertices").to_string();
    assert!(error.contains("import budget"), "{error}");
    let deep = format!("<svg width=\"1mm\" height=\"1mm\">{}{}</svg>", "<g>".repeat(5000), "</g>".repeat(5000));
    let error = svg::parse(deep.as_bytes(), None).expect_err("depth").to_string();
    assert!(error.contains("nested"), "{error}");
    // A `use` that refers to itself is cut off, not followed for ever.
    let looped = page(r##"<g id="a"><use href="#a"/><line x2="1"/></g>"##);
    let error = svg::parse(looped.as_bytes(), None).expect_err("loop").to_string();
    assert!(error.contains("use"), "{error}");
}

#[test]
fn svg_short_curve_vertices_cannot_silently_break_the_chord_bound() {
    // Removing the inner vertices makes the midpoint 0.01 mm from the emitted chord.
    // This exceeds the 0.005 mm bound, although the endpoints can form an open chain.
    let text = r#"<svg width="1mm" height="1mm" viewBox="0 0 1 1"><path d="M0 0 Q0 0.02 0.02 0"/></svg>"#;
    refuses(text, "curve");
    refuses(text, "chord");
}

#[test]
fn svg_beziers_above_the_segment_cap_refuse_the_import_budget() {
    for path in [
        "M0 0 Q0 100000000 100000000 0",
        "M0 0 C0 100000000 100000000 100000000 100000000 0",
    ] {
        let text = format!(
            r#"<svg width="100000000mm" height="100000000mm" viewBox="0 0 100000000 100000000"><path d="{path}"/></svg>"#
        );
        refuses(&text, "import budget");
    }
}

#[test]
fn svg_ellipses_above_the_segment_cap_refuse_the_import_budget() {
    let text = r#"<svg width="100000000mm" height="100000000mm" viewBox="0 0 100000000 100000000"><ellipse cx="50000000" cy="50000000" rx="40000000" ry="20000000"/></svg>"#;
    refuses(text, "import budget");
}

#[test]
fn svg_symbol_viewport_cannot_silently_ignore_the_use_size() {
    // The symbol is 10 units wide, but its referenced viewport must be 50 mm wide.
    let text = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="100mm" height="100mm" viewBox="0 0 100 100">
      <defs><symbol id="s" viewBox="0 0 10 10"><rect width="10" height="10"/></symbol></defs>
      <use xlink:href="#s" width="50" height="50"/>
    </svg>"##;
    refuses(text, "symbol");
    refuses(text, "viewport");
}

#[test]
fn svg_acyclic_shared_references_refuse_excess_traversal_work() {
    let mut body = String::from(r#"<defs><g id="g0"/>"#);
    for level in 1..=6 {
        body.push_str(&format!(r#"<g id="g{level}">"#));
        for _ in 0..10 {
            body.push_str(&format!(r##"<use href="#g{}"/>"##, level - 1));
        }
        body.push_str("</g>");
    }
    body.push_str(r##"</defs><use href="#g6"/><rect width="1" height="1"/>"##);
    let text = page(&body);
    // Fewer than 100 XML elements expand to over a million empty group visits.
    assert!(text.matches('<').count() < 100);
    let limits = Limits { max_bytes: 1 << 20, max_entities: 100, max_vertices: 10 };
    let error = svg::parse_with_limits(text.as_bytes(), None, limits).expect_err("traversal budget").to_string();
    assert!(error.contains("import budget") && error.contains("traversal"), "{error}");
}

#[test]
fn svg_combined_group_and_reference_depth_is_refused_without_stack_overflow() {
    // Each XML subtree and reference chain is legal by itself. Their combined
    // active traversal must be refused before it exhausts a normal test stack.
    let mut body = String::from("<defs>");
    for level in 0..16 {
        body.push_str(&format!(r#"<g id="g{level}">"#));
        body.push_str(&"<g>".repeat(230));
        if level < 15 {
            body.push_str(&format!(r##"<use href="#g{}"/>"##, level + 1));
        } else {
            body.push_str(r#"<rect width="10" height="10"/>"#);
        }
        body.push_str(&"</g>".repeat(231));
    }
    body.push_str(r##"</defs><use href="#g0"/>"##);
    refuses(&page(&body), "depth");
}

#[test]
fn svg_near_circular_large_ellipse_cannot_exceed_the_absolute_chord_bound() {
    // The old relative test emitted a circle 0.0095 mm from each axis.
    // The 0.005 mm chord bound requires flattening, which exceeds this budget.
    let text = r#"<svg width="100000000mm" height="100000000mm" viewBox="0 0 100000000 100000000"><ellipse rx="40000000" ry="40000000.019"/></svg>"#;
    refuses(text, "import budget");
}

#[test]
fn svg_deterministic_smoke_fuzz_never_panics() {
    let seed = page(
        r#"<g transform="translate(5 5) rotate(30)"><path d="M0 0 C10 0 10 10 0 10 S-10 20 0 20 Q5 25 10 20 T20 20 A5 3 20 1 0 30 30 Z"/>
        <rect x="1" y="1" width="8" height="4" rx="2" ry="1"/><ellipse rx="3" ry="2"/><polygon points="0,0 4,0 4,4"/></g>"#,
    )
    .into_bytes();
    for cut in 0..seed.len() {
        let _ = svg::parse(&seed[..cut], None);
    }
    let mut state = 0x1880u64;
    for _ in 0..1000 {
        let mut bytes = seed.clone();
        for _ in 0..4 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let index = (state as usize) % bytes.len();
            bytes[index] = (state >> 32) as u8;
        }
        let _ = svg::parse(&bytes, None);
        let _ = svg::parse(&bytes, Some(camcraft_geometry::dxf::Unit::Inches));
    }
}
