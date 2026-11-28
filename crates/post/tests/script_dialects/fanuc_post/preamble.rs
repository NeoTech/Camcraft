//! #4346: each section preamble of the Fanuc and Doosan DNM mill posts
//! states the plane (G17) and the feed mode (G94) before its first motion.
//! A FANUC control can keep G18 / G19 or G95 from an earlier program or
//! from MDI (FANUC Series 0i-F operator's manual B-64604EN-2, the power-on
//! state of the plane and of the feed mode). Then an XY arc, the cutter
//! compensation or a drilling cycle runs in the wrong plane, and an mm/min
//! F word is read as mm per revolution.
//! Provenance: our own tests over our own posts and stream format.

use camcraft_post::post::engine;
use serde_json::Value;

const POSTS: &[&str] = &["fanuc", "doosan-dnm"];

/// Two sections, two tools: each section opens a preamble of its own.
const STREAM: &str = "PARAM SpindleSpeed 6000
PARAM ClearancePlane 50
PARAM RetractPlane 25
TOOL 1 10MM END MILL
SECTION
Rapid 30000 30000 50000
Feed 30000 30000 -8000 F300000
Rapid 30000 30000 50000
ENDSECTION
PARAM SpindleSpeed 5000
PARAM ClearancePlane 50
PARAM RetractPlane 25
TOOL 2 6MM END MILL
SECTION
Rapid 10000 10000 50000
Feed 10000 10000 -4000 F200000
Rapid 10000 10000 50000
ENDSECTION
";

fn words(line: &str) -> Vec<&str> {
    line.split('(').next().unwrap_or("").split_whitespace().collect()
}

fn is_motion(line: &str) -> bool {
    words(line).iter().any(|w| matches!(*w, "G0" | "G00" | "G1" | "G01" | "G2" | "G02" | "G3" | "G03"))
}

#[test]
fn the_fanuc_family_preamble_states_g17_and_g94_before_the_first_motion() {
    for stem in POSTS {
        let cps = super::read(&format!("assets/posts/{stem}.cps"));
        let out = engine::run(&cps, &Value::Null, STREAM).unwrap_or_else(|e| panic!("{stem}: {e}"));
        assert!(out.warnings.is_empty(), "{stem}: {:?}", out.warnings);
        let lines: Vec<&str> = out.program.lines().collect();
        // The preamble block is the one that cancels compensation, length
        // offset and cycles; there is one for each section.
        let preambles: Vec<usize> = (0..lines.len())
            .filter(|&i| {
                let w = words(lines[i]);
                w.contains(&"G40") && w.contains(&"G49") && w.contains(&"G80")
            })
            .collect();
        assert_eq!(preambles.len(), 2, "{stem}: one preamble for each section\n{}", out.program);
        for &i in &preambles {
            let w = words(lines[i]);
            assert!(w.contains(&"G17"), "{stem}: the preamble `{}` states G17\n{}", lines[i], out.program);
            assert!(w.contains(&"G94"), "{stem}: the preamble `{}` states G94\n{}", lines[i], out.program);
        }
        let first_motion = lines.iter().position(|l| is_motion(l)).unwrap_or(usize::MAX);
        assert!(preambles[0] < first_motion, "{stem}: the preamble comes before the first motion\n{}", out.program);
    }
}
