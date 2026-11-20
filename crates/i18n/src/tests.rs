//! The catalogue's own tests (#853): the shipped file reads clean, English is
//! the source, a miss falls back to English, and a bad line is refused by name.

use super::*;

#[test]
fn the_shipped_chinese_catalogue_reads_clean() {
    let n = report(Lang::ZhCn).expect("the shipped zh-CN catalogue is refused");
    assert!(n > 300, "the catalogue should cover the ribbon, tree and property rows, has {n}");
    let cat = shipped(Lang::ZhCn).expect("shipped");
    assert!(cat.refused().is_empty(), "refused lines: {:?}", cat.refused());
}

#[test]
fn english_is_the_source_and_a_miss_is_the_english_string() {
    assert_eq!(tr_in(Lang::En, "Save"), "Save");
    assert_eq!(tr_in(Lang::En, "not a key at all"), "not a key at all");
    assert_eq!(tr_in(Lang::ZhCn, "Save"), "保存");
    assert_eq!(tr_in(Lang::ZhCn, "not a key at all"), "not a key at all");
}

#[test]
fn a_key_is_unique_and_no_translation_is_empty() {
    let cat = shipped(Lang::ZhCn).expect("shipped");
    let mut seen = std::collections::BTreeSet::new();
    for (k, v) in cat.entries() {
        assert!(seen.insert(k), "duplicate key {k:?}");
        assert!(!v.trim().is_empty(), "empty translation for {k:?}");
        assert_ne!(k.trim(), v.trim(), "{k:?} is its own translation");
    }
}

#[test]
fn a_system_locale_names_its_language() {
    for t in ["zh", "zh-CN", "zh_CN", "zh-Hans", "zh-Hans-CN", " ZH_cn "] {
        assert_eq!(Lang::from_tag(t), Some(Lang::ZhCn), "{t}");
    }
    for t in ["en", "en-US", "en_GB"] {
        assert_eq!(Lang::from_tag(t), Some(Lang::En), "{t}");
    }
    for t in ["", "fr", "de-DE", "zz"] {
        assert_eq!(Lang::from_tag(t), None, "{t:?}");
    }
    assert_eq!(Lang::ZhCn.tag(), "zh-CN");
    assert_eq!(Lang::ZhCn.native_name(), "简体中文");
}

/// #853: when the person has not pinned a language, the system locale
/// decides - `LC_ALL`, then `LC_MESSAGES`, then `LANG`; `C`/`POSIX` and
/// values this build does not speak fall through to English.
#[test]
fn the_system_locale_names_the_language() {
    let env = |pairs: &'static [(&'static str, &'static str)]| {
        move |k: &str| pairs.iter().find(|(n, _)| *n == k).map(|(_, v)| (*v).to_owned())
    };
    assert_eq!(system_lang_from(|_: &str| None), Lang::En);
    // LC_ALL wins over LC_MESSAGES and LANG.
    assert_eq!(
        system_lang_from(env(&[
            ("LC_ALL", "zh_CN.UTF-8"),
            ("LC_MESSAGES", "en_US.UTF-8"),
            ("LANG", "en_US.UTF-8"),
        ])),
        Lang::ZhCn
    );
    assert_eq!(
        system_lang_from(env(&[("LC_MESSAGES", "zh_CN.UTF-8"), ("LANG", "en_US.UTF-8")])),
        Lang::ZhCn
    );
    assert_eq!(system_lang_from(env(&[("LANG", "zh_CN.UTF-8")])), Lang::ZhCn);
    // `C`/`POSIX` mean "no locale": the search goes on, and alone they are
    // English.
    assert_eq!(system_lang_from(env(&[("LC_ALL", "C"), ("LANG", "zh_CN.UTF-8")])), Lang::ZhCn);
    assert_eq!(system_lang_from(env(&[("LC_ALL", "C"), ("LANG", "C.UTF-8")])), Lang::En);
    assert_eq!(system_lang_from(env(&[("LC_ALL", "  "), ("LANG", "POSIX")])), Lang::En);
    // A locale this build does not speak is English, not an error.
    assert_eq!(system_lang_from(env(&[("LANG", "fr_FR.UTF-8")])), Lang::En);
    assert_eq!(system_lang_from(env(&[("LANG", "C.UTF-8")])), Lang::En);
}

#[test]
fn the_reader_refuses_a_bad_line_by_number() {
    let cat = Catalog::parse(
        "test",
        "\"A\" = \"甲\"\nno quote here\n\"B\" = \"乙\"\n\"C\" = \"\"\n",
    );
    assert_eq!(cat.len(), 2);
    let why = cat.refused().join("\n");
    assert!(why.contains("test: line 2"), "{why}");
    assert!(why.contains("test: line 4"), "{why}");
}

#[test]
fn a_duplicate_key_is_reported_and_the_later_line_wins() {
    let cat = Catalog::parse("t", "\"A\" = \"甲\"\n\"A\" = \"乙\"\n");
    assert_eq!(cat.get("A"), Some("乙"));
    assert!(cat.refused()[0].contains("duplicate key"));
}

#[test]
fn escapes_and_comments_read_as_written() {
    let cat = Catalog::parse(
        "t",
        "# a comment\n\n\"Cut\\nline\" = \"切\\t线\" # trailing comment\n\"Quote\" = \"引号 \\\"x\\\"\"\n",
    );
    assert_eq!(cat.len(), 2);
    assert_eq!(cat.get("Cut\nline"), Some("切\t线"));
    assert_eq!(cat.get("Quote"), Some("引号 \"x\""));
    assert!(cat.refused().is_empty(), "{:?}", cat.refused());
}

#[test]
fn count_keeps_english_plurals_and_translates_the_unit() {
    // `count` reads the global language, so it is pinned to English here:
    // the suite never leaves another language in force.
    assert_eq!(lang(), Lang::En);
    assert_eq!(count(1, "tool"), "1 tool");
    assert_eq!(count(3, "tool"), "3 tools");
}
