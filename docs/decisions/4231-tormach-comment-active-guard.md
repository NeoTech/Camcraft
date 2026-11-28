# 4231 - Guard active comment keywords in Tormach lathe and plasma posts

- **Chosen:**
  1. Add the LinuxCNC `ACTIVE` regular expression guard to `assets/posts/tormach-lathe.cps`.
  2. Add the LinuxCNC `ACTIVE` regular expression guard to `assets/posts/tormach-plasma.cps`.
  3. Prepend `"- "` to any comment line that starts with LinuxCNC active comment keywords (`MSG`, `DEBUG`, `PRINT`, `LOG`, `PROBEOPEN`, `ABORT`, `AXIS`, `PY`).
  4. Add unit test suites in `crates/post/tests/tormach_lathe_post.rs` and `crates/post/tests/tormach_plasma_post.rs`.
- **Why:**
  1. PathPilot runs on LinuxCNC. LinuxCNC interprets comments that begin with active keywords as control commands rather than informational text.
  2. The milling post `assets/posts/tormach.cps` already guards comments against active keywords. The lathe and plasma posts lacked this guard.
  3. Without this guard, a machine name, setup name, tool name, or user comment starting with `MSG,` can trigger unwanted control actions or popups on the machine.
- **Dropped:**
  1. Changing `clean()` to remove the keywords: `clean()` only sanitizes delimiter characters (`(`, `)`, `%`, `;`). Prefixing with `"- "` keeps the original text readable and follows `tormach.cps`.
