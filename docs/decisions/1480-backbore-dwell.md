## #1480 - Back bore (G89) carries the dwell and feeds out at the plunge feed

**What was decided.** The drill generator (`camcraft_motion::drilling::drill`)
now writes the user's dwell into the `Cycle` record of a Back bore hole. It
also feeds the tool out at the plunge feed, the same feed as the way in.
Before, the dwell was forced to 0 and the feed out used the cutting feed.

**Why.**
- G89 is "feed in, dwell, feed out" (RS-274/NGC, NIST IR 6556; LinuxCNC
  G-code reference). The cycle exists for its dwell, and the user could not
  get one.
- A canned G89 (and the G85 that several posts write for a Back bore with no
  dwell) has one F word. The post takes that F from the first cutting move
  of the hole's chain, which is the plunge. So the control always fed out at
  the plunge feed. The chain said the cutting feed. The verifier, the
  simulation time and the `.cl` dump then disagreed with the machine. The
  Ream cycle (G85, #1445) already uses the one-feed chain; Back bore now
  uses the same law.
- Each post reads the dwell in seconds from the stream and writes its own
  P unit, or refuses by name. Fanuc, Mazak, Doosan, Haas and Sinumerik write
  whole milliseconds; grblHAL, Tormach, LinuxCNC and Mach4 write seconds;
  Heidenhain writes an explicit dwell block. The lathe posts and the posts
  with no canned cycle refuse the Back bore.
  `crates/post/tests/dwell_cycles_modal_p.rs`, `haas_dwell_cycles.rs`,
  `mach4_post.rs`, `heidenhain_cycles.rs` and `sinumerik_g89_dwell.rs` pin
  these words (#2350, #2360).
- Sinumerik G89 now writes `P0` for zero dwell. This clears a previous
  positive dwell instead of leaving its modal P active. The Siemens
  [ISO dialects Function Manual, 07/2021](https://cache.industry.siemens.com/dl/files/254/109801254/att_1077768/v1/ONE_840Dsl_828D_iso_dialects_fct_man_0721_en-US.pdf),
  table 6-1, printed page 95, gives the G89 dwell unit in milliseconds.
  Section 6.3.1 describes modal cycle parameters. The tests
  `sinumerik_g89_dwell_posts_p_in_milliseconds` and
  `sinumerik_g89_zero_dwell_clears_previous_p` pin the unit and its reset.

**Considered and dropped.**
- Keep the feed out at the cutting feed, and make each post write a second F
  word. A canned cycle has no second F word. A post would have to expand the
  cycle into G1 moves, which changes every Back bore program.
- A separate "retract feed" parameter for bore cycles. This is a new
  parameter and a new command field. It is not necessary for this issue.
- A new G89 golden for each post in this PR. Direct assertions pin each
  P word. The Sinumerik tests also pin the transition from positive dwell
  to zero.

**Cost to change later.** Small. The feed out is one line in
`crates/motion/src/drilling/cycle.rs`, pinned by
`back_bore_feeds_in_and_out_at_the_plunge_feed`. A Back bore program with no
dwell keeps the same F. Its Sinumerik G89 block now includes `P0` to clear
a previous dwell. The `.cl` chain of a Back bore changes its feed-out feed.
On the Heidenhain post,
which writes the Back bore as an explicit chain, the retract line now has
the plunge feed.
