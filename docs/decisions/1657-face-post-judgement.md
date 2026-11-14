## #1657 - The UI posting test faced through its own part: the judgement is right, the fixture is fixed

**Decided:** the post-merge failure of
`each_setup_picks_its_own_machine_and_post_and_posts_one_file_per_mount` is a
true gouge. PR #1622 (#1607) put Face under the stock-source keep law, so a
stock-source Face is now judged against the setup's parts. The test's fixture
set a 60 x 60 x 20 box centred on the origin under the seeded 60 x 60 x 30
block and faced it at Z 19.5 with a 25 mm clearance: the face ran through the
block, and the clearance plane was inside it. The law is kept as merged. The
fixture now holds the block inside the stock (66 x 66 x 32 at (30, 30, 0)),
faces at Z 31.5 and clears at Z 45; Setup2's work offset moves to the new top
(Z 32). An app test pins the judgement:
`a_face_of_a_stock_lower_than_the_part_cuts_the_part_and_never_posts`
(`crates/app/tests/stock_source_law.rs`).

**Why:** before #1622 the program verify did not see this cut, so the test
posted a program that cut the part. Loosening the law for Face would let that
gouge through to the post (Prime Directive).

**Considered and dropped:**
- Take Face out of `keep_law::is_stock_kept_kind`. That makes the test pass
  (checked), but it lets a face through the part post again.
- A 64 x 64 x 32 stock (2 mm side spare). The 4 mm mill's last pass is then
  tangent to the stock edge and gets a `Gouge` with no part at all. That looks
  like a separate false positive, filed as #1659 (P3); the fixture uses a
  3 mm spare so this test does not depend on it.

**Cost to change later:** none; test fixture only.
