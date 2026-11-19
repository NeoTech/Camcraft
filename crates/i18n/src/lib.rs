//! Interface languages (#853): the catalogue, the lookup seam, and nothing else.
//!
//! Provenance: our own catalogue format and lookup, specced from the
//! maintainer's request in #853 ("English as the source and fallback").
//! No vendor text, catalogue or tooling was copied.
//!
//! The law this crate keeps:
//!
//! - **English is the source.** The key of every entry is the English string
//!   the code already writes, so a missing translation shows English - never
//!   a key, never a blank. A wording change in the source simply falls back to
//!   the new English until a translator follows it, and
//!   [`missing`] names the stragglers.
//! - **Nothing else is localised.** Posted G-code, CL, project files, `.mch`,
//!   `.cps`, MCP names and log lines never pass through [`tr`]. Call sites in
//!   those paths must not call it (AGENTS.md; #853's "must never be localised").
//! - **No panic.** A malformed catalogue line is refused by name and the
//!   language falls back to English whole, so a bad translation file cannot
//!   take the window down.

mod parse;

use std::collections::BTreeMap;
use std::borrow::Cow;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

/// A language the interface can speak. The variants are the shipped
/// catalogues; adding one is a catalogue file plus a variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {
    /// The source language: every string is written in it.
    En,
    /// Simplified Chinese.
    ZhCn,
}

impl Lang {
    /// Every language, in the picker's order (English first: it is the source).
    pub const ALL: [Lang; 2] = [Lang::En, Lang::ZhCn];

    /// The BCP 47 tag, which is also the catalogue's file name.
    #[must_use]
    pub fn tag(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::ZhCn => "zh-CN",
        }
    }

    /// The language's own name, never translated - a picker a user cannot
    /// read is a picker they cannot use.
    #[must_use]
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::ZhCn => "简体中文",
        }
    }

    /// The catalogue's name (the tag), for a refusal that names the file.
    #[must_use]
    pub fn catalog_name(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::ZhCn => "zh-CN",
        }
    }

    /// Parse a tag or a system locale: `zh`, `zh-CN`, `zh_Hans_CN` and
    /// `zh-Hans` all name [`Lang::ZhCn`]; an unknown or empty tag is `None`.
    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Lang> {
        let primary = tag
            .trim()
            .split(['-', '_'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match primary.as_str() {
            "en" => Some(Lang::En),
            "zh" => Some(Lang::ZhCn),
            _ => None,
        }
    }

    /// True for the source language: the lookup is a no-op there.
    #[must_use]
    pub fn is_source(self) -> bool {
        self == Lang::En
    }

    /// The shipped catalogue's text, `None` for the source language.
    #[must_use]
    pub fn catalog_text(self) -> Option<&'static str> {
        match self {
            Lang::En => None,
            Lang::ZhCn => Some(include_str!("../../../assets/i18n/zh-CN.toml")),
        }
    }
}

/// One language's `English source -> translation` map.
#[derive(Debug, Default)]
pub struct Catalog {
    map: BTreeMap<String, String>,
    /// Lines the reader refused, with their reasons. Empty in a good file.
    refused: Vec<String>,
}

impl Catalog {
    /// Read a catalogue. Refused lines are not fatal: the file still loads
    /// what it can, and [`Catalog::refused`] carries every reason, so a
    /// startup diagnostic (and the crate's own test) can print them.
    #[must_use]
    pub fn parse(name: &str, text: &str) -> Catalog {
        let mut cat = Catalog::default();
        for (i, line) in text.lines().enumerate() {
            match parse::row(line, i + 1) {
                parse::Row::Pair(key, value) => {
                    if let Some(prev) = cat.map.insert(key.clone(), value) {
                        // A duplicated key is a translator's mistake; the
                        // later line wins and the report names it.
                        cat.refused.push(format!(
                            "{name}: duplicate key at line {}: {key:?} (was {prev:?})",
                            i + 1
                        ));
                    }
                }
                parse::Row::Ignore => {}
                parse::Row::Refuse(why) => cat.refused.push(format!("{name}: {why}")),
            }
        }
        cat
    }

    /// The translation for an English source string, if this file has one.
    #[must_use]
    pub fn get(&self, source: &str) -> Option<&str> {
        self.map.get(source).map(String::as_str)
    }

    /// How many pairs the file holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// True when the file holds no pair at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Every refused line, in file order.
    #[must_use]
    pub fn refused(&self) -> &[String] {
        &self.refused
    }

    /// Every pair, for a coverage test and the untranslated report.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.map.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

/// The parsed catalogue of a language, built once. `Err` is the refusal
/// report of a file that could not be read at all.
fn catalog(lang: Lang) -> Result<&'static Catalog, &'static [String]> {
    static ZH: OnceLock<Result<Catalog, Vec<String>>> = OnceLock::new();
    match lang {
        Lang::En => Err(&[]),
        Lang::ZhCn => {
            let slot = ZH.get_or_init(|| {
                let name = "zh-CN";
                let text = lang
                    .catalog_text()
                    .ok_or_else(|| vec![format!("{name}: no catalogue text shipped")])?;
                let cat = Catalog::parse(name, text);
                if cat.is_empty() {
                    return Err(cat.refused.clone());
                }
                Ok(cat)
            });
            match slot {
                Ok(c) => Ok(c),
                Err(why) => Err(why.as_slice()),
            }
        }
    }
}

/// The shipped catalogue of a language (parse once, cached).
#[must_use]
pub fn shipped(lang: Lang) -> Option<&'static Catalog> {
    catalog(lang).ok()
}

/// How a language's catalogue reads: the pairs it holds, or the reasons it
/// holds none. Startup prints this once (the diag seam) and the crate's test
/// refuses a file with any `Err`.
pub fn report(lang: Lang) -> Result<usize, Vec<String>> {
    match catalog(lang) {
        Ok(c) => {
            let mut refused = c.refused().to_vec();
            refused.truncate(20);
            if refused.is_empty() { Ok(c.len()) } else { Err(refused) }
        }
        Err(why) => Err(why.to_vec()),
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

/// The interface language in force.
#[must_use]
pub fn lang() -> Lang {
    match CURRENT.load(Ordering::Relaxed) {
        1 => Lang::ZhCn,
        _ => Lang::En,
    }
}

/// Choose the interface language; returns the one now in force. A language
/// whose catalogue did not load stays English, so the app is never left with
/// keys or blanks where its words should be.
pub fn set_lang(lang: Lang) -> Lang {
    let chosen = if lang.is_source() || catalog(lang).is_ok() { lang } else { Lang::En };
    CURRENT.store(match chosen {
        Lang::En => 0,
        Lang::ZhCn => 1,
    }, Ordering::Relaxed);
    chosen
}

/// Read `CAMCRAFT_LANG` (a BCP 47 tag; `zh`, `zh-CN`, `zh_Hans` ...) and set
/// the language. An unreadable or unknown value leaves English in force and
/// returns it - the caller logs the value it ignored.
pub fn init_from_env() -> Lang {
    let wanted = std::env::var("CAMCRAFT_LANG").ok();
    match wanted.as_deref().and_then(|v| {
        if v.trim().is_empty() {
            Some(Lang::En)
        } else {
            Lang::from_tag(v)
        }
    }) {
        Some(l) => set_lang(l),
        None => set_lang(Lang::En),
    }
}

/// Translate an English source string in the language in force.
///
/// A miss is the English string itself, borrowed - so a call site that is
/// never translated costs no allocation and reads correctly.
#[must_use]
pub fn tr(source: &str) -> Cow<'_, str> {
    tr_in(lang(), source)
}

/// [`tr`], for an explicit language (tests, previews, the translation editor).
#[must_use]
pub fn tr_in(lang: Lang, source: &str) -> Cow<'_, str> {
    if lang.is_source() {
        return Cow::Borrowed(source);
    }
    match catalog(lang) {
        Ok(c) => match c.get(source) {
            Some(v) => Cow::Borrowed(v),
            None => Cow::Borrowed(source),
        },
        Err(_) => Cow::Borrowed(source),
    }
}

/// True when `source` has no translation in `lang` (a coverage report line).
#[must_use]
pub fn missing(lang: Lang, source: &str) -> bool {
    !lang.is_source() && catalog(lang).is_ok_and(|c| c.get(source).is_none())
}

/// Translate, owning the result - for a `format!` argument or a `String` field.
#[must_use]
pub fn tr_owned(source: &str) -> String {
    tr(source).into_owned()
}

/// `3 tools` in English, `3 把刀具` in a language whose unit word carries its
/// own classifier (the catalogue's `tool` entry). `unit` is a catalogue key.
#[must_use]
pub fn count(n: usize, unit: &str) -> String {
    let word = tr(unit);
    if lang().is_source() {
        if n == 1 { format!("1 {word}") } else { format!("{n} {word}s") }
    } else {
        format!("{n} {word}")
    }
}

/// `tr!` is [`tr`] as a macro, so a call site reads as one word beside egui's.
#[macro_export]
macro_rules! tr {
    ($s:expr) => {
        $crate::tr($s)
    };
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
