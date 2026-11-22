## #853 - Interface languages: English-keyed catalogues, a bundled CJK face, and the seams a translation rides

**What was decided.** A pilot of the multi-language direction, one complete
language (Simplified Chinese) end to end:

- A new dependency-free crate, `camcraft-i18n`, holds `Lang`, the catalogue
  reader and the lookup (`tr`, `tr_in`, `count`, `set_lang`,
  `init_from_env`, `missing`, `report`). `ui -> app -> i18n` stays acyclic.
- **English is the key.** A catalogue line is `"English source" = "translation"`,
  so a miss shows the English string (never a key, never a blank), and a
  rewording in the source degrades to English until a translator follows it.
  Transported CL/G-code/project text never passes through the lookup.
- The catalogue is `assets/i18n/zh-CN.toml` (3 638 pairs), checked without a
  build by `scripts/i18n/check.py`, with a translator's guide beside it.
- The language follows the system locale on first start (`LC_ALL`, then
  `LC_MESSAGES`, then `LANG`). The View door carries a **Language** list -
  *Follow system*, English, 简体中文 - which switches at once and writes the
  pin to `prefs.json` beside the theme (`language`, absent = follow the
  system). `CAMCRAFT_LANG` (BCP 47: `zh`, `zh-CN`, `zh_Hans`) overrides both
  for that run. The picker adds rows to an existing painted door rather than a
  new ribbon cell, so the English window's pixels do not move. English
  installs no font and translates no string, so it is byte-identical to
  before.
- A translated language installs `NotoSansSC-Regular-Subset.ttf` (Noto Sans
  SC 400, OFL 1.1) first in `Proportional`, so Latin, digits and hanzi in one
  label share one face and one line height. The subset is GB2312 plus
  punctuation: 8 090 code points, 2.3 MB, reproducible from the upstream
  variable font with `scripts/fonts/subset_sc.py`.
- Translation happens at five seams, not at 4 600 call sites: the property
  grid (`label_cell`, `group`, `action_bar`), the model tree (`t_row`), the
  ribbon (`text_of`, `caption`, dropdown rows, machine tabs), the operation
  tabs, and a handful of page details.

**Why.**

- The maintainer's goal in #853 starts with "English as the source and
  fallback". English-as-key is the smallest shape that keeps that promise
  and needs no dependency, against #853's own warning that the mechanism
  "adds a dependency, so it needs a decision". If the maintainer prefers
  Fluent, only `crates/i18n` changes: every call site already says `tr`.
- The seams are where nearly every label funnels, so a five-file change
  covers the ribbon, the tree, every property page and its footer. The
  remaining work (hints, status line, app refusals, panel titles) is listed
  in `docs/I18N.md` rather than half-done.
- Layout: the worry was that Chinese would break the fixed ribbon cells. The
  measurement says otherwise - at 11 px the English operation captions
  already overflow the 74 px cell in 13 of 42 cases (`Multi-Axis Contour`
  = 101 px), while their Chinese translations measure 33-55 px. Two tests
  pin it: every translated ribbon caption fits the cell, and the translated
  set is narrower in aggregate than the English one. The snapshot
  `tests/snapshots/egui_app_zh.png` shows the whole window in Chinese.

**What was considered and dropped.**

- Rewriting call sites to symbolic keys (`ribbon.save`): every string would
  need a name, and a missing one would show a key. Dropped for this pilot.
- A separate languages page or a new ribbon cell: the door already exists and
  is where panel/view choices live; a new cell would move the English ribbon's
  pixels for every user, including those who never change language.
- Shipping the full Noto Sans SC (16 MB) or the whole CJK TTC: the subset
  carries every shipped string, which a test proves by reading the TTF
  `cmap`; a rare hanzi outside GB2312 falls back to a box like any missing
  glyph in any app.
- Translating the app's refusal and diagnostic sentences now: #853 requires
  each to keep a stable name or code so issue reports and tests still match,
  which is its own change.

**Cost to change later.** The catalogue format and the lookup live in
`crates/i18n`; swapping them for Fluent replaces that crate and keeps every
`tr` call site. Adding a language is a row in the `crates/i18n` `REGISTRY`, a
catalogue file and, if the script is not covered, one font; `ALL`, locale
parsing, the cache and the picker all read that row, with no call-site change.
The seams are ordinary functions, so widening the translated surface is
additive.
