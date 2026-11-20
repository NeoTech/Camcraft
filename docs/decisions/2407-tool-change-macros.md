## #2407 - Post-defined tool-change macros

**Decided:** A post may declare `toolChangeMacros = { before: "M100\nM101", after: "M102" }`.
Both fields are optional strings and default to empty.
The engine emits their literal NC lines immediately before and after `onToolChange` for the first TOOL record and every later tool-number transition.
It still calls `onToolChange` for every TOOL record, but repeated records with the same tool number emit no macros.
Returning to an earlier tool number emits the macros again.
The resulting lines belong to that TOOL record in the preview trace and line map.
An absent or empty macro preserves existing output.

**Why:** The issue allows configuration in a machine file or a post, and its acceptance criterion asks for a post with macros.
A post manifest meets that criterion through the existing posting and preview path.
The existing Tormach post suppresses M6 for repeated TOOL records with the same number.
Suppressing the macros on those records prevents an aggregate head from unlocking and locking when its tool has not changed.
The engine checks the object and string types and rejects control characters other than newlines, tabs and CRLF by name.
It escapes macro text as data rather than evaluating it as JavaScript.
The post author remains responsible for choosing valid controller instructions and a safe machine sequence.

**Considered and dropped:** Adding machine-file fields now would also require changes to the typed Machine parser, mirror and persistence surface.
That broader configuration path is deferred.
Inserting macros inside individual post hooks would require changes to every post and leave the insertion point inconsistent.
Wrapping the existing hook gives all posts the same lifecycle boundary.
Executing macro text as JavaScript was rejected because the requested content is controller code.

**Cost to change later:** Machine-file configuration can reuse the validation and emission rules after adding explicit parser and mirror fields and defining override precedence.
Moving a macro inside a particular post's tool-change sequence requires a separate placement contract and new ordering tests.
Custom posts that force a physical tool change for a repeated tool number would need a future explicit placement contract rather than the current tool-number transition rule.
