//! BLOCK definitions and their INSERT references expand into world geometry (#1865): moves,
//! turns, equal scales and mirrors keep arcs exact; unequal scales give tangent arcs within the
//! curve tolerance; arrays and nested blocks compose; layer "0" takes the INSERT's layer.
//! Provenance: hand-built fixtures from the public DXF BLOCK, INSERT and ARC group codes; the
//! expected values are worked by hand from `at + R(rotation) S (p - base)`.

use camcraft_geometry::dxf::{CURVE_TOLERANCE_MM, Document, Primitive, parse};
use camcraft_geometry::geo2d::Point2;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// A drawing in millimetres with `blocks` in its BLOCKS section and `entities` in ENTITIES.
fn blocks_drawing(blocks: &str, entities: &str) -> String {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n\
         0\nSECTION\n2\nBLOCKS\n{blocks}0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
}

/// A BLOCK named `name` with base point `base` and the entity records `body`.
fn block(name: &str, base: (f64, f64), body: &str) -> String {
    format!("0\nBLOCK\n8\n0\n2\n{name}\n70\n0\n10\n{}\n20\n{}\n30\n0\n3\n{name}\n{body}0\nENDBLK\n8\n0\n", base.0, base.1)
}

/// An INSERT of `name` on `layer` at `at`, with the extra group pairs `more`.
fn insert(name: &str, layer: &str, at: (f64, f64), more: &str) -> String {
    format!("0\nINSERT\n8\n{layer}\n2\n{name}\n10\n{}\n20\n{}\n30\n0\n{more}", at.0, at.1)
}

fn read(text: &str) -> Document {
    parse(text.as_bytes()).unwrap_or_else(|e| panic!("reads: {e}"))
}

fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-12, "{a} != {b}");
}

fn near_point(p: Point2, x: f64, y: f64) {
    near(p.x, x);
    near(p.y, y);
}

fn arc_ends(p: &Primitive) -> (Point2, Point2) {
    match *p {
        Primitive::Line { start, end } => (start, end),
        Primitive::Arc { center, radius, start_angle, sweep_angle } => {
            let at = |a: f64| Point2::new(center.x + radius * a.cos(), center.y + radius * a.sin());
            (at(start_angle), at(start_angle + sweep_angle))
        }
        Primitive::Circle { center, radius } => {
            let p = Point2::new(center.x + radius, center.y);
            (p, p)
        }
    }
}

const SHAPES: &str = "0\nLINE\n8\n0\n10\n1\n20\n1\n11\n3\n21\n1\n\
    0\nARC\n8\n0\n10\n1\n20\n1\n40\n2\n50\n0\n51\n90\n\
    0\nCIRCLE\n8\nholes\n10\n2\n20\n2\n40\n0.5\n";

#[test]
fn insert_moves_turns_and_scales_a_block_keeping_arcs_exact() {
    let text = blocks_drawing(&block("Plate", (1.0, 1.0), SHAPES), &insert("plate", "cut", (10.0, 20.0), "41\n2\n42\n2\n50\n90\n"));
    let doc = read(&text);
    assert_eq!(doc.primitives.len(), 3, "{:?}", doc.primitives);
    assert!(doc.summary.skipped.is_empty(), "{:?}", doc.summary.skipped);
    assert_eq!(doc.curve_tolerance, None, "no curve was fitted");
    let [line, arc, circle] = [0, 1, 2].map(|i| &doc.primitives[i]);
    assert_eq!((line.layer.as_str(), arc.layer.as_str(), circle.layer.as_str()), ("cut", "cut", "holes"));
    let Primitive::Line { start, end } = line.primitive else { panic!("{line:?}") };
    near_point(start, 10.0, 20.0);
    near_point(end, 10.0, 24.0);
    let Primitive::Arc { center, radius, start_angle, sweep_angle } = arc.primitive else { panic!("{arc:?}") };
    near_point(center, 10.0, 20.0);
    near(radius, 4.0);
    near(start_angle, FRAC_PI_2);
    near(sweep_angle, FRAC_PI_2);
    let Primitive::Circle { center, radius } = circle.primitive else { panic!("{circle:?}") };
    near_point(center, 8.0, 22.0);
    near(radius, 1.0);
}

#[test]
fn a_mirrored_insert_reverses_the_arc() {
    let text = blocks_drawing(&block("Plate", (0.0, 0.0), SHAPES), &insert("Plate", "cut", (0.0, 0.0), "41\n-1\n"));
    let doc = read(&text);
    let Primitive::Arc { center, radius, start_angle, sweep_angle } = doc.primitives[1].primitive else { panic!() };
    near_point(center, -1.0, 1.0);
    near(radius, 2.0);
    near(start_angle, PI);
    near(sweep_angle, -FRAC_PI_2);
    // The mirror images of the drawn ends (3, 1) and (1, 3), in the drawn order.
    let (a, b) = arc_ends(&doc.primitives[1].primitive);
    assert!((a.x + 3.0).abs() < 1e-12 && (a.y - 1.0).abs() < 1e-12, "{a:?}");
    assert!((b.x + 1.0).abs() < 1e-12 && (b.y - 3.0).abs() < 1e-12, "{b:?}");
}

/// The distance from `q` to the primitive `p`.
fn distance(q: Point2, p: &Primitive) -> f64 {
    match *p {
        Primitive::Line { start, end } => {
            let (dx, dy) = (end.x - start.x, end.y - start.y);
            let t = (((q.x - start.x) * dx + (q.y - start.y) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
            (start.x + t * dx - q.x).hypot(start.y + t * dy - q.y)
        }
        Primitive::Circle { center, radius } => ((q.x - center.x).hypot(q.y - center.y) - radius).abs(),
        Primitive::Arc { center, radius, start_angle, sweep_angle } => {
            let a = (q.y - center.y).atan2(q.x - center.x);
            let (lo, sweep) = if sweep_angle >= 0.0 { (start_angle, sweep_angle) } else { (start_angle + sweep_angle, -sweep_angle) };
            if (a - lo).rem_euclid(TAU) <= sweep {
                ((q.x - center.x).hypot(q.y - center.y) - radius).abs()
            } else {
                let (s, e) = arc_ends(p);
                (q.x - s.x).hypot(q.y - s.y).min((q.x - e.x).hypot(q.y - e.y))
            }
        }
    }
}

#[test]
fn unequal_scale_turns_a_circle_into_tangent_arcs_within_the_bound() {
    let body = "0\nCIRCLE\n8\n0\n10\n0\n20\n0\n40\n5\n";
    let text = blocks_drawing(&block("Hole", (0.0, 0.0), body), &insert("Hole", "cut", (50.0, 0.0), "41\n2\n42\n1\n50\n30\n"));
    let doc = read(&text);
    assert!(doc.primitives.len() >= 4, "{:?}", doc.primitives);
    assert_eq!(doc.curve_tolerance, Some(CURVE_TOLERANCE_MM));
    let out: Vec<Primitive> = doc.primitives.iter().map(|p| p.primitive.clone()).collect();
    // The true ellipse: (50, 0) + R(30 deg) (10 cos t, 5 sin t).
    let (s, c) = 30f64.to_radians().sin_cos();
    let ellipse = |t: f64| {
        let (x, y) = (10.0 * t.cos(), 5.0 * t.sin());
        Point2::new(50.0 + c * x - s * y, s * x + c * y)
    };
    for i in 0..4000 {
        let q = ellipse(TAU * i as f64 / 4000.0);
        let d = out.iter().map(|p| distance(q, p)).fold(f64::INFINITY, f64::min);
        assert!(d <= CURVE_TOLERANCE_MM * 1.01, "ellipse point {q:?} is {d} from the arcs");
    }
    // The pieces join end to end.
    for w in out.windows(2) {
        let (_, a) = arc_ends(&w[0]);
        let (b, _) = arc_ends(&w[1]);
        assert!((a.x - b.x).hypot(a.y - b.y) < 1e-9, "{w:?}");
    }
    assert!(doc.primitives.iter().all(|p| p.layer == "cut"));
}

#[test]
fn a_mirrored_unequal_scale_keeps_the_arc_ends() {
    // A half circle from (5, 0) over (0, 5) to (-5, 0), drawn counterclockwise.
    let body = "0\nARC\n8\n0\n10\n0\n20\n0\n40\n5\n50\n0\n51\n180\n";
    let text = blocks_drawing(&block("Half", (0.0, 0.0), body), &insert("Half", "cut", (0.0, 0.0), "41\n-2\n42\n1\n"));
    let doc = read(&text);
    let out: Vec<Primitive> = doc.primitives.iter().map(|p| p.primitive.clone()).collect();
    let (first, _) = arc_ends(&out[0]);
    let (_, last) = arc_ends(out.last().expect("pieces"));
    // x is doubled and mirrored: the ends go to (-10, 0) and (10, 0), over (0, 5).
    assert!((first.x + 10.0).abs() < 1e-9 && first.y.abs() < 1e-9, "{first:?}");
    assert!((last.x - 10.0).abs() < 1e-9 && last.y.abs() < 1e-9, "{last:?}");
    let top = Point2::new(0.0, 5.0);
    let d = out.iter().map(|p| distance(top, p)).fold(f64::INFINITY, f64::min);
    assert!(d <= CURVE_TOLERANCE_MM, "{d}");
}

#[test]
fn array_places_columns_and_rows_along_the_turned_axes() {
    let body = "0\nLINE\n8\n0\n10\n0\n20\n0\n11\n1\n21\n0\n";
    let more = "50\n90\n70\n3\n71\n2\n44\n10\n45\n5\n";
    let doc = read(&blocks_drawing(&block("Tick", (0.0, 0.0), body), &insert("Tick", "cut", (100.0, 0.0), more)));
    assert_eq!(doc.primitives.len(), 6);
    let mut starts: Vec<(i64, i64)> = doc
        .primitives
        .iter()
        .map(|p| {
            let Primitive::Line { start, end } = p.primitive else { panic!() };
            // Each copy is turned by 90 degrees: it runs along +Y.
            assert!((end.x - start.x).abs() < 1e-12 && (end.y - start.y - 1.0).abs() < 1e-12, "{p:?}");
            (start.x.round() as i64, start.y.round() as i64)
        })
        .collect();
    starts.sort_unstable();
    // Column spacing 10 along the turned X axis (+Y), row spacing 5 along the turned Y (-X).
    assert_eq!(starts, vec![(95, 0), (95, 10), (95, 20), (100, 0), (100, 10), (100, 20)]);
}

#[test]
fn nested_blocks_compose_and_layer_zero_takes_the_insert_layer() {
    let unit = block("A", (0.0, 0.0), "0\nLINE\n8\n0\n10\n0\n20\n0\n11\n1\n21\n0\n");
    let middle = block("B", (0.0, 0.0), &insert("A", "0", (5.0, 0.0), ""));
    let outer = |layer: &str| block("C", (0.0, 0.0), &insert("B", layer, (0.0, 0.0), "50\n90\n"));
    for (layer, want) in [("0", "cut"), ("mid", "mid")] {
        let text = blocks_drawing(&format!("{unit}{middle}{}", outer(layer)), &insert("C", "cut", (100.0, 100.0), ""));
        let doc = read(&text);
        assert_eq!(doc.primitives.len(), 1);
        assert_eq!(doc.primitives[0].layer, want);
        let Primitive::Line { start, end } = doc.primitives[0].primitive else { panic!() };
        near_point(start, 100.0, 105.0);
        near_point(end, 100.0, 106.0);
    }
}

#[test]
fn a_block_reference_that_was_skipped_now_reads_its_block() {
    let blocks = format!(
        "{}{}",
        block("BOLT", (0.0, 0.0), "0\nCIRCLE\n8\n0\n10\n0\n20\n0\n40\n3\n0\nTEXT\n8\n0\n10\n0\n20\n0\n40\n1\n1\nM6\n"),
        block("NUT", (0.0, 0.0), "0\nCIRCLE\n8\n0\n10\n0\n20\n0\n40\n5\n"),
    );
    let entities = format!(
        "{}0\nATTRIB\n8\n0\n1\nM6\n2\nSIZE\n10\n5\n20\n5\n0\nSEQEND\n8\n0\n0\nPOINT\n8\n0\n10\n1\n20\n1\n{}\
         0\nLINE\n8\ncut\n10\n0\n20\n0\n11\n10\n21\n0\n",
        insert("BOLT", "0", (5.0, 5.0), "66\n1\n"),
        insert("NUT", "0", (9.0, 9.0), ""),
    );
    let doc = read(&blocks_drawing(&blocks, &entities));
    assert_eq!(doc.primitives.len(), 3, "{:?}", doc.primitives);
    assert_eq!(doc.summary.skipped.get("INSERT"), None);
    assert_eq!(doc.summary.skipped.get("POINT"), Some(&1));
    // TEXT in a block is skipped and counted, and is not kept as an annotation.
    assert_eq!(doc.summary.skipped.get("TEXT"), Some(&1));
    assert!(doc.annotations.is_empty());
    let circles: Vec<(f64, f64, f64)> = doc
        .primitives
        .iter()
        .filter_map(|p| match p.primitive {
            Primitive::Circle { center, radius } => Some((center.x, center.y, radius)),
            _ => None,
        })
        .collect();
    assert_eq!(circles, vec![(5.0, 5.0, 3.0), (9.0, 9.0, 5.0)]);
}
