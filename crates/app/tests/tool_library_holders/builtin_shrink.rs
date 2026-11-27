//! #3919: the holder library's shrink-fit chucks as built-in components
//! (`component_library::builtins`). Pinned: each holder entry with a
//! taper is a built-in holder, a shrink-fit chuck's workpiece side is a
//! plain bore of its own size (no collet), a solid tool of that shank mounts in it and a
//! tool of another shank is refused by name, and its geometry is the
//! holder library's own outline at the library gauge length - never
//! smaller than the steel. The SK50 holders and ER50 chucks of slice 3
//! take the same conversion. Provenance: our own tests over our own
//! holder table and commands.

use camcraft_app::commands::{AddTool, History, SetToolAssembly, UpsertDocumentComponent};
use camcraft_app::component_library::{self as lib, LibraryComponent};
use camcraft_app::model;
use camcraft_app::tool_body::body_of;
use camcraft_tools::component::{ADAPTIVE_PORT, ComponentKind, CuttingItem};
use camcraft_tools::component_geometry::ComponentGeometry;
use camcraft_tools::connection_code::ConnectionCode;
use camcraft_tools::holder_library::{self, Clamp};
use camcraft_tools::tool_assembly::Mount;

use super::assembly_cmds::{code, link};

const CHUCK: &str = "BT40 Shrink 12 mm";

fn builtin(id: &str) -> LibraryComponent {
    lib::builtins().into_iter().find(|c| c.component.id() == id).unwrap_or_else(|| panic!("no built-in '{id}'"))
}

/// A built-in solid tool whose shank is (or is not) Ø12.
fn solid(shank_12: bool) -> LibraryComponent {
    lib::builtins()
        .into_iter()
        .find(|c| {
            matches!(c.component.kind(), ComponentKind::CuttingItem(CuttingItem::Solid { mcs: ConnectionCode::Cylindrical { d }, .. })
                if ((*d - 12.0).abs() < 1e-9) == shank_12)
        })
        .expect("a built-in solid tool")
}

#[test]
fn every_holder_with_a_taper_is_a_built_in_and_a_shrink_fit_bore_is_plain() {
    let all = lib::builtins();
    let mut n = 0;
    for e in holder_library::library() {
        let Some(taper) = e.taper else { continue };
        let id = format!("builtin:holder:{}", e.name());
        // No entry with a taper drops out of the catalogue in silence.
        // ISO30 (the router name of DIN 69871 size 30, the SK30 steel)
        // is the one library taper that is not a connection code yet, so
        // its entries are not built-ins (main since #4233); a new taper
        // outside the table fails here.
        if ConnectionCode::parse(taper.name).is_err() {
            assert_eq!(taper.name, "ISO30", "'{id}': its taper is not a connection code");
            assert!(all.iter().all(|c| c.component.id() != id), "{id}");
            continue;
        }
        let c = all.iter().find(|c| c.component.id() == id).unwrap_or_else(|| panic!("'{id}' is not a built-in"));
        let Clamp::Shrink { bore, .. } = e.clamp else { continue };
        let ComponentKind::AdaptiveItem { mcs, csw } = c.component.kind() else { panic!("'{id}' is not an adaptive item") };
        assert_eq!(*mcs, code(taper.name), "{id}");
        assert_eq!(*csw, ConnectionCode::Cylindrical { d: bore }, "{id}: a shrink-fit bore, no collet");
        n += 1;
    }
    assert!(n >= 10, "the library has shrink-fit chucks on tapers ({n})");
}

#[test]
fn a_built_in_shrink_fit_chuck_is_the_library_outline_at_the_library_gauge() {
    let entry = holder_library::find(CHUCK).expect("library entry");
    let Clamp::Shrink { nose_d, body_d, .. } = entry.clamp else { panic!("{CHUCK} is a shrink-fit chuck") };
    let c = builtin(&format!("builtin:holder:{CHUCK}"));
    let want = ComponentGeometry::holder(&entry, ADAPTIVE_PORT, entry.gauge_len).expect("library geometry");
    assert_eq!(c.geometry, want);
    let port = c.geometry.port(ADAPTIVE_PORT).expect("the one port");
    assert!((port.origin()[2] + entry.gauge_len).abs() < 1e-9, "the nose face is the gauge length below the gauge line");
    let body = c.geometry.body().expect("a body");
    assert!(body.profile().max_radius() >= body_d * 0.5 && body.profile().max_radius() >= nose_d * 0.5);
}

#[test]
fn a_built_in_shrink_fit_chuck_takes_its_own_shank_and_refuses_another() {
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    let chuck = builtin(&format!("builtin:holder:{CHUCK}"));
    let (fits, other) = (solid(true), solid(false));
    for c in [&chuck, &fits, &other] {
        h.execute(&w, UpsertDocumentComponent::new(&w, c.component.clone(), c.geometry.clone()).expect("upsert"));
    }
    let stickout = Some(Mount { parent: 0, port: None, z_rot_deg: 0.0, projection_mm: Some(40.0) });
    for (name, c) in [("Fits", &fits), ("Other", &other)] {
        let ComponentKind::CuttingItem(CuttingItem::Solid { tool, .. }) = c.component.kind() else { panic!("solid") };
        h.execute(&w, AddTool::new(&w, name, (**tool).clone()).expect("tool"));
    }
    let stack = |c: &LibraryComponent| vec![link(chuck.component.id(), None), link(c.component.id(), stickout.clone())];
    h.execute(&w, SetToolAssembly::new(&w, "Fits", stack(&fits)).expect("a Ø12 shank in a Ø12 shrink-fit bore"));
    let e = model::find_by_name(&w, "Fits").expect("tool");
    assert!(body_of(&w, &e).is_some(), "the chuck outline is the tool's body");
    let Err(err) = SetToolAssembly::new(&w, "Other", stack(&other)) else { panic!("another shank is refused") };
    assert!(err.to_string().contains("Other"), "refused by name: {err}");
}

/// #3919 slice 3: the SK50 entries (ER and Weldon, with the ER50 chucks)
/// take the same conversion: SK50 on the machine side, the collet or
/// the Weldon bore on the workpiece side.
#[test]
fn the_sk50_holders_and_the_er50_chucks_are_built_ins() {
    let sk50: Vec<LibraryComponent> =
        lib::builtins().into_iter().filter(|c| c.component.id().starts_with("builtin:holder:SK50 ")).collect();
    assert!(!sk50.is_empty(), "SK50 built-ins");
    let sides = |c: &LibraryComponent| match c.component.kind() {
        ComponentKind::AdaptiveItem { mcs, csw } => (mcs.clone(), csw.clone()),
        _ => panic!("'{}' is not an adaptive item", c.component.id()),
    };
    assert!(sk50.iter().all(|c| sides(c).0 == code("SK50")));
    assert!(sk50.iter().any(|c| sides(c).1 == code("ER50")), "an SK50 ER50 chuck");
    assert!(sk50.iter().any(|c| matches!(sides(c).1, ConnectionCode::Weldon { .. })), "an SK50 Weldon holder");
}
