//! Unit tests of the curved-face fit's reported height (#2718): the value
//! the fit reports for a face is never below the largest distance that the
//! kept facets stand off the true surface, for each curved kind it measures.
//! The meshes are made here on a triangular lattice in the surface's own
//! parameters, coarse enough that their edges stand between one and two
//! pass tolerances off: the band the fit keeps and did not report.

use super::*;
use truck_meshalgo::prelude::{Faces, PolygonMesh, StandardAttributes, Vector2};

const TOL: f64 = 0.005;

type At = dyn Fn(f64, f64) -> [f64; 3];

/// A lattice of `nu` x `nv` points from `(u0, v0)`, steps `du`, `dv`, odd
/// rows shifted half a step, so the facets are near equilateral in (u, v).
fn lattice(at: &At, (u0, du, nu): (f64, f64, usize), (v0, dv, nv): (f64, f64, usize)) -> PolygonMesh {
    let mut positions = Vec::new();
    let mut uv_coords = Vec::new();
    for j in 0..nv {
        for i in 0..nu {
            let u = u0 + (i as f64 + 0.5 * (j % 2) as f64) * du;
            let v = v0 + j as f64 * dv;
            let p = at(u, v);
            positions.push(truck_meshalgo::prelude::Point3::new(p[0], p[1], p[2]));
            uv_coords.push(Vector2::new(u, v));
        }
    }
    let id = |i: usize, j: usize| j * nu + i;
    let mut tris: Vec<[usize; 3]> = Vec::new();
    for j in 0..nv - 1 {
        for i in 0..nu - 1 {
            if j % 2 == 0 {
                tris.push([id(i, j), id(i + 1, j), id(i, j + 1)]);
                tris.push([id(i + 1, j), id(i + 1, j + 1), id(i, j + 1)]);
            } else {
                tris.push([id(i, j), id(i + 1, j + 1), id(i, j + 1)]);
                tris.push([id(i, j), id(i + 1, j), id(i + 1, j + 1)]);
            }
        }
    }
    type Corner = (usize, Option<usize>, Option<usize>);
    let faces: Vec<[Corner; 3]> = tris.iter().map(|t| t.map(|k| (k, Some(k), None))).collect();
    let faces = Faces::from_iter(faces.iter().map(|f| &f[..]));
    PolygonMesh::new(StandardAttributes { positions, uv_coords, normals: Vec::new() }, faces)
}

/// The kept facets: the fit's own, or the kernel's when it kept them.
fn kept(fit: &Fit, poly: &PolygonMesh) -> Vec<[[f64; 3]; 3]> {
    if let Some(f) = &fit.facets {
        return f.iter().map(|t| [t.a, t.b, t.c].map(|p| [p.x, p.y, p.z])).collect();
    }
    let pos = poly.positions();
    poly.face_iter().map(|f| [0, 1, 2].map(|k| {
        let p = pos[f[k].pos];
        [p.x, p.y, p.z]
    })).collect()
}

/// The largest distance a dense barycentric sample of the facets stands
/// off the surface `gap` measures.
fn worst(facets: &[[[f64; 3]; 3]], gap: &dyn Fn([f64; 3]) -> f64) -> f64 {
    const N: usize = 12;
    let mut w = 0.0f64;
    for t in facets {
        for a in 0..=N {
            for b in 0..=N - a {
                let (la, lb) = (a as f64 / N as f64, b as f64 / N as f64);
                let lc = 1.0 - la - lb;
                let p = [0, 1, 2].map(|k| la * t[0][k] + lb * t[1][k] + lc * t[2][k]);
                w = w.max(gap(p));
            }
        }
    }
    w
}

/// The pin: what the fit reports bounds what it kept, and what it kept
/// stands past the pass tolerance (the value the import reported before).
fn pins(name: &str, kind: Curved, at: &At, gap: &dyn Fn([f64; 3]) -> f64, u: (f64, f64, usize), v: (f64, f64, usize)) {
    let poly = lattice(at, u, v);
    let fit = fit(&poly, at, TOL, 1_000_000, kind);
    let facets = kept(&fit, &poly);
    let w = worst(&facets, gap);
    eprintln!("{name}: reported {:.5} mm, sampled worst {w:.5} mm, {} facets", fit.off, facets.len());
    assert!(w > TOL, "{name}: the lattice must stand past the pass tolerance to pin #2718 (worst {w})");
    assert!(fit.off >= w, "{name}: reported {} mm is below the kept facets' worst {w} mm", fit.off);
    assert!(fit.off <= 2.0 * w, "{name}: reported {} mm is far over the worst {w} mm", fit.off);
}

fn rho(p: [f64; 3]) -> f64 {
    (p[0] * p[0] + p[1] * p[1]).sqrt()
}

#[test]
fn a_cylinder_face_reports_no_less_than_its_facets_stand_off() {
    let r = 10.0;
    let at = move |u: f64, v: f64| [r * u.cos(), r * u.sin(), v];
    let du = 0.08;
    pins("cylinder", Curved::Cylinder, &at, &|p| (r - rho(p)).abs(), (0.0, du, 12), (0.0, r * du * 0.866, 12));
}

#[test]
fn a_cone_face_reports_no_less_than_its_facets_stand_off() {
    let (s, c) = (30f64.to_radians().sin(), 30f64.to_radians().cos());
    // `v` is the slant distance from the apex.
    let at = move |u: f64, v: f64| [v * s * u.cos(), v * s * u.sin(), v * c];
    let gap = move |p: [f64; 3]| (rho(p) * c - p[2] * s).abs();
    let du = 0.12;
    pins("cone", Curved::Cone, &at, &gap, (0.0, du, 12), (10.0, 5.0 * du * 0.866, 12));
}

#[test]
fn a_sphere_face_reports_no_less_than_its_facets_stand_off() {
    let r = 10.0;
    let at = move |u: f64, v: f64| [r * v.cos() * u.cos(), r * v.cos() * u.sin(), r * v.sin()];
    let gap = move |p: [f64; 3]| (r - (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()).abs();
    let du = 0.0775;
    pins("sphere", Curved::Sphere, &at, &gap, (0.0, du, 12), (-0.3, du * 0.866, 12));
}

#[test]
fn a_torus_face_reports_no_less_than_its_facets_stand_off() {
    let (big, small) = (20.0, 5.0);
    let at = move |u: f64, v: f64| {
        let w = big + small * v.cos();
        [w * u.cos(), w * u.sin(), small * v.sin()]
    };
    let gap = move |p: [f64; 3]| (small - ((rho(p) - big).powi(2) + p[2] * p[2]).sqrt()).abs();
    let a = 0.63;
    // The outer side (curved two ways alike) and the inner side (a saddle).
    pins("torus outer", Curved::Torus, &at, &gap, (0.0, a / 25.0, 12), (-0.3, a * 0.866 / small, 12));
    pins("torus inner", Curved::Torus, &at, &gap, (0.0, a / 15.0, 12), (std::f64::consts::PI - 0.3, a * 0.866 / small, 12));
}
