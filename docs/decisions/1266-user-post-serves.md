## #1266 - A user's post script states the machine kinds it serves

**Decided.** The manifest key is `serves` (top-level, like `capabilities`):
`serves = ["plasma"];`, or one string, over the machine file's kind words
(mill, lathe, mill-turn, plasma, wire_edm, sinker_edm; `Kind::from_word`'s
tolerant spelling). #1265's door reads it for a user's own script: a
script that states it posts to exactly those kinds and is refused by name
on any other. One that states none keeps the dialect reading only where
it is unambiguous (turning serves lathe and mill-turn, a non-turning one
a mill and a mill-turn) and is refused on plasma and EDM, the refusal
saying how to declare it. A `serves` that is empty or names no kind is a
clean refusal, never read as stated.

**Shipped posts** are judged by registry name and need no key: no .cps
changed, goldens untouched.

**Considered, dropped.** Reusing the `capabilities` mask with a plasma
bit (the shipped plasma post declares the milling bit, changing masks
would touch every post); inferring plasma from `onTorchOn` (a mill script
could define it); a stated `mill` also serving a mill-turn (stricter: list
`mill-turn` to say so).
