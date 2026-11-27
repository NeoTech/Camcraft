## #4349 - Doosan DNM header: clean the machine name, refuse a program number outside O0001-O9999

**Decided.**

- `onOpen` writes the machine name through the post's own `clean()`.
  This is the same helper that the operation and tool notes use. It
  replaces `(`, `)`, `%` and `;` with a space and cuts the name to 40
  characters. The rule is the same as in `fanuc.cps` and
  `fanuc-lathe.cps` (#594).
- `oWord()` refuses a program number below 1 or above 9999 by name. The
  text is the same as in `fanuc.cps`: "program number N is outside
  O0001-O9999, the control's program numbers".
- `oWord()` reads the number with `isNaN`, not with `|| 1`. Thus a
  program number of 0 is refused, and it does not become O0001.

**Why.**

- A `(` or `)` in the name gives a nested comment. A `%` in the name ends
  the tape on a FANUC control.
- The old code clamped 10000 and more to O9999. O9999 is a different
  program, and O9000-O9999 is a protected range on many controls. The
  FANUC Series 0i-F operator's manual gives O0001-O9999 as the program
  numbers. The 8-digit program number is an option that the post does
  not assume.

**Dropped.**

- Clamp with a warning (`message()`). A clamped number still names
  another program. A refusal is the only safe answer.
- An option for 8-digit program numbers. No issue asks for it, and the
  sister posts do not have it.
- `|| 1` as in `fanuc.cps` `progNo()`. It changes 0 to 1 without a
  word. `fanuc.cps` keeps its form; this PR does not change it.

**Cost to change later.** Low. The guard is one line in `oWord()`. The
pins are in `crates/post/tests/comment_escape.rs`.
