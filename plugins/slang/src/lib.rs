//! Adds the Slang shading language: Khronos' grammar from the Slang VS Code extension, which highlights
//! `.slang` and `.slangh` files and `slang` code fences. Slang's language server, `slangd`, is one of Drift's
//! own, started like any other when it is installed, so this plugin declares none and has no settings.

wit_bindgen::generate!({ world: "languages", path: "../../wit" });

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
        Vec::new()
    }

    fn language_servers() -> Vec<LanguageServer> {
        Vec::new()
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
