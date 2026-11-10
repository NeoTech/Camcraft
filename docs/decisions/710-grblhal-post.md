## #710 - grblHAL post processor (3-axis mill)

**Decided:** a new built-in post, `assets/posts/grblhal.cps`, specced from the
G-code list in grblHAL's public core README and the LinuxCNC G-code reference
(behaviour only; grblHAL's source is GPL and was not read). Structure from
`tormach.cps`, coolant and after-operation handling from `mazak.cps`.

- **No tool length word at all** (no G43 H, no G43.1, no G49). The issue
  proposed G43.1 as the default, but G43.1 needs a length value the post cannot
  know. On grblHAL the length offset belongs to the controller's tool-change
  handling (tool setter, or a sender that measures and sets G43.1), and a G49
  in a preamble would throw that measured length away. *Dropped:* `G43 H<n>`
  (needs a tool table, which is driver-dependent); a G43/G43.1 post option
  (can be added later without changing the default).
- **`T<n> M6` alone on its line**, after `G53 G0 Z<safe>`. Some senders hold
  the M6 block and drop the rest of it. No G28/G30 before the change: their
  stored positions are controller settings, machine zero when unset.
- **No control compensation**: grblHAL lists G40 only, so Control and Wear ops
  are refused by name, like Heidenhain and Tormach.
- **Canned cycles always on**: G73/G81/G82/G83/G86/G89 and G98 are in the core
  list without a configuration marker, so the issue's "optional, else
  expanded" switch was dropped. Dwell P in seconds.
- **Tapping refused by name**: G84 is not in the list and a tap cannot be
  plain moves.
- **Probing refused by the engine** (no probe hooks): G38.2-G38.5 exist, but
  the result test and offset arithmetic need parameters and expressions the
  core does not list.
- **Go home after an operation refused by name**: same reason as the G30 above.
- **Safe Z from the machine file**, refused by name when unstated (only "no
  machine" keeps G53 Z0): where machine Z0 sits is the builder's choice on
  grblHAL (homing direction, pull-off, origin setting).
- **Header without G64, O word, % or N numbers**: grblHAL lists G61 only, and
  O words need its macro plugins.

**Cost to change later:** low - a post option for the length offset or for G30
before M6 adds words without moving any existing golden.
