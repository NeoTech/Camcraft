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

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn every_qualified_group_size_has_a_unique_file_work_and_quality_for_each_punch() {
    let temp = tempfile::tempdir().unwrap();
    let make_job = |i: usize| {
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
    };
    for count in 2..=MAX_PUNCH_WORKS {
        let jobs: Vec<_> = (0..count).map(make_job).collect();
        let dir = temp.path().join(count.to_string());
        let files = export_works(&jobs, &dir, "PUNCH", 1.0).unwrap();
        assert_eq!(files.len(), count + 2);
        let piece = std::fs::read_to_string(dir.join("PUNCH.SBL")).unwrap();
        let map = std::fs::read_to_string(dir.join("PUNCH.SBR")).unwrap();
        assert_eq!(map.lines().count(), count);
        for n in 1..=count {
            let alias = format!("PUNCH{n}.ISO");
            assert!(dir.join(&alias).is_file());
            assert!(map.contains(&format!("AGIE.USING_{alias} IMPORT {alias};")));
            assert!(piece.contains(&format!(
                "\"CUT{n}\",NULL,ID_GEOMETRY,\"AGIE.USING_{alias}\""
            )));
            assert!(piece.contains(&format!("JE_CreateQuality(je_piece,\"QN{}\"", n + 1)));
        }
    }
    let jobs: Vec<_> = (0..=MAX_PUNCH_WORKS).map(make_job).collect();
    let dir = temp.path().join("TOOMANY");
    assert!(export_works(&jobs, &dir, "TOOMANY", 1.0).is_err());
    assert!(!dir.exists());
}
