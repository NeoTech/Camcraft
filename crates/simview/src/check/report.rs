//! The report of `camcraft-sim check` (#2586): one line for each pass,
//! each finding and each check that was not judged, and the exit code
//! that they give. A check that did not run never reads as a pass.
//! Provenance: our own report format.

use crate::{EXIT_FAILED, EXIT_NOT_JUDGED, EXIT_OK};

/// The most findings printed for one check; the count of the rest is
/// printed, so the count never lies.
pub const CAP: usize = 40;

#[derive(Debug, Default)]
pub struct Report {
    lines: Vec<String>,
    /// (check, findings withheld past the cap)
    withheld: Vec<(String, usize)>,
    failed: bool,
    unjudged: bool,
}

impl Report {
    /// A check ran and found nothing.
    pub fn pass(&mut self, check: &str, what: &str) {
        self.lines.push(format!("PASS        {check}: {what}"));
    }

    /// A check found a problem: `what` names the NC line.
    pub fn fail(&mut self, check: &str, what: &str) {
        self.failed = true;
        let shown = self.lines.iter().filter(|l| l.starts_with("FAIL") && l.contains(&format!(" {check}: "))).count();
        if shown < CAP {
            self.lines.push(format!("FAIL        {check}: {what}"));
        } else if let Some(w) = self.withheld.iter_mut().find(|(c, _)| c == check) {
            w.1 += 1;
        } else {
            self.withheld.push((check.to_owned(), 1));
        }
    }

    /// A check (or a part of it) did not run, and why.
    pub fn not_judged(&mut self, check: &str, why: &str) {
        self.unjudged = true;
        self.lines.push(format!("NOT JUDGED  {check}: {why}"));
    }

    /// A fact for the reader that judges nothing.
    pub fn note(&mut self, what: &str) {
        self.lines.push(format!("note        {what}"));
    }

    #[must_use]
    pub fn failed(&self) -> bool {
        self.failed
    }

    /// 2 when a check failed, else 3 when one was not judged, else 0.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        if self.failed {
            EXIT_FAILED
        } else if self.unjudged {
            EXIT_NOT_JUDGED
        } else {
            EXIT_OK
        }
    }

    /// The report text, the result line last.
    #[must_use]
    pub fn text(&self) -> String {
        let mut out: Vec<String> = self.lines.clone();
        for (check, n) in &self.withheld {
            out.push(format!("FAIL        {check}: {n} more findings are not printed"));
        }
        out.push(
            match self.exit_code() {
                EXIT_FAILED => "result: FAILED",
                EXIT_NOT_JUDGED => "result: NOT JUDGED (each check that did not run is named above)",
                _ => "result: ok (each check ran and passed)",
            }
            .to_owned(),
        );
        out.join("\n")
    }
}
