//! Tells you when Drift needs you or has finished: a notice in the app, and optionally a webhook
//! (Slack, Discord, anything that takes JSON) so you hear about it away from the screen.

wit_bindgen::generate!({ world: "plugin", path: "../../wit" });

use drift::plugin::host::{config, log, Level};
use drift::plugin::http::fetch;
use drift::plugin::notify::{show, Tone};
use serde_json::Value;

struct Notify;

fn settings() -> Value {
    serde_json::from_str(&config()).unwrap_or(Value::Null)
}

fn enabled(setting: &str, default: bool) -> bool {
    settings()[setting].as_bool().unwrap_or(default)
}

fn post(text: &str) {
    let settings = settings();
    let Some(url) = settings["webhook"].as_str().filter(|url| !url.is_empty()) else { return };
    // Slack and Discord both read a `text` or `content` field; sending both satisfies either.
    let body = serde_json::json!({ "text": text, "content": text }).to_string();
    if let Err(error) = fetch("POST", url, &[("content-type".to_owned(), "application/json".to_owned())], Some(&body)) {
        log(Level::Warn, &format!("webhook failed: {error}"));
    }
}

fn title(session: &Session) -> String {
    if session.title.trim().is_empty() { "Untitled thread".to_owned() } else { session.title.clone() }
}

impl Guest for Notify {
    fn name() -> String {
        "notify".into()
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

    fn turn_end(reply: Reply) -> TurnEnd {
        if enabled("onReply", true) {
            let first = reply.text.lines().find(|line| !line.trim().is_empty()).unwrap_or("Reply ready").trim();
            let line: String = first.chars().take(140).collect();
            post(&format!("Drift finished a turn ({}): {line}", reply.agent));
        }
        TurnEnd::Accept
    }

    fn permission(ask: PermissionAsk) -> Permission {
        if enabled("onPermission", true) {
            show("Waiting for your approval", &ask.title, Tone::Info);
            post(&format!("Drift is waiting for approval: {}", ask.title));
        }
        Permission::Pass
    }

    fn compaction(_compaction: Compaction) -> Compacting {
        Compacting::Proceed
    }

    fn session(session: Session, kind: SessionKind) {
        if kind == SessionKind::Idle && enabled("onIdle", false) {
            show("Thread finished", &title(&session), Tone::Success);
        }
    }
}

export!(Notify);
