//! Pin the dispatch instructions, not an atomic lock. Provenance: #3661 and its maintainer follow-ups.

const WORK: &str = include_str!("../../.claude/skills/work-issue/SKILL.md");
const REVIEW: &str = include_str!("../../.claude/skills/review-prs/SKILL.md");
const REFRESH: &str = include_str!("../../.claude/skills/resolve-conflicts/SKILL.md");
const MERGE: &str = include_str!("../../.claude/skills/merge-ready/SKILL.md");
const COORDINATOR: &str = include_str!("../../.claude/agents/coordinator.md");
const START: &str = include_str!("../../.claude/skills/start-orchestrator/SKILL.md");
const REVIEWER: &str = include_str!("../../.claude/agents/PR-reviewer.md");

fn section<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    text.split_once(start)
        .expect("section start")
        .1
        .split_once(end)
        .expect("section end")
        .0
}

#[test]
fn ownership_review_prompt_passes_the_record_and_head() {
    requires(
        section(REVIEW, "2. Agent tool", "3. On its completion"),
        &["record URL", "Base:", "Starting exact head:", "release"],
    );
}

#[test]
fn ownership_reviewer_checks_and_releases_the_record() {
    requires(
        REVIEWER,
        &[
            "../skills/issue-claims/references/ownership.md",
            "0. **Ownership",
            "record URL",
            "directly",
            "collaborator",
            "pre-check",
            "no reviewer record",
        ],
    );
    for (start, end) in [
        ("6. **Law check", "7. **Post"),
        ("7. **Post", "8. **Label"),
        ("8. **Label", "9. **Clean up"),
    ] {
        requires(
            section(REVIEWER, start, end),
            &["re-read", "head", "ownership"],
        );
    }
    requires(
        section(REVIEWER, "9. **Clean up", "## Report back"),
        &["release", "record URL", "failure", "skip", "read back"],
    );
}

#[test]
fn ownership_delta_dispatches_carry_the_full_lifecycle() {
    requires(
        section(REFRESH, "- **DELTA**", "- **NEW CODE**"),
        &[
            "collaborator",
            "record URL",
            "before work",
            "re-read",
            "release",
        ],
    );
    requires(
        section(MERGE, "**Wave 2**", "**After all merges:**"),
        &[
            "collaborator",
            "record URL",
            "before work",
            "re-read",
            "release",
        ],
    );
}

#[test]
fn ownership_refresh_shared_inputs_keep_per_package_checks() {
    for text in [GUARD, REFRESH, MERGE] {
        requires(
            text,
            &[
                "cargo check --tests -p",
                "Cargo.lock",
                "one package at a time",
                "never `--workspace`",
            ],
        );
    }
}

fn requires(text: &str, phrases: &[&str]) {
    let text = words(text);
    for phrase in phrases {
        assert!(
            text.contains(phrase),
            "missing ownership contract: {phrase}"
        );
    }
}

#[test]
fn ownership_coordinator_passes_a_verified_record_to_each_actor() {
    requires(
        COORDINATOR,
        &[
            "../skills/issue-claims/references/ownership.md",
            "before dispatch",
            "writer",
            "reviewer",
            "record URL",
            "starting exact head",
            "read it back",
            "stand down",
            "release",
            "must not push",
        ],
    );
}

#[test]
fn ownership_startup_and_review_fallback_use_the_same_guard() {
    requires(
        START,
        &[
            "../issue-claims/references/ownership.md",
            "before dispatch",
            "coordinator",
            "record URL",
            "read-only Opus",
            "release",
            "claim lifetimes",
        ],
    );
}

#[test]
fn ownership_worker_distinguishes_dispatched_and_self_started_claims() {
    requires(
        WORK,
        &[
            "Caller-dispatched",
            "Self-started",
            "do not post another `claimed:`",
            "same claimant",
            "record URL",
            "read it back",
            "before it edits",
            "named review fix",
            "reuse the existing branch",
        ],
    );
}

#[test]
fn ownership_reviewer_records_require_collaborator_authority() {
    requires(
        GUARD,
        &[
            "collaborator-authored reviewer records count",
            "record author",
            "reviewer's GitHub login",
            "permissions.push",
            "Failed authority reads mean STOP",
            "pre-check uses no reviewer record",
            "non-collaborator record does not block",
            "no expiry timer",
        ],
    );
}

#[test]
fn ownership_writer_liveness_keeps_each_existing_claim_case() {
    requires(
        GUARD,
        &[
            "only activity tests",
            "open PR for the issue",
            "branch has a commit in the last 48 hours",
            "commented on the issue in the last 48 hours",
            "All three absent",
            "Withdrawn or released",
            "new claim",
            "4 hours",
            "no writer-specific timeout",
            "released writer record stays released",
        ],
    );
}

#[test]
fn ownership_dispatch_paths_load_the_shared_guard() {
    for (name, skill) in [
        ("work", WORK),
        ("review", REVIEW),
        ("refresh", REFRESH),
        ("merge", MERGE),
    ] {
        assert!(
            skill.contains("../issue-claims/references/ownership.md"),
            "{name} must load the shared public ownership procedure before dispatch"
        );
        assert!(
            words(skill).contains("before dispatch"),
            "{name} must guard dispatch"
        );
    }
}

const GUARD: &str = include_str!("../../.claude/skills/issue-claims/references/ownership.md");

fn words(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn ownership_record_and_lifecycle_contract() {
    let text = words(GUARD);
    for required in [
        "canonical writer record",
        "canonical review record",
        "Owner:",
        "Dispatcher:",
        "Issue:",
        "PR:",
        "Branch:",
        "Base:",
        "Starting head:",
        "Expected remote:",
        "Started:",
        "State: active",
        "comment URL",
        "threads and remote head",
        "after posting and before dispatch",
        "4 hours",
        "48-hour",
        "An ended claim stays ended",
        "does not release the issue claim",
        "Age alone does not release",
        "Ownership released: <record URL>",
        "both sessions STOP",
        "one owner explicitly releases",
    ] {
        assert!(
            text.contains(required),
            "ownership procedure must retain: {required}"
        );
    }
}

#[test]
fn ownership_stop_and_non_atomic_limits_are_explicit() {
    let text = words(GUARD);
    for required in [
        "live writer means stand down",
        "live reviewer means stand down",
        "owner and record URL",
        "incomplete reads mean STOP",
        "If the remote head moved, STOP",
        "A rejected push is STOP",
        "Do not merge that unexpected head",
        "Never force-push",
        "including `--force-with-lease`",
        "not an atomic distributed lock",
        "plain push rejects non-fast-forward updates",
        "can accept a fast-forward after a race",
        "not execution by an agent or distributed exclusion",
    ] {
        assert!(
            text.contains(required),
            "ownership procedure must retain: {required}"
        );
    }
    let review = words(REVIEW);
    assert!(review.contains("Keep the local `.claude/pr-review.lock`"));
    assert!(review.contains("stop without a current-head verdict"));
}

#[test]
fn ownership_refresh_checks_precede_push_and_preserve_merge_validation() {
    for skill in [REFRESH, MERGE, GUARD] {
        let text = words(skill);
        assert!(
            text.contains("cargo check --tests"),
            "refresh must compile affected code"
        );
        assert!(
            text.contains("before push"),
            "refresh compilation must precede publication"
        );
    }
    let guard = words(GUARD);
    assert!(guard.contains("without a build or edit between them and the push"));
    assert!(guard.contains("Any later tree change invalidates those results"));
    let merge = words(MERGE);
    assert!(merge.contains("cargo xtask merge-plan --check <N> <N> ..."));
    assert!(merge.contains("before the first merge, read them again"));
    assert!(merge.contains("Never refresh a PR that is not next"));
}
