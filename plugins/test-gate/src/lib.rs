//! Runs the test suite before a turn that changed files is allowed to end. Failures go back to
//! the model as a prompt, so it keeps working until they pass.

wit_bindgen::generate!({ world: "plugin", path: "../../wit" });

use drift::plugin::host::{config, log, Level};
use drift::plugin::notify::{show, Tone};
use drift::plugin::process::run;
use drift::plugin::store::{get, set};
use serde_json::Value;

struct Gate;

const WRITE_TOOLS: &[&str] = &["edit", "write", "apply_patch"];
const TIMEOUT_MS: u32 = 300_000;

fn settings() -> Value {
    serde_json::from_str(&config()).unwrap_or(Value::Null)
}

fn command() -> Vec<String> {
    settings()["test"].as_array().map(|items| items.iter().filter_map(|item| item.as_str().map(str::to_owned)).collect()).unwrap_or_default()
}

fn dirty_key(session: &str) -> String {
    format!("dirty:{session}")
}

fn tail(stderr: &str, stdout: &str) -> String {
    let text = if stderr.trim().is_empty() { stdout } else { stderr };
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(60)..].join("\n")
}

impl Guest for Gate {
    fn name() -> String {
        "test-gate".into()
    }

    fn before_tool(_call: ToolCall) -> BeforeTool {
        BeforeTool::Allow
    }

    fn after_tool(outcome: ToolResult) -> AfterTool {
        if !outcome.failed && WRITE_TOOLS.contains(&outcome.tool.as_str()) {
            set(&dirty_key(&outcome.session_id), "1");
        }
        AfterTool::Keep
    }

    fn prompt_submit(_prompt: Prompt) -> PromptSubmit {
        PromptSubmit::Keep
    }

    fn turn_end(reply: Reply) -> TurnEnd {
        let key = dirty_key(&reply.session_id);
        if get(&key).is_none_or(|value| value.is_empty()) {
            return TurnEnd::Accept;
        }
        let command = command();
        let Some((program, args)) = command.split_first() else {
            log(Level::Warn, "no test command configured; set config.test to a program and its arguments");
            set(&key, "");
            return TurnEnd::Accept;
        };
        let timeout = settings()["timeoutMs"].as_u64().map(|ms| ms as u32).unwrap_or(TIMEOUT_MS);
        match run(program, args, timeout) {
            Ok(output) if output.code == 0 => {
                set(&key, "");
                TurnEnd::Note(format!("{} passed", command.join(" ")))
            }
            Ok(output) => {
                show("Tests failed", &format!("{} exited with {}; the agent is fixing them", program, output.code), Tone::Warning);
                TurnEnd::Continue(format!("Files changed this turn and `{}` failed with exit code {}. Fix the failures before finishing.\n\n{}", command.join(" "), output.code, tail(&output.stderr, &output.stdout)))
            }
            Err(error) => {
                set(&key, "");
                TurnEnd::Note(format!("tests could not run: {error}"))
            }
        }
    }

    fn permission(_ask: PermissionAsk) -> Permission {
        Permission::Pass
    }

    fn compaction(_compaction: Compaction) -> Compacting {
        Compacting::Proceed
    }

    fn session(session: Session, kind: SessionKind) {
        if kind == SessionKind::Deleted {
            set(&dirty_key(&session.id), "");
        }
    }
}

export!(Gate);
