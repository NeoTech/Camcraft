//! JEScript native piece/work attributes for a single punch, per Quintin Smits's MIT Agie Processor.
use super::AgieJob;

fn attribute(lines: &mut Vec<String>, object: &str, context: &str, key: &str, value: &str) {
    lines.push(format!(
        "  ok=JE_AttribObject({object},SETATTR,je_piece,{context},{key},{value},je_dim)"
    ));
}

fn quoted(value: impl std::fmt::Display) -> String {
    format!("\"{value}\"")
}

pub(super) fn piece(
    job: &AgieJob,
    name: &str,
    separation_length_mm: f64,
    entry: [f64; 2],
    exit: Option<[f64; 2]>,
) -> String {
    let [x, y, z] = job.stock_min;
    let [right, back, top] = job.stock_max;
    let height = format!("{:.3}", top - z);
    let group = quoted(name);
    let work = format!("{group},\"CUT1\"");
    let work_context = format!("{work},NULL");
    let start_context = format!("{work},\"stp1\"");
    let mut lines = vec!["'$Include \"jescript.sbh\"".into(), "sub SetOrigin".into()];
    for (field, value) in [
        ("ID_HEIGHT", height.clone()),
        ("ID_NOMEOBJ", name.into()),
        ("ID_MATERIAL", "0001".into()),
        ("ID_THREAD", "Berco Cut 0.25".into()),
        ("ID_POSX", "0".into()),
        ("ID_POSY", "0".into()),
        ("ID_POSZ", "0".into()),
        ("ID_ROTATION", "0".into()),
        ("ID_DIML", format!("{:.3}", right - x)),
        ("ID_DIMB", format!("{:.3}", back - y)),
        ("ID_POSPOSX", format!("{x:.3}")),
        ("ID_POSPOSY", format!("{y:.3}")),
        ("ID_POSPOSZ", format!("{z:.3}")),
        ("ID_VALSECP", "3.0".into()),
        ("ID_VALRETP", "30.0".into()),
    ] {
        attribute(&mut lines, "PIECE", "NULL,NULL,NULL", field, &quoted(value));
    }
    lines.extend([
        "  ok=JE_GenerateAttrib(c_tecauto,c_tecauto_t)".into(),
        "  ok=JE_GenerateAttrib(c_tecassign,c_tecass_fi)".into(),
        "  ok=JE_GenerateAttrib(c_tecalert,c_tecalert_sea)".into(),
        "end sub".into(),
        "sub CutPunch".into(),
        format!("  ok=JE_Create(GROUP,je_piece,{group},NULL)"),
        format!("  ok=JE_Create(WORK,je_piece,{work})"),
    ]);
    attribute(
        &mut lines,
        "WORK",
        &work_context,
        "ID_GEOMETRY",
        &quoted(format!("AGIE.USING_{name}.ISO")),
    );
    let side = match job.side {
        super::AgieSide::Punch => "c_punch",
        super::AgieSide::Die => "c_die",
        super::AgieSide::Left => "c_left",
        super::AgieSide::Right => "c_right",
    };
    attribute(&mut lines, "WORK", &work_context, "ID_PUNCH", side);
    attribute(&mut lines, "WORK", &work_context, "ID_WORKCOND", "\"2\"");
    // The control selects its own offsets and passes for the expressly selected Fine Ra0.30
    // technology. No controller technology tables or previously programmed parts are embedded.
    lines.push("  ok=JE_CreateQuality(je_piece,\"QN2\",\"0.30,1.50,1.00,F,F,F\",je_q1)".into());
    for (field, value) in [
        ("ID_QUALITY", "je_q1".into()),
        ("ID_THREAD", quoted("Berco Cut 0.25")),
        ("ID_ENTRYMODE", "c_entper".into()),
        ("ID_HEIGHT", quoted(&height)),
        ("ID_POSX", quoted("0")),
        ("ID_POSY", quoted("0")),
        ("ID_SEPCUT", quoted(format!("{separation_length_mm:.1}"))),
        ("ID_REVCUT", quoted("T")),
        ("ID_COMMPOINTENTRY", quoted("3.0")),
        ("ID_COMMPOINTEXIT", quoted("0")),
        ("ID_PRIORITY", quoted("10")),
        ("ID_RADIUSMINLA", quoted("T")),
        ("ID_I_TYPE", "c_radius_rm".into()),
        ("ID_E_TYPE", "c_radius_rm".into()),
    ] {
        attribute(&mut lines, "WORK", &work_context, field, &value);
    }
    if exit.is_some() {
        attribute(&mut lines, "WORK", &work_context, "ID_EXITMODE", "c_entper");
    } else {
        attribute(&mut lines, "WORK", &work_context, "ID_ENTRY", "\"2\"");
    }
    lines.push(format!(
        "  ok=JE_CreateStartPoint(WORK,\"stp1\",je_piece,{work})"
    ));
    for (field, value) in [("ID_POSX", entry[0]), ("ID_POSY", entry[1])] {
        attribute(
            &mut lines,
            "STARTPOINT",
            &start_context,
            field,
            &quoted(format!("{value:.3}")),
        );
    }
    let (point_key, mode_key) = if exit.is_some() {
        ("ID_STPENTRY", "ID_STPOPEN")
    } else {
        ("ID_STP", "ID_STPMODE")
    };
    attribute(&mut lines, "WORK", &work_context, point_key, "\"stp1\"");
    attribute(&mut lines, "WORK", &work_context, mode_key, "c_work");
    attribute(
        &mut lines,
        "STARTPOINT",
        &start_context,
        "ID_STP_PL",
        "\"4\"",
    );
    attribute(
        &mut lines,
        "STARTPOINT",
        &start_context,
        "ID_DIAMETER",
        "\"2.0\"",
    );
    if let Some([ex, ey]) = exit {
        let context = format!("{work},\"stp2\"");
        lines.push(format!(
            "  ok=JE_CreateStartPoint(WORK,\"stp2\",je_piece,{work})"
        ));
        attribute(&mut lines, "WORK", &work_context, "ID_STPEXIT", "\"stp2\"");
        attribute(&mut lines, "WORK", &work_context, "ID_STPOPEX", "c_work");
        attribute(
            &mut lines,
            "STARTPOINT",
            &context,
            "ID_POSX",
            &quoted(format!("{ex:.3}")),
        );
        attribute(
            &mut lines,
            "STARTPOINT",
            &context,
            "ID_POSY",
            &quoted(format!("{ey:.3}")),
        );
    }
    lines.extend([
        format!("  ok=JE_GenerateCuts(WORK,je_piece,{work},ASDELETE,NULL)"),
        format!("  ok=JE_GenerateCuts(WORK,je_piece,{work},ASAGIE,NULL)"),
        "end sub".into(),
        "sub main".into(),
        "  ok=JE_Initiate".into(),
        "  if (ok=0) then".into(),
        "    SetOrigin".into(),
        "    CutPunch".into(),
        "    ok=JE_AttribObject(PIECE,SETATTR,je_piece,je_grp,NULL,NULL,ID_STATEGY,\"EW\",je_dim)"
            .into(),
        "    ok=JE_ClosePiece(je_piece)".into(),
        "  else".into(),
        "    stop".into(),
        "  end if".into(),
        "end sub".into(),
        String::new(),
    ]);
    lines.join("\r\n")
}
