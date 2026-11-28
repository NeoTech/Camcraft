//! Report capture completion regressions (#2745).
//! Provenance: our own callback/channel and status-strip tests; no device loss.
use super::*;
use crate::egui_app::{panes, report, theme};
use egui_kittest::{kittest::Queryable, Harness};

fn harness() -> Harness<'static, State> {
    Harness::builder().with_size(egui::vec2(1200.0, 400.0)).build_ui_state(
        |ui, state| {
            report::window(ui.ctx(), state);
            let pal = theme::current();
            egui::Panel::bottom("status")
                .frame(egui::Frame::NONE)
                .exact_size(theme::status_h(&pal))
                .show(ui, |ui| {
                    panes::status(ui, state, &pal, None);
                });
        },
        State::new(),
    )
}

#[test]
fn failed_readback_reaches_visible_status_without_copying_an_image() {
    let mut h = harness();
    h.state_mut().status = "copying the whole window".to_owned();
    let mut capture = Capture::new();
    capture.in_flight.store(true, Ordering::Release);
    capture.shot_since = Some(Instant::now());
    // None is the real callback's result for a map/range/decode failure.
    complete(
        &capture.tx,
        &capture.in_flight,
        vec![report::shot_token().clone()],
        None,
    );
    assert!(capture.busy(), "a queued failure still needs a frame");
    let events = capture.drain_events(h.state_mut());
    h.input_mut().events.extend(events);
    h.step();
    assert_eq!(
        h.state().status,
        "Screenshot: picture was not copied - the text report is complete"
    );
    h.get_by_label("Screenshot: picture was not copied - the text report is complete");
    assert!(!h
        .output()
        .platform_output
        .commands
        .iter()
        .any(|c| matches!(c, egui::OutputCommand::CopyImage(_))));
    assert!(!capture.busy());
}

#[test]
fn cancelled_map_uses_the_real_callback_failure_path() {
    let Some(gpu) = crate::headless_gpu::test_gpu("report capture cancellation").expect("GPU policy") else {
        return;
    };
    let mut capture = Capture::new();
    capture.want(report::shot_token().clone(), Instant::now());
    let shot = capture
        .begin(&gpu.device, [2, 2], wgpu::TextureFormat::Rgba8UnormSrgb)
        .expect("shot");
    let buffer = shot.buffer.clone();
    shot.finish();
    // wgpu's public cancellation API fails the pending map. This is not
    // device loss and requires no permanent fault-injection option.
    buffer.unmap();
    capture.pump(&gpu.device);
    let mut h = harness();
    h.state_mut().status = "copying the whole window".to_owned();
    let events = capture.drain_events(h.state_mut());
    h.input_mut().events.extend(events);
    h.step();
    assert_eq!(
        h.state().status,
        "Screenshot: picture was not copied - the text report is complete"
    );
    assert!(!capture.busy());
}

#[test]
fn foreign_failure_preserves_status_and_closed_report_still_gets_its_failure() {
    let mut h = harness();
    assert!(!h.state().report_open);
    let mut capture = Capture::new();
    for (token, expected) in [
        (UserData::new("another door"), "keep me"),
        (
            report::shot_token().clone(),
            "Screenshot: picture was not copied - the text report is complete",
        ),
    ] {
        h.state_mut().status = "keep me".to_owned();
        capture.shot_since = Some(Instant::now());
        complete(&capture.tx, &capture.in_flight, vec![token], None);
        let events = capture.drain_events(h.state_mut());
        assert!(events.is_empty(), "failure never carries an image");
        h.step();
        assert_eq!(h.state().status, expected);
        h.get_by_label(expected);
        assert!(!capture.busy());
    }
}

#[test]
fn success_answers_all_tokens_and_a_waiting_request_recovers_after_failure() {
    let Some(gpu) = crate::headless_gpu::test_gpu("report capture recovery").expect("GPU policy") else {
        return;
    };
    let mut capture = Capture::new();
    capture.want(report::shot_token().clone(), Instant::now());
    let failed = capture
        .begin(&gpu.device, [2, 2], wgpu::TextureFormat::Rgba8UnormSrgb)
        .expect("shot");
    let other = UserData::new("another door");
    capture.want(other.clone(), Instant::now());
    capture.want(report::shot_token().clone(), Instant::now());
    complete(&failed.tx, &failed.in_flight, failed.tokens, None);
    let mut h = harness();
    assert!(capture.drain_events(h.state_mut()).is_empty());
    assert!(capture.busy(), "later requests survive the failed shot");
    assert!(capture.due(h.state_mut(), Instant::now()));
    let shot = capture
        .begin(&gpu.device, [2, 2], wgpu::TextureFormat::Rgba8UnormSrgb)
        .expect("fresh shot");
    shot.finish();
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("map callback");
    assert!(capture.busy(), "a completed image still needs a frame");
    let events = capture.drain_events(h.state_mut());
    assert_eq!(events.len(), 2);
    assert!(matches!(&events[0], egui::Event::Screenshot {user_data, ..} if *user_data == other));
    assert!(matches!(&events[1], egui::Event::Screenshot {user_data, ..} if user_data == report::shot_token()));
    h.input_mut().events.extend(events);
    h.step();
    assert!(h
        .output()
        .platform_output
        .commands
        .iter()
        .any(|c| matches!(c, egui::OutputCommand::CopyImage(_))));
    assert_eq!(
        h.state().status,
        "Screenshot: CamCraft copied the whole window to the clipboard"
    );
    assert!(!capture.busy());
}

#[test]
fn timeout_discards_late_replies_without_releasing_a_fresh_shot() {
    let Some(gpu) = crate::headless_gpu::test_gpu("report capture timeout").expect("GPU policy") else {
        return;
    };
    let now = Instant::now();
    let mut capture = Capture::new();
    let mut state = State::new();
    capture.want(report::shot_token().clone(), now);
    let old = capture
        .begin(&gpu.device, [2, 2], wgpu::TextureFormat::Rgba8UnormSrgb)
        .expect("old shot");
    let later = capture.shot_since.expect("started") + CAPTURE_TIMEOUT + Duration::from_secs(1);
    assert!(!capture.due(&mut state, later));
    assert!(!capture.busy());
    for image in [None, Some(Arc::new(ColorImage::filled([1, 1], Color32::RED)))] {
        capture.want(report::shot_token().clone(), later);
        let fresh = capture
            .begin(&gpu.device, [2, 2], wgpu::TextureFormat::Rgba8UnormSrgb)
            .expect("fresh shot");
        state.status = "fresh request".to_owned();
        complete(&old.tx, &old.in_flight, old.tokens.clone(), image);
        assert!(capture.drain_events(&mut state).is_empty());
        assert_eq!(state.status, "fresh request");
        assert!(capture.in_flight.load(Ordering::Acquire));
        assert!(capture.shot_since.is_some());
        complete(&fresh.tx, &fresh.in_flight, fresh.tokens, None);
        capture.drain_events(&mut state);
        assert!(!capture.busy());
    }
    capture.want(report::shot_token().clone(), now);
    assert!(!capture.due(&mut state, later));
    assert!(!capture.busy(), "an unread waiting request also times out");
}
