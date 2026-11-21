//! Native Agie group ownership, resource names and measured work count.
use super::*;

#[test]
fn a_group_rejects_an_unqualified_height_or_reserved_geometry_alias() {
    let temp = tempfile::tempdir().unwrap();
    for (height, name, expected) in [
        (-40., "PAIR", "20 mm"),
        (-20., "LPT", "non-reserved"),
        (-20., "ABCDEFGH", "seven"),
    ] {
        let mut a = punch();
        let mut b = punch();
        a.stock_min[2] = height;
        b.stock_min[2] = height;
        let dir = temp.path().join(name);
        assert!(
            export_works(&[a, b], &dir, name, 1.0)
                .unwrap_err()
                .contains(expected)
        );
        assert!(!dir.exists());
    }
}

#[test]
fn a_punch_and_a_die_cannot_share_a_group() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("MIXED");
    let mut die = punch();
    die.side = AgieSide::Die;
    assert!(
        export_works(&[punch(), die], &destination, "MIXED", 1.0)
            .unwrap_err()
            .contains("independent closed punches or two die openings")
    );
    assert!(!destination.exists());
}

fn group_punch(i: usize) -> AgieJob {
    let x = (i % 3) as f64 * 70.0 - 70.0;
    let y = (i / 3) as f64 * 40.0 - 40.0;
    let mut job = punch();
    for p in &mut job.contour {
        p[0] += x;
        p[1] += y;
    }
    job.start = Some([x - 30.0, y]);
    job.stock_min = [-125.0, -80.0, -20.0];
    job.stock_max = [125.0, 80.0, 0.0];
    job
}

#[test]
fn mixed_wire_conditions_refuse_the_group_before_writing_any_files() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("MIXED");
    for fine_plus_index in [0, 1] {
        let mut jobs = [group_punch(0), group_punch(1)];
        jobs[fine_plus_index].quality = AgieQuality::FinePlusRa035;
        assert_eq!(
            export_works(&jobs, &destination, "MIXED", 1.0).unwrap_err(),
            "All works in an Agie group must use the same qualified wire condition"
        );
        assert!(!destination.exists());
        assert!(temp.path().read_dir().unwrap().next().is_none());
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn every_punch_has_its_own_geometry_and_uses_the_requested_piece_quality() {
    let temp = tempfile::tempdir().unwrap();
    for count in 2..=MAX_PUNCH_WORKS {
        let jobs: Vec<_> = (0..count).map(group_punch).collect();
        let dir = temp.path().join(count.to_string());
        let files = export_works(&jobs, &dir, "PUNCH", 1.0).unwrap();
        assert_eq!(files.len(), count + 2);
        let piece = std::fs::read_to_string(dir.join("PUNCH.SBL")).unwrap();
        let map = std::fs::read_to_string(dir.join("PUNCH.SBR")).unwrap();
        assert_eq!(map.lines().count(), count);
        assert_eq!(piece.matches("JE_CreateQuality(").count(), 1);
        assert!(
            piece.contains(r#"JE_CreateQuality(je_piece,"CCB030","0.30,1.50,1.00,F,F,F",je_q1)"#)
        );
        assert!(!piece.contains("ASAGIE"));
        assert_eq!(piece.matches(",ASDELETE,NULL)").count(), count);
        let first_reset = piece.find("JE_GenerateCuts(WORK").unwrap();
        for n in 1..=count {
            let alias = format!("PUNCH{n}.ISO");
            assert!(dir.join(&alias).is_file());
            assert!(map.contains(&format!("AGIE.USING_{alias} IMPORT {alias};")));
            assert!(piece.contains(&format!(
                "\"CUT{n}\",NULL,ID_GEOMETRY,\"AGIE.USING_{alias}\""
            )));
            let binding = format!(r#""CUT{n}",NULL,ID_QUALITY,je_q1,je_dim"#);
            assert!(piece[..first_reset].contains(&binding));
        }
    }
    let jobs: Vec<_> = (0..=MAX_PUNCH_WORKS).map(group_punch).collect();
    let dir = temp.path().join("TOOMANY");
    assert!(export_works(&jobs, &dir, "TOOMANY", 1.0).is_err());
    assert!(!dir.exists());
}
