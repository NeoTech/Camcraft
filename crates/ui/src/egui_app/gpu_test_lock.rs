//! Test-only: one process-wide lock so the lib tests that make a wgpu
//! instance (directly, or through egui_kittest's screenshot renderer) use
//! the GPU one at a time (#3589: parallel instance creation and teardown
//! ended the test process with STATUS_ACCESS_VIOLATION on Windows 11).
//! Provenance: our own test harness; std::sync primitives only.
//!
//! Take the guard before the first call that can make an instance (for a
//! kittest harness: before the harness itself, so the harness and its lazy
//! renderer drop first and the guard drops last).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

static GPU_TEST_LOCK: Mutex<()> = Mutex::new(());
/// Guards alive now. The lock keeps it at 0 or 1.
static HOLDERS: AtomicUsize = AtomicUsize::new(0);
/// The highest count of [`HOLDERS`] this process saw.
static MAX_HOLDERS: AtomicUsize = AtomicUsize::new(0);

/// Holds the GPU for one test. Drop order: declare it before the harness.
pub(crate) struct GpuGuard {
    _lock: MutexGuard<'static, ()>,
}

impl Drop for GpuGuard {
    fn drop(&mut self) {
        // Runs before `_lock` is released (fields drop after `drop`), and
        // on a panic's unwind too, so a failed test does not leave a count.
        HOLDERS.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Wait for the GPU, then hold it until the guard drops. A test that
/// panicked with the lock held poisons it; the next test takes it anyway,
/// so one failure does not cascade into the other GPU tests.
pub(crate) fn gpu_guard() -> GpuGuard {
    let lock = GPU_TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let now = HOLDERS.fetch_add(1, Ordering::SeqCst) + 1;
    MAX_HOLDERS.fetch_max(now, Ordering::SeqCst);
    assert_eq!(now, 1, "two tests hold the GPU at once (#3589)");
    GpuGuard { _lock: lock }
}

/// The lib tests that make a wgpu instance. Each must take [`gpu_guard`];
/// `every_gpu_test_takes_the_lock` reads their source to pin it.
const GPU_TESTS: [(&str, &str); 8] = [
    ("tests.rs", "fn headless_run_succeeds()"),
    ("gpu.rs", "fn adapter_probe_names_the_gpu()"),
    ("report_tests.rs", "fn the_door_opens_the_window_and_a_kind_button_files_the_url()"),
    ("report_tests.rs", "fn the_title_field_travels_into_the_url()"),
    ("report_tests.rs", "fn a_kind_button_asks_the_shell_for_the_whole_window()"),
    ("report_tests.rs", "fn the_answer_copies_the_picture_and_says_the_word()"),
    ("report_tests.rs", "fn the_picture_is_still_copied_after_the_window_is_closed()"),
    ("report_tests.rs", "fn no_answer_copies_nothing()"),
];

fn source_of(file: &str) -> &'static str {
    match file {
        "tests.rs" => include_str!("tests.rs"),
        "gpu.rs" => include_str!("gpu.rs"),
        "report_tests.rs" => include_str!("report_tests.rs"),
        other => panic!("no source table entry for {other}"),
    }
}

#[test]
fn every_gpu_test_takes_the_lock() {
    for (file, sig) in GPU_TESTS {
        let src = source_of(file);
        let start = src.find(sig).unwrap_or_else(|| panic!("{file}: {sig} not found"));
        let body = &src[start..];
        let open = body.find('{').unwrap_or_else(|| panic!("{file}: {sig} has no body"));
        // The guard is the first statement, before any harness or instance.
        let first = body[open + 1..].trim_start();
        let first = first.lines().find(|l| !l.trim_start().starts_with("//")).unwrap_or("");
        assert!(
            first.trim_start().starts_with("let _gpu = ") && first.contains("gpu_guard()"),
            "{file}: {sig} must start with `let _gpu = gpu_guard();`, found {first:?}"
        );
    }
}

#[test]
fn the_lock_lets_one_holder_in_at_a_time() {
    let threads: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| {
                for _ in 0..20 {
                    let _gpu = gpu_guard();
                    assert_eq!(HOLDERS.load(Ordering::SeqCst), 1);
                    std::thread::yield_now();
                }
            })
        })
        .collect();
    for t in threads {
        assert!(t.join().is_ok(), "a holder saw a second holder");
    }
    assert_eq!(MAX_HOLDERS.load(Ordering::SeqCst), 1, "the GPU was held twice at once");
}

#[test]
fn a_panic_with_the_lock_held_does_not_block_the_next_test() {
    let failed = std::thread::spawn(|| {
        let _gpu = gpu_guard();
        panic!("a GPU test fails while it holds the lock");
    })
    .join();
    assert!(failed.is_err());
    // The lock is poisoned now; the next holder still gets in, alone.
    let _gpu = gpu_guard();
    assert_eq!(HOLDERS.load(Ordering::SeqCst), 1);
}
