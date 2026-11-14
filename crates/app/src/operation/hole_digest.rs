//! The hole-list words of an operation's stamp (#1560): a Drill, Bore
//! or Thread op's holes digest in the form the project file stores them,
//! snapped to the 1 um lattice ([`crate::cl::lattice`]) exactly as
//! `project::params`'s `HoleDto` / `CrossHoleDto` write them. A typed
//! off-lattice value (`0.190 in - 0.040 in` = 3.8099999999999996 mm)
//! and the 3.81 the file reloads then stamp alike, so an op saved Ok
//! reopens Ok (the #169 law). On-lattice values (what recognition
//! emits) hash bit-for-bit as before; a change of half a micron or more
//! still moves the stamp. Same remedy as #1351's path signature.
//!
//! Provenance: our own design (the stored form is the stamped form); no
//! vendor material.

use std::hash::Hasher;

use crate::cl::lattice;
use crate::drilling::{CrossHole, Hole};

/// One float as the project file stores it.
fn put(h: &mut impl Hasher, v: f64) {
    h.write(&lattice(v).to_bits().to_le_bytes());
}

/// One axial hole: the words and order the stamp always wrote.
pub(super) fn hole(h: &mut impl Hasher, x: &Hole) {
    put(h, x.xy.x);
    put(h, x.xy.y);
    put(h, x.radius);
    put(h, x.entry_z);
    put(h, x.floor_z);
    h.write_u8(u8::from(x.through));
    // Changes what the bore door says, so it belongs in the digest
    // (M8.3): a re-detect that folds a step marks the op Stale.
    h.write_u8(u8::from(x.stepped));
}

/// One radial hole (M5.2 F2). The direction is stored verbatim by the
/// file (recognition emits axis-aligned unit vectors), so it hashes
/// verbatim; every other float takes the file's snap.
pub(super) fn cross(h: &mut impl Hasher, c: &CrossHole) {
    put(h, c.origin.x);
    put(h, c.origin.y);
    put(h, c.origin.z);
    for v in [c.dir.x, c.dir.y, c.dir.z] {
        h.write(&v.to_bits().to_le_bytes());
    }
    for v in [c.radius, c.entry_d, c.floor_d] {
        put(h, v);
    }
    h.write_u8(u8::from(c.through));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo2d::Point2;

    fn sig(x: &Hole) -> u64 {
        let mut h = std::hash::DefaultHasher::new();
        hole(&mut h, x);
        h.finish()
    }

    fn at(floor_z: f64) -> Hole {
        Hole { xy: Point2::new(10.0, -5.0), radius: 3.0, entry_z: 15.334, floor_z, through: false, stepped: false }
    }

    #[test]
    fn a_typed_floor_stamps_like_the_one_the_file_reloads() {
        let typed: f64 = 0.190 * 25.4 - 0.040 * 25.4;
        assert_ne!(typed, 3.81, "the probe must be off the lattice");
        assert_eq!(sig(&at(typed)), sig(&at(3.81)));
        // Half a micron or more is a different hole.
        assert_ne!(sig(&at(3.81)), sig(&at(3.811)));
    }
}
