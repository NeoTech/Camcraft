//! Klartext source splitting: physical lines into numbered blocks, `~` continuation lines, `;` comments, word tokens and literal numbers.
//! Provenance: HEIDENHAIN iTNC 530 conversational user's manual 533 190-22 (block layout, `~` continuation, `;` comments) as summarized in docs/research/heidenhain/, and CamCraft's own Klartext post output (assets/posts/heidenhain-klartext.cps).

use crate::types::GcodeError;

/// One logical Klartext block: a numbered line plus its `~` continuation lines.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// 1-based physical line where the block starts.
    pub line: usize,
    /// The block number written at the start of the line, if any.
    pub number: Option<u64>,
    /// Upper-case word tokens without comments; a quoted token keeps its quotes and case.
    pub words: Vec<String>,
}

impl Block {
    /// "block N: " for a numbered block, empty otherwise. Refusals start with it.
    pub fn tag(&self) -> String {
        match self.number {
            Some(n) => format!("block {n}: "),
            None => String::new(),
        }
    }

    pub fn unsupported(&self, code: &str, why: &str) -> GcodeError {
        GcodeError::UnsupportedCode {
            line: self.line,
            code: code.to_string(),
            reason: format!("{}{why}", self.tag()),
        }
    }

    pub fn syntax(&self, why: &str) -> GcodeError {
        GcodeError::Syntax {
            line: self.line,
            message: format!("{}{why}", self.tag()),
        }
    }

    pub fn unknown(&self, word: &str) -> GcodeError {
        GcodeError::UnknownWord {
            line: self.line,
            word: format!("{}{word}", self.tag()),
        }
    }

    pub fn invalid_arc(&self, why: &str) -> GcodeError {
        GcodeError::InvalidArc {
            line: self.line,
            message: format!("{}{why}", self.tag()),
        }
    }

    pub fn invalid_cycle(&self, why: &str) -> GcodeError {
        GcodeError::InvalidCycle {
            line: self.line,
            message: format!("{}{why}", self.tag()),
        }
    }

    pub fn invalid_modal(&self, why: &str) -> GcodeError {
        GcodeError::InvalidModal {
            line: self.line,
            message: format!("{}{why}", self.tag()),
        }
    }
}

/// Split a Klartext program into logical blocks. Blank and comment-only lines
/// give no block. Returns the blocks and the count of physical lines.
pub fn split_blocks(program: &str) -> Result<(Vec<Block>, usize), GcodeError> {
    let mut blocks = Vec::new();
    let mut pending: Option<(usize, String)> = None;
    let mut count = 0;
    for (idx, raw) in program.lines().enumerate() {
        let line = idx + 1;
        count = line;
        let mut text = raw.trim();
        let cont = text.ends_with('~');
        if cont {
            text = text[..text.len() - 1].trim_end();
        }
        let code = strip_comment(text);
        let (start, mut buf) = pending.take().unwrap_or((line, String::new()));
        buf.push(' ');
        buf.push_str(code);
        if cont {
            pending = Some((start, buf));
            continue;
        }
        if let Some(b) = make_block(start, &buf)? {
            blocks.push(b);
        }
    }
    if let Some((start, _)) = pending {
        return Err(GcodeError::Syntax {
            line: start,
            message:
                "the block ends with '~' but no continuation line follows (truncated program?)"
                    .to_string(),
        });
    }
    Ok((blocks, count))
}

/// The text before the first `;` that is not inside double quotes.
fn strip_comment(text: &str) -> &str {
    let mut quoted = false;
    for (i, c) in text.char_indices() {
        match c {
            '"' => quoted = !quoted,
            ';' if !quoted => return &text[..i],
            _ => {}
        }
    }
    text
}

fn make_block(line: usize, text: &str) -> Result<Option<Block>, GcodeError> {
    let mut words = tokenize(line, text)?;
    let mut number = None;
    if let Some(first) = words.first()
        && !first.is_empty()
        && first.bytes().all(|b| b.is_ascii_digit())
    {
        let n = first.parse::<u64>().map_err(|_| GcodeError::Syntax {
            line,
            message: format!("block number '{first}' is too large"),
        })?;
        number = Some(n);
        words.remove(0);
    }
    // A structure block (`* - text`) is a comment.
    if words.is_empty() || words[0].starts_with('*') {
        return Ok(None);
    }
    Ok(Some(Block {
        line,
        number,
        words,
    }))
}

/// Split on white space; a double-quoted run is one token (quotes and case kept).
fn tokenize(line: usize, text: &str) -> Result<Vec<String>, GcodeError> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for c in text.chars() {
        if quoted {
            cur.push(c);
            if c == '"' {
                quoted = false;
            }
        } else if c == '"' {
            cur.push(c);
            quoted = true;
        } else if c.is_whitespace() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else {
            cur.extend(c.to_uppercase());
        }
    }
    if quoted {
        return Err(GcodeError::Syntax {
            line,
            message: "unterminated '\"' in the block".to_string(),
        });
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    Ok(out)
}

/// A literal Klartext number: an optional sign, digits and one optional decimal
/// point (`+10`, `-2.5`, `.5`). No exponent, no `inf`/`nan`; the value is finite.
pub fn literal(tok: &str) -> Option<f64> {
    let body = tok.strip_prefix(['+', '-']).unwrap_or(tok);
    let mut digits = 0usize;
    let mut points = 0usize;
    for b in body.bytes() {
        match b {
            b'0'..=b'9' => digits += 1,
            b'.' => points += 1,
            _ => return None,
        }
    }
    if digits == 0 || points > 1 {
        return None;
    }
    tok.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// A Q parameter reference `Q5`, `+Q5` or `-Q5`: the index and the sign.
pub fn q_ref(tok: &str) -> Option<(u32, f64)> {
    let (sign, body) = match tok.as_bytes().first() {
        Some(b'-') => (-1.0, &tok[1..]),
        Some(b'+') => (1.0, &tok[1..]),
        _ => (1.0, tok),
    };
    let idx = q_index(body)?;
    Some((idx, sign))
}

/// Highest Q parameter index of the iTNC 530 (Q0 to Q1999).
pub const MAX_Q: u32 = 1999;

/// The index of a `Q<n>` token, with `n` in 0..=[`MAX_Q`].
pub fn q_index(tok: &str) -> Option<u32> {
    let digits = tok.strip_prefix('Q')?;
    if digits.is_empty() || digits.len() > 4 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse::<u32>().ok().filter(|n| *n <= MAX_Q)
}
