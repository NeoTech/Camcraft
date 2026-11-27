//! #3642: the ISO 1832 insert code, the ISO 5608 holder code and the
//! connection codes - parsed, written back, and refused by name.

use crate::connection_code::{ConnectionCode, SteepTaper};
use crate::insert_codes::{Clearance, Fixing, Hand, InsertShape};
use crate::insert_iso1832::{Corner, DesignationError, InsertSpec};
use crate::insert_seat::{Clamping, HolderDesignation};

fn ok(code: &str) -> InsertSpec {
    match InsertSpec::parse(code) {
        Ok(s) => s,
        Err(e) => panic!("{code}: {e}"),
    }
}

fn close(a: Option<f64>, b: f64) -> bool {
    a.is_some_and(|a| (a - b).abs() < 1e-9)
}

#[test]
fn common_turning_inserts_parse() {
    let c = ok("CNMG120408");
    assert_eq!((c.shape, c.clearance, c.tolerance, c.fixing, c.size), (InsertShape::C, Clearance::N, 'M', Fixing::G, 12));
    assert_eq!(c.thickness.code(), "04");
    assert!(close(c.thickness.nominal_mm(), 4.76));
    assert!(close(c.corner.nose_radius_mm(), 0.8));
    assert!(close(c.inscribed_circle_mm(), 12.7));
    assert_eq!(c.hand, None);

    // Each shape's edge length maps back to its inscribed circle.
    for (code, ic) in [
        ("DNMG150608", 12.7),
        ("VBMT160404", 9.525),
        ("TNMG160408", 9.525),
        ("WNMG080408", 12.7),
        ("SNMG120408", 12.7),
        ("CCMT09T304", 9.525),
        ("TCMT110204", 6.35),
    ] {
        assert!(close(ok(code).inscribed_circle_mm(), ic), "{code}");
    }
    let cc = ok("CCMT09T304");
    assert_eq!(cc.thickness.code(), "T3");
    assert!(close(cc.thickness.nominal_mm(), 3.97));
    let round = ok("RCMT1204M0");
    assert_eq!(round.corner, Corner::Round);
    assert_eq!(round.inscribed_circle_mm(), None);
}

#[test]
fn common_milling_inserts_parse() {
    let s = ok("SEKN1203AFTN");
    assert_eq!((s.shape, s.clearance, s.fixing), (InsertShape::S, Clearance::E, Fixing::N));
    assert_eq!(s.corner, Corner::Facet { angle: 'A', clearance: Clearance::F });
    assert_eq!(s.hand, Some(Hand::N));
    assert_eq!(s.tail, "TN");
    assert_eq!(ok("SPKN1203EDR").hand, Some(Hand::R));
    assert_eq!(ok("TPKN1603PDL").hand, Some(Hand::L));
    assert_eq!(ok("RPMT10T3M0").corner, Corner::Round);
}

#[test]
fn designations_round_trip_through_their_text() {
    let spaced = ok("cnmg 12 04 08-pm");
    assert_eq!(spaced.to_string(), "CNMG120408-PM");
    assert_eq!(spaced.hand, None);
    for code in ["CNMG120408", "CNMG120408-PM", "SEKN1203AFTN", "RCMT1204M0", "CCMT09T304", "VBMT160400"] {
        let s = ok(code);
        assert_eq!(s.to_string(), code);
        assert_eq!(ok(&s.to_string()), s);
    }
    assert_eq!(ok("VBMT160400").corner, Corner::Sharp);
}

fn refused(code: &str) -> DesignationError {
    match InsertSpec::parse(code) {
        Ok(s) => panic!("{code} parsed as {s}"),
        Err(e) => e,
    }
}

fn position(code: &str) -> u8 {
    match refused(code) {
        DesignationError::Position { pos, standard, .. } => {
            assert_eq!(standard, "ISO 1832");
            pos
        }
        e => panic!("{code}: {e}"),
    }
}

#[test]
fn malformed_insert_codes_are_refused_by_position() {
    assert_eq!(refused(""), DesignationError::Empty { standard: "ISO 1832" });
    assert_eq!(refused("   "), DesignationError::Empty { standard: "ISO 1832" });
    assert!(matches!(refused("CNMG12"), DesignationError::TooShort { len: 6, need: 10, .. }));
    // An inch code is too short for the metric layout.
    assert!(matches!(refused("CNMG432"), DesignationError::TooShort { .. }));
    assert!(matches!(refused("CNMG\u{e9}20408"), DesignationError::NonAscii { .. }));
    assert_eq!(position("QNMG120408"), 1);
    assert_eq!(position("CZMG120408"), 2);
    assert_eq!(position("CNBG120408"), 3);
    assert_eq!(position("CNMZ120408"), 4);
    assert_eq!(position("CNMG000408"), 5);
    assert_eq!(position("CNMGAB0408"), 5);
    assert_eq!(position("CNMG12XX08"), 6);
    assert_eq!(position("CNMG120000"), 6);
    assert_eq!(position("CNMG1204M0"), 7);
    assert_eq!(position("RCMT120408"), 7);
    assert_eq!(position("SEKN1203QFTN"), 7);
    assert_eq!(position("CNMG120408#"), 8);
    let text = refused("QNMG120408").to_string();
    assert!(text.contains("ISO 1832 position 1 (shape): 'Q'"), "{text}");
}

#[test]
fn turning_holder_codes_parse_and_round_trip() {
    let Ok(h) = HolderDesignation::parse("PCLNR2525M12") else { panic!("PCLNR2525M12") };
    let s = h.seat;
    assert_eq!((s.clamping, s.shape, s.clearance, s.hand, s.size), (Clamping::P, InsertShape::C, Clearance::N, Hand::R, 12));
    assert_eq!(s.style.approach_deg(), Some(95.0));
    assert_eq!((h.shank_h, h.shank_b, h.length_mm()), (25, 25, Some(150.0)));
    assert_eq!(h.to_string(), "PCLNR2525M12");
    let Ok(v) = HolderDesignation::parse("svjbl 2020 k16") else { panic!("svjbl") };
    assert_eq!((v.seat.clamping, v.seat.style.approach_deg(), v.seat.hand), (Clamping::S, Some(93.0), Hand::L));
    assert_eq!(v.to_string(), "SVJBL2020K16");
    assert_eq!(HolderDesignation::parse(&v.to_string()), Ok(v));
}

#[test]
fn malformed_holder_codes_are_refused_by_position() {
    let pos = |code: &str| match HolderDesignation::parse(code) {
        Err(DesignationError::Position { pos, standard: "ISO 5608", .. }) => pos,
        other => panic!("{code}: {other:?}"),
    };
    assert_eq!(pos("ZCLNR2525M12"), 1);
    assert_eq!(pos("PQLNR2525M12"), 2);
    assert_eq!(pos("PCINR2525M12"), 3);
    assert_eq!(pos("PCLZR2525M12"), 4);
    assert_eq!(pos("PCLNX2525M12"), 5);
    assert_eq!(pos("PCLNR0025M12"), 6);
    assert_eq!(pos("PCLNR25A5M12"), 7);
    assert_eq!(pos("PCLNR2525I12"), 8);
    assert_eq!(pos("PCLNR2525M1A"), 9);
    assert!(matches!(HolderDesignation::parse("PCLNR25"), Err(DesignationError::TooShort { need: 12, .. })));
    assert!(matches!(HolderDesignation::parse(""), Err(DesignationError::Empty { .. })));
}

#[test]
fn connection_codes_round_trip_and_refuse_by_name() {
    for code in [
        "BT40", "CAT40", "SK50", "HSK-A63", "PSC63", "KM63", "ER32", "WELDON 12", "CYL 12.7", "TTS", "RECT 25x25",
        "VDI40", "BMT65", "ARBOR 22", "CUSTOM:Shop pallet",
    ] {
        let Ok(c) = ConnectionCode::parse(code) else { panic!("{code}") };
        assert_eq!(c.to_string(), code);
        assert_eq!(c.check(), Ok(()));
    }
    assert_eq!(ConnectionCode::parse(" hsk-a 63 "), Ok(ConnectionCode::Hsk { form: 'A', size: 63 }));
    assert_eq!(ConnectionCode::parse("bt 40"), Ok(ConnectionCode::SteepTaper { family: SteepTaper::Bt, size: 40 }));
    for bad in ["", "BT45", "HSK-Z63", "ER33", "CYL 0", "CYL nan", "CYL 900", "XYZ", "CUSTOM:", "RECT 25", "ER32\u{e9}"] {
        let Err(e) = ConnectionCode::parse(bad) else { panic!("{bad} parsed") };
        assert!(e.to_string().starts_with("the connection code"), "{e}");
    }
    assert!(ConnectionCode::Hsk { form: 'Z', size: 63 }.check().is_err());
    assert!(ConnectionCode::Cylindrical { d: f64::NAN }.check().is_err());
}

/// #4096 review: a spec built in code (its fields are public) is checked
/// as its text would be, and each refusal names the position.
#[test]
fn directly_built_inserts_and_seats_are_refused_by_position() {
    let good = ok("CNMG120408-PM");
    assert_eq!(good.check(), Ok(()));
    let at = |s: &InsertSpec| match s.check() {
        Err(DesignationError::Position { standard: "ISO 1832", pos, name, .. }) => (pos, name),
        other => panic!("{s:?}: {other:?}"),
    };
    let with = |f: &dyn Fn(&mut InsertSpec)| {
        let mut s = good.clone();
        f(&mut s);
        s
    };
    assert_eq!(at(&with(&|s| s.size = 0)), (5, "size"));
    assert_eq!(at(&with(&|s| s.size = 100)), (5, "size"));
    for t in ['?', 'm', 'B', ' ', '\u{e9}'] {
        assert_eq!(at(&with(&|s| s.tolerance = t)), (3, "tolerance class"), "{t}");
    }
    assert_eq!(at(&with(&|s| s.corner = Corner::Radius(0))), (7, "corner"));
    assert_eq!(at(&with(&|s| s.corner = Corner::Radius(100))), (7, "corner"));
    assert_eq!(at(&with(&|s| s.corner = Corner::Round)), (7, "corner"));
    assert_eq!(at(&with(&|s| s.corner = Corner::Facet { angle: 'Q', clearance: Clearance::N })), (7, "corner"));
    assert_eq!(at(&with(&|s| s.hand = Some(Hand::L))), (8, "hand"));
    for tail in ["-pm", "- PM", "-P\u{e9}"] {
        assert_eq!(at(&with(&|s| s.tail = tail.into())), (8, "suffix"), "{tail}");
    }
    let mut round = ok("RCMT1204M0");
    round.corner = Corner::Radius(8);
    assert_eq!(at(&round), (7, "corner"));
    let Ok(h) = HolderDesignation::parse("PCLNR2525M12") else { panic!("holder") };
    assert_eq!(h.seat.check(), Ok(()));
    for size in [0, 100, 255] {
        let mut seat = h.seat;
        seat.size = size;
        let refused = seat.check();
        let named = matches!(
            &refused,
            Err(DesignationError::Position { standard: "ISO 5608", pos: 9, name: "insert size", .. })
        );
        assert!(named, "{size}: {refused:?}");
    }
}

/// #4096 review: a typed code that is not in its text form normalizes
/// to the value its text parses to. Validation, the text round trip and
/// the fit then agree.
#[test]
fn typed_connection_codes_normalize_to_their_text_form() {
    let cases = [
        (ConnectionCode::Hsk { form: 'a', size: 63 }, ConnectionCode::Hsk { form: 'A', size: 63 }),
        (ConnectionCode::Custom(" Shop pallet ".into()), ConnectionCode::Custom("Shop pallet".into())),
        (ConnectionCode::Weldon { d: 12.0 }, ConnectionCode::Weldon { d: 12.0 }),
    ];
    for (raw, canon) in cases {
        assert_eq!(raw.check(), Ok(()), "{raw:?}");
        assert_eq!(raw.canonical().as_ref(), Ok(&canon));
        assert_eq!(ConnectionCode::parse(&raw.to_string()).as_ref(), Ok(&canon));
        assert_eq!(canon.canonical().as_ref(), Ok(&canon));
        assert!(raw.takes(&canon) && canon.takes(&raw), "{raw:?}");
    }
    let bad = [
        ConnectionCode::Hsk { form: 'z', size: 63 },
        ConnectionCode::Custom("  ".into()),
        ConnectionCode::Weldon { d: -1.0 },
        ConnectionCode::Weldon { d: f64::INFINITY },
        ConnectionCode::Er { size: 33 },
    ];
    for code in bad {
        assert!(code.check().is_err() && code.canonical().is_err(), "{code:?}");
        assert!(!code.takes(&code), "an invalid code mates nothing: {code:?}");
    }
}

#[test]
fn a_plain_bore_takes_a_weldon_shank_but_not_the_reverse() {
    let cyl = |d| ConnectionCode::Cylindrical { d };
    let weldon = |d| ConnectionCode::Weldon { d };
    assert!(cyl(12.0).takes(&weldon(12.0)));
    assert!(cyl(12.0).takes(&cyl(12.01)));
    assert!(!cyl(12.0).takes(&cyl(12.5)));
    assert!(!weldon(12.0).takes(&cyl(12.0)));
    assert!(!ConnectionCode::Er { size: 32 }.takes(&ConnectionCode::Er { size: 25 }));
}

/// Degenerate text never panics: a fixed pseudo-random walk over an
/// alphabet with digits, letters, separators and non-ASCII characters.
#[test]
fn degenerate_codes_never_panic() {
    let alphabet: Vec<char> = "CNMGRTSVDW0123456789-TXM: ._\u{e9}\u{0}\u{1F600}x".chars().collect();
    let mut seed: u64 = 0x3642;
    let mut next = || {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (seed >> 33) as usize
    };
    for _ in 0..5000 {
        let len = next() % 24;
        let text: String = (0..len).map(|_| alphabet[next() % alphabet.len()]).collect();
        if let Ok(s) = InsertSpec::parse(&text) {
            assert_eq!(InsertSpec::parse(&s.to_string()), Ok(s.clone()), "{text}");
        }
        let _ = HolderDesignation::parse(&text);
        let _ = ConnectionCode::parse(&text);
    }
}
