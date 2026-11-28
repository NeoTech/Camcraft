## #4529 - Refuse reverse-spindle Mazak G86

**Decision.** Fail the Mazak post with a named error if a G86 bore meets
a spindle started with M4. Use the spindle word the post emitted, from
the start through the next M5. Forward G86 and the other cycle types
keep their existing output.

**Evidence.** Yamazaki Mazak, [MAZATROL MATRIX Programming Manual for
Machining Centers, EIA/ISO Program][matrix], H740PB0041E, §13-2-17,
printed page 13-23 (PDF page 217), figure MEP154, labels the G86 restart
at the return level M03. The figure gives no M04 restart or direction
override. No live controller test confirms how a particular control
handles an M04 start.

**Alternatives.** Do not assume that G86 restores the previous direction
or add M4 between holes. The figure does not support either sequence.
Use a posting error because the standalone `post` CLI can write a
program with warnings. A future implementation needs a documented
reverse sequence and posting tests that cover the spindle stop and
restart before it can replace this refusal.

[matrix]: https://www.passeidireto.com/arquivo/150733552/mazak-510-c-matrix-prog-manual
