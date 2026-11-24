use super::*;
use crate::dxf::LayeredPrimitive;
use crate::sketch_chains::{ChainOptions, chain};

fn closed(primitives: Vec<Primitive>) -> ClosedChain {
    let items: Vec<_> = primitives
        .into_iter()
        .map(|primitive| LayeredPrimitive {
            layer: "profile".into(),
            primitive,
        })
        .collect();
    let chains = chain(&items, ChainOptions::default()).unwrap();
    assert_eq!(chains.closed.len(), 1, "{chains:?}");
    chains.closed[0].clone()
}

/// An R10 racetrack with caps at X0 and X16, drawn as four separate primitives.
fn racetrack(cap_sweep: f64) -> Vec<Primitive> {
    let line = |a: [f64; 2], b: [f64; 2]| Primitive::Line {
        start: Point2::new(a[0], a[1]),
        end: Point2::new(b[0], b[1]),
    };
    vec![
        line([0., -10.], [16., -10.]),
        Primitive::Arc {
            center: Point2::new(16., 0.),
            radius: 10.,
            start_angle: -PI / 2.,
            sweep_angle: cap_sweep,
        },
        line([16., 10.], [0., 10.]),
        Primitive::Arc {
            center: Point2::new(0., 0.),
            radius: 10.,
            start_angle: PI / 2.,
            sweep_angle: PI,
        },
    ]
}

#[test]
fn a_tangent_racetrack_chain_gives_its_two_cap_centres() {
    let shape = RoundProfile::from_chain(&closed(racetrack(PI))).unwrap();
    assert_eq!(shape.radius, 10.);
    // The chain walk sets the order of the two caps.
    let Spine::Stadium(mut caps) = shape.spine else {
        panic!("a racetrack");
    };
    caps.sort_by(|a, b| a.x.total_cmp(&b.x));
    assert_eq!(caps, [Point2::new(0., 0.), Point2::new(16., 0.)]);
}

#[test]
fn a_circle_reads_its_centre_and_an_unqualified_radius_is_refused() {
    let circle = |radius| {
        closed(vec![Primitive::Circle {
            center: Point2::new(3., -2.),
            radius,
        }])
    };
    let shape = RoundProfile::from_chain(&circle(10.)).unwrap();
    assert_eq!(shape.spine, Spine::Circle(Point2::new(3., -2.)));
    for radius in [0.4, 20.5] {
        assert!(RoundProfile::from_chain(&circle(radius)).is_err());
    }
}

#[test]
fn a_chain_that_is_not_a_tangent_racetrack_is_refused() {
    // A rectangle has no arcs.
    let line = |a: [f64; 2], b: [f64; 2]| Primitive::Line {
        start: Point2::new(a[0], a[1]),
        end: Point2::new(b[0], b[1]),
    };
    let square = vec![
        line([0., 0.], [10., 0.]),
        line([10., 0.], [10., 10.]),
        line([10., 10.], [0., 10.]),
        line([0., 10.], [0., 0.]),
    ];
    assert!(RoundProfile::from_chain(&closed(square)).is_err());
    // A short straight side: the caps are closer than 6 mm.
    let short = vec![
        line([0., -10.], [5., -10.]),
        Primitive::Arc {
            center: Point2::new(5., 0.),
            radius: 10.,
            start_angle: -PI / 2.,
            sweep_angle: PI,
        },
        line([5., 10.], [0., 10.]),
        Primitive::Arc {
            center: Point2::new(0., 0.),
            radius: 10.,
            start_angle: PI / 2.,
            sweep_angle: PI,
        },
    ];
    assert!(RoundProfile::from_chain(&closed(short)).is_err());
}

#[test]
fn the_circle_preview_stays_outside_the_true_wire_boundary() {
    let center = Point2::new(3.5, -2.0);
    let shape = RoundProfile {
        spine: Spine::Circle(center),
        radius: 10.0,
    };
    let circle = shape.preview().unwrap();
    for (&p, &q) in circle
        .points()
        .iter()
        .zip(circle.points().iter().cycle().skip(1))
    {
        let signed =
            ((q.x - p.x) * (center.y - p.y) - (q.y - p.y) * (center.x - p.x)) / distance(p, q);
        assert!((signed - shape.radius).abs() < 1e-9);
        assert!(distance(center, p) <= shape.radius + PREVIEW_SAG + 1e-9);
    }
}

#[test]
fn the_preview_contains_each_true_cap_with_bounded_excess() {
    let shape = RoundProfile {
        spine: Spine::Stadium([Point2::new(1., -2.), Point2::new(13., 14.)]),
        radius: 10.,
    };
    let ring = shape.preview().unwrap();
    let [a, b] = shape.centers();
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    for (&p, &q) in ring
        .points()
        .iter()
        .zip(ring.points().iter().cycle().skip(1))
    {
        // A capsule is the convex hull of its two circles. A CCW edge
        // contains both discs if their centres clear its inner half-plane.
        let length = distance(p, q);
        for center in [a, b] {
            let signed = ((q.x - p.x) * (center.y - p.y) - (q.y - p.y) * (center.x - p.x)) / length;
            assert!(signed >= shape.radius - 1e-9, "{signed}");
        }
        let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / (dx * dx + dy * dy)).clamp(0., 1.);
        let spine = Point2::new(a.x + t * dx, a.y + t * dy);
        assert!(distance(p, spine) <= shape.radius + PREVIEW_SAG + 1e-9);
    }
}
