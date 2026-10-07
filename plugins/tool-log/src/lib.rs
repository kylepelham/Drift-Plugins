//! Keeps a record of every tool call: one JSON line per call in a file under the workspace, and
//! optionally posted to an endpoint, for audit or for watching what the agent does.

wit_bindgen::generate!({ world: "plugin", path: "../../wit" });

use drift::plugin::files::{read, write};
use drift::plugin::host::{config, log, Level};
use drift::plugin::http::fetch;
use serde_json::Value;

struct Log;

const DEFAULT_FILE: &str = ".drift/tool-log.jsonl";
const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
const MAX_OUTPUT_CHARS: usize = 2000;

fn settings() -> Value {
    serde_json::from_str(&config()).unwrap_or(Value::Null)
}

fn clipped(text: &str) -> String {
    if text.chars().count() <= MAX_OUTPUT_CHARS { text.to_owned() } else { format!("{}...", text.chars().take(MAX_OUTPUT_CHARS).collect::<String>()) }
}

fn record(outcome: &ToolResult) -> String {
    let input: Value = serde_json::from_str(&outcome.input).unwrap_or(Value::Null);
    serde_json::json!({
        "session": outcome.session_id,
        "agent": outcome.agent,
        "tool": outcome.tool,
        "input": input,
        "failed": outcome.failed,
        "output": clipped(&outcome.output),
    })
    .to_string()
}

/// Appends a line; when the file outgrows its limit the oldest half goes, so it never grows without bound.
fn append(path: &str, line: &str) {
    let mut text = read(path).unwrap_or_default();
    if text.len() + line.len() > MAX_FILE_BYTES {
        let keep_from = text.len() / 2;
        let cut = text[keep_from..].find('\n').map(|at| keep_from + at + 1).unwrap_or(text.len());
        text = text[cut..].to_owned();
    }
    text.push_str(line);
    text.push('\n');
    if let Err(error) = write(path, &text) {
        log(Level::Warn, &format!("could not write {path}: {error}"));
    }
}

impl Guest for Log {
    fn name() -> String {
        "tool-log".into()
    }

    fn before_tool(_call: ToolCall) -> BeforeTool {
        BeforeTool::Allow
    }

    fn after_tool(outcome: ToolResult) -> AfterTool {
        let settings = settings();
        let line = record(&outcome);
        if settings["file"].as_bool() != Some(false) {
            let path = settings["file"].as_str().unwrap_or(DEFAULT_FILE);
            append(path, &line);
        }
        if let Some(url) = settings["endpoint"].as_str().filter(|url| !url.is_empty()) {
            if let Err(error) = fetch("POST", url, &[("content-type".to_owned(), "application/json".to_owned())], Some(&line)) {
                log(Level::Warn, &format!("endpoint failed: {error}"));
            }
        }
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

export!(Log);
