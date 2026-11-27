//! #3642: tool components - made and checked, fitted insert to seat
//! and shank to coupling, queried both ways, and refused by name.

use crate::component::{ComponentError, ComponentKind, CuttingItem, InsertSeat, ToolComponent};
use crate::component_fit::{
    check_coupling_fit, check_insert_fit, compatible_children, compatible_parents, FitError, FitProperty,
};
use crate::connection_code::ConnectionCode;
use crate::insert_iso1832::{InsertSpec, Thickness};
use crate::insert_seat::HolderDesignation;
use crate::tools::{catalog, Tool};

fn made(id: &str, kind: ComponentKind) -> ToolComponent {
    match ToolComponent::new(id, id, kind) {
        Ok(c) => c,
        Err(e) => panic!("{id}: {e}"),
    }
}

fn insert(code: &str) -> ToolComponent {
    let Ok(spec) = InsertSpec::parse(code) else { panic!("{code}") };
    made(code, ComponentKind::CuttingItem(CuttingItem::Insert(spec)))
}

/// A turning holder from its ISO 5608 code: a rectangular shank and
/// one seat, `CSW1`, of the stated thickness.
fn holder(code: &str, thickness: Option<&str>) -> ToolComponent {
    let Ok(h) = HolderDesignation::parse(code) else { panic!("{code}") };
    let mut spec = h.seat;
    spec.thickness = thickness.and_then(Thickness::from_code);
    let mcs = ConnectionCode::RectShank { h: u16::from(h.shank_h), b: u16::from(h.shank_b) };
    made(code, ComponentKind::ToolItem { mcs, seats: vec![InsertSeat { port: "CSW1".into(), spec }] })
}

fn code(text: &str) -> ConnectionCode {
    let Ok(c) = ConnectionCode::parse(text) else { panic!("{text}") };
    c
}

fn adaptive(id: &str, mcs: &str, csw: &str) -> ToolComponent {
    made(id, ComponentKind::AdaptiveItem { mcs: code(mcs), csw: code(csw) })
}

fn end_mill() -> Tool {
    let Some((_, t)) = catalog().into_iter().next() else { panic!("the catalog is empty") };
    t
}

fn solid(id: &str, tool: Tool, mcs: ConnectionCode) -> Result<ToolComponent, ComponentError> {
    ToolComponent::new(id, id, ComponentKind::CuttingItem(CuttingItem::Solid { tool: Box::new(tool), mcs }))
}

fn refusal(r: Result<(), FitError>) -> FitError {
    match r {
        Ok(()) => panic!("the fit was accepted"),
        Err(e) => e,
    }
}

#[test]
fn inserts_fit_the_holders_made_for_them() {
    let pairs = [
        ("PCLNR2525M12", "CNMG120408"),
        ("DDJNR2525M15", "DNMG150608"),
        ("SCLCR2020K09", "CCMT09T304"),
        ("SVJBL2020K16", "VBMT160404"),
        ("CSSPR2525M12", "SPKN1203EDR"),
    ];
    for (h, i) in pairs {
        assert_eq!(check_insert_fit(&holder(h, None), "CSW1", &insert(i)), Ok(()), "{i} in {h}");
    }
    assert_eq!(check_insert_fit(&holder("PCLNR2525M12", Some("04")), "CSW1", &insert("CNMG120408")), Ok(()));
}

#[test]
fn insert_mismatches_are_refused_by_name() {
    let pclnr = holder("PCLNR2525M12", None);
    let cases = [
        (&pclnr, "DNMG150608", FitProperty::Shape, "C", "D"),
        (&pclnr, "CCMT09T304", FitProperty::Clearance, "N", "C"),
        (&pclnr, "CNMG090308", FitProperty::Size, "12", "09"),
        (&pclnr, "CNMN120408", FitProperty::Fixing, "a cylindrical hole (clamping P)", "no hole (fixing N)"),
    ];
    for (h, i, property, expected, actual) in cases {
        let e = refusal(check_insert_fit(h, "CSW1", &insert(i)));
        assert_eq!((e.property, e.expected.as_str(), e.actual.as_str()), (property, expected, actual), "{i}");
        assert_eq!((e.parent.as_str(), e.child.as_str()), ("PCLNR2525M12", i));
    }
    let thick = refusal(check_insert_fit(&holder("PCLNR2525M12", Some("04")), "CSW1", &insert("CNMG120608")));
    assert_eq!((thick.property, thick.expected.as_str(), thick.actual.as_str()), (FitProperty::Thickness, "04", "06"));
    let hand = refusal(check_insert_fit(&holder("CSSPR2525M12", None), "CSW1", &insert("SPKN1203EDL")));
    assert_eq!((hand.property, hand.expected.as_str(), hand.actual.as_str()), (FitProperty::Hand, "R", "L"));
    let text = hand.to_string();
    assert_eq!(text, "'SPKN1203EDL' does not fit 'CSSPR2525M12': hand - expected R, actual L");
    let port = refusal(check_insert_fit(&pclnr, "CSW9", &insert("CNMG120408")));
    assert_eq!((port.property, port.actual.as_str()), (FitProperty::Port, "CSW9"));
    let kind = refusal(check_insert_fit(&pclnr, "CSW1", &adaptive("ER32 collet 10", "ER32", "CYL 10")));
    assert_eq!(kind.property, FitProperty::Kind);
}

#[test]
fn couplings_match_by_connection_code() {
    let chuck = adaptive("BT40 ER32 chuck", "BT40", "ER32");
    let collet = adaptive("ER32 collet 4", "ER32", "CYL 4");
    let tool = end_mill();
    let mill = match solid("EM 4 mm shank", tool.clone(), ConnectionCode::Cylindrical { d: tool.shank_d }) {
        Ok(m) => m,
        Err(e) => panic!("{e}"),
    };
    assert_eq!(check_coupling_fit(&chuck, "CSW", &collet), Ok(()));
    assert_eq!(check_coupling_fit(&collet, "CSW", &mill), Ok(()));
    let er25 = adaptive("ER25 collet 4", "ER25", "CYL 4");
    let e = refusal(check_coupling_fit(&chuck, "CSW", &er25));
    assert_eq!((e.property, e.expected.as_str(), e.actual.as_str()), (FitProperty::Coupling, "ER32", "ER25"));
    assert_eq!((e.parent.as_str(), e.child.as_str()), ("BT40 ER32 chuck", "ER25 collet 4"));
    // A Weldon bore needs the flat; a plain bore takes a Weldon shank.
    let weldon_bore = adaptive("BT40 Weldon 12", "BT40", "WELDON 12");
    let plain_shank = adaptive("Extension 12", "CYL 12", "ER16");
    let flat_shank = adaptive("Extension 12 W", "WELDON 12", "ER16");
    let w = refusal(check_coupling_fit(&weldon_bore, "CSW", &plain_shank));
    assert_eq!((w.property, w.expected.as_str(), w.actual.as_str()), (FitProperty::Coupling, "WELDON 12", "CYL 12"));
    assert_eq!(check_coupling_fit(&weldon_bore, "CSW", &flat_shank), Ok(()));
    assert_eq!(check_coupling_fit(&adaptive("Collet 12", "ER25", "CYL 12"), "CSW", &flat_shank), Ok(()));
    // An insert has no machine side, and a seat is not a coupling.
    assert_eq!(refusal(check_coupling_fit(&chuck, "CSW", &insert("CNMG120408"))).property, FitProperty::Kind);
    let seat = refusal(check_coupling_fit(&holder("PCLNR2525M12", None), "CSW1", &collet));
    assert_eq!(seat.property, FitProperty::Kind);
    assert_eq!(refusal(check_coupling_fit(&chuck, "CSW2", &collet)).property, FitProperty::Port);
}

#[test]
fn the_library_queries_find_parents_and_children() {
    let library = vec![
        holder("PCLNR2525M12", None),
        holder("PCLNL2525M12", None),
        holder("DDJNR2525M15", None),
        holder("SCLCR2020K09", None),
        insert("CNMG120408"),
        insert("DNMG150608"),
        insert("CCMT09T304"),
        adaptive("BT40 ER32 chuck", "BT40", "ER32"),
        adaptive("ER32 collet 4", "ER32", "CYL 4"),
        adaptive("ER25 collet 4", "ER25", "CYL 4"),
    ];
    let names = |fits: Vec<crate::component_fit::Fit<'_>>| -> Vec<String> {
        fits.iter().map(|f| format!("{}@{}", f.component.name(), f.port)).collect()
    };
    let cnmg = insert("CNMG120408");
    assert_eq!(names(compatible_parents(&cnmg, &library)), ["PCLNR2525M12@CSW1", "PCLNL2525M12@CSW1"]);
    assert_eq!(names(compatible_children(&holder("PCLNR2525M12", None), &library)), ["CNMG120408@CSW1"]);
    assert_eq!(names(compatible_children(&library[3], &library)), ["CCMT09T304@CSW1"]);
    let collet = adaptive("ER32 collet 4", "ER32", "CYL 4");
    assert_eq!(names(compatible_parents(&collet, &library)), ["BT40 ER32 chuck@CSW"]);
    assert_eq!(names(compatible_children(&library[7], &library)), ["ER32 collet 4@CSW"]);
    let tool = end_mill();
    let Ok(mill) = solid("EM", tool.clone(), ConnectionCode::Cylindrical { d: tool.shank_d }) else { panic!("EM") };
    assert_eq!(names(compatible_parents(&mill, &library)), ["ER32 collet 4@CSW", "ER25 collet 4@CSW"]);
}

#[test]
fn malformed_components_are_refused_by_name() {
    let rect = ConnectionCode::RectShank { h: 25, b: 25 };
    let Ok(h) = HolderDesignation::parse("PCLNR2525M12") else { panic!("holder") };
    let seat = |port: &str| InsertSeat { port: port.into(), spec: h.seat };
    let tool_item = |seats| ComponentKind::ToolItem { mcs: rect.clone(), seats };
    assert_eq!(ToolComponent::new(" ", "x", tool_item(vec![seat("CSW1")])), Err(ComponentError::EmptyId));
    assert!(matches!(ToolComponent::new("h", "", tool_item(vec![seat("CSW1")])), Err(ComponentError::EmptyName { .. })));
    assert!(matches!(ToolComponent::new("h", "H", tool_item(vec![])), Err(ComponentError::NoSeat { .. })));
    for bad in ["SEAT1", "CSW", "CSW1_", "CSWa"] {
        assert!(matches!(ToolComponent::new("h", "H", tool_item(vec![seat(bad)])), Err(ComponentError::BadPort { .. })), "{bad}");
    }
    let dup = ToolComponent::new("h", "H", tool_item(vec![seat("CSW1_1"), seat("CSW1_1")]));
    assert!(matches!(dup, Err(ComponentError::DuplicatePort { .. })));
    let bad_hsk = ComponentKind::AdaptiveItem { mcs: ConnectionCode::Hsk { form: 'Z', size: 63 }, csw: code("ER32") };
    let Err(e) = ToolComponent::new("a", "Chuck", bad_hsk) else { panic!("HSK-Z63 accepted") };
    assert!(e.to_string().starts_with("the component 'Chuck' machine side"), "{e}");
    let tool = end_mill();
    let wrong = solid("EM", tool.clone(), ConnectionCode::Cylindrical { d: tool.shank_d + 2.0 });
    assert!(matches!(wrong, Err(ComponentError::ShankMismatch { .. })));
    let mut broken = tool;
    broken.diameter = f64::NAN;
    assert!(matches!(solid("EM", broken, ConnectionCode::Tts), Err(ComponentError::Tool { .. })));
}

/// #4096 review: a component checks the insert and seat specs that it
/// is given (their fields are public), names the component and the
/// property, and stores its connections in their text form.
#[test]
fn components_check_their_specs_and_normalize_their_connections() {
    let Ok(good) = InsertSpec::parse("CNMG120408") else { panic!("insert") };
    let insert_with = |f: &dyn Fn(&mut InsertSpec)| {
        let mut spec = good.clone();
        f(&mut spec);
        ToolComponent::new("i", "Bad insert", ComponentKind::CuttingItem(CuttingItem::Insert(spec)))
    };
    for (r, property) in [
        (insert_with(&|s| s.size = 0), "(size)"),
        (insert_with(&|s| s.tolerance = '?'), "(tolerance class)"),
        (insert_with(&|s| s.corner = crate::insert_iso1832::Corner::Radius(0)), "(corner)"),
    ] {
        let Err(e) = r else { panic!("{property} accepted") };
        assert!(matches!(e, ComponentError::Insert { .. }), "{e:?}");
        let text = e.to_string();
        assert!(text.contains("'Bad insert'") && text.contains(property), "{text}");
    }
    let Ok(h) = HolderDesignation::parse("PCLNR2525M12") else { panic!("holder") };
    let mut seat = h.seat;
    seat.size = 0;
    let kind = ComponentKind::ToolItem { mcs: code("RECT 25x25"), seats: vec![InsertSeat { port: "CSW1".into(), spec: seat }] };
    let Err(e) = ToolComponent::new("h", "Bad holder", kind) else { panic!("seat size 00 accepted") };
    assert!(matches!(e, ComponentError::Seat { .. }), "{e:?}");
    let text = e.to_string();
    assert!(text.contains("'Bad holder'") && text.contains("'CSW1'") && text.contains("(insert size)"), "{text}");
    // Connections are stored in their text form, so they fit their twins.
    let hsk_a = ConnectionCode::Hsk { form: 'a', size: 63 };
    let pallet = ConnectionCode::Custom(" Shop pallet ".into());
    let chuck = made("HSK pallet", ComponentKind::AdaptiveItem { mcs: hsk_a.clone(), csw: pallet });
    assert_eq!(chuck.mcs(), Some(&code("HSK-A63")));
    assert_eq!(check_coupling_fit(&chuck, "CSW", &adaptive("Pallet adapter", "CUSTOM:Shop pallet", "ER32")), Ok(()));
    let spindle = made("BT40 to HSK", ComponentKind::AdaptiveItem { mcs: code("BT40"), csw: hsk_a });
    assert_eq!(check_coupling_fit(&spindle, "CSW", &adaptive("HSK holder", "HSK-A63", "ER32")), Ok(()));
}
