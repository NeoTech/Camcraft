use super::*;

#[test]
fn arc_with_radius_difference_of_0_0003_mm_after_rounding_posts() {
    let stream = STREAM.replace(
        "Arc CCW X 100000 Y 35000 Z 9000 F 350000 I 100000 J 25000\nArc CW X 110000 Y 25000 Z 9000 F 350000 I 100000 J 25000",
        "Arc CCW X 100000 Y 35000.3 Z 9000 F 350000 I 100000 J 25000\nFeed X 100000 Y 35000.3 Z 9000 F 220000",
    );
    let nc = post(MCH, &stream, &Value::Null).expect("0.0003 mm arc radius difference must post");
    assert!(nc.contains("CC X+0 Y+0"), "{nc}");
    assert!(nc.contains("C X+0 Y+10.0003 DR+"), "{nc}");
}

#[test]
fn arc_with_radius_difference_above_machine_tolerance_is_refused_with_radii_and_limit() {
    let stream = STREAM.replace(
        "Arc CCW X 100000 Y 35000 Z 9000 F 350000 I 100000 J 25000",
        "Arc CCW X 100000 Y 35020 Z 9000 F 350000 I 100000 J 25000",
    );
    let err = post(MCH, &stream, &Value::Null).expect_err("arc above tolerance must be refused");
    assert!(err.contains("inconsistent XY arc centre or radius"), "{err}");
    assert!(err.contains("start radius 10.0000 mm"), "{err}");
    assert!(err.contains("end radius 10.0200 mm"), "{err}");
    assert!(err.contains("limit 0.016 mm"), "{err}");
}

#[test]
fn custom_machine_arc_tolerance_is_respected_and_named_in_refusal() {
    let mut machine: Value = serde_json::from_str(MCH).unwrap();
    machine["arcTolerance"] = serde_json::json!(0.0002);
    let stream = STREAM.replace(
        "Arc CCW X 100000 Y 35000 Z 9000 F 350000 I 100000 J 25000\nArc CW X 110000 Y 25000 Z 9000 F 350000 I 100000 J 25000",
        "Arc CCW X 100000 Y 35000.3 Z 9000 F 350000 I 100000 J 25000\nFeed X 100000 Y 35000.3 Z 9000 F 220000",
    );
    let err = post(&machine.to_string(), &stream, &Value::Null)
        .expect_err("0.0003 mm radius difference must exceed 0.0002 mm custom limit");
    assert!(err.contains("limit 0.0002 mm"), "{err}");
    assert!(err.contains("start radius 10.0000 mm"), "{err}");
    assert!(err.contains("end radius 10.0003 mm"), "{err}");
}

#[test]
fn arc_tolerance_above_controller_maximum_is_refused() {
    let mut machine: Value = serde_json::from_str(MCH).unwrap();
    machine["arcTolerance"] = serde_json::json!(0.1);
    let err = post(&machine.to_string(), STREAM, &Value::Null)
        .expect_err("arc tolerance above 0.016 mm must be refused");
    assert!(err.contains("exceeds the 0.016 mm controller maximum (MP 7431)"), "{err}");
}

