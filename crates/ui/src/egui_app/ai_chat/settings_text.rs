//! Provenance: PR #3798 reporter hand-test; HTTP status meanings from RFC 9110 and provider API docs.

/// The status of a cloud provider without a key. Test stays disabled while it shows.
pub(crate) const NO_KEY: &str = "No key - Set API key...";

/// The Ollama server address that the panel uses (no endpoint field yet).
pub(crate) const OLLAMA_SERVER: &str = "http://localhost:11434";

/// The model that Test and the status check use: the Model field like Send reads it.
/// Spaces are trimmed. An empty field uses the provider default.
#[must_use]
pub(crate) fn resolve_model(input: &str, default: &str) -> String {
    let typed = input.trim();
    if typed.is_empty() {
        default.trim().to_owned()
    } else {
        typed.to_owned()
    }
}

/// The HTTP status in a transport error (`<host> answered HTTP <code>: ...`).
fn http_status(error: &str) -> Option<u16> {
    let rest = error.split("answered HTTP ").nth(1)?;
    rest.get(..3)?.parse().ok()
}

/// One cause and one "what to try" hint for a failed cloud Test.
/// `error` is the transport error; `http.rs` removed the key from it before and
/// kept the provider's message, type, code and request endpoint in it.
/// `endpoint` is the provider base URL (scheme, host and path).
#[must_use]
pub(crate) fn test_failure(error: &str, model: &str, endpoint: &str) -> String {
    let lower = error.to_ascii_lowercase();
    let billing = lower.contains("credit") || lower.contains("billing");
    let status = http_status(error);
    let (cause, hint) = match status {
        Some(401 | 403) => (
            "the provider rejected the key",
            "Click Set API key... and paste a new key for this provider.",
        ),
        Some(402) => (
            "billing or credits",
            "Check the credits and the spending limits on the provider's billing page.",
        ),
        Some(_) if billing => (
            "billing or credits",
            "Check the credits and the spending limits on the provider's billing page.",
        ),
        Some(404) => (
            "the provider does not know this model",
            "Check the Model name on the provider's model list. Clear Model to use the default.",
        ),
        Some(400 | 422) => (
            "the provider refused the request or the model name",
            "Check the Model name. Clear Model to use the default.",
        ),
        Some(429) => (
            "the rate limit or the quota",
            "Wait one minute, then click Test again. Check the usage limits.",
        ),
        Some(500..=599) => (
            "the provider is not available now",
            "Wait some minutes, then click Test again.",
        ),
        Some(_) => (
            "the provider refused the request",
            "Check the key, the Model name and the billing.",
        ),
        None if lower.contains("not a valid model name") => (
            "the Model name is not valid",
            "Type a model name with letters, digits, '.', '-', '_', ':' or '/' only.",
        ),
        None if lower.contains("could not be reached") => (
            "the network did not reach the provider",
            "Check the network connection, a firewall or a proxy.",
        ),
        None => (
            "the provider gave no usable answer",
            "Check the network connection, then click Test again.",
        ),
    };
    let code = status.map_or_else(String::new, |code| format!(" (HTTP {code})"));
    let text = format!(
        "Test failed: {cause}{code}.\nModel tried: {model} at {endpoint}.\nTry: {hint}\nDetails: {error}"
    );
    tracing::warn!(target: "camcraft::ai_settings", "{text}");
    text
}

/// The Ollama status for a probe result. `test` marks a Test click, so the result
/// always names the outcome and differs from a background check.
pub(crate) fn ollama_status(
    probe: Result<(), String>,
    model: &str,
    test: bool,
) -> Result<String, String> {
    let ready = |text: String| {
        if test {
            format!("{text} - Test passed")
        } else {
            text
        }
    };
    let failed = |text: String| {
        let text = if test {
            format!("Test failed: {text}")
        } else {
            text
        };
        tracing::warn!(target: "camcraft::ai_settings", "{text}");
        text
    };
    match probe {
        Ok(()) if model.is_empty() => Ok(ready(
            "Ollama is running. Type an installed model name in Model (see ollama list).".into(),
        )),
        Ok(()) => Ok(ready(format!("Ready (local model {model})"))),
        Err(e) if e.contains("could not be reached") => Err(failed(format!(
            "Ollama is not running at {OLLAMA_SERVER}. CamCraft does not start it. \
             Start the Ollama app or run ollama serve. CamCraft checks again every 5 s."
        ))),
        Err(e) if model.is_empty() && (e.contains("HTTP 404") || e.contains("not found")) => {
            Ok(ready(
                "Ollama is running. Type an installed model name in Model (see ollama list)."
                    .into(),
            ))
        }
        Err(e) if e.contains("HTTP 404") || e.contains("not found") => Err(failed(format!(
            "model {model} is not installed. Run ollama pull {model}, or type a name from ollama list."
        ))),
        Err(e) if e.contains("not a valid model name") => Err(failed(format!(
            "the Model name '{model}' is not valid. Type a name from ollama list."
        ))),
        Err(e) => Err(failed(format!(
            "the Ollama check failed for model {model}. Details: {e}"
        ))),
    }
}
