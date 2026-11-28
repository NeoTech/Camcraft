//! #3643: tool component bodies from STEP, STL and OBJ files - placed by
//! their own mounting frame (MCS, MCS_INSERT), judged never smaller than
//! the facets, resolved in a spindle stack, and refused by name on
//! degenerate, corrupt or over-budget input, never with a panic. Every
//! fixture is made here or is our own STEP test model.

use crate::component::{ComponentKind, CuttingItem, ToolComponent};
use crate::component_geometry::GeometryError;
use crate::connection_code::ConnectionCode;
use crate::frame::Frame;
use crate::holder_profile::{EnvelopeError, HolderProfile};
use crate::lathe_holder::LatheClearance;
use crate::mesh_body::{MeshBody, MeshBodyError, MeshFormat};
use crate::mesh_file::{read_obj, read_stl, MeshFileError, Tri, MAX_BODY_FACETS};
use crate::tool_assembly::{resolve, AssemblyNode, Mount};
use crate::tools::{catalog, Tool, ToolKind};

fn frame(origin: [f64; 3], z: [f64; 3], x: [f64; 3]) -> Frame {
    Frame::new(origin, z, x).unwrap_or_else(|e| panic!("{e}"))
}

/// An `n`-sided prism about the Z axis, corners `r` from it, from `z0`
/// up to `z1`, closed at both ends, each point mapped by `place`.
pub(crate) fn prism(n: usize, r: f64, z0: f64, z1: f64, place: &Frame) -> Vec<Tri> {
    let c = |k: usize, z: f64| {
        let a = std::f64::consts::TAU * (k % n) as f64 / n as f64;
        place.map_point([r * a.cos(), r * a.sin(), z])
    };
    let (b, t) = (place.map_point([0.0, 0.0, z0]), place.map_point([0.0, 0.0, z1]));
    let mut tris = Vec::new();
    for k in 0..n {
        tris.push([c(k, z0), c(k + 1, z0), c(k + 1, z1)]);
        tris.push([c(k, z0), c(k + 1, z1), c(k, z1)]);
        tris.push([b, c(k + 1, z0), c(k, z0)]);
        tris.push([t, c(k, z1), c(k + 1, z1)]);
    }
    tris
}

fn ascii_stl(tris: &[Tri]) -> String {
    let mut s = String::from("solid made_here\n");
    for t in tris {
        s.push_str("  facet normal 0 0 0\n    outer loop\n");
        for p in t {
            s.push_str(&format!("      vertex {} {} {}\n", p[0], p[1], p[2]));
        }
        s.push_str("    endloop\n  endfacet\n");
    }
    s.push_str("endsolid made_here\n");
    s
}

fn binary_stl(tris: &[Tri]) -> Vec<u8> {
    let mut b = b"solid but binary".to_vec();
    b.resize(80, b' ');
    b.extend_from_slice(&u32::try_from(tris.len()).unwrap_or_else(|_| panic!("too many")).to_le_bytes());
    for t in tris {
        b.extend_from_slice(&[0u8; 12]);
        for v in t.iter().flatten() {
            b.extend_from_slice(&(*v as f32).to_le_bytes());
        }
        b.extend_from_slice(&[0u8; 2]);
    }
    b
}

fn obj(tris: &[Tri]) -> String {
    let mut s = String::from("# made here\no body\n");
    for p in tris.iter().flatten() {
        s.push_str(&format!("v {} {} {}\n", p[0], p[1], p[2]));
    }
    for k in 0..tris.len() {
        // Mixed index forms: plain, v/vt and negative.
        let n = 3 * tris.len();
        s.push_str(&format!("f {} {}/1 {}\n", 3 * k + 1, 3 * k + 2, 3 * k as i64 + 3 - n as i64 - 1));
    }
    s
}

fn step_fixture() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models/tool_csys_box_iso13399.step");
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("fixture: {e}")).replace("\r\n", "\n")
}

/// Our STEP box (20 x 20 mm, 40 mm tall, z -40..0) with its MCS moved to
/// the top face and its one port, `CSW`, on the axis at the bottom face.
pub(crate) fn step_holder(mount_name: &str) -> String {
    let swap = |s: String, a: &str, b: &str| {
        assert!(s.contains(a), "the fixture has {a}");
        s.replacen(a, b, 1)
    };
    let s = swap(step_fixture(), "#211 = CARTESIAN_POINT('', (0.0, 0.0, -40.0));", "#211 = CARTESIAN_POINT('', (0.0, 0.0, 0.0));");
    let s = swap(s, "AXIS2_PLACEMENT_3D('CSW1_2',", "AXIS2_PLACEMENT_3D('CSW',");
    let s = swap(s, "#231 = CARTESIAN_POINT('', (10.0, 0.0, -20.0));", "#231 = CARTESIAN_POINT('', (0.0, 0.0, -40.0));");
    swap(s, "AXIS2_PLACEMENT_3D('MCS',", &format!("AXIS2_PLACEMENT_3D('{mount_name}',"))
}

fn refused<T>(r: Result<T, MeshBodyError>) -> MeshBodyError {
    match r {
        Ok(_) => panic!("accepted a {}", std::any::type_name::<T>()),
        Err(e) => e,
    }
}

/// A body read from an STL with stored frames - its MCS in the file
/// upside down and off the origin - sits on its own mounting frame: in
/// a stack with a solid end mill it gives the gauge length and the band
/// of the plain cylinder it stands for.
#[test]
fn an_stl_holder_with_stored_frames_resolves_in_a_spindle_stack() {
    let (r, len) = (16.0, 60.0);
    let mcs = frame([5.0, 5.0, 100.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0]);
    let csw = mcs.then(&Frame::at_z(-len).unwrap_or_else(|e| panic!("{e}")));
    let tris = prism(8, r, -len, 0.0, &mcs);
    let body = MeshBody::read(MeshFormat::Stl, ascii_stl(&tris).as_bytes())
        .and_then(|b| b.with_frames(vec![("MCS".into(), mcs), (" csw ".into(), csw)]))
        .unwrap_or_else(|e| panic!("{e}"));
    let g = body.spindle_geometry().unwrap_or_else(|e| panic!("{e}"));
    let port = g.port("CSW").unwrap_or_else(|| panic!("no CSW: {:?}", g.ports()));
    assert!(port.origin().iter().zip([0.0, 0.0, -len]).all(|(a, b)| (a - b).abs() < 1e-9), "{:?}", port.origin());
    assert!((port.z()[2] - 1.0).abs() < 1e-12, "the port's +Z points to the machine");

    let tool = catalog()
        .into_iter()
        .map(|(_, t)| t)
        .find(|t| t.kind == ToolKind::EndMill && t.corner_r == 0.0)
        .map(|t| Tool { diameter: 10.0, shank_d: 10.0, ..t })
        .unwrap_or_else(|| panic!("no flat end mill"));
    let code = |s: &str| ConnectionCode::parse(s).unwrap_or_else(|e| panic!("{e}"));
    let made = |id: &str, kind| ToolComponent::new(id, id, kind).unwrap_or_else(|e| panic!("{e}"));
    let p = tool.flute_len + 10.0;
    let nodes = [
        AssemblyNode {
            component: made("STL holder", ComponentKind::AdaptiveItem { mcs: code("BT30"), csw: code("CYL 10") }),
            geometry: g,
            mount: None,
        },
        AssemblyNode {
            component: made(
                "end mill",
                ComponentKind::CuttingItem(CuttingItem::Solid { tool: Box::new(tool.clone()), mcs: code("CYL 10") }),
            ),
            geometry: Default::default(),
            mount: Some(Mount { parent: 0, port: None, z_rot_deg: 0.0, projection_mm: Some(p) }),
        },
    ];
    let a = resolve(&nodes).unwrap_or_else(|e| panic!("{e}"));
    assert!((a.gauge_len - (p + len)).abs() < 1e-9, "gauge {}", a.gauge_len);
    let bands = a.body.unwrap_or_else(|| panic!("no body")).profile.bands().to_vec();
    assert_eq!(bands.len(), 1, "{bands:?}");
    let b = bands[0];
    assert!((b.above - p).abs() < 1e-9 && (b.len - len).abs() < 1e-9, "{b:?}");
    // The prism's corners stand r from the axis: the band is r, never less.
    assert!(b.r >= r && b.r - r < 1e-5, "{b:?}");
}

#[test]
fn ascii_and_binary_stl_and_obj_read_the_same_facets() {
    let tris = prism(6, 12.5, -30.0, 0.0, &Frame::IDENTITY);
    // Round the corners to f32, as a binary STL holds them.
    let tris: Vec<Tri> = tris.iter().map(|t| t.map(|p| p.map(|v| f64::from(v as f32)))).collect();
    let ascii = read_stl(ascii_stl(&tris).as_bytes()).unwrap_or_else(|e| panic!("{e}"));
    let binary = read_stl(&binary_stl(&tris)).unwrap_or_else(|e| panic!("{e}"));
    let from_obj = read_obj(obj(&tris).as_bytes()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(ascii, tris);
    assert_eq!(binary, tris);
    assert_eq!(from_obj, tris);
    // A quad face is split as a fan.
    let quad = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
    assert_eq!(read_obj(quad.as_bytes()).map(|t| t.len()), Ok(2));
    assert_eq!(MeshFormat::from_extension(".STP"), Some(MeshFormat::Step));
    assert_eq!(MeshFormat::from_extension("stl"), Some(MeshFormat::Stl));
    assert_eq!(MeshFormat::from_extension("Obj"), Some(MeshFormat::Obj));
    assert_eq!(MeshFormat::from_extension("iges"), None);
}

/// A vendor-style STEP places by its own frames: the MCS (or an
/// insert's MCS_INSERT) is the origin, the CSW port and the CIP are read
/// in it, and the body is never smaller than the box plus the certified
/// chord deviation.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_step_body_places_by_its_own_iso_13399_frames() {
    for mount in ["MCS", "MCS_INSERT"] {
        let body = MeshBody::read(MeshFormat::Step, step_holder(mount).as_bytes()).unwrap_or_else(|e| panic!("{e}"));
        let dev = body.deviation_mm();
        assert!((0.0..=0.01).contains(&dev), "deviation {dev}");
        let parts = body.parts().unwrap_or_else(|e| panic!("{mount}: {e}"));
        let names: Vec<&str> = parts.ports.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["CSW"]);
        assert!((parts.ports[0].1.origin()[2] + 40.0).abs() < 1e-9);
        let cip = parts.cip.unwrap_or_else(|| panic!("no CIP"));
        assert!((cip.origin()[2] + 75.0).abs() < 1e-9, "{:?}", cip.origin());

        let g = body.spindle_geometry().unwrap_or_else(|e| panic!("{mount}: {e}"));
        let b = g.body().unwrap_or_else(|| panic!("no body"));
        assert!((b.base() - (-40.0 - dev)).abs() < 1e-9, "base {}", b.base());
        let corner = 10.0 * std::f64::consts::SQRT_2;
        let reach = b.profile().max_radius();
        assert!(reach >= corner + dev && reach < corner + dev + 1e-5, "reach {reach}");

        let (lg, lathe) = body.lathe_geometry().unwrap_or_else(|e| panic!("{mount}: {e}"));
        assert!(lg.body().is_none(), "a lathe body is no revolved profile");
        assert!(lathe.facets().len() >= 12 && lathe.pad() == dev);
    }
}

/// The lathe bands of a meshed body grow by its pad in every direction,
/// so they still hold every point within the pad of a facet.
#[test]
fn lathe_bands_grow_by_the_pad() {
    let tris = prism(4, 8.0, 2.0, 30.0, &frame([12.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]));
    let base = LatheClearance::of(&tris).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(base.padded(0.0), base);
    for d in [0.004, 0.1, 0.7] {
        let grown = base.padded(d);
        assert!((grown.z_lo() - (base.z_lo() - d)).abs() < 1e-9 && (grown.z_hi() - (base.z_hi() + d)).abs() < 1e-9);
        for p in tris.iter().flatten() {
            for q in [[p[0] + d, p[1], p[2] + d], [p[0] - d, p[1] - d, p[2] - d], [p[0], p[1] + d, p[2]]] {
                let r = grown.over(q[2] - 1e-9, q[2] + 1e-9).unwrap_or_else(|| panic!("no band at {q:?}, d {d}"));
                assert!(r.x_lo <= q[0] + 1e-9 && q[0] <= r.x_hi + 1e-9, "{q:?} outside {r:?}");
                assert!(q[1].abs() <= r.y_hi + 1e-9, "{q:?} outside {r:?}");
            }
        }
    }
}

/// The drawn outline only grows the envelope: many steps (more than an
/// outline holds), steps finer than a drawn segment, and a gap along the
/// axis are all enclosed.
#[test]
fn the_drawn_outline_never_falls_inside_the_envelope() {
    let mut tris = Vec::new();
    for k in 0..48 {
        let z = -2.0 * (k + 1) as f64;
        let r = 10.0 + if k % 2 == 0 { 3.0 } else { 0.005 * k as f64 };
        tris.extend(prism(12, r, z, z + 2.0, &Frame::IDENTITY));
    }
    // A second piece below a 5 mm gap.
    tris.extend(prism(12, 6.0, -120.0, -101.0, &Frame::IDENTITY));
    let body = MeshBody::from_facets(tris.clone())
        .and_then(|b| b.with_frames(vec![("MCS".into(), Frame::IDENTITY)]))
        .unwrap_or_else(|e| panic!("{e}"));
    let g = body.spindle_geometry().unwrap_or_else(|e| panic!("{e}"));
    let b = g.body().unwrap_or_else(|| panic!("no body"));
    let drawn = b.profile().bands();
    let lifted: Vec<Tri> = tris.iter().map(|t| t.map(|p| [p[0], p[1], p[2] - b.base()])).collect();
    let env = HolderProfile::envelope(&lifted).unwrap_or_else(|e| panic!("{e}"));
    for e in env.bands() {
        let mid = e.above + 0.5 * e.len;
        let r = drawn.iter().filter(|d| d.above <= mid && mid <= d.above + d.len).map(|d| d.r).fold(0.0, f64::max);
        assert!(r >= e.r - 1e-9, "envelope band {e:?} reaches past the outline ({r})");
    }
}

#[test]
fn a_missing_doubled_or_upside_down_frame_is_refused_by_name() {
    let tris = prism(8, 10.0, -50.0, 0.0, &Frame::IDENTITY);
    let below = Frame::at_z(-50.0).unwrap_or_else(|e| panic!("{e}"));
    let above = Frame::at_z(20.0).unwrap_or_else(|e| panic!("{e}"));
    let body = MeshBody::from_facets(tris.clone()).unwrap_or_else(|e| panic!("{e}"));
    let with = |frames: Vec<(&str, Frame)>| {
        body.clone().with_frames(frames.into_iter().map(|(n, f)| (n.to_owned(), f)).collect())
    };
    let e = refused(body.spindle_geometry());
    assert_eq!(e, MeshBodyError::NoMount { found: "it names no coordinate systems".into() });
    assert!(e.to_string().contains("MCS or MCS_INSERT"), "{e}");
    let e = refused(with(vec![("MCS", Frame::IDENTITY), ("MCS_INSERT", below)]).and_then(|b| b.parts()));
    assert_eq!(e, MeshBodyError::TwoMounts(vec!["MCS".into(), "MCS_INSERT".into()]));
    let e = refused(with(vec![("MCS", Frame::IDENTITY), ("CSW", above)]).and_then(|b| b.parts()));
    assert_eq!(e, MeshBodyError::NotBelowMount { name: "CSW".into(), z: 20.0 });
    let e = refused(with(vec![("MCS", Frame::IDENTITY), ("CIP", below), ("cip", below)]).and_then(|b| b.parts()));
    assert_eq!(e, MeshBodyError::TwoCutPoints(vec!["CIP".into(), "CIP".into()]));
    let e = refused(with(vec![("MCS", Frame::IDENTITY), ("CSW", below), ("csw", below)]).and_then(|b| b.spindle_geometry()));
    assert_eq!(e, MeshBodyError::Geometry(GeometryError::DuplicatePort("CSW".into())));
    assert_eq!(refused(with(vec![("  ", Frame::IDENTITY)])), MeshBodyError::EmptyFrameName);
    // A body beside the axis is no spindle body.
    let beside = prism(8, 3.0, -50.0, 0.0, &frame([20.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]));
    let e = refused(
        MeshBody::from_facets(beside)
            .and_then(|b| b.with_frames(vec![("MCS".into(), Frame::IDENTITY)]))
            .and_then(|b| b.spindle_geometry()),
    );
    assert_eq!(e, MeshBodyError::Envelope(EnvelopeError::OffAxis));
}

#[test]
fn degenerate_and_over_budget_files_are_refused_by_name() {
    let file = |e: MeshBodyError| match e {
        MeshBodyError::File(f) => f,
        other => panic!("not a file refusal: {other}"),
    };
    let stl = |s: &str| file(refused(MeshBody::read(MeshFormat::Stl, s.as_bytes())));
    let objf = |s: &str| file(refused(MeshBody::read(MeshFormat::Obj, s.as_bytes())));
    assert_eq!(file(refused(MeshBody::read(MeshFormat::Stl, b""))), MeshFileError::Empty);
    assert_eq!(stl("solid empty\nendsolid empty\n"), MeshFileError::NoFacets);
    assert_eq!(stl("solid x\nfacet normal 0 0 1\nouter loop\nvertex 0 0 nan\n"), MeshFileError::NonFinite { line: 4 });
    assert!(matches!(stl("solid x\nfacet\nvertex 0 0 0\nvertex 1 0 0\nendfacet\n"), MeshFileError::Syntax { line: 5, .. }));
    assert!(matches!(stl("solid x\nvertex 0 0 0\n"), MeshFileError::Syntax { line: 2, .. }));
    assert!(matches!(stl("solid x\nfacet\nvertex 0 0 0\n"), MeshFileError::Syntax { .. }));
    assert!(matches!(stl("solid x\nfacet\nvertex 0 zero 0\n"), MeshFileError::Syntax { line: 3, .. }));
    let mut cut = binary_stl(&prism(3, 1.0, 0.0, 1.0, &Frame::IDENTITY));
    cut.truncate(cut.len() - 7);
    assert!(matches!(file(refused(MeshBody::read(MeshFormat::Stl, &cut))), MeshFileError::BinaryLength { facets: 12, .. }));
    assert_eq!(objf("v 0 0 0\nv 1 0 0\nf 1 2 9\n"), MeshFileError::BadIndex { line: 3, word: "9".into() });
    assert_eq!(objf("v 0 0 0\nv 1 0 0\nf 1 2 0\n"), MeshFileError::BadIndex { line: 3, word: "0".into() });
    assert_eq!(objf("v 0 0 0\nv 1 0 0\nf 1 2 -3\n"), MeshFileError::BadIndex { line: 3, word: "-3".into() });
    assert!(matches!(objf("v 0 0 0\nv 1 0 0\nf 1 2\n"), MeshFileError::Syntax { line: 3, .. }));
    assert_eq!(objf("v 0 0 inf\n"), MeshFileError::NonFinite { line: 1 });
    assert_eq!(objf("v 0 0 0\n"), MeshFileError::NoFacets);
    assert_eq!(file(refused(MeshBody::read(MeshFormat::Obj, &[0xff, 0xfe, b'v']))), MeshFileError::NotText);
    // Every facet of zero area: refused, not judged as a thin body.
    let flat = vec![[[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, -5.0]]; 3];
    let e = refused(
        MeshBody::from_facets(flat)
            .and_then(|b| b.with_frames(vec![("MCS".into(), Frame::IDENTITY)]))
            .and_then(|b| b.spindle_geometry()),
    );
    assert_eq!(e, MeshBodyError::Envelope(EnvelopeError::Degenerate));
    // Over the facet budget, by name: as facets, and as one OBJ fan.
    let t = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let e = file(refused(MeshBody::from_facets(vec![t; MAX_BODY_FACETS + 1])));
    assert_eq!(e, MeshFileError::OverBudget { facets: MAX_BODY_FACETS + 1 });
    assert!(e.to_string().contains("#1041"), "{e}");
    let mut fan = String::from("v 0 0 0\nv 1 0 0\nv 0 1 0\nf");
    // A fan of n corners is n - 2 facets: one past the budget.
    for _ in 0..(MAX_BODY_FACETS + 3) {
        fan.push_str(" 1");
    }
    assert_eq!(objf(&fan), MeshFileError::OverBudget { facets: MAX_BODY_FACETS + 1 });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_corrupt_step_body_is_refused_by_name() {
    for bad in ["", "not a step file", "ISO-10303-21;\nHEADER;\nENDSEC;\nDATA;\nENDSEC;\nEND-ISO-10303-21;\n"] {
        let e = refused(MeshBody::read(MeshFormat::Step, bad.as_bytes()));
        assert!(!e.to_string().is_empty(), "{bad:?}");
    }
    assert!(matches!(refused(MeshBody::read(MeshFormat::Step, &[0xc3, 0x28])), MeshBodyError::Step(_)));
    // A good model with no mounting frame reads, and is refused when placed.
    let no_mount = step_holder("PCS_2");
    let body = MeshBody::read(MeshFormat::Step, no_mount.as_bytes()).unwrap_or_else(|e| panic!("{e}"));
    assert!(matches!(refused(body.parts()), MeshBodyError::NoMount { .. }));
}

/// A small fuzz pass: cut short and bit-flipped STL, OBJ and STEP files
/// are read or refused, never a panic or a hang.
#[test]
fn mangled_files_never_panic() {
    let tris = prism(5, 4.0, -9.0, 0.0, &Frame::IDENTITY);
    let seeds: [(MeshFormat, Vec<u8>); 3] = [
        (MeshFormat::Stl, ascii_stl(&tris).into_bytes()),
        (MeshFormat::Stl, binary_stl(&tris)),
        (MeshFormat::Obj, obj(&tris).into_bytes()),
    ];
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let place = |b: MeshBody| b.with_frames(vec![("MCS".into(), Frame::IDENTITY)]).and_then(|b| b.spindle_geometry());
    for (format, seed) in &seeds {
        for _ in 0..120 {
            let mut m = seed.clone();
            let cut = (next() as usize) % (m.len() + 1);
            if next() % 2 == 0 {
                m.truncate(cut);
            }
            for _ in 0..(next() % 6) {
                if m.is_empty() {
                    break;
                }
                let at = (next() as usize) % m.len();
                m[at] ^= 1 << (next() % 8);
            }
            // Either result is fine; the call returns.
            let _ = MeshBody::read(*format, &m).and_then(place);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let step = step_holder("MCS").into_bytes();
        for k in 1..=6 {
            let mut m = step.clone();
            m.truncate(step.len() * k / 7);
            let _ = MeshBody::read(MeshFormat::Step, &m).and_then(|b| b.spindle_geometry());
        }
    }
}
