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

use std::borrow::Cow;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// A language the interface can speak.
///
/// This is a descriptor, not a closed enum: every shipped language is one row
/// of [`REGISTRY`]. Adding one is a catalogue file plus a row - lookup,
/// caching, locale parsing and the View door's picker all read the registry,
/// so no matching logic and no call site has to change. The `id` is kept
/// first so `Ord` follows the registry (and therefore the picker's) order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lang {
    id: usize,
    tag_str: &'static str,
    native: &'static str,
    /// `None` for the source language, which ships no catalogue.
    catalog: Option<&'static str>,
}

// ---- The language registry ------------------------------------------------
//
// ONE ROW PER LANGUAGE. To ship another language: drop in
// `assets/i18n/<tag>.toml`, add a row here, and - only if call sites name it -
// a `pub const` alias below. Nothing else in this crate has a language-
// specific branch; nothing outside this crate changes at all.

const EN: Lang = Lang {
    id: 0,
    tag_str: "en",
    native: "English",
    catalog: None,
};
const ZH_CN: Lang = Lang {
    id: 1,
    tag_str: "zh-CN",
    native: "简体中文",
    catalog: Some(include_str!("../../../assets/i18n/zh-CN.toml")),
};

const REGISTRY: &[Lang] = &[EN, ZH_CN];

#[allow(non_upper_case_globals)]
impl Lang {
    /// Every language, in the picker's order (the source language first).
    /// Built from the registry, so a new row appears here automatically.
    pub const ALL: [Lang; REGISTRY.len()] = {
        let mut all = [EN; REGISTRY.len()];
        let mut i = 0;
        while i < REGISTRY.len() {
            all[i] = REGISTRY[i];
            i += 1;
        }
        all
    };

    /// The source language: English, whose strings need no catalogue.
    pub const En: Lang = EN;
    /// Simplified Chinese.
    pub const ZhCn: Lang = ZH_CN;

    /// The BCP 47 tag, which is also the catalogue's file name.
    #[must_use]
    pub fn tag(self) -> &'static str {
        self.tag_str
    }

    /// The language's own name, never translated - a picker a user cannot
    /// read is a picker they cannot use.
    #[must_use]
    pub fn native_name(self) -> &'static str {
        self.native
    }

    /// The catalogue's name (the tag), for a refusal that names the file.
    #[must_use]
    pub fn catalog_name(self) -> &'static str {
        self.tag_str
    }

    /// Parse a tag or a system locale: `zh`, `zh-CN`, `zh_Hans_CN` and
    /// `zh-Hans` all name the registered `zh-CN`; an unknown or empty tag is
    /// `None`. An exact tag wins over a bare primary, so a future `zh-TW`
    /// row does not steal `zh-CN`'s locale.
    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Lang> {
        let wanted = tag.trim();
        if wanted.is_empty() {
            return None;
        }
        let normalise = |s: &str| s.replace('_', "-").to_ascii_lowercase();
        let wanted_norm = normalise(wanted);
        if let Some(exact) = REGISTRY
            .iter()
            .copied()
            .find(|l| normalise(l.tag_str) == wanted_norm)
        {
            return Some(exact);
        }
        let primary = wanted.split(['-', '_']).next().unwrap_or("");
        REGISTRY.iter().copied().find(|l| {
            l.tag_str
                .split('-')
                .next()
                .is_some_and(|p| p.eq_ignore_ascii_case(primary))
        })
    }

    /// True for the source language: the lookup is a no-op there.
    #[must_use]
    pub fn is_source(self) -> bool {
        self.catalog.is_none()
    }

    /// The shipped catalogue's text, `None` for the source language.
    #[must_use]
    pub fn catalog_text(self) -> Option<&'static str> {
        self.catalog
    }

    /// This language's slot in the parse-once cache.
    fn slot(self) -> usize {
        self.id
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
    // The source language has no catalogue by design, not a failed one.
    if lang.is_source() {
        return Err(&[]);
    }
    // One cache for the whole registry: a row added to REGISTRY gets a slot
    // here without a language-specific `static` or `match`.
    static CACHE: OnceLock<Vec<Result<Catalog, Vec<String>>>> = OnceLock::new();
    let slots = CACHE.get_or_init(|| {
        REGISTRY
            .iter()
            .map(|l| {
                let Some(text) = l.catalog_text() else {
                    return Ok(Catalog::default());
                };
                let cat = Catalog::parse(l.tag(), text);
                if cat.is_empty() {
                    Err(cat.refused.clone())
                } else {
                    Ok(cat)
                }
            })
            .collect()
    });
    match &slots[lang.slot()] {
        Ok(c) => Ok(c),
        Err(why) => Err(why.as_slice()),
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
    // The source language has no catalogue by design; starting in English is
    // not a refusal and must not read like one.
    if lang.is_source() {
        return Ok(0);
    }
    match catalog(lang) {
        Ok(c) => {
            let mut refused = c.refused().to_vec();
            refused.truncate(20);
            if refused.is_empty() { Ok(c.len()) } else { Err(refused) }
        }
        Err(why) => Err(why.to_vec()),
    }
}

thread_local! {
    /// The language in force on this thread. It is a thread-local, not a
    /// process global: every unit test runs on its own thread, so a test that
    /// switches language cannot change the labels another test asserts on.
    /// The interface draws on one thread - the window's - so a session still
    /// has exactly one language.
    static CURRENT: Cell<Lang> = const { Cell::new(Lang::En) };
    /// True when `CAMCRAFT_LANG` named the language: the stored preference
    /// then must not override it for this run.
    static FROM_ENV: Cell<bool> = const { Cell::new(false) };
}

/// The interface language in force.
#[must_use]
pub fn lang() -> Lang {
    CURRENT.with(Cell::get)
}

/// Choose the interface language; returns the one now in force. A language
/// whose catalogue did not load stays English, so the app is never left with
/// keys or blanks where its words should be.
pub fn set_lang(lang: Lang) -> Lang {
    let chosen = if lang.is_source() || catalog(lang).is_ok() { lang } else { Lang::En };
    CURRENT.with(|c| c.set(chosen));
    chosen
}

/// True when `CAMCRAFT_LANG` chose the language for this run (an explicit
/// override; a stored preference must not win over it).
#[must_use]
pub fn env_override() -> bool {
    FROM_ENV.with(Cell::get)
}

/// The language the operating system asks for, from the POSIX locale
/// variables (`LC_ALL`, then `LC_MESSAGES`, then `LANG`). `C` and `POSIX` mean
/// "no locale" and are skipped; anything this build does not speak - or a
/// platform that sets none of them, such as Windows - is English.
#[must_use]
pub fn system_lang() -> Lang {
    system_lang_from(|k| std::env::var(k).ok())
}

/// [`system_lang`] over a variable reader, so a test can pose a locale.
#[must_use]
fn system_lang_from(get: impl Fn(&str) -> Option<String>) -> Lang {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        let Some(v) = get(key) else { continue };
        let v = v.trim();
        if v.is_empty() || v == "C" || v == "POSIX" {
            continue;
        }
        if let Some(l) = Lang::from_tag(v) {
            return l;
        }
    }
    Lang::En
}

/// Read `CAMCRAFT_LANG` (a BCP 47 tag; `zh`, `zh-CN`, `zh_Hans` ...) and set
/// the language, or follow the system locale when it is absent. An unreadable
/// or unknown value is logged by the caller and the system locale applies.
pub fn init_from_env() -> Lang {
    let wanted = std::env::var("CAMCRAFT_LANG").ok();
    match wanted.as_deref().and_then(Lang::from_tag) {
        Some(l) => {
            FROM_ENV.with(|c| c.set(true));
            set_lang(l)
        }
        None => set_lang(system_lang()),
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

/// Translate a `format!`-style template, then substitute its arguments.
///
/// The **template** is the catalogue key, so wording (and argument order) can
/// differ by language: `"Cut {} passes"` becomes `"切削 {} 刀"`. `{}` and
/// `{name}` placeholders are filled in encounter order from `args`; `{{` /
/// `}}` are literal braces. A missing key falls back to the English template,
/// so this never shows a key or a blank. Arguments are already rendered
/// strings, which keeps the call sites free of runtime format-string parsing.
///
/// # Examples
/// ```
/// use camcraft_i18n::{set_lang, tf, Lang};
/// set_lang(Lang::En);
/// assert_eq!(tf("Cut {} passes", &["3".into()]), "Cut 3 passes");
/// ```
#[must_use]
pub fn tf(template: &str, args: &[String]) -> String {
    let rendered = tr(template);
    let mut out = String::with_capacity(rendered.len() + 16);
    let bytes = rendered.as_bytes();
    let mut i = 0;
    let mut next = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                out.push('{');
                i += 2;
                continue;
            }
            if let Some(rel) = rendered[i + 1..].find('}') {
                let end = i + 1 + rel;
                if next < args.len() {
                    out.push_str(&args[next]);
                } else {
                    out.push_str(&rendered[i..=end]);
                }
                next += 1;
                i = end + 1;
                continue;
            }
        }
        if bytes[i] == b'}' && i + 1 < bytes.len() && bytes[i + 1] == b'}' {
            out.push('}');
            i += 2;
            continue;
        }
        let ch = rendered[i..].chars().next().expect("index is a char boundary");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
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
