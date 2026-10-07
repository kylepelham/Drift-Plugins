//! Runs a formatter over each file the model writes, chosen by extension. Drift formats with the
//! project's own tools already; this covers formatters it does not know, or a house style.

wit_bindgen::generate!({ world: "plugin", path: "../../wit" });

use drift::plugin::host::config;
use drift::plugin::process::run;
use serde_json::Value;

struct Format;

const WRITE_TOOLS: &[&str] = &["edit", "write", "apply_patch"];
const TIMEOUT_MS: u32 = 30_000;

/// Formatters by extension, as `{ "ts,tsx,js": ["prettier", "--write", "$FILE"] }`.
fn defaults() -> Value {
    serde_json::json!({
        "js,jsx,ts,tsx,json,css,md,yaml,yml": ["prettier", "--write", "$FILE"],
        "rs": ["rustfmt", "--edition", "2021", "$FILE"],
        "py": ["black", "-q", "$FILE"],
        "go": ["gofmt", "-w", "$FILE"],
    })
}

fn formatter_for(path: &str) -> Option<Vec<String>> {
    let settings: Value = serde_json::from_str(&config()).unwrap_or(Value::Null);
    let table = if settings["formatters"].is_object() { settings["formatters"].clone() } else { defaults() };
    let extension = path.rsplit('.').next()?.to_ascii_lowercase();
    let entry = table.as_object()?.iter().find(|(key, _)| key.split(',').any(|ext| ext.trim().eq_ignore_ascii_case(&extension)))?;
    let command: Vec<String> = entry.1.as_array()?.iter().filter_map(|item| item.as_str()).map(|arg| arg.replace("$FILE", path)).collect();
    (!command.is_empty()).then_some(command)
}

fn written(input: &Value) -> Vec<String> {
    let direct = ["path", "filePath", "file_path"].iter().filter_map(|key| input[*key].as_str().map(str::to_owned));
    let patched = input["patch"].as_str().into_iter().flat_map(|patch| patch.lines().filter_map(|line| line.strip_prefix("*** Update File: ").or_else(|| line.strip_prefix("*** Add File: "))).map(|path| path.trim().to_owned()));
    direct.chain(patched).collect()
}

impl Guest for Format {
    fn name() -> String {
        "auto-format".into()
    }

    fn before_tool(_call: ToolCall) -> BeforeTool {
        BeforeTool::Allow
    }

    fn after_tool(outcome: ToolResult) -> AfterTool {
        if outcome.failed || !WRITE_TOOLS.contains(&outcome.tool.as_str()) {
            return AfterTool::Keep;
        }
        let input: Value = serde_json::from_str(&outcome.input).unwrap_or(Value::Null);
        let mut formatted = Vec::new();
        for path in written(&input) {
            let Some(command) = formatter_for(&path) else { continue };
            let (program, args) = command.split_first().unwrap();
            if run(program, args, TIMEOUT_MS).is_ok_and(|output| output.code == 0) {
                formatted.push(format!("{path} with {program}"));
            }
        }
        if formatted.is_empty() { AfterTool::Keep } else { AfterTool::Note(format!("formatted {}", formatted.join(", "))) }
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

export!(Format);
