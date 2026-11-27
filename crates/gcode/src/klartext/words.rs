//! Word decoding for Klartext path blocks (`L`, `CC`, `C`, `CR`, `CT`): coordinates, radius compensation, feed, direction, radius and M functions.
//! Provenance: HEIDENHAIN iTNC 530 conversational user's manual 533 190-22 (path function words), as summarized in docs/research/heidenhain/differences.md, and the blocks of assets/posts/heidenhain-klartext.cps.

use std::collections::HashMap;

use super::source::{Block, literal, q_ref};
use crate::types::GcodeError;

/// The feed word of a path block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Feed {
    /// `FMAX`: rapid traverse for this block only.
    Max,
    /// `F<n>`: feed rate in mm/min, modal.
    Rate(f64),
}

/// The `MB` word of `M140`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mb {
    /// `MB MAX`: retract to the end of the traverse range.
    Max,
    /// `MB <n>`: retract by `n` mm in the tool axis.
    Dist(f64),
}

/// Decoded words of one path block.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Words {
    /// Absolute `X`/`Y`/`Z`.
    pub abs: [Option<f64>; 3],
    /// Incremental `IX`/`IY`/`IZ`.
    pub inc: [Option<f64>; 3],
    /// `R0` was written.
    pub r0: bool,
    pub feed: Option<Feed>,
    /// `DR+` (true, counter-clockwise) or `DR-` (false, clockwise).
    pub ccw: Option<bool>,
    /// `R+n` / `R-n` of a `CR` block.
    pub radius: Option<f64>,
    pub m: Vec<u32>,
    pub mb: Option<Mb>,
}

impl Words {
    pub fn has_coords(&self) -> bool {
        self.abs.iter().chain(self.inc.iter()).any(Option::is_some)
    }
}

/// Q parameter values set by `FN 0:` or `Qn =` blocks.
pub type QTable = HashMap<u32, f64>;

/// A numeric value: a literal, or a Q parameter that a literal assignment set.
pub fn value(b: &Block, what: &str, tok: &str, q: &QTable) -> Result<f64, GcodeError> {
    if let Some(v) = literal(tok) {
        return Ok(v);
    }
    if let Some((idx, sign)) = q_ref(tok) {
        return q.get(&idx).map(|v| sign * v).ok_or_else(|| {
            b.syntax(&format!(
                "{what} uses Q{idx}, but no block before sets Q{idx}"
            ))
        });
    }
    Err(b.syntax(&format!("{what} value '{tok}' is not a number")))
}

const AXES: [&str; 3] = ["X", "Y", "Z"];

/// Decode the words after the block keyword.
pub fn decode(b: &Block, toks: &[String], q: &QTable) -> Result<Words, GcodeError> {
    let mut w = Words::default();
    let mut i = 0;
    while i < toks.len() {
        let t = toks[i].as_str();
        i += 1;
        if t == "MB" {
            let Some(arg) = toks.get(i) else {
                return Err(b.syntax("MB needs MAX or a distance"));
            };
            i += 1;
            set_mb(b, &mut w, arg, q)?;
            continue;
        }
        if let Some(arg) = t.strip_prefix("MB") {
            set_mb(b, &mut w, arg, q)?;
            continue;
        }
        decode_one(b, &mut w, t, q)?;
    }
    Ok(w)
}

fn set_mb(b: &Block, w: &mut Words, arg: &str, q: &QTable) -> Result<(), GcodeError> {
    if w.mb.is_some() {
        return Err(b.syntax("MB appears twice"));
    }
    w.mb = Some(if arg == "MAX" {
        Mb::Max
    } else {
        Mb::Dist(value(b, "MB", arg, q)?)
    });
    Ok(())
}

fn decode_one(b: &Block, w: &mut Words, t: &str, q: &QTable) -> Result<(), GcodeError> {
    match t {
        "R0" => return once(b, &mut w.r0, t),
        "RL" | "RR" => return Err(b.unsupported(
            t,
            "radius compensation RL/RR is not read; this reader needs the tool-centre path (R0)",
        )),
        "FMAX" => return set(b, &mut w.feed, Feed::Max, "F"),
        "FAUTO" | "FU" | "FZ" | "FT" => {
            return Err(b.unsupported(t, "only F in mm/min and FMAX are read"));
        }
        "DR+" => return set(b, &mut w.ccw, true, "DR"),
        "DR-" => return set(b, &mut w.ccw, false, "DR"),
        _ => {}
    }
    for (k, axis) in AXES.iter().enumerate() {
        if let Some(rest) = t.strip_prefix('I').and_then(|r| r.strip_prefix(axis)) {
            if w.abs[k].is_some() {
                return Err(b.syntax(&format!("{axis} and I{axis} in one block")));
            }
            let v = value(b, &format!("I{axis}"), rest, q)?;
            return set(b, &mut w.inc[k], v, &format!("I{axis}"));
        }
        if let Some(rest) = t.strip_prefix(axis) {
            if w.inc[k].is_some() {
                return Err(b.syntax(&format!("{axis} and I{axis} in one block")));
            }
            let v = value(b, axis, rest, q)?;
            return set(b, &mut w.abs[k], v, axis);
        }
    }
    if let Some(rest) = t.strip_prefix('M')
        && !rest.is_empty()
        && rest.len() <= 4
        && rest.bytes().all(|c| c.is_ascii_digit())
    {
        let n: u32 = rest
            .parse()
            .map_err(|_| b.syntax(&format!("bad M word '{t}'")))?;
        w.m.push(n);
        return Ok(());
    }
    if let Some(rest) = t.strip_prefix('F') {
        let f = value(b, "F", rest, q)?;
        return set(b, &mut w.feed, Feed::Rate(f), "F");
    }
    if let Some(rest) = t.strip_prefix('R')
        && (rest.starts_with(['+', '-']) || literal(rest).is_some() || q_ref(rest).is_some())
    {
        let r = value(b, "R", rest, q)?;
        return set(b, &mut w.radius, r, "R");
    }
    if let Some(axis) = [
        "A", "B", "C", "U", "V", "W", "IA", "IB", "IC", "IU", "IV", "IW",
    ]
    .iter()
    .find(|a| {
        t.strip_prefix(**a)
            .is_some_and(|r| literal(r).is_some() || q_ref(r).is_some())
    }) {
        return Err(b.unsupported(
            &format!("axis {axis}"),
            "only the X, Y and Z axes are read (3-axis reader)",
        ));
    }
    if ["PA", "PR", "IPA", "IPR", "PDX", "PDY"]
        .iter()
        .any(|p| t.starts_with(p))
    {
        return Err(b.unsupported(t, "polar coordinates are not read"));
    }
    Err(b.unknown(t))
}

fn once(b: &Block, flag: &mut bool, t: &str) -> Result<(), GcodeError> {
    if *flag {
        return Err(b.syntax(&format!("{t} appears twice")));
    }
    *flag = true;
    Ok(())
}

fn set<T>(b: &Block, slot: &mut Option<T>, v: T, name: &str) -> Result<(), GcodeError> {
    if slot.is_some() {
        return Err(b.syntax(&format!("{name} appears twice")));
    }
    *slot = Some(v);
    Ok(())
}
