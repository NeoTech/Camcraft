//! Klartext block dispatch and machine state: program frame (`BEGIN PGM` / `END PGM`), M functions, sections and the final `ParsedProgram`.
//! Provenance: HEIDENHAIN iTNC 530 conversational user's manual 533 190-22 (program structure, M function table and effect at block start or end), as summarized in docs/research/heidenhain/.

use camcraft_geometry::geo2d::Point2;
use camcraft_geometry::geometry::Point3;
use camcraft_geometry::geometry::units::Unit;

use super::KlartextOptions;
use super::cycles::DrillDef;
use super::source::Block;
use super::words::QTable;
use crate::section::SectionBuilder;
use crate::types::{
    DwellEvent, GcodeError, LineMap, ModalState, MoveLine, ParseOptions, ParsedProgram,
    ParsedSection, ProgramEnd, SpindleDir, StopEvent, StopKind,
};

pub struct Machine<'a> {
    pub opts: &'a KlartextOptions,
    pub modal: ModalState,
    pub pos: Point3,
    /// Circle centre of the last `CC` block (modal).
    pub cc: Option<Point2>,
    /// Unit XY direction at the end of the last move, for `CT`.
    pub tangent: Option<Point2>,
    pub cycle: Option<DrillDef>,
    pub q: QTable,
    pub tool_name: Option<String>,
    section_tool_name: Option<String>,
    pgm: Option<String>,
    pgm_ended: bool,
    ended: Option<ProgramEnd>,
    pub sections: Vec<ParsedSection>,
    pub current: SectionBuilder,
    pub dwells: Vec<DwellEvent>,
    stops: Vec<StopEvent>,
    pub tools: Vec<u32>,
}

impl<'a> Machine<'a> {
    pub fn new(opts: &'a KlartextOptions) -> Self {
        let mut modal = ModalState::from_dialect(&ParseOptions::default().dialect);
        modal.units = Some(Unit::Millimeters);
        let pos = opts.initial_position;
        let current = SectionBuilder::new(0, pos, &modal);
        Self {
            opts,
            modal,
            pos,
            cc: None,
            tangent: None,
            cycle: None,
            q: QTable::new(),
            tool_name: None,
            section_tool_name: None,
            pgm: None,
            pgm_ended: false,
            ended: None,
            sections: Vec::new(),
            current,
            dwells: Vec::new(),
            stops: Vec::new(),
            tools: Vec::new(),
        }
    }

    pub fn step(&mut self, b: &Block) -> Result<(), GcodeError> {
        let w: Vec<&str> = b.words.iter().map(String::as_str).collect();
        if self.pgm_ended {
            return Err(b.syntax("block after END PGM"));
        }
        if w.first() == Some(&"BEGIN") {
            return self.begin(b, &w);
        }
        if self.pgm.is_none() {
            return Err(b.syntax("the program must start with BEGIN PGM"));
        }
        if w.first() == Some(&"END") {
            return self.end(b, &w);
        }
        if self.ended.is_some() {
            return Err(b.syntax("executable block after program end (M2/M30)"));
        }
        self.dispatch(b, &w)
    }

    fn begin(&mut self, b: &Block, w: &[&str]) -> Result<(), GcodeError> {
        if self.pgm.is_some() {
            return Err(b.syntax("a second BEGIN PGM"));
        }
        match w {
            ["BEGIN", "PGM", name, "MM"] => {
                self.pgm = Some((*name).to_string());
                Ok(())
            }
            ["BEGIN", "PGM", _, "INCH"] => {
                Err(b.unsupported("BEGIN PGM INCH", "inch programs are not read (only MM)"))
            }
            _ => Err(b.syntax("expected BEGIN PGM <name> MM")),
        }
    }

    fn end(&mut self, b: &Block, w: &[&str]) -> Result<(), GcodeError> {
        let ["END", "PGM", name, unit] = w else {
            return Err(b.syntax("expected END PGM <name> MM"));
        };
        if self.pgm.as_deref() != Some(*name) || *unit != "MM" {
            return Err(b.syntax(&format!(
                "END PGM {name} {unit} does not match BEGIN PGM {} MM",
                self.pgm.as_deref().unwrap_or("")
            )));
        }
        self.pgm_ended = true;
        Ok(())
    }

    fn dispatch(&mut self, b: &Block, w: &[&str]) -> Result<(), GcodeError> {
        let rest = &b.words[1..];
        match w[0] {
            "L" => self.line_block(b, rest),
            "CC" => self.cc_block(b, rest),
            "C" => self.arc_block(b, rest),
            "CR" => self.radius_arc_block(b, rest),
            "CT" => self.tangent_arc_block(b, rest),
            "BLK" => self.blk_form(b, rest),
            "TOOL" => self.tool_block(b, rest),
            "CYCL" => self.cycle_block(b, rest),
            "FN" => self.fn_block(b, rest),
            "STOP" => self.stop_block(b, rest),
            k if k.starts_with('Q') => self.q_assign(b, &b.words),
            k if k.starts_with('M') && k.len() > 1 => {
                // An M-only block, as written by the post (`M3`, `M0`, `M2`).
                let words = super::words::decode(b, &b.words, &self.q)?;
                if words.has_coords() || words.feed.is_some() || words.r0 {
                    return Err(b.syntax("an M block takes only M functions"));
                }
                self.pre_m(b, &words.m)?;
                self.post_m(b, &words.m, words.mb)
            }
            "RND" | "CHF" => Err(b.unsupported(
                w[0],
                "rounding and chamfer blocks change the neighbour elements; this reader version refuses them",
            )),
            "LP" | "CP" | "CTP" | "PA" => Err(b.unsupported(w[0], "polar coordinates are not read")),
            "APPR" | "DEP" => Err(b.unsupported(w[0], "approach and departure functions are not read")),
            "LBL" | "CALL" => Err(b.unsupported(
                w[0],
                "labels and program calls (LBL, CALL LBL, CALL PGM) are not read in this reader version",
            )),
            "FK" | "FL" | "FLT" | "FC" | "FCT" | "FPOL" => {
                Err(b.unsupported(w[0], "FK free contour programming is not read"))
            }
            other => Err(b.unknown(other)),
        }
    }

    /// M functions that act at the start of the block.
    pub fn pre_m(&mut self, b: &Block, m: &[u32]) -> Result<(), GcodeError> {
        for &n in m {
            match n {
                3 => self.modal.spindle_dir = SpindleDir::Cw,
                4 => self.modal.spindle_dir = SpindleDir::Ccw,
                8 => self.modal.coolant.flood = true,
                13 | 14 => {
                    self.modal.spindle_dir = if n == 13 {
                        SpindleDir::Cw
                    } else {
                        SpindleDir::Ccw
                    };
                    self.modal.coolant.flood = true;
                }
                0 | 1 | 2 | 5 | 9 | 30 | 99 | 137 | 140 => {}
                136 => return Err(b.unsupported("M136", "feed per revolution is not read")),
                89 => {
                    return Err(b.unsupported(
                        "M89",
                        "the modal cycle call is not read; use CYCL CALL or M99",
                    ));
                }
                91 | 92 => {
                    return Err(
                        b.unsupported(&format!("M{n}"), "machine-based coordinates are not read")
                    );
                }
                _ => return Err(b.unsupported(&format!("M{n}"), "this M function is not read")),
            }
        }
        self.sync_section();
        Ok(())
    }

    /// M functions that act at the end of the block. M99 calls the active cycle.
    pub fn post_m(
        &mut self,
        b: &Block,
        m: &[u32],
        mb: Option<super::words::Mb>,
    ) -> Result<(), GcodeError> {
        if mb.is_some() && !m.contains(&140) {
            return Err(b.syntax("MB without M140"));
        }
        if m.contains(&99) {
            self.cycle_call(b)?;
        }
        if m.contains(&140) {
            self.m140(b, mb)?;
        }
        for &n in m {
            let after_move = self.current.builder.moves_len().checked_sub(1);
            match n {
                0 | 1 => {
                    let kind = if n == 0 {
                        StopKind::ProgramStop
                    } else {
                        StopKind::OptionalStop
                    };
                    self.stops.push(StopEvent {
                        line: b.line,
                        kind,
                        section: self.current.index,
                        after_move,
                    });
                    if n == 0 {
                        // iTNC M0: program stop, spindle stop, coolant off.
                        self.modal.spindle_dir = SpindleDir::Off;
                        self.modal.coolant = Default::default();
                    }
                }
                5 => self.modal.spindle_dir = SpindleDir::Off,
                9 => self.modal.coolant = Default::default(),
                2 => self.ended = Some(ProgramEnd::M2),
                30 => self.ended = Some(ProgramEnd::M30),
                _ => {}
            }
        }
        if self.ended.is_some() {
            self.cycle = None;
        }
        Ok(())
    }

    /// Start a new section when the tool, offset, spindle or coolant changed after a move.
    pub fn sync_section(&mut self) {
        if !self.current.has_moves() {
            self.current = SectionBuilder::new(self.current.index, self.pos, &self.modal);
            self.section_tool_name = self.tool_name.clone();
            return;
        }
        if self.current.state_matches(&self.modal) && self.section_tool_name == self.tool_name {
            return;
        }
        let next = self.sections.len() + 1;
        let prev = std::mem::replace(
            &mut self.current,
            SectionBuilder::new(next, self.pos, &self.modal),
        );
        let name = std::mem::replace(&mut self.section_tool_name, self.tool_name.clone());
        let mut s = prev.finish();
        s.tool_name = name;
        self.sections.push(s);
    }

    pub fn push_stop(&mut self, s: StopEvent) {
        self.stops.push(s);
    }

    pub fn finish(mut self, lines: usize) -> Result<ParsedProgram, GcodeError> {
        if !self.pgm_ended {
            return Err(GcodeError::Syntax {
                line: lines.max(1),
                message: "the program has no END PGM block (truncated program?)".to_string(),
            });
        }
        let idx = self.current.index;
        let named = self.dwells.iter().any(|d| d.section == idx)
            || self.stops.iter().any(|s| s.section == idx);
        if self.current.has_moves() || named {
            let mut s = self.current.finish();
            s.tool_name = self.section_tool_name.take();
            self.sections.push(s);
        }
        let mut entries = Vec::new();
        for (section, sec) in self.sections.iter().enumerate() {
            for (move_index, &line) in sec.move_lines.iter().enumerate() {
                entries.push(MoveLine {
                    section,
                    move_index,
                    line,
                });
            }
        }
        Ok(ParsedProgram {
            program_number: None,
            sections: self.sections,
            line_map: LineMap { entries },
            dwells: self.dwells,
            stops: self.stops,
            ended: self.ended,
            tools: self.tools,
            final_modal: self.modal,
        })
    }
}
