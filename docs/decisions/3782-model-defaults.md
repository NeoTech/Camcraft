## #3782 - Select installed local models and maintain cloud defaults

**What was decided.** An empty local model field selects an installed model automatically. The transport reads `GET /api/tags`. It checks each candidate with `POST /api/show` and selects the first model with the `tools` capability. All local presets use this selection rule. Each preset keeps its existing fallback mode. A model name that the user enters stays explicit.

If no installed model supports tools, the diagnostic gives `ollama pull ornith-1.5:35b` and links to <https://ollama.com/library>.

The selection has a time limit: 5 s for each request and 20 s for the whole selection. A failed probe of one model does not stop the selection; the next model is checked. Each cause has its own message: the server did not answer its model list, no installed model can call tools, the probe failed for a named model, or the time limit ended. A chat failure after an automatic selection names the selected model.

An empty cloud Model field displays the provider default and uses that default for a request. The automatic-selection hint applies only to local providers. Cloud defaults live in one table in the app layer. The defaults are `claude-sonnet-5-5`, `gpt-6.2-sol` for both OpenAI endpoints, and `gemini-4-hydrogen`. These IDs were checked against the official model pages on 2026-11-24:

- <https://platform.claude.com/docs/en/models/overview>
- <https://developers.openai.com/api/docs/models/gpt-6.2-sol>
- <https://aistudio.google.com/docs/models/gemini-4-hydrogen>

Each row in the shared `CLOUD_DEFAULTS` table includes its retirement status. An offline test rejects a default when its row marks it as retired. The table and its status fields need maintenance when providers change model availability.

**Why.** The reporter found that an empty cloud Model showed the local hint and failed a request. The shared table supplies the displayed and effective cloud default. The install suggestion uses `ornith-1.5:35b` because the reporter's 2026-11-22 benchmark scored it 16/20 CamCraft tasks. The previous `qwen3:8b` suggestion scored 9/20. An installed model can work on the first Send without a model name that the user must enter. The capability probe excludes models that cannot call tools. One table prevents separate defaults from drifting between callers. Recorded responses keep the tests independent of installed models, provider accounts, and network access.

**What was dropped.** The first version stopped the selection at the first failed probe and used the 300 s chat time limit for each probe. The reporter's machine has about 30 installed models, so one damaged model or a slow server could block each Send. A display of the selected model after a successful Send was dropped from this PR: the panel status line is the work of #3798. The local automatic-selection hint was dropped for cloud providers because cloud requests use the table default. The `qwen3:8b` suggestion was dropped because the reporter's benchmark showed unreliable project changes. The app does not download a model automatically. A download can consume substantial disk space and bandwidth. The app does not silently replace an explicit model name. That would hide the user's choice. A fixed local model name was dropped because the selected machine might not have that model. Live cloud checks were dropped because they require network access and can require API keys.

**Cost to change later.** A different automatic selection policy changes the candidate selection and its recorded-response tests. Provider changes require an update to the shared defaults table or its retirement status fields. A provider wire-format change can also require transport work. These changes do not require a project format migration.

### PR #3799 review: enforce the total discovery time limit

**What was decided.** The model list request and each model probe use the remaining total discovery time. Each request also keeps its 5 s maximum.

**Why.** A check before each probe lets the last request exceed the total time limit. The request timeout must also use the remaining time.

**What was dropped.** A fixed 5 s request timeout without the remaining total time was dropped. It can exceed the stated 20 s selection limit.

**Cost to change later.** A different time limit requires an update to the discovery timeout policy and its slow-response tests.
