//! Pins of the name follow (#3915) on meshes built here, with records of
//! our own: a box at two widths, and a faceted cylinder whose wall is
//! one patch. Each revision lists its names in its own order, so a
//! follow that read the record index instead of the name fails.

use super::*;
use crate::geometry::feature::features_of_facets;
use crate::geometry::{Point3, Triangle, cube, cylinder};
use crate::source_faces::{FaceName, FacetRange, SourceFaces};

/// The six faces of `cube()`, in its facet order (two facets each).
const CUBE: [&str; 6] = ["bottom", "top", "front", "back", "right", "left"];
const FRONT: usize = 2;
const BACK: usize = 3;
const LEFT: usize = 5;

fn name(n: &str) -> FaceName {
    FaceName {
        name: n.into(),
        verified: true,
        former: None,
    }
}

fn names(list: &[&str]) -> Vec<FaceName> {
    list.iter().map(|n| name(n)).collect()
}

fn box_tris(width: f64) -> Vec<Triangle> {
    cube(Point3::ZERO, Point3::new(width, 20.0, 10.0))
        .facets
        .to_vec()
}

/// A record that gives facet `t` the record face `of(t, facet)`.
fn label(
    tris: &[Triangle],
    names: Vec<FaceName>,
    of: impl Fn(usize, &Triangle) -> usize,
) -> SourceFaces {
    let mut ranges: Vec<FacetRange> = Vec::new();
    for (t, tri) in tris.iter().enumerate() {
        let face = u32::try_from(of(t, tri)).expect("a small index");
        match ranges.last_mut() {
            Some(r) if r.face == face && r.end == t => r.end = t + 1,
            _ => ranges.push(FacetRange {
                start: t,
                end: t + 1,
                face,
            }),
        }
    }
    SourceFaces { names, ranges }
}

/// Cube face `k` (in [`CUBE`] order) is record face `slot[k]`.
fn box_record(tris: &[Triangle], names: Vec<FaceName>, slot: [usize; 6]) -> SourceFaces {
    label(tris, names, |t, _| slot[t / 2])
}

/// The old box (60 wide) in [`CUBE`] order, and the new box (80 wide)
/// with its names reversed - the same names, other record indexes.
fn reversed() -> (Vec<FaceName>, [usize; 6]) {
    let mut list = CUBE;
    list.reverse();
    (names(&list), [5, 4, 3, 2, 1, 0])
}

/// The id of the face that holds facet `t`.
fn id_at(f: &Features, t: usize) -> u64 {
    f.faces[f.facet_face[t] as usize].id
}

struct Pair {
    fa: Features,
    fb: Features,
    ra: SourceFaces,
    rb: SourceFaces,
}

impl Pair {
    fn new(a: &[Triangle], ra: SourceFaces, b: &[Triangle], rb: SourceFaces) -> Self {
        Pair {
            fa: features_of_facets(a),
            fb: features_of_facets(b),
            ra,
            rb,
        }
    }

    fn boxes(old: (Vec<FaceName>, [usize; 6]), new: (Vec<FaceName>, [usize; 6])) -> Self {
        let (a, b) = (box_tris(60.0), box_tris(80.0));
        let (ra, rb) = (box_record(&a, old.0, old.1), box_record(&b, new.0, new.1));
        Pair::new(&a, ra, &b, rb)
    }

    fn follow(&self, kind: PickKind, id: u64) -> Option<Verdict> {
        let old = Revision::new(&self.fa, &self.ra).expect("the old record tiles");
        let new = Revision::new(&self.fb, &self.rb).expect("the new record tiles");
        follow(&old, &new, kind, id)
    }

    fn face(&self, k: usize) -> Option<Verdict> {
        self.follow(PickKind::Face, id_at(&self.fa, 2 * k))
    }
}

fn plain() -> (Vec<FaceName>, [usize; 6]) {
    (names(&CUBE), [0, 1, 2, 3, 4, 5])
}

#[test]
fn a_side_face_follows_its_name_into_the_wider_box() {
    let p = Pair::boxes(plain(), reversed());
    let v = p.face(FRONT).expect("the front face moved");
    assert_eq!(v.to, Ok(id_at(&p.fb, 2 * FRONT)));
    assert_eq!(v.names, vec!["front".to_owned()]);
    assert_ne!(
        id_at(&p.fa, 2 * FRONT),
        id_at(&p.fb, 2 * FRONT),
        "the width change moved the id"
    );
}

#[test]
fn a_carried_face_and_a_pick_lost_before_have_nothing_to_follow() {
    let p = Pair::boxes(plain(), reversed());
    assert_eq!(
        id_at(&p.fa, 2 * LEFT),
        id_at(&p.fb, 2 * LEFT),
        "x = 0 keeps its corners"
    );
    assert_eq!(p.face(LEFT), None, "a carried face is untouched");
    assert_eq!(
        p.follow(PickKind::Face, 0xdead_beef),
        None,
        "an id in neither revision"
    );
}

#[test]
fn a_removed_name_does_not_follow() {
    let mut new = reversed();
    new.0[4] = name("lid"); // the new record's "top"
    let p = Pair::boxes(plain(), new);
    let v = p.face(1).expect("the top face moved");
    assert_eq!(v.to, Err(Lost::NoMatch));
    assert_eq!(v.names, vec!["top".to_owned()]);
}

#[test]
fn an_unverified_name_does_not_follow() {
    let mut old = plain();
    old.0[FRONT].verified = false;
    let p = Pair::boxes(old, reversed());
    assert_eq!(
        p.face(FRONT).expect("moved").to,
        Err(Lost::Unverified("front".into()))
    );
}

#[test]
fn an_unverified_new_name_does_not_follow() {
    let mut new = reversed();
    new.0[5 - FRONT].verified = false;
    let p = Pair::boxes(plain(), new);
    assert_eq!(p.face(FRONT).expect("moved").to, Err(Lost::NewUnverified));
}

#[test]
fn two_candidates_do_not_follow() {
    // The new record names the front and the back face both "front".
    let new = (
        names(&["bottom", "top", "front", "right", "left"]),
        [0, 1, 2, 2, 3, 4],
    );
    let p = Pair::boxes(plain(), new);
    assert_eq!(p.face(FRONT).expect("moved").to, Err(Lost::Many(2)));
}

#[test]
fn two_faces_with_one_name_before_do_not_follow() {
    let old = (
        names(&["bottom", "top", "side", "right", "left"]),
        [0, 1, 2, 2, 3, 4],
    );
    let p = Pair::boxes(old, reversed());
    assert_eq!(p.face(BACK).expect("moved").to, Err(Lost::TwinNames));
}

#[test]
fn a_former_name_follows() {
    let mut new = reversed();
    new.0[5 - FRONT] = FaceName {
        name: "front.1".into(),
        verified: true,
        former: Some("front".into()),
    };
    let p = Pair::boxes(plain(), new);
    assert_eq!(
        p.face(FRONT).expect("moved").to,
        Ok(id_at(&p.fb, 2 * FRONT))
    );
}

#[test]
fn two_old_faces_never_meet_on_one_new_face() {
    // The new front face is "front" and, by its former name, "back"; the
    // new back face has a name of its own. Each old pick finds one
    // candidate, but the two would meet on it: neither follows.
    let mut new = reversed();
    new.0[5 - FRONT] = FaceName {
        name: "front".into(),
        verified: true,
        former: Some("back".into()),
    };
    new.0[5 - BACK] = name("back2");
    let p = Pair::boxes(plain(), new);
    assert_eq!(p.face(FRONT).expect("moved").to, Err(Lost::Shared));
    assert_eq!(p.face(BACK).expect("moved").to, Err(Lost::Shared));
}

#[test]
fn a_name_that_moved_to_a_carried_face_does_not_follow() {
    // The new record gives "front" to the carried x = 0 face.
    let new = (
        names(&["bottom", "top", "front2", "back", "right", "front"]),
        [0, 1, 2, 3, 4, 5],
    );
    let p = Pair::boxes(plain(), new);
    assert_eq!(p.face(FRONT).expect("moved").to, Err(Lost::NotNew));
}

/// Cylinder facets: a cap by its normal, else the wall - or, with
/// `halves`, the wall's +x half and its -x half under two names.
fn cylinder_record(tris: &[Triangle], halves: bool) -> SourceFaces {
    label(tris, names(&["top", "bottom", "wall", "wall.b"]), |_, t| {
        let n = t.normal();
        let x = (t.a.x + t.b.x + t.c.x) / 3.0;
        if n.z > 0.9 {
            0
        } else if n.z < -0.9 {
            1
        } else if halves && x < 0.0 {
            3
        } else {
            2
        }
    })
}

fn cyl(d: f64) -> Vec<Triangle> {
    cylinder(d, 10.0, 16).facets.to_vec()
}

fn wall_patch(f: &Features) -> u64 {
    let all = patches(f);
    assert_eq!(all.len(), 1, "the faceted wall is one patch");
    all[0].id
}

#[test]
fn a_patch_follows_its_names_and_its_strips_do_not() {
    for halves in [false, true] {
        let (a, b) = (cyl(20.0), cyl(24.0));
        let (ra, rb) = (cylinder_record(&a, halves), cylinder_record(&b, halves));
        let p = Pair::new(&a, ra, &b, rb);
        let (pa, pb) = (wall_patch(&p.fa), wall_patch(&p.fb));
        assert_ne!(pa, pb, "the diameter change moved the patch id");
        let v = p.follow(PickKind::Patch, pa).expect("the wall moved");
        assert_eq!(v.to, Ok(pb), "halves {halves}");
        // One strip of the wall shares its names with its neighbours.
        let strip =
            p.fa.faces
                .iter()
                .find(|f| f.normal[2].abs() < 0.5)
                .expect("a strip")
                .id;
        assert_eq!(
            p.follow(PickKind::Face, strip).expect("moved").to,
            Err(Lost::TwinNames)
        );
    }
}

#[test]
fn a_kind_change_does_not_follow() {
    // The wall becomes the flat front face of a box.
    let (a, b) = (cyl(20.0), box_tris(60.0));
    let ra = cylinder_record(&a, false);
    let rb = box_record(
        &b,
        names(&["bottom", "top", "wall", "back", "right", "left"]),
        [0, 1, 2, 3, 4, 5],
    );
    let p = Pair::new(&a, ra, &b, rb);
    let v = p
        .follow(PickKind::Patch, wall_patch(&p.fa))
        .expect("the wall is gone");
    assert_eq!(v.to, Err(Lost::KindChanged));
}

#[test]
fn a_record_that_does_not_tile_is_never_read() {
    let a = box_tris(60.0);
    let f = features_of_facets(&a);
    let mut short = box_record(&a, names(&CUBE), [0, 1, 2, 3, 4, 5]);
    if let Some(r) = short.ranges.last_mut() {
        r.end -= 1;
    }
    assert!(
        Revision::new(&f, &short).is_none(),
        "a facet with no record face"
    );
    let mut past = box_record(&a, names(&CUBE), [0, 1, 2, 3, 4, 5]);
    past.names.pop();
    assert!(
        Revision::new(&f, &past).is_none(),
        "an index past the names"
    );
}

#[test]
fn every_reason_has_words() {
    for why in [
        Lost::TwinId,
        Lost::Unnamed,
        Lost::Unverified("a".into()),
        Lost::TwinNames,
        Lost::NoMatch,
        Lost::Many(3),
        Lost::KindChanged,
        Lost::NotNew,
        Lost::Shared,
        Lost::NewUnverified,
    ] {
        assert!(!why.reason().is_empty(), "{why:?}");
    }
}
