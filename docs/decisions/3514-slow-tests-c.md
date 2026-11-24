## #3514 - slice C: the slow adaptive trochoid integration tests

**Decided.** Each of the four tests of slice C ran alone in debug
(`cargo test -j 6 -p camcraft-motion --test <binary> -- --exact <name>
--test-threads=1`, origin/main at 88545cf44, libtest time):

| Test | Binary | Alone (debug) | #3427 said | Result |
|---|---|---|---|---|
| `triangle_and_wide_to_narrow_neck_are_cleared` | `adaptive_troch_neck` | 167.9 s | 687.3 s | heavy lane + `_light` twin |
| `pocket_with_island_is_cleared_without_crossing_or_excess_rest` | `adaptive_troch_island` | 153.9 s | 671.9 s | heavy lane + `_light` twin |
| `ring_round_island_is_cleared_with_closed_lane` | `adaptive_troch_island` | 50.2 s | 401.2 s | stays in the default suite |
| `corner_cleanup_reduces_34x16_rest_to_at_most_hsm_rest` | `adaptive_troch_parallel` | 29.2 s | 168.6 s | stays in the default suite |

The two tests over the 60 s limit of AGENTS.md "Testing" get the reason
`heavy lane: motion heavy: <binary> (...)` and keep all their cases and
asserts. Each gets a `<name>_light` twin in the same file. The twin runs
one case of the full test through the same `check_cleared` laws:

- neck: the wide-to-narrow neck (48x30 D6) only; the 60x60 triangle stays
  in the full test.
- island: the 30x30 island D6 only; the 48x40 island D10 stays in the full
  test.

**Why.** The #3427 times come from a parallel run of the whole crate. Alone,
the tests are about 4-8 times faster, the same as in slices A and B. Only a
test over about 60 s alone moves. The twin keeps the smaller case, so that
the light lane checks the same laws in release at review.

**Dropped.** A twin in a separate file (the context pack's option): the
gate refuses a twin whose file has no `#[ignore]`d full test, so the twin
stays next to its full test. A smaller debug grid in `check_cleared`: it
changes the readings of all the tests that share the fixture.

**Cost to change.** One attribute line per full test. The `#[test]` count
goes up by two (two new twins); no test is removed.
