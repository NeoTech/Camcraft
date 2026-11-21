//! JEScript native piece/work attributes for a piece and its native works, per Quintin Smits's MIT Agie Processor.
use super::{AgieJob, Work};

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
    works: &[Work],
) -> String {
    let [x, y, z] = job.stock_min;
    let [right, back, top] = job.stock_max;
    let height = format!("{:.3}", top - z);
    let group = quoted(name);
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
    ]);
    for (index, item) in works.iter().enumerate() {
        append_work(
            &mut lines,
            &group,
            &height,
            index,
            item,
            separation_length_mm,
        );
    }
    lines.extend([
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

fn append_work(
    lines: &mut Vec<String>,
    group: &str,
    height: &str,
    index: usize,
    item: &Work,
    separation_length_mm: f64,
) {
    let work = format!("{group},\"CUT{}\"", index + 1);
    let work_context = format!("{work},NULL");
    let start_context = format!("{work},\"stp1\"");
    lines.push(format!("  ok=JE_Create(WORK,je_piece,{work})"));
    attribute(
        lines,
        "WORK",
        &work_context,
        "ID_GEOMETRY",
        &quoted(format!("AGIE.USING_{}.ISO", item.geometry)),
    );
    let side = match item.side {
        super::AgieSide::Punch => "c_punch",
        super::AgieSide::Die => "c_die",
        super::AgieSide::Left => "c_left",
        super::AgieSide::Right => "c_right",
    };
    attribute(lines, "WORK", &work_context, "ID_PUNCH", side);
    attribute(lines, "WORK", &work_context, "ID_WORKCOND", "\"2\"");
    // The control selects its own offsets and passes for the expressly selected Fine Ra0.30
    // technology. No controller technology tables or previously programmed parts are embedded.
    lines.push(format!(
        "  ok=JE_CreateQuality(je_piece,\"QN{}\",\"0.30,1.50,1.00,F,F,F\",je_q1)",
        index + 2
    ));
    for (field, value) in [
        ("ID_QUALITY", "je_q1".into()),
        ("ID_THREAD", quoted("Berco Cut 0.25")),
        ("ID_ENTRYMODE", "c_entper".into()),
        ("ID_HEIGHT", quoted(height)),
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
        attribute(lines, "WORK", &work_context, field, &value);
    }
    if item.exit.is_some() {
        attribute(lines, "WORK", &work_context, "ID_EXITMODE", "c_entper");
    } else {
        attribute(lines, "WORK", &work_context, "ID_ENTRY", "\"2\"");
    }
    lines.push(format!(
        "  ok=JE_CreateStartPoint(WORK,\"stp1\",je_piece,{work})"
    ));
    for (field, value) in [("ID_POSX", item.entry[0]), ("ID_POSY", item.entry[1])] {
        attribute(
            lines,
            "STARTPOINT",
            &start_context,
            field,
            &quoted(format!("{value:.3}")),
        );
    }
    let (point_key, mode_key) = if item.exit.is_some() {
        ("ID_STPENTRY", "ID_STPOPEN")
    } else {
        ("ID_STP", "ID_STPMODE")
    };
    attribute(lines, "WORK", &work_context, point_key, "\"stp1\"");
    attribute(lines, "WORK", &work_context, mode_key, "c_work");
    attribute(lines, "STARTPOINT", &start_context, "ID_STP_PL", "\"4\"");
    attribute(
        lines,
        "STARTPOINT",
        &start_context,
        "ID_DIAMETER",
        "\"2.0\"",
    );
    if let Some([ex, ey]) = item.exit {
        let context = format!("{work},\"stp2\"");
        lines.push(format!(
            "  ok=JE_CreateStartPoint(WORK,\"stp2\",je_piece,{work})"
        ));
        attribute(lines, "WORK", &work_context, "ID_STPEXIT", "\"stp2\"");
        attribute(lines, "WORK", &work_context, "ID_STPOPEX", "c_work");
        attribute(
            lines,
            "STARTPOINT",
            &context,
            "ID_POSX",
            &quoted(format!("{ex:.3}")),
        );
        attribute(
            lines,
            "STARTPOINT",
            &context,
            "ID_POSY",
            &quoted(format!("{ey:.3}")),
        );
    }
    lines.extend([
        format!("  ok=JE_GenerateCuts(WORK,je_piece,{work},ASDELETE,NULL)"),
        format!("  ok=JE_GenerateCuts(WORK,je_piece,{work},ASAGIE,NULL)"),
    ]);
}
