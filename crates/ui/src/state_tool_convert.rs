//! #4339: the tool library window's "Convert to components" preview -
//! the `tool_convert` plan of every standalone document tool, one tick
//! for each row (the "same" rows start ticked), and the handler that
//! lands the ticked rows as one undoable `ConvertToComponents`. The plan
//! is computed when the preview opens, on Refresh and after a convert,
//! never on a still frame (the frame-cost rule, #2238); `at` is the
//! history length it was computed at, so the page can say when the
//! document moved on. Provenance: our own view state over our own
//! command; no HSMWorks material involved.

use camcraft_app::commands::ConvertToComponents;
use camcraft_app::tool_convert::{plan, ConvertRow, Verdict};

use super::State;

/// The preview of the conversion.
#[derive(Clone, Debug, Default)]
pub struct ConvertView {
    pub rows: Vec<ConvertRow>,
    /// One tick for each row of `rows`.
    pub picked: Vec<bool>,
    /// The history length when `rows` was computed.
    pub at: usize,
    /// The plan's refusal, by name.
    pub error: Option<String>,
}

impl ConvertView {
    /// The ticked tools that can convert, in row order.
    #[must_use]
    pub fn selection(&self) -> Vec<String> {
        self.rows
            .iter()
            .zip(&self.picked)
            .filter(|(r, p)| **p && !matches!(r.verdict, Verdict::NoMatch(_)))
            .map(|(r, _)| r.tool.clone())
            .collect()
    }
}

impl State {
    /// The preview of every standalone tool, as the document stands.
    fn convert_preview(&self) -> ConvertView {
        let at = self.history.done_len();
        match plan(&self.world, &[]) {
            Ok(rows) => {
                let picked = rows.iter().map(|r| r.verdict == Verdict::Same).collect();
                ConvertView { rows, picked, at, error: None }
            }
            Err(e) => ConvertView { at, error: Some(e), ..ConvertView::default() },
        }
    }

    /// Open (or compute again) the preview in the tool library window.
    pub(crate) fn open_convert(&mut self) {
        let view = self.convert_preview();
        if let Some(v) = self.tool_library.as_mut() {
            v.convert = Some(view);
        }
    }

    /// Close the preview; nothing changes in the document.
    pub(crate) fn close_convert(&mut self) {
        if let Some(v) = self.tool_library.as_mut() {
            v.convert = None;
        }
    }

    /// The handler of `ComponentAction::Convert`: one undo step for all
    /// the selected tools, then the preview follows the document.
    pub(crate) fn convert_tools(&mut self, tools: &[String]) -> Result<String, String> {
        let cmd = ConvertToComponents::new(&self.world, tools).map_err(|e| e.to_string())?;
        let done = cmd.tools();
        self.history.execute(&self.world, cmd);
        if self.tool_library.as_ref().is_some_and(|v| v.convert.is_some()) {
            self.open_convert();
        }
        Ok(format!("converted to component stacks (one undo step): {}", done.join(", ")))
    }
}
