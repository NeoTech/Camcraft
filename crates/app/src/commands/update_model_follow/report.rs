//! The re-import report of the name follow (#3915): the picks that
//! followed, each pick that could not, the window's phrase, the import
//! answer's `picks` key, and the `needs_review` line of an operation.
//! Provenance: our own wording and format (#4213, D5, D6, D10).

use serde_json::{Value, json};

use camcraft_geometry::source_follow::PickKind;

/// The longest `needs_review` line a project file holds
/// (`project::validate` refuses a longer one as corrupt).
const NOTE_MAX: usize = 240;
/// The longest SolidCraft name the window and a review line show, bytes.
const NAME_SHOWN: usize = 60;
/// How many lost picks the window's phrase names; the import answer
/// names them all.
const STATUS_LOST: usize = 8;
/// The longest phrase the window's status gets from the follow, bytes.
const STATUS_MAX: usize = 900;

/// One pick of the follow, for the re-import report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickNote {
    /// The operation's stable key (MCP and scripts address it by this).
    pub operation: String,
    /// The operation's label, as people see it.
    pub label: String,
    /// `"face"` or `"curved face"`.
    pub kind: &'static str,
    /// The face's SolidCraft names before the import, sorted.
    pub names: Vec<String>,
    /// Why it did not follow; `None` for a followed pick.
    pub why: Option<String>,
}

/// What the follow did, for the re-import report.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FollowReport {
    pub followed: Vec<PickNote>,
    pub lost: Vec<PickNote>,
}

impl FollowReport {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.followed.is_empty() && self.lost.is_empty()
    }

    /// The window's phrase: how many picks followed, and each pick that
    /// could not (the first [`STATUS_LOST`], then a count). `None` when
    /// the follow did nothing.
    #[must_use]
    pub fn describe(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }
        let mut parts: Vec<String> = Vec::new();
        let n = self.followed.len();
        if n > 0 {
            parts.push(format!("{n} pick{} followed by SolidCraft name", plural(n)));
        }
        let n = self.lost.len();
        if n > 0 {
            let mut each: Vec<String> = self
                .lost
                .iter()
                .take(STATUS_LOST)
                .map(|p| {
                    format!(
                        "{} in '{}' ({})",
                        shown(p),
                        clean(&p.label),
                        p.why.as_deref().unwrap_or("")
                    )
                })
                .collect();
            if n > STATUS_LOST {
                each.push(format!("and {} more", n - STATUS_LOST));
            }
            parts.push(format!(
                "{n} pick{} could not follow: {}",
                plural(n),
                each.join("; ")
            ));
        }
        Some(clamp(&parts.join("; "), STATUS_MAX))
    }

    /// The `picks` key of the import answer: every followed and every
    /// lost pick, by operation key, with its names. `None` when the
    /// follow did nothing, so the answer is as before.
    #[must_use]
    pub fn to_json(&self) -> Option<Value> {
        if self.is_empty() {
            return None;
        }
        let row = |p: &PickNote| {
            let mut r = json!({ "operation": p.operation, "pick": p.kind, "names": p.names });
            if let Some(why) = &p.why {
                r["why"] = json!(why);
            }
            r
        };
        Some(json!({
            "followed": self.followed.iter().map(row).collect::<Vec<_>>(),
            "lost": self.lost.iter().map(row).collect::<Vec<_>>(),
        }))
    }
}

pub(super) fn noun(kind: PickKind) -> &'static str {
    match kind {
        PickKind::Face => "face",
        PickKind::Patch => "curved face",
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// A pick in words: `face 'F2:side:b'`, `curved face 'a + b'`, or the
/// bare kind when it had no names.
fn shown(p: &PickNote) -> String {
    if p.names.is_empty() {
        return p.kind.to_owned();
    }
    let names: Vec<String> = p
        .names
        .iter()
        .map(|n| clamp(&clean(n), NAME_SHOWN))
        .collect();
    format!("{} '{}'", p.kind, names.join(" + "))
}

/// The `needs_review` line of one operation: which picks of model `name`
/// followed their names. At most [`NOTE_MAX`] bytes, no control
/// characters - the project loader refuses anything else as corrupt.
pub(super) fn review_line(name: &str, followed: &[PickNote]) -> String {
    let mut each: Vec<String> = followed.iter().map(shown).collect();
    each.dedup();
    let model = clamp(&clean(name), NAME_SHOWN);
    let line = match each.as_slice() {
        [one] => format!(
            "{one} of '{model}' followed by its SolidCraft name after the model was imported again"
        ),
        many => format!(
            "{} picks of '{model}' followed by their SolidCraft names after the model was imported again: {}",
            many.len(),
            many.join(", ")
        ),
    };
    clamp(&line, NOTE_MAX)
}

/// `s` with each control character (a newline, a tab) as a space.
fn clean(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// `s` cut to at most `max` bytes on a character boundary, with `...`
/// when it was cut and there is room for it. Never panics.
fn clamp(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_owned();
    }
    let dots = if max >= 3 { "..." } else { "" };
    let room = max.saturating_sub(dots.len());
    let mut end = 0;
    for (i, c) in s.char_indices() {
        let next = i.saturating_add(c.len_utf8());
        if next > room {
            break;
        }
        end = next;
    }
    format!("{}{dots}", s.get(..end).unwrap_or(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(names: &[&str]) -> PickNote {
        PickNote {
            operation: "op1".into(),
            label: "Face1".into(),
            kind: "face",
            names: names.iter().map(|n| (*n).to_owned()).collect(),
            why: None,
        }
    }

    #[test]
    fn a_review_line_is_short_and_clean_whatever_the_names() {
        let long = "x".repeat(300);
        let hostile = ["F2:side:b\nnext line", long.as_str(), "\u{e9}\u{e9}\u{e9}"];
        for names in [
            vec![hostile[0]],
            vec![hostile[1]],
            vec![hostile[0], hostile[1], hostile[2]],
        ] {
            let line = review_line("part\tname", &[note(&names), note(&["top"])]);
            assert!(line.len() <= NOTE_MAX, "{} bytes: {line}", line.len());
            assert!(!line.chars().any(char::is_control), "{line:?}");
            assert!(!line.trim().is_empty());
        }
        let one = review_line("part", &[note(&["F2:side:b"])]);
        assert_eq!(
            one,
            "face 'F2:side:b' of 'part' followed by its SolidCraft name after the model was imported again"
        );
    }

    #[test]
    fn clamp_cuts_on_a_character_boundary() {
        let s = "\u{e9}".repeat(200); // two bytes each
        let c = clamp(&s, 11);
        assert!(c.len() <= 11 && c.ends_with("..."), "{c}");
        assert_eq!(clamp("short", 11), "short");
        assert_eq!(clamp("", 0), "");
        assert_eq!(clamp("abcdef", 2), "ab");
        assert!(clamp(&"x".repeat(300), NOTE_MAX).len() <= NOTE_MAX);
    }

    #[test]
    fn the_report_names_each_lost_pick_and_counts_the_rest() {
        let mut r = FollowReport::default();
        assert_eq!(r.describe(), None);
        assert_eq!(r.to_json(), None);
        r.followed.push(note(&["front"]));
        for i in 0..10 {
            let name = format!("lost{i}");
            let mut n = note(&[name.as_str()]);
            n.why = Some("no face has these names now".into());
            r.lost.push(n);
        }
        let said = r.describe().expect("a phrase");
        assert!(said.starts_with("1 pick followed by SolidCraft name; 10 picks could not follow: face 'lost0' in 'Face1'"), "{said}");
        assert!(said.contains("and 2 more"), "{said}");
        let j = r.to_json().expect("json");
        assert_eq!(j["lost"].as_array().map(Vec::len), Some(10));
        assert_eq!(j["followed"][0]["names"][0], "front");
        assert_eq!(j["lost"][9]["why"], "no face has these names now");
    }
}
