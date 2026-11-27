//! Klartext set-up blocks: `BLK FORM`, `TOOL CALL` / `TOOL DEF`, literal Q parameter assignment (`FN 0:` and `Qn =`) and `STOP`.
//! Provenance: HEIDENHAIN iTNC 530 conversational user's manual 533 190-22 (workpiece blank, tool call, FN 0 assignment, STOP), as summarized in docs/research/heidenhain/.

use super::exec::Machine;
use super::source::{Block, literal, q_index};
use super::words::value;
use crate::domain::check_feed;
use crate::types::{GcodeError, SpindleDir, StopEvent, StopKind};

impl Machine<'_> {
    /// `BLK FORM 0.1 Z X.. Y.. Z..` and `BLK FORM 0.2 X.. Y.. Z..`: checked, then
    /// ignored (the blank is not part of the toolpath).
    pub fn blk_form(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let t: Vec<&str> = toks.iter().map(String::as_str).collect();
        let coords = match t.as_slice() {
            ["FORM", "0.1", "Z", rest @ ..] => rest,
            ["FORM", "0.2", rest @ ..] => rest,
            ["FORM", "0.1", axis, ..] => {
                return Err(
                    b.unsupported(&format!("BLK FORM 0.1 {axis}"), "only tool axis Z is read")
                );
            }
            ["FORM", kind, ..] => {
                return Err(b.unsupported(
                    &format!("BLK FORM {kind}"),
                    "only the 0.1 / 0.2 box blank is read",
                ));
            }
            _ => return Err(b.syntax("expected BLK FORM 0.1 Z ... or BLK FORM 0.2 ...")),
        };
        for c in coords {
            let body = c.strip_prefix('I').unwrap_or(c);
            let Some(v) = body.strip_prefix(['X', 'Y', 'Z']) else {
                return Err(b.unknown(c));
            };
            value(b, "BLK FORM", v, &self.q)?;
        }
        Ok(())
    }

    /// `TOOL CALL [n | "NAME"] Z [S..] [F..] [DL..] [DR..]`; `TOOL DEF` is a no-op.
    pub fn tool_block(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let Some((kind, rest)) = toks.split_first() else {
            return Err(b.syntax("expected TOOL CALL or TOOL DEF"));
        };
        match kind.as_str() {
            "DEF" => return Ok(()),
            "CALL" => {}
            other => return Err(b.unknown(&format!("TOOL {other}"))),
        }
        let mut i = 0;
        let mut number = None;
        let mut name = None;
        if let Some(t) = rest.first() {
            if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
                let n = t[1..t.len() - 1].trim();
                if n.is_empty() {
                    return Err(b.syntax("empty tool name"));
                }
                name = Some(n.to_string());
                i = 1;
            } else if t.bytes().all(|c| c.is_ascii_digit()) {
                number = Some(
                    t.parse::<u32>()
                        .map_err(|_| b.syntax(&format!("tool number '{t}' is too large")))?,
                );
                i = 1;
            }
        }
        match rest.get(i).map(String::as_str) {
            Some("Z") => i += 1,
            Some(a @ ("X" | "Y")) => {
                return Err(b.unsupported(&format!("TOOL CALL {a}"), "only tool axis Z is read"));
            }
            _ => return Err(b.syntax("TOOL CALL needs the tool axis Z")),
        }
        let (mut s, mut f) = (None, None);
        for t in &rest[i..] {
            if let Some(v) = t.strip_prefix("DL").or_else(|| t.strip_prefix("DR")) {
                if value(b, "TOOL CALL delta", v, &self.q)? != 0.0 {
                    return Err(b.unsupported(&t[..2], "a TOOL CALL delta value is not read"));
                }
            } else if let Some(v) = t.strip_prefix('S') {
                let rpm = value(b, "S", v, &self.q)?;
                if !(0.0..=1.0e6).contains(&rpm) {
                    return Err(
                        b.invalid_modal(&format!("spindle speed S ({rpm}) is out of range"))
                    );
                }
                s = Some(rpm);
            } else if let Some(v) = t.strip_prefix('F') {
                f = Some(check_feed(b.line, value(b, "F", v, &self.q)?)?);
            } else {
                return Err(b.unknown(t));
            }
        }
        if number.is_some() || name.is_some() {
            self.modal.active_tool = number;
            self.tool_name = name;
            if let Some(n) = number
                && !self.tools.contains(&n)
            {
                self.tools.push(n);
            }
        }
        if let Some(rpm) = s {
            self.modal.spindle_rpm = Some(rpm);
        }
        if f.is_some() {
            self.modal.feed_rate_mm_min = f;
        }
        self.sync_section();
        Ok(())
    }

    /// `FN 0: Qn = <literal>`; any other FN function is refused by name.
    pub fn fn_block(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let Some(first) = toks.first() else {
            return Err(b.syntax("expected FN 0: Qn = value"));
        };
        let (num, rest) = match first.strip_suffix(':') {
            Some(n) => (n.to_string(), &toks[1..]),
            None => {
                if toks.get(1).map(String::as_str) != Some(":") {
                    return Err(b.syntax("expected FN <n>:"));
                }
                (first.clone(), &toks[2..])
            }
        };
        if num != "0" {
            return Err(b.unsupported(
                &format!("FN {num}"),
                "only FN 0 (assign a literal value) is read",
            ));
        }
        self.q_assign(b, rest)
    }

    /// `Qn = <literal>` (also `Qn=<literal>`).
    pub fn q_assign(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let joined: String = toks.concat();
        let Some((lhs, rhs)) = joined.split_once('=') else {
            return Err(b.syntax("expected Qn = value"));
        };
        let Some(idx) = q_index(lhs) else {
            return Err(b.unsupported(lhs, "only Q parameters Q0 to Q1999 are read"));
        };
        let Some(v) = literal(rhs) else {
            return Err(b.unsupported(
                &format!("Q{idx} = {rhs}"),
                "only a literal value is read in a Q parameter assignment",
            ));
        };
        self.q.insert(idx, v);
        Ok(())
    }

    /// `STOP` (optionally with M functions) is a programmed stop. The block
    /// records exactly one stop: `STOP` is unconditional, so an `M0` or `M1`
    /// in the same block adds no second stop (an `M1` cannot weaken it). `M0`
    /// keeps its modal effect (spindle stop, coolant off).
    pub fn stop_block(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let w = super::words::decode(b, toks, &self.q)?;
        if w.has_coords() || w.feed.is_some() || w.r0 {
            return Err(b.syntax("STOP takes only M functions"));
        }
        self.pre_m(b, &w.m)?;
        let after_move = self.current.builder.moves_len().checked_sub(1);
        self.push_stop(StopEvent {
            line: b.line,
            kind: StopKind::ProgramStop,
            section: self.current.index,
            after_move,
        });
        if w.m.contains(&0) {
            self.modal.spindle_dir = SpindleDir::Off;
            self.modal.coolant = Default::default();
        }
        let rest: Vec<u32> = w.m.iter().copied().filter(|&n| n != 0 && n != 1).collect();
        self.post_m(b, &rest, w.mb)
    }
}
