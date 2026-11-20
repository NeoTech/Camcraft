## #2876 - Name retained shape keys in parameter patch refusals

**What was decided.** Keep RFC 7386 merge-patch behavior for operation
parameters. When the parser requires a single map key, add context for
former single-key objects that now contain multiple keys. Keep the original
parser error. Give dotted field paths, including `links.approach.lead_out`,
and the merged keys. Tell the caller to remove unwanted keys with `null`.
Document this removal for both `add_operation` and `edit_operation`.

**Why.** The issue accepts a refusal that names the field and conflicting
shapes. The shared parser serves both tools and nested parameters. This
change preserves parameter values, command validation and undo history. It
requires no dependency or toolpath behavior change. Context describes merged
objects because a single-key object is not necessarily an enum.

**Considered and dropped.** Automatic shape replacement would change merge
semantics. A Lead-only check would miss other enums and nested fields. A new
parser dependency would expand this small diagnostic fix.

**Cost to change later.** Exact typed error paths may require a parser that
uses the parameter schema. Keep validation before command execution and
preserve history when a patch is refused.

**Validation.** Three pins cover creation, editing, nested paths, explicit
removal and unchanged state after refusal. All three fail with the original
parser and pass with the fix. The quick gate passes 17 steps. Clippy passes
for `camcraft-app`.
