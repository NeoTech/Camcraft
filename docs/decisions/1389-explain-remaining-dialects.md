## #1389 - Explain comments for the remaining post dialects

**Decided:** Enable Explain on eight more posts through the shared helper.
Sinumerik uses semicolon comments. grblHAL, TinyG, Carvera, PathPilot
plasma and lathe, and generic ISO wire and sinker EDM use parentheses.
Each writer appends a comment only when the complete line fits within
80 characters. Existing executable text stays unchanged.

**Why:** Each control has its own word meanings. Sinumerik describes
G700 and G710 as length and feed unit choices. It describes G505 and
G506 as work offsets 5 and 6. It leaves G58 and G59 unexplained.
PathPilot plasma describes its probe, torch and plasma parameter words.
PathPilot lathe describes its extended offsets and retains the existing
tool station and offset decoder.

Machine-supplied EDM commands stay unexplained. The wire post marks
these calls to `block()` so the helper skips them. This includes a
motion block that carries a machine-supplied UV code. The sinker post
already writes its machine commands through `emit()`. A machine can
assign meanings to these words that differ from the shared mill table.

Heidenhain and Sodick retain their named refusal when Explain is on.
Heidenhain's public ISO manual describes semicolon comments after the
last word. It does not confirm placement beside the `*` terminator
that this post exports. Sodick uses T91 and T90 to thread and cut the
wire. The shared decoder would describe them as tool numbers. These
posts need confirmed syntax or a suitable decoder before they can
declare `explains`.

**Considered and dropped:** Do not enable every post with the shared
mill table. That would give some EDM commands incorrect descriptions.
Do not guess where Heidenhain accepts a comment beside `*`. Do not
shorten executable text to make room for an explanation. Omit the
explanation when the complete line would exceed the chosen limit.
Keep the shared engine unchanged because the existing helper supports
the eight enabled posts.

**Cost to change later:** Heidenhain needs a confirmed exported block
example and tests for its terminator and tool-axis words. Sodick needs
a decoder for its EDM T words and tests for each supported meaning.
A different line limit needs updated boundary tests for the affected
post. Machine-specific EDM explanations need an explicit meaning
contract instead of an inference from the code letters.

Public behavior references:

- [Siemens 840D Fundamentals](https://cache.industry.siemens.com/dl/files/241/109763241/att_971137/v1/840Dsl_fundamentals_progr_man_1218_en-US.pdf), section 3.2.4, page 44: semicolon comments.
- [Heidenhain TNC 640 ISO programming](https://content.heidenhain.de/doku/tnc_guide/pdf_files/TNC640/34059x-07/diniso/892909-25.pdf), section 4.1, pages 172-173: comments after the last word.
- The existing post provenance comments identify the sources for the
  supported control words. This change uses those behavior descriptions.
