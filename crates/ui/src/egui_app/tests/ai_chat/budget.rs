//! #1944: the chat worker keeps each request inside a context budget: a
//! long tool reply is cut with a hint, a local model gets the core tool
//! set, and a turn stops at the round cap. No network: scripted transport.
//! Provenance: our own offline shell tests; no vendor material.

use std::sync::atomic::Ordering;

use camcraft_app::mcp::provider::CORE_TOOLS;
use serde_json::{Value, json};

use super::super::super::ai_chat::{TransportOutput, TurnStatus, budget};
use super::{ScriptedTransport, blank_harness, pump_until_idle, select_provider, submit_prompt, toggle_ai_chat_via_view_door};

fn tool_use(id: &str, name: &str) -> Result<TransportOutput, String> {
    Ok(TransportOutput::AssistantResponse {
        text: String::new(),
        envelope: json!({"content": [{"type": "tool_use", "id": id, "name": name, "input": {}}]}),
        fallback_override: None,
    })
}

fn final_text(text: &str) -> Result<TransportOutput, String> {
    Ok(TransportOutput::AssistantResponse {
        text: text.into(),
        envelope: json!({"content": [{"type": "text", "text": text}]}),
        fallback_override: None,
    })
}

#[test]
fn a_long_tool_reply_is_cut_to_the_character_cap_with_a_hint() {
    let mut h = blank_harness();
    h.run();
    toggle_ai_chat_via_view_door(&mut h);
    // The whole operation table is far longer than one reply may be.
    let transport = ScriptedTransport::new(vec![tool_use("t1", "list_operation_types"), final_text("Done.")]);
    h.state_mut().state.ai_chat.set_transport(transport.clone());
    submit_prompt(&mut h, "List the operation kinds");
    pump_until_idle(&mut h);

    let reqs = transport.requests.lock().expect("lock");
    assert_eq!(reqs.len(), 2);
    let reply = reqs[1].messages.last().expect("tool reply");
    assert_eq!(reply.role, "tool");
    // The Anthropic tool_result carries the reply as one JSON text.
    let content = reply.raw.as_ref().and_then(|r| r["content"].as_str()).unwrap_or_default();
    assert!(content.contains("truncated"), "the cut reply says so");
    assert!(
        content.chars().count() <= budget::MAX_TOOL_REPLY_CHARS + 400,
        "the reply sent back is {} characters",
        content.chars().count()
    );
    assert!(reply.text.chars().count() <= budget::MAX_TOOL_REPLY_CHARS + 400);
}

#[test]
fn a_local_model_gets_the_core_tool_set_and_a_cloud_model_the_full_table() {
    let mut h = blank_harness();
    h.run();
    toggle_ai_chat_via_view_door(&mut h);
    let transport = ScriptedTransport::new(vec![final_text("Cloud."), final_text("Local."), final_text("Prompt JSON.")]);
    h.state_mut().state.ai_chat.set_transport(transport.clone());
    submit_prompt(&mut h, "Hello cloud");
    pump_until_idle(&mut h);
    select_provider(&mut h, "Offline local (Ollama)");
    submit_prompt(&mut h, "Hello local");
    pump_until_idle(&mut h);
    select_provider(&mut h, "Local Prompt-JSON");
    submit_prompt(&mut h, "Hello Prompt-JSON");
    pump_until_idle(&mut h);

    let reqs = transport.requests.lock().expect("lock");
    assert_eq!(reqs.len(), 3);
    let count = |v: &Value| v["tools"].as_array().map_or(0, Vec::len);
    let cloud = count(&reqs[0].tools_payload);
    let local = count(&reqs[1].tools_payload);
    assert!(cloud > 40, "cloud has the full table: {cloud}");
    assert_eq!(local, CORE_TOOLS.len(), "native local has the core set");
    let mut expected = CORE_TOOLS.to_vec();
    expected.sort_unstable();
    for (request, native) in [(&reqs[1], true), (&reqs[2], false)] {
        let tools = request.tools_payload["tools"].as_array().expect("tools");
        let mut names: Vec<_> = tools.iter().map(|tool| {
            let name = if native { &tool["function"]["name"] } else { &tool["name"] };
            name.as_str().expect("tool name")
        }).collect();
        names.sort_unstable();
        assert_eq!(names, expected, "both local payloads contain exactly the core tools");
        for name in ["rename_operation", "edit_tool", "set_work_material"] {
            assert!(names.contains(&name), "local payload lacks {name}");
        }
    }
    let fallback = &reqs[2].tools_payload;
    assert!(fallback["project"].is_object(), "fallback retains the project context");
    assert!(fallback["instructions"].as_str().expect("instructions")
        .contains("Mutating edits require user confirmation before execution."));
    assert!(fallback["tools"].as_array().expect("tools").iter()
        .all(|tool| tool["inputSchema"].is_object()), "fallback retains the raw MCP schemas");
    let (wire_tools, system_prompt) = camcraft_app::ai::wire::split_payload(fallback);
    assert!(wire_tools.is_empty(), "Prompt-JSON sends its tools in the system prompt");
    assert!(system_prompt.expect("fallback system prompt")
        .contains("Mutating edits require user confirmation before execution."));
}

#[test]
fn a_local_native_rename_changes_the_document_and_undo_turn_restores_it() {
    use super::super::super::ai_chat::ToolCardStatus;
    use camcraft_app::{mcp, model};
    use egui::accesskit::Role;
    use egui_kittest::kittest::Queryable;

    let mut h = blank_harness();
    for (name, arguments) in [
        ("set_stock", json!({"box":{"dx":60,"dy":40,"dz":10}})),
        ("add_tool", json!({"catalog":"EM-0400-2FL"})),
        (
            "add_operation",
            json!({"kind":"Face","tool":"EM-0400-2FL","name":"Face1"}),
        ),
    ] {
        let st = &mut h.state_mut().state;
        let request = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":arguments}});
        let answer = mcp::answer_live(&st.world, &mut st.history, None, &request.to_string());
        let reply: Value = serde_json::from_str(&answer.reply.expect("reply")).expect("JSON");
        assert_eq!(reply["result"]["isError"], false, "{reply}");
    }
    let label = |h: &egui_kittest::Harness<'_, super::super::ShellUi>| {
        let world = &h.state().state.world;
        let op = model::find_by_name(world, "Face1").expect("stable key remains");
        model::operation_label(world, "Face1", &op)
    };
    assert_eq!(label(&h), "Face1");
    let depth = h.state().state.history.done_len();
    h.run();
    toggle_ai_chat_via_view_door(&mut h);
    select_provider(&mut h, "Offline local (Ollama)");
    let transport = ScriptedTransport::new(vec![
        Ok(TransportOutput::AssistantResponse {
            text: String::new(),
            envelope: json!({"choices":[{"message":{"tool_calls":[{"id":"rename-1","type":"function",
                "function":{"name":"rename_operation","arguments":json!({"name":"Face1","new_name":"Roughing"}).to_string()}}]}}]}),
            fallback_override: None,
        }),
        Ok(TransportOutput::AssistantResponse {
            text: "Renamed.".into(),
            envelope: json!({"choices":[{"message":{"content":"Renamed."}}]}),
            fallback_override: None,
        }),
    ]);
    h.state_mut().state.ai_chat.set_transport(transport.clone());
    // Deterministic version of the benchmark task, not a live model rerun.
    submit_prompt(&mut h, "Rename Face1 to Roughing");
    pump_until_idle(&mut h);
    assert_eq!(label(&h), "Roughing");
    assert_eq!(h.state().state.history.done_len(), depth + 1);
    let turn = h.state().state.ai_chat.turns().last().expect("turn");
    assert_eq!(turn.tool_cards.len(), 1);
    assert_eq!(turn.tool_cards[0].status, ToolCardStatus::Succeeded);
    {
        let requests = transport.requests.lock().expect("requests");
        assert_eq!(requests.len(), 2);
        let tools = requests[0].tools_payload["tools"]
            .as_array()
            .expect("native tools");
        assert!(
            tools
                .iter()
                .any(|t| t["function"]["name"] == "rename_operation"),
            "rename must be offered"
        );
        let result = requests[1]
            .messages
            .last()
            .expect("tool reply")
            .raw
            .as_ref()
            .expect("raw");
        assert_eq!(result["tool_call_id"], "rename-1");
        assert!(
            result["content"]
                .as_str()
                .expect("result")
                .contains("Roughing")
        );
    }
    h.get_by_role_and_label(Role::Button, "Undo Turn").click();
    h.run_steps(2);
    assert_eq!(label(&h), "Face1");
    assert_eq!(h.state().state.history.done_len(), depth);
    assert!(h.state().state.ai_chat.turns().last().expect("turn").undone);
}

#[test]
fn a_turn_stops_at_the_round_cap() {
    let mut h = blank_harness();
    h.run();
    toggle_ai_chat_via_view_door(&mut h);
    let rounds: Vec<_> = (0..budget::MAX_TOOL_ROUNDS + 2).map(|i| tool_use(&format!("r{i}"), "get_selection")).collect();
    let transport = ScriptedTransport::new(rounds);
    h.state_mut().state.ai_chat.set_transport(transport.clone());
    submit_prompt(&mut h, "Loop forever");
    pump_until_idle(&mut h);

    assert_eq!(transport.calls.load(Ordering::Relaxed), budget::MAX_TOOL_ROUNDS);
    let turn = h.state().state.ai_chat.turns().last().expect("turn");
    assert!(
        matches!(&turn.status, TurnStatus::Failed(e) if e.contains("tool steps")),
        "{:?}",
        turn.status
    );
}
