//! Adds the Slang shading language: Khronos' grammar from the Slang VS Code extension, which highlights
//! `.slang` and `.slangh` files and `slang` code fences, and Slang's language server, so an edit to a
//! Slang file comes back with the errors it left. The server is `slangd`, which the Slang releases and
//! the Vulkan SDK ship, unless `config.server` names another command; an empty one turns it off.

wit_bindgen::generate!({ world: "languages", path: "../../wit" });

use drift::plugin::host::config;
use drift::plugin::types::{SettingKind, TextSetting};

const SERVER: &str = "slangd";
const EXTENSIONS: [&str; 2] = [".slang", ".slangh"];

struct Slang;

fn extensions() -> Vec<String> {
    EXTENSIONS.map(String::from).to_vec()
}

impl Guest for Slang {
    fn name() -> String {
        "slang".into()
    }

    fn settings() -> Vec<Setting> {
        vec![Setting {
            key: "server".into(),
            label: "Language server".into(),
            description: "The command that starts Slang's language server, with its arguments; empty turns it off."
                .into(),
            kind: SettingKind::Text(TextSetting {
                default: SERVER.into(),
                placeholder: SERVER.into(),
                multiline: false,
                secret: false,
            }),
        }]
    }

    fn language_servers() -> Vec<LanguageServer> {
        let config: serde_json::Value = serde_json::from_str(&config()).unwrap_or_default();
        let command: Vec<String> = config["server"]
            .as_str()
            .unwrap_or(SERVER)
            .split_whitespace()
            .map(String::from)
            .collect();
        if command.is_empty() {
            return Vec::new();
        }

        vec![LanguageServer {
            id: "slang".into(),
            command,
            extensions: extensions(),
            language: Some("slang".into()),
        }]
    }

    fn grammars() -> Vec<Grammar> {
        vec![Grammar {
            name: "slang".into(),
            aliases: Vec::new(),
            extensions: extensions(),
            json: include_str!("../slang.tmLanguage.json").into(),
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

export!(Slang);
