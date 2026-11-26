//! Finding 6 of the #4123 review: the guide set has a total order. The
//! same curves in any input order and from any start give the same set,
//! also when curves share their index and family (split rings).

use super::common::{curve, pts};
use camcraft_pathkernel::guide::{BAD_CURVE, Curve, Family, GuideSet, REPEATED_START, ZERO_LENGTH};

fn all_orders(c: &[Curve]) -> Vec<Vec<Curve>> {
    let n = c.len();
    let mut out = Vec::new();
    let mut idx: Vec<usize> = (0..n).collect();
    // Heap's algorithm, iterative.
    let mut cnt = vec![0usize; n];
    out.push(idx.iter().map(|&i| c[i].clone()).collect());
    let mut i = 0;
    while i < n {
        if cnt[i] < i {
            let j = if i % 2 == 0 { 0 } else { cnt[i] };
            idx.swap(j, i);
            out.push(idx.iter().map(|&i| c[i].clone()).collect());
            cnt[i] += 1;
            i = 0;
        } else {
            cnt[i] = 0;
            i += 1;
        }
    }
    out
}

#[test]
fn curves_with_equal_keys_give_one_set_in_any_input_order() {
    // Three split pieces of ring k=1, all Ring family, plus one open row: equal (k, family) keys.
    let a = curve(&[(0.0, 0.0), (2.0, 0.0), (1.0, 1.0)], true, 1, Family::Ring);
    let b = curve(&[(5.0, 0.0), (7.0, 0.0), (6.0, 1.0)], true, 1, Family::Ring);
    let c = curve(&[(0.0, 5.0), (2.0, 5.0), (1.0, 6.0)], true, 1, Family::Ring);
    let d = curve(&[(3.0, 3.0), (9.0, 3.0)], false, 1, Family::Ring);
    let want = GuideSet::new(vec![a.clone(), b.clone(), c.clone(), d.clone()]);
    for input in all_orders(&[a, b, c, d]) {
        assert_eq!(GuideSet::new(input), want);
    }
}

#[test]
fn a_restarted_closed_curve_or_a_reversed_open_curve_gives_the_same_set() {
    let a = curve(&[(1.0, 1.0), (0.0, 0.0), (1.0, 0.0)], true, 2, Family::Ring);
    let a2 = curve(&[(1.0, 0.0), (1.0, 1.0), (0.0, 0.0)], true, 2, Family::Ring);
    assert_eq!(GuideSet::new(vec![a.clone()]), GuideSet::new(vec![a2]));
    assert_eq!(GuideSet::new(vec![a]).curves()[0].pts()[0].at(), camcraft_pathkernel::Point2::new(0.0, 0.0));
    let o = curve(&[(3.0, 2.0), (0.0, 1.0)], false, 0, Family::Row);
    let o2 = curve(&[(0.0, 1.0), (3.0, 2.0)], false, 0, Family::Row);
    assert_eq!(GuideSet::new(vec![o]), GuideSet::new(vec![o2]));
}

#[test]
fn a_closed_curve_with_two_equal_lowest_points_has_one_canonical_start() {
    // The lowest point (0,0) appears twice; both starts give one set.
    let p = [(0.0, 0.0), (2.0, 1.0), (0.0, 0.0), (-2.0, 1.0), (0.0, 3.0)];
    let mut q = p.to_vec();
    q.rotate_left(2);
    let a = Curve::new(pts(&p), None, true, 0, Family::Ring).unwrap();
    let b = Curve::new(pts(&q), None, true, 0, Family::Ring).unwrap();
    assert_eq!(GuideSet::new(vec![a]), GuideSet::new(vec![b]));
}

#[test]
fn the_set_sorts_by_index_then_family() {
    let s = GuideSet::new(vec![
        curve(&[(1.0, 1.0), (0.0, 0.0), (1.0, 0.0)], true, 2, Family::Ring),
        curve(&[(0.0, 5.0), (2.0, 5.0)], false, 0, Family::Row),
        curve(&[(0.0, 9.0), (2.0, 9.0)], false, 0, Family::Ring),
    ]);
    let keys: Vec<(u32, Family)> = s.curves().iter().map(|c| (c.k(), c.family())).collect();
    assert_eq!(keys, [(0, Family::Ring), (0, Family::Row), (2, Family::Ring)]);
}

#[test]
fn a_degenerate_curve_is_refused() {
    assert_eq!(Curve::new(pts(&[(0.0, 0.0), (1.0, 0.0)]), None, true, 0, Family::Ring).unwrap_err().reason(), BAD_CURVE);
    assert_eq!(Curve::new(pts(&[(0.0, 0.0)]), None, false, 0, Family::Trace).unwrap_err().reason(), BAD_CURVE);
    assert_eq!(Curve::new(pts(&[(1.0, 1.0), (1.0, 1.0)]), None, false, 0, Family::Trace).unwrap_err().reason(), ZERO_LENGTH);
    let closed_twice = pts(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.0)]);
    assert_eq!(Curve::new(closed_twice, None, true, 0, Family::Ring).unwrap_err().reason(), REPEATED_START);
}
