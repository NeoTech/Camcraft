## #1232 - A post writes only for a machine kind it serves

**Decided.** `post_serves` (the Setup pane's listing law) is now also a
gate at post time. A new door, `posting_serves.rs`, asks for every mount
of the program - whole-program `post_setup` / `post_script`, per-mount
`post_mounts` / `post_mounts_script`, per-operation `post_per_op` /
`post_per_op_script` (and so the GUI, script `cam.post`, CLI and MCP,
which all call these) - whether its post serves its machine's kind. A
refusal names the setup(s), their operations, the post and the kind
("Setup2 (Facing2): post Haas does not serve Bench lathe, a turning
machine - ..."), and nothing is written.

**Root cause.** `post_serves` was only the picker's filter. Its own doc
said a mismatch "is not refused here ... the post refuses by name what it
cannot express", but only the turning dialects refuse (#1190's door, for
milling ops); the milling dialects express a turning section happily as
G17 / T M6 / G43 / S M3 / Y moves. No posting path asked the question, so
the per-mount path (and the single-file one) posted a lathe through Haas.

**How a post is identified.** A shipped post by its registry name (the
script text is matched against `BUILTINS`, so a caller handing a shipped
script verbatim is judged as that post). A user's own script by the
dialect its `capabilities` manifest declares: a lathe takes only a
script that declares turning (an undeclared one is refused - the
stricter reading, as #1174/#1199 read undeclared scripts); a mill or a
plasma table refuses a turning-only script; a mill-turn takes any; an
EDM takes none (no EDM dialect ships, #533). Dropped: refusing a user's
milling-mask script on a plasma table - the shipped plasma post itself
declares the milling bit, so the mask cannot tell a user's plasma script
from a milling one; the torch law in the engine still refuses torch
records through a script without `onTorchOn`.

**All or nothing.** In the repro, Setup1 (mill, Fanuc) is served and
Setup2 (lathe, Haas) is not. Setup1's file is NOT written either: the
per-mount post already judges every file before writing any ("a refusal
anywhere writes nothing"), and one job half on disk is the incomplete
program the Prime Directive warns about. The kind refusal joins the
program gate's problem list (first), so any other refusal (e.g. the
workholding rpm limit) is still named beside it.

**Order.** #1190's lathe door (a milling op through a turning post)
still runs first and keeps its own wording; the family gate (an op the
machine cannot run at all, e.g. a mill Face on an EDM) refuses before
either. Each is by name, before any NC text.

**Cost to change later.** One module; dropping or widening the rule is
the `serves` match in `crates/app/src/posting_serves.rs`.
