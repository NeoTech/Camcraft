# #1230 - part-alignment refusal names the machine kind

**Chosen.** The aligned-setup refusal in `setup_align::generation_problem`
names the setup's actual machine kind ("sinker EDM", "wire EDM", "plasma",
...) instead of "a lathe or mill-turn machine". The name comes from one
table, `stock_sim_carry::machine_kind_name`, which #1229's
`carry_machine_refusal` now also calls. The `setup_stock` doc comment that
said "a lathe or mill-turn machine" now says "a machine that is not a mill".

**Why.** The refusal fired for every non-mill kind, so an EDM setup was told
it was a lathe. One table keeps the carry refusal and the alignment refusal
in step.

**Considered and dropped.**
- A second name table in `setup_align` - dropped, the brief asked for one.
- Loosening or tightening the condition - dropped, the brief asked that the
  refusal stay exactly as strict; only the words change.
- Reformatting the other rustfmt diffs in the touched files - dropped, they
  predate this branch and are out of scope.

**Pinned by.** `setup_align::an_aligned_setup_on_an_edm_names_the_edm_kind_it_is_refused_on`
(sinker and wire fixtures). The existing lathe and mill-turn carry tests
(`carried_fixtures_cache`) still pass.
