## #3540 - The program end returns the rotary axes to zero (Fanuc, Doosan DNM)

**What was decided.** A program whose last section leaves a rotary off
zero ends with a return to zero. The Fanuc and Doosan DNM posts write,
after the last section's M5:

```
G49
G91 G28 Z0          (or G53 G00 Z<safe_retract> with safeRetract G53)
G90                 (G28 mode only)
G00 A0. B0.         (only the rotaries held off zero; no X/Y/Z word)
G91 G28 X0 Y0       (G28 mode only)
M30
```

Doosan writes M11 before and M10 after the return when A turns, as at
the return of #3466. A program whose last section is upright ends byte
for byte as before.

**Why.** The angle words are modal. Without a return, the operator
unloads the part on a tilted table, and a program from a different
source that does not state its pose starts with a lean. The blocks are
the same as the section return of #3466, so the program has one form of
return. G49 comes first because the Doosan program end still had G43.4
active at the reference return, and many controls alarm on a reference
return under TCPM. The X/Y home is incremental (`G91 G28 X0 Y0`): after
the G90 that the angle block needs, a plain `G28 X0 Y0` would use work
X0 Y0 as its via point.

When a trailing "Go home" after-operation ask already wrote the
reference return, the end writes only the zero angle block, with no
second retract.

**Considered and dropped.**
- Leave the rotaries and write a note. Dropped: the issue recommends the
  return, and a note does not help a program from a different source.
- Return with `G28 A0 B0` (a reference return of the rotaries). Dropped:
  the reference position of a rotary is a machine setting, not zero in
  every case, and #3466 already uses `G00` zero words.
- Change the other rotary posts. Tormach keeps its final tilt by the
  decision of #1594. Mach4 keeps its final pose too, and is not in the
  scope of this issue. Haas and Sinumerik refuse tilted output.

**Cost to change later.** One function (`uprightAtEnd`) in each of the two
posts, and the end blocks of the catalog program `multiaxis/indexed`.
