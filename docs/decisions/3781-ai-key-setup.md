## #3781 - Set up AI provider keys inside the app

**What was decided:** Add a password entry field, save, remove, and test controls to AI Chat. Use the existing app keychain functions and the existing environment-first lookup. Saving replaces the provider's stored key. Remove deletes only the keychain entry. Clear the entry field after save, cancel, and provider changes. Do not return a stored secret to the UI.

Run credential operations and readiness checks in background work. Show the key source as soon as the provider is selected. A ready status means that a key was found. An explicit Test sends a small request to the selected model and can incur a provider charge. Check local Ollama readiness and model availability as well.

Explain API keys, provider setup, billing, environment precedence, and OS permission prompts beside the controls and in the user guide. Keep the PR a draft until the reporter confirms the macOS hand-test.

**Why:** A desktop user must be able to set up a cloud provider without Terminal or an app restart. Reusing the existing storage preserves its service and account names. Background operations keep keychain prompts and network requests from blocking the UI. Distinguishing a found key from a successful test avoids a false claim about account access or credit.

**Considered and dropped:** Environment-only setup does not work well for an app started from the Dock. Storing a key in preferences or the project would expose it. Displaying a saved key would create an unnecessary disclosure. Sending cloud requests automatically on provider selection could cause charges without an explicit action. A separate credential store would duplicate the existing platform support.

**Cost to change later:** Provider instructions and links can change without a storage migration. Changing the keychain service or account names requires a migration for existing keys. Changing lookup precedence requires updates to the transport, readiness checks, tests, and guide.

**Review fix scope:** Add the missing Ollama server commands and environment options to the panel and guide. Explain that a Gemini app subscription does not include an API key. Keep the current endpoint limit explicit. Track the live-ai smoke lane in #3803 and the distinct key-rejected status in #3804. Change this PR to reference #3781 because those requirements remain open.

**Why:** The review permits a bounded fix with explicit follow-ups. The live lane needs real provider credentials and fixture tool calls. The rejected-key status needs provider error classification. Neither belongs in the Model edit scheduling fix.

**Considered and dropped:** Do not claim that the full issue is complete or ask for the reporter's hand-test before the live lane runs. Do not add endpoint controls in this fix; #3783 tracks them.

**Cost to change later:** The follow-up work can reuse the existing credential backend and provider transports. Its tests must preserve secret redaction.

**Model edit scheduling:** Use a cloud readiness identity that excludes the model text. For Ollama, wait for a 500 ms pause in Model edits before checking the model.

**Why:** A cloud readiness check reads the credential but does not use the model. Repeating it per character can show repeated OS keychain prompts. Ollama needs the model name, but a short delay avoids an `/api/show` request for each character.

**Considered and dropped:** Per-character validation repeats credential reads and server requests. Checking only when the field loses focus delays feedback while the user stays in the field. A short pause gives feedback without requiring a focus change.

**Cost to change later:** The delay can change without a storage migration. Backend-call-count tests pin cloud reuse and one Ollama check after the edit pause.

**Reporter hand-test fixes (2026-11-24):** Test and the status check read the Model field as Send does: trim the spaces, and use the preset default when the field is empty. Disable Test while no key is found. A failed cloud Test names a cause from the HTTP status, the model and host that were tried, one "what to try" hint, and the provider's error text; the app log gets the same text. Test shows "Testing..." and a passed or failed result for Ollama too. A failed Ollama check runs again every 5 s, and an empty Ollama Model still tells if the server answers. The panel says that CamCraft does not start Ollama. The Anthropic help names the console choice "Somewhere else - Use an API key".

**Why:** The reporter could not make an empty cloud Model work, a leading space failed Test, and a failed Test gave no cause. A status from before `ollama serve` stayed on the screen. The provider's error text is safe to show because `http.rs` removes the key value from it before it returns, for each provider's auth header.

**Considered and dropped:** Keep the generic "Test failed" text: it gave the user nothing to act on. Re-check Ollama on each frame: it sends a request per frame. Start Ollama from CamCraft: it is a separate feature. Show the default model as the Model hint here: PR #3799 changes the same line, so this PR only resolves the default and names it in the Test result.

**Cost to change later:** The cause table is one function (`settings_text::test_failure`) with a unit test for each class.
