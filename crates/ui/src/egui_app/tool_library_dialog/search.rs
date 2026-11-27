//! #4335 (U9 of #3663): the matcher of the tool list's one search box. A
//! query is split at white space into terms; a row matches when EACH term
//! matches one of its facts ("6 ball" finds the 6 mm ball nose mills).
//! - A text term matches a part of the row's text: the name, the type, the
//!   category, the holder, the part number and the vendor (#898), without
//!   case.
//! - A number term ("6", "6.35", "Ø8", "10mm", "0,5") matches a cutting or
//!   shank diameter within [`TOLERANCE_MM`] (in the document's unit), the
//!   tool number when it is a whole number, and the text when it has four
//!   or more characters (a numeric part number).
//! - "T5" matches the tool number 5 (or the text "t5").
//!
//! The caller lowers the row's text once, when it builds the row, so a
//! still frame does no work here (`state_tool_list`).
//! Provenance: our own matcher over our own tool records; no HSMWorks
//! material involved.

use camcraft_app::units::{self, Unit};

/// How near a typed diameter must be to a row's diameter, in mm. Finer
/// than the step of any common tool series (0.1 mm, 1/64 in = 0.4 mm), so
/// "6" does not find a 1/4 in (6.35 mm) tool.
pub(crate) const TOLERANCE_MM: f64 = 0.05;

/// The least length of a number term that also matches the text.
const NUMBER_IN_TEXT: usize = 4;

/// What one row offers the search.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Facts<'a> {
    /// The row's text, lowered ([`haystack`]).
    pub text: &'a str,
    /// The cutting and the shank diameter (mm); a value that is not finite
    /// or not above zero matches nothing.
    pub diameters_mm: &'a [f64],
    /// The tool number, if the row has one.
    pub number: Option<u32>,
}

/// One term of a query.
#[derive(Clone, Debug, PartialEq)]
enum Term {
    Text(String),
    /// A number in the document's unit, with the term's text.
    Number(f64, String),
    /// "T5": a tool number, with the term's text.
    ToolNumber(u32, String),
}

/// A parsed query. An empty query matches each row.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Query {
    terms: Vec<Term>,
}

/// The row's text for the search: the parts, lowered, one to a line (so a
/// term never matches across two parts).
#[must_use]
pub(crate) fn haystack<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut out = String::new();
    for p in parts {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&p.to_lowercase());
    }
    out
}

/// A number as the user types it: an optional "Ø" or "ø" first, an
/// optional unit word after, a comma or a point as the decimal mark.
fn number(term: &str) -> Option<f64> {
    let t = term.trim_start_matches(['ø', 'Ø']);
    let t = t.strip_suffix("mm").or_else(|| t.strip_suffix("in")).or_else(|| t.strip_suffix('"')).unwrap_or(t);
    if t.is_empty() || !t.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
        return None;
    }
    t.replace(',', ".").parse::<f64>().ok().filter(|v| v.is_finite() && *v >= 0.0)
}

impl Query {
    /// Parse the box's text.
    #[must_use]
    pub(crate) fn parse(text: &str) -> Self {
        let terms = text
            .split_whitespace()
            .map(str::to_lowercase)
            .map(|t| {
                if let Some(v) = number(&t) {
                    return Term::Number(v, t);
                }
                if let Some(n) = t.strip_prefix('t').and_then(|r| r.parse::<u32>().ok()) {
                    return Term::ToolNumber(n, t);
                }
                Term::Text(t)
            })
            .collect();
        Self { terms }
    }

    /// The query has no term: each row matches.
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// Does the row match each term? `unit` is the document's unit, in
    /// which the user types a number.
    #[must_use]
    pub(crate) fn matches(&self, f: &Facts<'_>, unit: Unit) -> bool {
        self.terms.iter().all(|t| term_matches(t, f, unit))
    }
}

fn term_matches(t: &Term, f: &Facts<'_>, unit: Unit) -> bool {
    match t {
        Term::Text(s) => f.text.contains(s.as_str()),
        Term::ToolNumber(n, s) => f.number == Some(*n) || f.text.contains(s.as_str()),
        Term::Number(v, s) => {
            let tol = units::to_display(TOLERANCE_MM, unit);
            let near = f
                .diameters_mm
                .iter()
                .filter(|d| d.is_finite() && **d > 0.0)
                .any(|d| (units::to_display(*d, unit) - v).abs() <= tol);
            let whole = v.fract() == 0.0 && f.number.is_some_and(|n| f64::from(n) == *v);
            near || whole || (s.chars().count() >= NUMBER_IN_TEXT && f.text.contains(s.as_str()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts<'a>(text: &'a str, d: &'a [f64], n: Option<u32>) -> Facts<'a> {
        Facts { text, diameters_mm: d, number: n }
    }

    #[test]
    fn terms_combine_with_and_and_ignore_case() {
        let ball = haystack(["BN-0600", "ball nose", "Ball nose"]);
        let mill = haystack(["EM-0600-4FL", "end mill", "Flat end mills"]);
        let q = Query::parse("6 BALL");
        assert!(q.matches(&facts(&ball, &[6.0, 6.0], Some(3)), Unit::Millimeters));
        assert!(!q.matches(&facts(&mill, &[6.0, 6.0], Some(1)), Unit::Millimeters), "the end mill is not a ball");
        assert!(!q.matches(&facts(&ball, &[8.0, 8.0], None), Unit::Millimeters), "an 8 mm ball is not 6");
    }

    #[test]
    fn a_number_matches_a_diameter_within_the_tolerance_in_the_document_unit() {
        let t = haystack(["x"]);
        let quarter = [6.35, 6.35];
        assert!(!Query::parse("6").matches(&facts(&t, &quarter, None), Unit::Millimeters), "6 is not 1/4 in");
        assert!(Query::parse("6.35").matches(&facts(&t, &quarter, None), Unit::Millimeters));
        assert!(Query::parse("Ø6,35mm").matches(&facts(&t, &quarter, None), Unit::Millimeters));
        assert!(Query::parse("0.25").matches(&facts(&t, &quarter, None), Unit::Inches), "inches in an inch document");
        assert!(Query::parse("10").matches(&facts(&t, &[8.0, 10.0], None), Unit::Millimeters), "the shank diameter");
        assert!(!Query::parse("0").matches(&facts(&t, &[0.0, f64::NAN], None), Unit::Millimeters), "no diameter, no match");
    }

    #[test]
    fn a_tool_number_and_a_numeric_part_number_match() {
        let t = haystack(["Drill", "drill", "4711-22"]);
        assert!(Query::parse("T5").matches(&facts(&t, &[3.0], Some(5)), Unit::Millimeters));
        assert!(Query::parse("5").matches(&facts(&t, &[3.0], Some(5)), Unit::Millimeters));
        assert!(Query::parse("4711").matches(&facts(&t, &[3.0], None), Unit::Millimeters), "the part number");
        assert!(!Query::parse("47").matches(&facts(&t, &[3.0], None), Unit::Millimeters), "a short number is not text");
        assert!(!Query::parse("T6").matches(&facts(&t, &[3.0], Some(5)), Unit::Millimeters));
    }

    #[test]
    fn an_empty_query_matches_and_text_never_spans_two_parts() {
        let t = haystack(["ab", "cd"]);
        assert!(Query::parse("  ").is_empty() && Query::parse("").matches(&facts(&t, &[], None), Unit::Millimeters));
        assert!(!Query::parse("bc").matches(&facts(&t, &[], None), Unit::Millimeters));
    }
}
