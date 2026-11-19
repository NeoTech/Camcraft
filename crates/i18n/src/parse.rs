//! The catalogue reader: `"English source" = "translation"`, one pair a line.
//!
//! Provenance: our own reader over our own file format. The format is the
//! smallest thing that survives a translator's editor: a comment line, a
//! blank line, or one quoted pair. Anything else is refused with its line
//! number and the reason, never skipped - a silently ignored line is a
//! string that stays English forever and nobody finds out (#853).

/// One line of a catalogue, or the reason it is not one.
pub(crate) enum Row {
    /// A `"source" = "translation"` pair.
    Pair(String, String),
    /// A comment or a blank line: nothing to record.
    Ignore,
    /// A line the reader refuses, with the reason.
    Refuse(String),
}

/// Read one catalogue line. `line` is 1-based, for the refusal's name.
pub(crate) fn row(line: &str, number: usize) -> Row {
    let t = line.trim();
    if t.is_empty() || t.starts_with('#') {
        return Row::Ignore;
    }
    let Some(rest) = t.strip_prefix('"') else {
        return Row::Refuse(format!("line {number}: a key must start with a quote: {t}"));
    };
    let (key, rest) = match quoted(rest) {
        Some(v) => v,
        None => return Row::Refuse(format!("line {number}: unterminated key: {t}")),
    };
    let Some(rest) = rest.trim_start().strip_prefix('=') else {
        return Row::Refuse(format!("line {number}: expected `=` after the key: {t}"));
    };
    let Some(rest) = rest.trim_start().strip_prefix('"') else {
        return Row::Refuse(format!("line {number}: the translation must start with a quote: {t}"));
    };
    let (value, rest) = match quoted(rest) {
        Some(v) => v,
        None => return Row::Refuse(format!("line {number}: unterminated translation: {t}")),
    };
    let tail = rest.trim();
    if !tail.is_empty() && !tail.starts_with('#') {
        return Row::Refuse(format!("line {number}: trailing text after the pair: {t}"));
    }
    if key.is_empty() {
        return Row::Refuse(format!("line {number}: the key is empty"));
    }
    if value.is_empty() {
        return Row::Refuse(format!("line {number}: the translation is empty: {key}"));
    }
    Row::Pair(key, value)
}

/// Read a quoted run (the opening quote is already eaten) and return its
/// unescaped text and the rest of the line. `\"` `\\` `\n` `\t` are the only
/// escapes; a backslash before anything else stays literal, so a translator
/// typing `\d` in a regular expression keeps it.
fn quoted(s: &str) -> Option<(String, &str)> {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Some((out, &s[i + 1..])),
            '\\' => match chars.next() {
                Some((_, '"')) => out.push('"'),
                Some((_, '\\')) => out.push('\\'),
                Some((_, 'n')) => out.push('\n'),
                Some((_, 't')) => out.push('\t'),
                Some((_, other)) => {
                    out.push('\\');
                    out.push(other);
                }
                None => return None,
            },
            other => out.push(other),
        }
    }
    None
}
