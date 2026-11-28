//! #3898: the built-in ER collets. Pinned: each series of the holder
//! library's ER table has one collet for each whole millimetre of its
//! range; a built-in ER chuck takes only the collets of its own series;
//! a collet takes only the straight shanks of its bore; and a built-in
//! chuck, collet and solid tool resolve to a tool with the gauge length
//! of the chuck plus the stick-out. Provenance: our own tests over our
//! own catalogue.

use camcraft_app::component_fit::{compatible_children, compatible_parents};
use camcraft_app::component_library::{self as lib, LibraryComponent};
use camcraft_app::schema::{AssemblyLink, DocumentComponent};
use camcraft_app::tool_assembly::{Mount, derive};
use camcraft_tools::component::{ComponentKind, CuttingItem, ToolComponent};

const CHUCK: &str = "builtin:holder:BT40 ER32";
const COLLET: &str = "builtin:collet:ER32 12";
const TOOL: &str = "builtin:tool:EM-1200-2FL";

fn get<'a>(all: &'a [LibraryComponent], id: &str) -> &'a LibraryComponent {
    all.iter().find(|c| c.component.id() == id).unwrap_or_else(|| panic!("no built-in {id}"))
}

fn pool(all: &[LibraryComponent]) -> Vec<&ToolComponent> {
    all.iter().map(|c| &c.component).collect()
}

#[test]
fn each_er_series_has_a_collet_for_each_whole_millimetre() {
    let all = lib::builtins();
    for (series, first, last) in [("ER11", 1, 7), ("ER16", 1, 10), ("ER20", 1, 13), ("ER25", 1, 16), ("ER32", 2, 20), ("ER40", 3, 26)] {
        let bores: Vec<u32> = all
            .iter()
            .filter_map(|c| c.component.id().strip_prefix(&format!("builtin:collet:{series} ")).map(str::to_owned))
            .map(|d| d.parse().unwrap_or_else(|e| panic!("{series} {d}: {e}")))
            .collect();
        assert_eq!(bores, (first..=last).collect::<Vec<u32>>(), "{series}");
    }
    let c = get(&all, COLLET);
    assert_eq!(c.component.name(), "ER32 collet 12 mm");
    assert!(c.is_builtin());
    assert!(matches!(c.component.kind(), ComponentKind::AdaptiveItem { .. }));
}

#[test]
fn a_built_in_er_chuck_takes_its_collets_and_a_collet_takes_its_shank() {
    let all = lib::builtins();
    let comps = pool(&all);
    let children: Vec<&str> = compatible_children(&get(&all, CHUCK).component, comps.iter().copied()).into_iter().map(|f| f.component.id()).collect();
    assert!(children.contains(&COLLET), "{children:?}");
    assert!(children.iter().all(|id| id.starts_with("builtin:collet:ER32 ")), "only the ER32 collets: {children:?}");
    assert_eq!(children.len(), 19);

    let held: Vec<&str> = compatible_children(&get(&all, COLLET).component, comps.iter().copied()).into_iter().map(|f| f.component.id()).collect();
    assert!(held.contains(&TOOL), "{held:?}");
    for id in &held {
        let ComponentKind::CuttingItem(CuttingItem::Solid { tool, .. }) = get(&all, id).component.kind() else { panic!("{id} is not a solid tool") };
        assert!((tool.shank_d - 12.0).abs() < 1e-9, "{id}: shank {}", tool.shank_d);
    }

    // "Show compatible parts" on a built-in tool now finds its collets.
    let parents: Vec<&str> = compatible_parents(&get(&all, TOOL).component, comps.iter().copied()).into_iter().map(|f| f.component.id()).collect();
    for series in ["ER20", "ER25", "ER32", "ER40"] {
        assert!(parents.contains(&format!("builtin:collet:{series} 12").as_str()), "{series}: {parents:?}");
    }
    assert!(!parents.iter().any(|id| id.starts_with("builtin:collet:ER11 ") || id.starts_with("builtin:collet:ER16 ")), "{parents:?}");
}

#[test]
fn a_built_in_chuck_collet_and_tool_resolve_to_the_tool_with_its_gauge() {
    let all = lib::builtins();
    let table: Vec<DocumentComponent> = [CHUCK, COLLET, TOOL]
        .iter()
        .map(|id| {
            let c = get(&all, id);
            DocumentComponent::new(c.component.clone(), c.geometry.clone())
        })
        .collect();
    let ComponentKind::CuttingItem(CuttingItem::Solid { tool, .. }) = get(&all, TOOL).component.kind() else { panic!("solid") };
    let stickout = tool.flute_len + 10.0;
    let links = vec![
        AssemblyLink { component: CHUCK.into(), mount: None },
        AssemblyLink { component: COLLET.into(), mount: Some(Mount { parent: 0, port: None, z_rot_deg: 0.0, projection_mm: None }) },
        AssemblyLink { component: TOOL.into(), mount: Some(Mount { parent: 1, port: None, z_rot_deg: 0.0, projection_mm: Some(stickout) }) },
    ];
    let d = derive("T1", tool, &links, &table).unwrap_or_else(|e| panic!("refused: {e}"));
    let chuck_gauge = camcraft_tools::holder_library::find("BT40 ER32").unwrap_or_else(|e| panic!("{e}")).gauge_len;
    assert!((d.gauge_len - (chuck_gauge + stickout)).abs() < 1e-9, "gauge {}", d.gauge_len);
    assert_eq!((d.tool.diameter, d.tool.corner_r), (tool.diameter, tool.corner_r));
    assert!(d.body.is_some(), "the chuck's body is judged");
}
