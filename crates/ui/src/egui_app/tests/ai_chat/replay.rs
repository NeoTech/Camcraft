//! #3411: offline replay of raw q08 Ollama replies through the real panel and inbox.
//! Provenance: mkinney/Camcraft qa-artifacts, q08-20261122T131017Z; see fixtures/README.md.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use serde_json::{Value, json};

use super::super::super::ai_chat::{
    ChatTransport, ToolCardStatus, TransportOutput, TransportRequest,
};
use super::super::{CommandKind, State};
use super::{
    blank_harness, pump_until_idle, select_provider, submit_prompt, toggle_ai_chat_via_view_door,
};

struct ReplayTransport {
    replies: Mutex<VecDeque<Value>>,
    requests: Mutex<Vec<TransportRequest>>,
}

impl ReplayTransport {
    fn new(turn: &Value) -> Arc<Self> {
        Arc::new(Self {
            replies: Mutex::new(
                turn["rounds"]
                    .as_array()
                    .expect("rounds")
                    .iter()
                    .map(|round| round["model_reply"].clone())
                    .collect(),
            ),
            requests: Mutex::new(Vec::new()),
        })
    }
}

impl ChatTransport for ReplayTransport {
    fn complete(
        &self,
        req: &TransportRequest,
        cancel: &AtomicBool,
    ) -> Result<TransportOutput, String> {
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".into());
        }
        self.requests.lock().expect("requests").push(req.clone());
        let reply = self
            .replies
            .lock()
            .expect("replies")
            .pop_front()
            .ok_or("replay exhausted: unexpected extra model round")?;
        // q08 stores the message fields and finish_reason, not the outer HTTP envelope.
        // Do not parse/re-serialize function.arguments or turn content into tool calls.
        Ok(TransportOutput::AssistantResponse {
            text: reply["content"].as_str().unwrap_or_default().into(),
            envelope: json!({"choices":[{"message":{
                "role":"assistant", "content":reply["content"], "tool_calls":reply["tool_calls"]
            }, "finish_reason":reply["finish_reason"]}]}),
            fallback_override: None,
        })
    }
}

fn call(state: &mut State, name: &str, arguments: Value) -> Value {
    let request = json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
        "params":{"name":name,"arguments":arguments}});
    let answer = camcraft_app::mcp::answer_live(
        &state.world,
        &mut state.history,
        state.project_path.as_deref(),
        &request.to_string(),
    );
    let reply: Value = serde_json::from_str(&answer.reply.expect("reply")).expect("JSON");
    assert_eq!(reply["result"]["isError"], false, "{reply}");
    reply["result"]["structuredContent"].clone()
}

fn operations(state: &mut State) -> Value {
    call(state, "get_job", json!({}))["operations"].clone()
}

// New schema fields may be additive. Every recorded field must still match.
fn assert_recorded(actual: &Value, expected: &Value) {
    if let Some(fields) = expected.as_object() {
        for (key, value) in fields {
            assert_recorded(&actual[key], value);
        }
    } else if let Some(items) = expected.as_array() {
        assert_eq!(
            actual.as_array().expect("array").len(),
            items.len(),
            "{actual}"
        );
        for (a, e) in actual.as_array().expect("array").iter().zip(items) {
            assert_recorded(a, e);
        }
    } else {
        assert_eq!(actual, expected);
    }
}

fn replay(fixture: &str, deny_deletes: bool) {
    let fixture: Value = serde_json::from_str(fixture).expect("fixture JSON");
    assert_eq!(fixture["format_version"], 1);
    let project = match fixture["initial_project"].as_str().expect("project") {
        "q08-start.camproj" => include_str!("fixtures/q08-start.camproj"),
        "q08-after-step5.camproj" => include_str!("fixtures/q08-after-step5.camproj"),
        other => panic!("unknown project {other}"),
    };
    let mut h = blank_harness();
    let world = camcraft_app::world::new_world();
    camcraft_app::project::open_noted(&world, project).expect("recorded project loads");
    h.state_mut().state.world = world;
    for setup in fixture["setup"].as_array().expect("setup") {
        call(
            &mut h.state_mut().state,
            setup["name"].as_str().expect("tool"),
            setup["arguments"].clone(),
        );
    }
    h.run();
    toggle_ai_chat_via_view_door(&mut h);
    select_provider(&mut h, "Offline local (Ollama)");

    for turn in fixture["turns"].as_array().expect("turns") {
        let before = operations(&mut h.state_mut().state);
        let depth = h.state().state.history.done_len();
        let transport = ReplayTransport::new(turn);
        h.state_mut()
            .state
            .ai_chat
            .set_transport(Arc::clone(&transport) as Arc<dyn ChatTransport>);
        submit_prompt(&mut h, turn["user"].as_str().expect("user"));
        let confirmations = turn["rounds"]
            .as_array()
            .expect("rounds")
            .iter()
            .flat_map(|round| {
                round["model_reply"]["tool_calls"]
                    .as_array()
                    .into_iter()
                    .flatten()
            })
            .filter(|call| call["function"]["name"] == "delete_operation")
            .count();
        for _ in 0..confirmations {
            // The recorded edit turn verifies geometry between its two deletes.
            // Reuse the suite hang guard, not the short scripted-confirmation deadline.
            let deadline = Instant::now() + super::HANG_GUARD;
            while h.query_by_role_and_label(Role::Button, "Allow").is_none() {
                assert!(
                    Instant::now() < deadline && h.state().state.ai_chat.is_running(),
                    "replay stopped before confirmation: {:?}",
                    h.state().state.ai_chat.last_error()
                );
                std::thread::sleep(Duration::from_millis(5));
                h.step();
            }
            h.run_steps(2);
            assert!(h.query_by_label_contains("Confirmation required").is_some());
            if deny_deletes {
                assert_eq!(operations(&mut h.state_mut().state), before);
            }
            h.get_by_role_and_label(Role::Button, if deny_deletes { "Deny" } else { "Allow" })
                .click();
            h.run_steps(2);
        }
        pump_until_idle(&mut h);
        assert!(transport.replies.lock().expect("replies").is_empty());
        assert!(
            !h.state()
                .state
                .ai_chat
                .last_error()
                .is_some_and(|s| s.contains("replay exhausted"))
        );
        let after = operations(&mut h.state_mut().state);
        if deny_deletes {
            assert_eq!(after, before);
            assert_eq!(h.state().state.history.done_len(), depth);
        } else {
            assert_recorded(&after, &turn["expected_operations"]);
            assert_eq!(
                h.state().state.history.done_len(),
                depth + 1,
                "one undo group per AI turn"
            );
            h.state_mut().state.run(CommandKind::Undo);
            assert_recorded(&operations(&mut h.state_mut().state), &before);
            h.state_mut().state.run(CommandKind::Redo);
            assert_recorded(&operations(&mut h.state_mut().state), &after);
        }
        let chat_turn = h.state().state.ai_chat.turns().last().expect("turn");
        let raw_calls: Vec<&Value> = turn["rounds"]
            .as_array()
            .expect("rounds")
            .iter()
            .flat_map(|round| {
                round["model_reply"]["tool_calls"]
                    .as_array()
                    .into_iter()
                    .flatten()
            })
            .collect();
        assert_eq!(
            chat_turn.tool_cards.len(),
            raw_calls.len(),
            "every raw call reaches the panel"
        );
        for (card, raw) in chat_turn.tool_cards.iter().zip(raw_calls) {
            assert_eq!(card.id, raw["id"].as_str().expect("call id"));
            assert_eq!(
                card.name,
                raw["function"]["name"].as_str().expect("tool name")
            );
            let arguments: Value = serde_json::from_str(
                raw["function"]["arguments"]
                    .as_str()
                    .expect("raw arguments"),
            )
            .expect("arguments JSON");
            assert_eq!(
                card.arguments, arguments,
                "production parser preserves recorded arguments"
            );
        }
        let requests = transport.requests.lock().expect("requests");
        assert_eq!(
            requests.len(),
            turn["rounds"].as_array().expect("rounds").len()
        );
        for request in requests.iter() {
            assert_eq!(
                request
                    .messages
                    .iter()
                    .rev()
                    .find(|m| m.role == "user")
                    .expect("user message")
                    .text,
                turn["user"].as_str().expect("user")
            );
        }
        for id in turn["refused_call_ids"].as_array().expect("refusal IDs") {
            let card = chat_turn
                .tool_cards
                .iter()
                .find(|card| card.id == id.as_str().expect("id"))
                .expect("refusal card");
            assert_eq!(card.status, ToolCardStatus::Failed, "{}", card.summary);
            assert!(
                !card.summary.is_empty(),
                "refusal has a visible explanation"
            );
            assert!(
                h.query_by_label_contains(&card.summary).is_some(),
                "refusal is rendered in the real panel"
            );
            assert!(
                requests
                    .iter()
                    .flat_map(|r| &r.messages)
                    .any(
                        |m| m.raw.as_ref().is_some_and(|raw| raw["tool_call_id"] == *id
                            && raw["content"]
                                .as_str()
                                .is_some_and(|s| serde_json::from_str::<Value>(s)
                                    .is_ok_and(|result| result["isError"] == true)))
                    ),
                "real refusal is returned to the model"
            );
        }
        if deny_deletes {
            assert_eq!(
                chat_turn
                    .tool_cards
                    .iter()
                    .filter(|card| card.status == ToolCardStatus::Denied)
                    .count(),
                confirmations
            );
        }
    }
}

#[test]
fn recorded_q08_finishing_turn() {
    replay(include_str!("fixtures/q08-finishing.json"), false);
}

#[test]
fn recorded_q08_edit_turn() {
    replay(include_str!("fixtures/q08-edit.json"), false);
}

#[test]
fn recorded_q08_delete_denied_preserves_document() {
    replay(include_str!("fixtures/q08-delete.json"), true);
}

#[test]
fn recorded_q08_delete_allowed_is_one_undo_group() {
    replay(include_str!("fixtures/q08-delete.json"), false);
}
