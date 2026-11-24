## #2428 - Update check UI placement and prefs.json persistence (Part B)

**What was decided.**
1. Place the `"Update available: vX.Y.Z"` item as a compact clickable pill button in the ribbon's top band (`panes.rs` `band`), right-aligned immediately to the left of `"Settings"`, `"Home"`, and `"Donate"` (directly above the `#352` version badge in `badge_rect`), opening the validated GitHub release-notes URL in the user's browser on click.
2. Add a `"Settings"` button in the top-right band (left of `"Home"` and `"Donate"`) opening a foreground dropdown `Area` with the `"Check for updates"` toggle, persisted immediately to `prefs.json` (`checkForUpdates` written only when `false`, `lastUpdateCheckUtc` stored as UTC Unix epoch seconds `u64`).
3. Put the GitHub Releases HTTP client (`reqwest` + `tokio`), SemVer comparison, URL validation, and 24-hour throttle in `crates/app/src/update_check.rs`, gated off `wasm32`, and run the one-shot startup check only from `shell_window::run_windowed` on native desktop startup. Note on `AGENTS.md` ("`app` holds the document, nothing else"): `update_check.rs` reads none of the five document items (ECS world, project file, command, MCP, CLI) directly, and it stays in `crates/app` because `camcraft-app` already owns the workspace's `reqwest` and `tokio` dependencies (matching `crates/app/src/ai/http.rs`), whereas moving HTTP checks to `camcraft-ui` would add new dependencies to `camcraft-ui`.

**Why.**
- At the default 1280 px window width, `badge_rect` in the ribbon body is only 157 px wide (and shrinks to 2 px when four operations are pinned), so `"Update available: vX.Y.Z"` would clip or disappear inside `badge_rect` while the top band has ~490 px of unused width directly above `#352`.
- CamCraft has no File/Help/Settings menu yet; placing `"Check for updates"` in `"Theme"` or `"Panels"` would hide an unrelated preference, whereas a top-band `"Settings"` dropdown follows the `Area` + `painted::menu_item` + `Role::MenuItem` pattern of `view_drop` and `theme_drop` and leaves `History`/dirty state untouched.
- Storing `lastUpdateCheckUtc` as epoch seconds avoids adding any date/time crate dependency (`zero new dependencies`).
- Spawning the background check solely in `shell_window::run_windowed` (with winit `EventLoopProxy` wakeups) keeps idle `redraw_delay == None` and guarantees unit and headless `ShellUi` tests never hit the network.
- Keeping all update-check logic together in `crates/app/src/update_check.rs` reuses `camcraft-app`'s existing `reqwest` + `tokio` dependency pair without adding new crate dependencies to `camcraft-ui`.

**Considered and dropped.**
- Placing `"Update available: vX.Y.Z"` inside `badge_rect` beside the build label: dropped because `157 px` at 1280 px cannot fit both horizontally and clipping/hiding happens when operations are pinned.
- Placing `"Check for updates"` inside `theme_drop.rs` or `home.rs`: dropped because Theme is strictly appearance/colour vision and Home is a modal start page; a dedicated `"Settings"` dropdown stays reachable from any workspace view.
- Polling on a frame timer while HTTP is in flight: dropped to avoid burning idle frames or violating `frame_cost` / `state_redraw_delay == None`.
- Splitting the pure helpers (`parse_semver`, `is_newer_version`, `should_check_now`, `is_safe_release_url`, `parse_latest_release`) out of `crates/app/src/update_check.rs` into `camcraft-ui` while leaving only the `reqwest` HTTP fetch in `camcraft-app`: dropped because `fetch_latest_release` validates the URL and parses the JSON payload inside `camcraft-app` before returning `ReleaseInfo`, and `camcraft-app` cannot depend on `camcraft-ui`, so splitting would push unvalidated response strings across the crate boundary; maintainer may override if a thin `camcraft-app` transport wrapper is preferred.

**Cost to change later.**
Low: moving the toggle or notice widget only touches `settings_drop.rs`, `update_notice.rs`, and `panes.rs` (`~15` lines); Part C (download, SHA-256 verification, and self-update install) can replace or extend the click handler in `update_notice.rs` without changing `prefs.json` or `update_check.rs` semver/throttling.
