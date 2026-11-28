//! Klartext cycles written as numbered sub-blocks (`CYCL DEF n.0`, `n.1`, ...): cycle 9 DWELL TIME and cycle 32 TOLERANCE; every other such cycle is refused by name.
//! Provenance: HEIDENHAIN TNC cycle programming manuals (special cycles 9 DWELL TIME and 32 TOLERANCE: `CYCL DEF 9.1 DWELL <s>`, `CYCL DEF 32.1 T<mm>`, `CYCL DEF 32.2 HSC-MODE:<0|1> TA<deg>`), as listed in docs/research/heidenhain/cycles.md.

use super::exec::Machine;
use super::source::{Block, literal};
use super::words::value;
use crate::domain::{check_dwell, check_len};
use crate::types::{DwellEvent, GcodeError};

/// A sub-block cycle that waits for its next block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OldCycle {
    pub cycle: u32,
    /// The sub-block number that must (or may) come next.
    pub next: u32,
    /// True when the next sub-block can be left out (`32.2`).
    pub optional: bool,
    /// Physical line of the block that opened the wait.
    pub line: usize,
}

/// `n.s` of a `CYCL DEF n.s` token.
fn split_number(tok: &str) -> Option<(u32, u32)> {
    let (c, s) = tok.split_once('.')?;
    let all_digits = |t: &str| !t.is_empty() && t.len() <= 4 && t.bytes().all(|b| b.is_ascii_digit());
    if !all_digits(c) || !all_digits(s) {
        return None;
    }
    Some((c.parse().ok()?, s.parse().ok()?))
}

impl Machine<'_> {
    /// Before each block: a cycle that waits for a required sub-block refuses
    /// every block but its own `CYCL DEF n.s`; an optional wait ends.
    pub fn old_cycle_gate(&mut self, b: &Block) -> Result<(), GcodeError> {
        let Some(p) = self.old else {
            return Ok(());
        };
        let own = matches!(b.words.as_slice(), [c, d, n, ..]
            if c == "CYCL" && d == "DEF" && split_number(n).is_some_and(|(k, _)| k == p.cycle));
        if own {
            return Ok(());
        }
        if p.optional {
            self.old = None;
            return Ok(());
        }
        Err(b.syntax(&format!(
            "CYCL DEF {}.{} must follow the CYCL DEF {}.{} block on line {}",
            p.cycle,
            p.next,
            p.cycle,
            p.next.saturating_sub(1),
            p.line
        )))
    }

    /// `CYCL DEF n.s ...`: cycles 9 and 32; other sub-block cycles are refused by name.
    pub fn old_cycle_def(&mut self, b: &Block, num: &str, rest: &[String]) -> Result<(), GcodeError> {
        let Some((c, s)) = split_number(num) else {
            return Err(b.syntax(&format!("cycle number '{num}' is not a number")));
        };
        if c != 9 && c != 32 {
            return Err(b.unsupported(
                &format!("CYCL DEF {num}"),
                "sub-block cycles (n.0 / n.1 blocks) other than 9 DWELL TIME and 32 TOLERANCE are not read",
            ));
        }
        let pending = self.old.filter(|p| p.cycle == c);
        if s == 0 {
            if let Some(p) = pending.filter(|p| !p.optional) {
                return Err(b.syntax(&format!(
                    "CYCL DEF {c}.0 before the CYCL DEF {c}.{} block that line {} needs",
                    p.next, p.line
                )));
            }
            // The name words (DWELL TIME, TOLERANCE) depend on the control language.
            if rest.iter().any(|t| literal(t).is_some()) {
                return Err(b.syntax(&format!("CYCL DEF {c}.0 takes only the cycle name")));
            }
            self.old = Some(OldCycle {
                cycle: c,
                next: 1,
                optional: false,
                line: b.line,
            });
            return Ok(());
        }
        if pending.map(|p| p.next) != Some(s) {
            return Err(b.syntax(&format!(
                "CYCL DEF {c}.{s} out of order: it needs the blocks CYCL DEF {c}.0 to {c}.{} first",
                s.saturating_sub(1)
            )));
        }
        self.old = None;
        match (c, s) {
            (9, 1) => self.dwell_block(b, rest),
            (32, 1) => {
                self.tolerance_block(b, rest)?;
                self.old = Some(OldCycle {
                    cycle: 32,
                    next: 2,
                    optional: true,
                    line: b.line,
                });
                Ok(())
            }
            (32, 2) => hsc_block(b, rest, self),
            _ => Err(b.syntax(&format!("CYCL DEF {c}.{s} is not a block of cycle {c}"))),
        }
    }

    /// `CYCL DEF 9.1 DWELL <s>`: the cycle is DEF-active, so the dwell runs here.
    fn dwell_block(&mut self, b: &Block, rest: &[String]) -> Result<(), GcodeError> {
        let Some((v, name)) = rest.split_last() else {
            return Err(b.syntax("CYCL DEF 9.1 needs the dwell time in seconds"));
        };
        if name.len() > 1 || name.iter().any(|t| literal(t).is_some()) {
            return Err(b.syntax("CYCL DEF 9.1 takes one name word and the dwell time"));
        }
        let seconds = check_dwell(b.line, value(b, "dwell time", v, &self.q)?)?;
        self.dwells.push(DwellEvent {
            line: b.line,
            seconds,
            section: self.current.index,
            after_move: self.current.builder.moves_len().checked_sub(1),
        });
        Ok(())
    }

    /// `CYCL DEF 32.1 T<mm>`, or `CYCL DEF 32.1` alone (reset to the machine
    /// tolerance). The tolerance does not change the programmed path that the
    /// reader gives; it is checked and not stored.
    fn tolerance_block(&mut self, b: &Block, rest: &[String]) -> Result<(), GcodeError> {
        match rest {
            [] => Ok(()),
            [t] => {
                let Some(v) = t.strip_prefix('T') else {
                    return Err(b.unknown(t));
                };
                let mm = check_len(b.line, "tolerance T", value(b, "tolerance T", v, &self.q)?)?;
                if mm < 0.0 {
                    return Err(b.invalid_cycle(&format!("tolerance T ({mm}) must not be negative")));
                }
                Ok(())
            }
            _ => Err(b.syntax("CYCL DEF 32.1 takes one T value or nothing")),
        }
    }
}

/// `CYCL DEF 32.2 HSC-MODE:<0|1> [TA<deg>]`: checked, no effect on the path.
fn hsc_block(b: &Block, rest: &[String], m: &Machine<'_>) -> Result<(), GcodeError> {
    let joined = rest.concat();
    let Some(after) = joined.strip_prefix("HSC-MODE:") else {
        return Err(b.syntax("CYCL DEF 32.2 needs HSC-MODE:0 or HSC-MODE:1"));
    };
    let ta = match after.as_bytes().first() {
        Some(b'0' | b'1') => &after[1..],
        _ => return Err(b.syntax("CYCL DEF 32.2 needs HSC-MODE:0 or HSC-MODE:1")),
    };
    if ta.is_empty() {
        return Ok(());
    }
    let Some(v) = ta.strip_prefix("TA") else {
        return Err(b.unknown(ta));
    };
    let deg = value(b, "rotary tolerance TA", v, &m.q)?;
    if !(0.0..=360.0).contains(&deg) {
        return Err(b.invalid_cycle(&format!("rotary tolerance TA ({deg}) must be in [0, 360] degrees")));
    }
    Ok(())
}
