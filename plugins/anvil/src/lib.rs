//! Adds Anvil's manifests: Anvil's own grammar highlights `anvil.toml` and `tests.toml` and `anvil` code fences,
//! and `anvil lsp` checks each edit to them as a build would, so the model sees an unknown key or target at once.

wit_bindgen::generate!({ world: "languages", path: "../../wit" });

use drift::plugin::types::ServerSetup;

const FILES: [&str; 2] = ["anvil.toml", "tests.toml"];

struct Anvil;

fn files() -> Vec<String> {
    FILES.map(String::from).to_vec()
}

impl Guest for Anvil {
    fn name() -> String {
        "anvil".into()
    }

    fn settings() -> Vec<Setting> {
        Vec::new()
    }

    fn language_servers() -> Vec<LanguageServer> {
        vec![LanguageServer {
            id: "anvil".into(),
            command: vec!["anvil".into(), "lsp".into()],
            extensions: files(),
            language: Some("toml".into()),
            label: "Anvil".into(),
            setup: ServerSetup::None(
                "Anvil's own `anvil` program serves it: build Anvil with `cargo install --path .` in its folder, which \
                 puts `anvil` on PATH."
                    .into(),
            ),
        }]
    }

    fn grammars() -> Vec<Grammar> {
        vec![Grammar {
            name: "anvil".into(),
            aliases: vec!["anvil-toml".into()],
            extensions: files(),
            json: include_str!("../anvil.tmLanguage.json").into(),
        }]
    }

    fn before_tool(_call: ToolCall) -> BeforeTool {
        BeforeTool::Allow
    }

    fn after_tool(_outcome: ToolResult) -> AfterTool {
        AfterTool::Keep
    }

    fn prompt_submit(_prompt: Prompt) -> PromptSubmit {
        PromptSubmit::Keep
    }

    fn turn_end(_reply: Reply) -> TurnEnd {
        TurnEnd::Accept
    }

    fn permission(_ask: PermissionAsk) -> Permission {
        Permission::Pass
    }

    fn compaction(_compaction: Compaction) -> Compacting {
        Compacting::Proceed
    }

    fn session(_session: Session, _kind: SessionKind) {}
}

export!(Anvil);
