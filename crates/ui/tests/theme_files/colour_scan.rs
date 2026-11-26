//! The colour-literal scanner of the #1120 guard (`colour_literals`):
//! what counts as a colour literal in Rust source, and the comment
//! stripper it reads through. Split from the guard for the 400-line law.
//!
//! A literal is a named constant (`Color32::WHITE`) or a constructor
//! whose arguments are all numbers written in place
//! (`Color32::from_rgb(245, 240, 250)`). A constructor over variables
//! (`from_rgb(r, g, b)`) converts a colour chosen elsewhere and does not
//! count. Comments (`//` to the line end, `/* */`) do not count; the text
//! of a string or char literal is kept as it is.
//! Provenance: our own scanner, specced from the Rust reference's lexical
//! rules for comments, strings and char literals.

/// A colour written as a value at the call site. `Color32::TRANSPARENT`
/// is not in the list: it carries no hue for a theme to change.
const PATTERNS: &[&str] = &[
    "Color32::from_rgb(",
    "Color32::from_rgba_unmultiplied(",
    "Color32::from_rgba_premultiplied(",
    "Color32::from_gray(",
    "Color32::from_black_alpha(",
    "Color32::from_white_alpha(",
    "Color32::from_hex(",
    "Rgba::from_rgb(",
    "Color32::WHITE",
    "Color32::BLACK",
    "Color32::GRAY",
    "Color32::LIGHT_GRAY",
    "Color32::DARK_GRAY",
    "Color32::RED",
    "Color32::LIGHT_RED",
    "Color32::DARK_RED",
    "Color32::GREEN",
    "Color32::LIGHT_GREEN",
    "Color32::DARK_GREEN",
    "Color32::BLUE",
    "Color32::LIGHT_BLUE",
    "Color32::DARK_BLUE",
    "Color32::YELLOW",
    "Color32::LIGHT_YELLOW",
    "Color32::ORANGE",
    "Color32::GOLD",
    "Color32::BROWN",
    "Color32::KHAKI",
];

/// A number written in place: `255`, `0x2B`, `1.0`, `0.5_f32`.
fn is_number(arg: &str) -> bool {
    let a = arg.trim();
    a.starts_with(|c: char| c.is_ascii_digit())
        && a.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

/// The arguments of the call whose `(` ends at `open`, split at the
/// commas of its own depth; `None` when the call does not close.
fn call_args(text: &str, open: usize) -> Option<Vec<&str>> {
    let mut depth = 0usize;
    let mut start = open;
    let mut args = Vec::new();
    for (k, c) in text[open..].char_indices() {
        let at = open + k;
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    args.push(&text[start..at]);
                    return Some(args);
                }
            }
            ',' if depth == 1 => {
                args.push(&text[start..at]);
                start = at + 1;
            }
            _ => {}
        }
        if at == open {
            start = open + 1;
        }
    }
    None
}

/// The length of the string literal that starts at `chars[i]` (`"`,
/// `r"`, `r#"`, `b"`, `br#"`), or `None` when no string starts there.
fn string_len(chars: &[char], i: usize) -> Option<usize> {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    if i > 0 && ident(chars[i - 1]) {
        return None;
    }
    let mut j = i;
    if chars.get(j) == Some(&'b') {
        j += 1;
    }
    let raw = chars.get(j) == Some(&'r');
    if raw {
        j += 1;
    }
    let hashes = chars[j.min(chars.len())..]
        .iter()
        .take_while(|c| **c == '#')
        .count();
    if !raw && hashes > 0 {
        return None;
    }
    j += hashes;
    if chars.get(j) != Some(&'"') {
        return None;
    }
    j += 1;
    while j < chars.len() {
        match chars[j] {
            '\\' if !raw => j += 2,
            '"' if chars[j + 1..].iter().take(hashes).all(|c| *c == '#')
                && chars.len() > j + hashes =>
            {
                return Some(j + 1 + hashes - i);
            }
            _ => j += 1,
        }
    }
    Some(chars.len() - i)
}

/// The length of the char literal that starts at `chars[i]` (`'a'`,
/// `'\''`, `'"'`, `'\u{2F}'`), or `None` for a lifetime or a label.
fn char_len(chars: &[char], i: usize) -> Option<usize> {
    match (chars.get(i + 1), chars.get(i + 2)) {
        (Some('\\'), Some(_)) => {
            let end = chars.get(i + 3..)?.iter().position(|c| *c == '\'')?;
            Some(end + 4)
        }
        (Some(_), Some('\'')) => Some(3),
        _ => None,
    }
}

/// The source with its comments removed: `//` to the line end, and
/// `/* */` blocks (nested, as Rust reads them). The newlines stay, and
/// the text of string and char literals is kept as it is.
fn strip_comments(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let (c, next) = (chars[i], chars.get(i + 1).copied());
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && next == Some('*') {
            let mut depth = 0usize;
            while i < chars.len() {
                match (chars[i], chars.get(i + 1).copied()) {
                    ('/', Some('*')) => {
                        depth += 1;
                        i += 2;
                    }
                    ('*', Some('/')) => {
                        depth = depth.saturating_sub(1);
                        i += 2;
                        if depth == 0 {
                            break;
                        }
                    }
                    ('\n', _) => {
                        out.push('\n');
                        i += 1;
                    }
                    _ => i += 1,
                }
            }
            out.push(' ');
        } else if let Some(n) =
            string_len(&chars, i).or_else(|| if c == '\'' { char_len(&chars, i) } else { None })
        {
            let end = (i + n).min(chars.len());
            out.extend(&chars[i..end]);
            i = end;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// The colour literals of one file. A constructor counts only when each
/// argument is a number written in place: `from_rgb(r, g, b)` converts a
/// colour chosen elsewhere and is not a literal. A named constant
/// (`Color32::WHITE`) always counts. A comment does not count.
pub(super) fn count(text: &str) -> usize {
    let code = strip_comments(text);
    PATTERNS
        .iter()
        .map(|pat| {
            code.match_indices(pat)
                .filter(|(i, _)| {
                    let end = i + pat.len();
                    if pat.ends_with('(') {
                        call_args(&code, end - 1).is_some_and(|args| {
                            // A trailing comma leaves one empty argument.
                            let args: Vec<&&str> =
                                args.iter().filter(|a| !a.trim().is_empty()).collect();
                            !args.is_empty() && args.iter().all(|a| is_number(a))
                        })
                    } else {
                        // `Color32::RED` must not count `Color32::REDDISH`-like names.
                        !code[end..].starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
                    }
                })
                .count()
        })
        .sum()
}

#[test]
fn the_counter_sees_each_form_and_skips_comments() {
    let text = "let a = Color32::from_rgb(1, 2, 3);\n\
                // Color32::from_rgb(9, 9, 9) in a comment\n\
                let b = [Color32::WHITE, Color32::TRANSPARENT, Color32::from_gray(4)];\n\
                let c = Color32::RED; let d = Color32::REDDISH;\n\
                let e = Color32::from_rgb(\n    0x10,\n    0x20,\n    0x30,\n);\n\
                let f = Color32::from_rgba_unmultiplied(r, g, b, ALPHA);\n\
                let g = Color32::from_rgb(rgb.0, rgb.1, rgb.2);\n\
                let h = Color32::from_black_alpha(170);\n";
    assert_eq!(
        count(text),
        6,
        "from_rgb, WHITE, from_gray, RED, the split from_rgb and from_black_alpha; \
         not the comment, TRANSPARENT, REDDISH or a conversion of variables"
    );
}

#[test]
fn a_comment_after_code_neither_hides_nor_adds_a_literal() {
    // The two probes of the #3990 review.
    assert_eq!(
        count("let c = Color32::from_rgb(1, // red\n2, 3);\n"),
        1,
        "a comment inside the call does not hide its numbers"
    );
    assert_eq!(
        count("let x = 0; // Color32::WHITE\n"),
        0,
        "a colour in a comment after code does not count"
    );
}

#[test]
fn strings_and_char_literals_keep_their_text() {
    assert_eq!(
        count("let s = \"http://x\"; let c = Color32::WHITE;\n"),
        1,
        "a // inside a string is not a comment"
    );
    assert_eq!(
        count("let s = r#\"a \"// b\"#; let c = Color32::WHITE;\n"),
        1,
        "nor inside a raw string"
    );
    assert_eq!(
        count("let q = '\"'; let c = Color32::BLACK; let l: &'static str = \"\";\n"),
        1,
        "a quote char literal opens no string; a lifetime is not a char"
    );
    assert_eq!(
        count("let a = /* Color32::WHITE /* nested */ */ Color32::from_gray(7);\n"),
        1,
        "a block comment does not count, nested or not"
    );
}
