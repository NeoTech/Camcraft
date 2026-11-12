//! STEP nesting guard: one linear byte scan that refuses a record whose
//! parentheses nest past [`MAX_NESTING`] before the recursive-descent
//! parser can overflow its stack on it (#1075).
//!
//! Provenance: lexical rules (strings `'...'` with `''` as an escaped
//! quote, comments `/* ... */`, entity ids `#n`) per ISO 10303-21; the cap
//! is our own measurement, recorded in docs/decisions/1075-step-nesting-cap.md.
//!
//! The scan only measures depth. It does not judge syntax: an unbalanced
//! `)` (depth would go negative), an unterminated string or comment are
//! left for the parser to report.

use super::ImportError;

/// Deepest parenthesis nesting a STEP record may have. Real files nest a
/// handful deep (our fixtures reach 3); ruststep's parser, in a debug
/// build, dies at about 400 levels on a 1 MiB stack (the Windows main
/// thread, where an inline import runs) and about 820 on 2 MiB (release:
/// about 530 and 1080).
pub const MAX_NESTING: usize = 64;

/// Refuse `data` by name if any record nests deeper than [`MAX_NESTING`].
pub fn check(data: &[u8]) -> Result<(), ImportError> {
    check_with_cap(data, MAX_NESTING)
}

/// The deepest parenthesis nesting anywhere in `data` (strings and
/// comments skipped). For tests and diagnostics.
#[must_use]
pub fn max_depth(data: &[u8]) -> usize {
    let mut deepest = 0;
    scan(data, |depth, _, _| {
        deepest = deepest.max(depth);
        true
    });
    deepest
}

fn check_with_cap(data: &[u8], cap: usize) -> Result<(), ImportError> {
    let mut over: Option<(Option<u64>, usize)> = None;
    scan(data, |depth, entity, line| {
        if depth > cap {
            over = Some((entity, line));
            return false;
        }
        true
    });
    match over {
        None => Ok(()),
        Some((entity, line)) => {
            let at = entity.map_or_else(
                || "a record without an entity id".to_string(),
                |id| format!("entity #{id}"),
            );
            Err(ImportError::Step(format!(
                "record nests deeper than {cap} levels of parentheses at {at}, line {line}"
            )))
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Code,
    Str,
    Comment,
}

/// Walk `data` once, calling `on_open(depth, entity, line)` after every `(`
/// outside strings and comments; stop when it returns `false`. `entity` is
/// the `#id` of the record being read (the id that opened it at depth 0),
/// `line` is 1-based and counts `\n` only, so CRLF and LF agree.
fn scan(data: &[u8], mut on_open: impl FnMut(usize, Option<u64>, usize) -> bool) {
    let mut state = State::Code;
    let mut depth: usize = 0;
    let mut line: usize = 1;
    let mut entity: Option<u64> = None;
    let mut i = 0;
    while let Some(&b) = data.get(i) {
        let next = data.get(i + 1).copied();
        if b == b'\n' {
            line = line.saturating_add(1);
        }
        match state {
            State::Str => {
                if b == b'\'' {
                    if next == Some(b'\'') {
                        i += 1; // escaped quote, still inside the string
                    } else {
                        state = State::Code;
                    }
                }
            }
            State::Comment => {
                if b == b'*' && next == Some(b'/') {
                    state = State::Code;
                    i += 1;
                }
            }
            State::Code => match b {
                b'\'' => state = State::Str,
                b'/' if next == Some(b'*') => {
                    state = State::Comment;
                    i += 1;
                }
                b'(' => {
                    depth = depth.saturating_add(1);
                    if !on_open(depth, entity, line) {
                        return;
                    }
                }
                b')' => depth = depth.saturating_sub(1),
                b';' if depth == 0 => entity = None,
                b'#' if depth == 0 => {
                    // The record's own id: `#` then digits, at depth 0.
                    let mut id: u64 = 0;
                    let mut j = i + 1;
                    let mut any = false;
                    while let Some(d) = data.get(j).filter(|d| d.is_ascii_digit()) {
                        id = id.saturating_mul(10).saturating_add(u64::from(d - b'0'));
                        any = true;
                        j += 1;
                    }
                    if any {
                        entity = Some(id);
                        i = j - 1;
                    }
                }
                _ => {}
            },
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested(depth: usize) -> String {
        format!(
            "#7 = THING({}1{});\n",
            "(".repeat(depth - 1),
            ")".repeat(depth - 1)
        )
    }

    #[test]
    fn depth_counts_parentheses_of_a_record() {
        assert_eq!(max_depth(nested(5).as_bytes()), 5);
        assert_eq!(max_depth(b"#1 = A(1.0, (2, 3), $);"), 2);
    }

    #[test]
    fn strings_and_comments_do_not_count() {
        assert_eq!(max_depth(b"#1 = A('(((((', 'it''s ((( ');"), 1);
        assert_eq!(max_depth(b"/* ((((( */ #1 = A(1);"), 1);
        // A `''` escape must not end the string early.
        assert_eq!(max_depth(b"#1 = A('a''(((b');"), 1);
    }

    #[test]
    fn refusal_names_cap_entity_and_line() {
        let text = format!("DATA;\n#1 = A(1);\n{}ENDSEC;\n", nested(10));
        let err = check_with_cap(text.as_bytes(), 8)
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(err.contains("deeper than 8"), "{err}");
        assert!(err.contains("entity #7"), "{err}");
        assert!(err.contains("line 3"), "{err}");
    }

    #[test]
    fn at_the_cap_passes() {
        assert!(check_with_cap(nested(8).as_bytes(), 8).is_ok());
        assert!(check_with_cap(nested(9).as_bytes(), 8).is_err());
    }

    #[test]
    fn crlf_and_lf_give_the_same_line() {
        let lf = format!("A;\n\n#1 = B(1);\n{}", nested(10));
        let crlf = lf.replace('\n', "\r\n");
        let a = check_with_cap(lf.as_bytes(), 8)
            .err()
            .map(|e| e.to_string());
        let b = check_with_cap(crlf.as_bytes(), 8)
            .err()
            .map(|e| e.to_string());
        assert!(a.as_deref().is_some_and(|m| m.contains("line 4")), "{a:?}");
        assert_eq!(a, b);
    }

    #[test]
    fn unbalanced_and_non_utf8_are_left_to_the_parser() {
        assert!(check(b"#1 = A(1)));)))(((;").is_ok());
        assert!(check(&[0xff, 0xfe, b'(', 0x80, b')']).is_ok());
        assert!(check(b"#1 = A('unterminated (((((").is_ok());
    }
}
