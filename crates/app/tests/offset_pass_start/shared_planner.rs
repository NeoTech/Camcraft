//! #3750 equivalence pin on `fig8.step`: the 2D Offset with Auto, Inside
//! and Outside (and the Arc entry for the two sides) posts the same CL and
//! the same warnings as the head before the shared ring planner. The
//! goldens were recorded on that head (origin/main 138a592a1). Regenerate
//! only for an intended output change, with `WRITE_GOLDEN=1 cargo test
//! --release -p camcraft-app --test offset_pass_start -- --ignored shared_planner`.

use super::{PassStart, fig8_world, offset_op_entered};
use super::Entry3D;
use crate::common::{gen_to_cltext, write_golden};

#[test]
#[ignore = "heavy lane (#3750 fig8 Auto, Inside and Outside): five generations run past the 60 s debug budget - run with --release -- --ignored"]
fn fig8_offset_rings_unchanged_by_the_shared_planner() {
    let cases = [
        ("auto", PassStart::Auto, Entry3D::Helix),
        ("inside", PassStart::Inside, Entry3D::Helix),
        ("outside", PassStart::Outside, Entry3D::Helix),
        ("inside_arc", PassStart::Inside, Entry3D::Arc),
        ("outside_arc", PassStart::Outside, Entry3D::Arc),
    ];
    for (name, start, entry) in cases {
        let (world, mut h, part, stock) = fig8_world();
        let (path, warnings) = offset_op_entered(&world, &mut h, &part, stock.0.z, start, entry);
        let mut text = gen_to_cltext(&path);
        for n in &path.notes {
            text.push_str(&format!("; note: {}\n", n.replace('\n', " ")));
        }
        for w in &warnings {
            text.push_str(&format!("; warning: {}\n", w.replace('\n', " ")));
        }
        write_golden(&format!("offset2d_fig8_{name}.cl"), &text);
    }
}
