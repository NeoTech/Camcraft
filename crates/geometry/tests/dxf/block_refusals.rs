//! Bad BLOCK and INSERT data is refused by name and never panics or hangs (#1865): cycles,
//! missing blocks, nesting past the bound, copy counts past the vertex limit, zero or
//! non-finite values, a tilted extrusion and external references.
//! Provenance: hand-built records from the public DXF BLOCK and INSERT group codes; a
//! fixed-seed mutation loop.

use camcraft_geometry::dxf::{Limits, MAX_BLOCK_DEPTH, parse, parse_with_limits};
use std::time::Instant;

fn drawing(blocks: &str, entities: &str) -> String {
    format!("0\nSECTION\n2\nBLOCKS\n{blocks}0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n")
}

fn block(name: &str, body: &str) -> String {
    format!("0\nBLOCK\n8\n0\n2\n{name}\n70\n0\n10\n0\n20\n0\n30\n0\n{body}0\nENDBLK\n")
}

fn insert(name: &str, more: &str) -> String {
    format!("0\nINSERT\n8\n0\n2\n{name}\n10\n0\n20\n0\n{more}")
}

const LINE: &str = "0\nLINE\n8\n0\n10\n0\n20\n0\n11\n1\n21\n0\n";

fn refused(text: &str, start: &str, words: &str) {
    let e = parse(text.as_bytes()).expect_err(words);
    assert!(e.0.starts_with(start) && e.0.contains(words), "{e}");
}

#[test]
fn cyclic_missing_and_too_deep_blocks_are_refused_by_name() {
    refused(&drawing(&block("A", &insert("A", "")), &insert("A", "")), "INSERT A", "cyclic block reference");
    let pair = format!("{}{}", block("A", &insert("b", "")), block("B", &insert("a", "")));
    refused(&drawing(&pair, &insert("A", "")), "INSERT a", "cyclic block reference");
    refused(&drawing(&block("A", LINE), &insert("Nope", "")), "INSERT Nope", "block is not defined");
    refused(&drawing(&block("A", &insert("Gone", "")), &insert("A", "")), "INSERT Gone", "block is not defined");
    // A chain of `levels` nested blocks: L1 inserts L2, ... the last holds a line.
    let chain = |levels: usize| {
        let mut blocks: String = (1..levels).map(|i| block(&format!("L{i}"), &insert(&format!("L{}", i + 1), ""))).collect();
        blocks += &block(&format!("L{levels}"), LINE);
        drawing(&blocks, &insert("L1", ""))
    };
    let doc = parse(chain(MAX_BLOCK_DEPTH).as_bytes()).expect("nesting at the bound reads");
    assert_eq!(doc.primitives.len(), 1);
    refused(&chain(MAX_BLOCK_DEPTH + 1), "INSERT L9", "blocks nest deeper than 8 levels");
    // A cycle in a block that no INSERT uses is never expanded.
    assert!(parse(drawing(&block("A", &insert("A", "")), LINE).as_bytes()).is_ok());
}

#[test]
fn copy_counts_past_the_vertex_limit_are_refused_without_the_work() {
    let small = Limits { max_vertices: 10_000, ..Limits::default() };
    let wide = drawing(&block("A", LINE), &insert("A", "70\n1000\n71\n1000\n44\n1\n45\n1\n"));
    let e = parse_with_limits(wide.as_bytes(), small).expect_err("array");
    assert!(e.0.starts_with("INSERT A") && e.0.contains("array of 1000000 copies exceeds the vertex limit"), "{e}");
    // Nested arrays of an empty block: each copy counts, so no hang without geometry.
    let bomb = format!(
        "{}{}{}",
        block("E", ""),
        block("M", &insert("E", "70\n100\n71\n100\n")),
        block("T", &insert("M", "70\n100\n71\n100\n"))
    );
    let started = Instant::now();
    let e = parse_with_limits(drawing(&bomb, &insert("T", "70\n100\n71\n100\n")).as_bytes(), small).expect_err("bomb");
    assert!(e.0.contains("expanded block geometry exceeds the vertex limit"), "{e}");
    assert!(started.elapsed().as_secs() < 10, "refused quickly");
    // The default limit holds the same bomb too.
    let e = parse(drawing(&bomb, &insert("T", "70\n100\n71\n100\n")).as_bytes()).expect_err("bomb");
    assert!(e.0.contains("exceeds the vertex limit"), "{e}");
}

#[test]
fn bad_insert_and_block_values_are_refused_by_name() {
    let one = |more: &str| drawing(&block("A", LINE), &insert("A", more));
    assert!(parse(one("").as_bytes()).is_ok());
    refused(&one("41\n0\n"), "INSERT A", "scale factor must be nonzero");
    refused(&one("43\n0\n"), "INSERT A", "scale factor must be nonzero");
    refused(&one("42\n1e12\n"), "INSERT A", "scale factor");
    refused(&one("41\nnan\n"), "INSERT", "non-finite numeric value");
    refused(&one("50\ninf\n"), "INSERT", "non-finite numeric value");
    refused(&one("44\n1e300\n70\n2\n"), "INSERT", "coordinate outside");
    refused(&one("70\n0\n"), "INSERT A", "array count must be at least 1");
    refused(&one("71\n-3\n"), "INSERT A", "array count must be at least 1");
    refused(&one("230\n-1\n"), "INSERT", "extrusion must point along +Z");
    refused(&one("210\n1\n"), "INSERT", "extrusion must point along +Z");
    let at = |x: &str, more: &str| drawing(&block("A", LINE), &format!("0\nINSERT\n8\n0\n2\nA\n10\n{x}\n20\n0\n{more}"));
    refused(&at("1e300", ""), "INSERT", "coordinate outside");
    // A placement whose geometry leaves the coordinate range is refused, not clamped.
    refused(&at("999500000000", "41\n1e9\n42\n1e9\n"), "INSERT A", "coordinate outside");
    // Attributes must end with SEQEND.
    let attributes = drawing(&block("A", LINE), &format!("{}0\nATTRIB\n8\n0\n1\nM6\n{LINE}", insert("A", "66\n1\n")));
    refused(&attributes, "INSERT", "expected ATTRIB or SEQEND");
    refused(&drawing(&block("A", LINE), "0\nINSERT\n8\n0\n10\n0\n20\n0\n"), "INSERT", "missing group 2");
    let xref = drawing(&block("A", LINE).replace("70\n0\n", "70\n4\n"), &insert("A", ""));
    refused(&xref, "BLOCK A", "external reference (XREF) is unsupported");
    let twice = drawing(&format!("{}{}", block("A", LINE), block("a", LINE)), &insert("A", ""));
    refused(&twice, "BLOCK a", "duplicate block definition");
    let open = format!("0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n{LINE}0\nENDSEC\n0\nEOF\n");
    refused(&open, "BLOCK A", "missing ENDBLK");
    // Bad geometry inside a used block names the block and the entity.
    let zero = drawing(&block("A", "0\nLINE\n8\n0\n10\n0\n20\n0\n11\n0\n21\n0\n"), &insert("A", ""));
    refused(&zero, "BLOCK A: LINE", "zero-length");
    // A block whose Z plane differs from the drawing's is not planar.
    let high = drawing(&block("A", LINE), &format!("{}0\nLINE\n8\n0\n10\n0\n20\n0\n30\n2\n11\n1\n21\n0\n31\n2\n", insert("A", "")));
    refused(&high, "INSERT", "varying Z heights");
}

#[test]
fn mutated_block_records_never_panic() {
    let base = drawing(
        &format!(
            "{}{}",
            block("A", "0\nARC\n8\n0\n10\n0\n20\n0\n40\n2\n50\n0\n51\n90\n0\nCIRCLE\n8\n0\n10\n3\n20\n0\n40\n1\n"),
            block("B", &format!("{LINE}{}", insert("A", "41\n2\n42\n1\n50\n30\n70\n2\n71\n2\n44\n10\n45\n10\n")))
        ),
        &insert("B", "41\n-1\n50\n45\n"),
    );
    // Millimetres: a drawing with no unit fits its curves 25.4 times finer (inches).
    let base = format!("0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n{base}");
    assert!(parse(base.as_bytes()).is_ok(), "{:?}", parse(base.as_bytes()));
    let lines: Vec<&str> = base.lines().collect();
    let values = ["0", "-1", "1e308", "-1e-308", "NaN", "inf", "3", "1e9", "0.5", "x", "", "2147483647", "A", "B"];
    let mut seed: u64 = 0x1865_b10c;
    let started = Instant::now();
    for _ in 0..300 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let at = (seed >> 33) as usize % lines.len();
        let value = values[(seed >> 13) as usize % values.len()];
        let mut mutated: Vec<&str> = lines.clone();
        if at % 2 == 1 {
            mutated[at] = value;
        }
        let text = mutated.join("\n") + "\n";
        let _ = parse(text.as_bytes());
    }
    assert!(started.elapsed().as_secs() < 60, "no mutation hangs");
}
