//! Refuses shell commands that rewrite history, notes failed commands, and when a reply says
//! `@guard test` runs the configured test command, keeping the turn going if it fails.

wit_bindgen::generate!({ world: "plugin", path: "../../wit" });

use drift::plugin::host::{config, log, Level};
use drift::plugin::notify::{show, Tone};
use drift::plugin::process::run;
use serde_json::Value;

struct Guard;

const DEFAULT_DENY: &[&str] = &["git push --force", "git push -f", "git reset --hard", "git clean -f", "git checkout --", "git branch -D"];
const TRIGGER: &str = "@guard test";
const TEST_TIMEOUT_MS: u32 = 300_000;

fn settings() -> Value {
    serde_json::from_str(&config()).unwrap_or(Value::Null)
}

fn strings(value: &Value) -> Vec<String> {
    value.as_array().map(|items| items.iter().filter_map(|item| item.as_str().map(str::to_owned)).collect()).unwrap_or_default()
}

impl Guest for Guard {
    fn name() -> String {
        "guard".into()
    }

    fn before_tool(call: ToolCall) -> BeforeTool {
        if call.tool != "bash" {
            return BeforeTool::Allow;
        }
        let input: Value = serde_json::from_str(&call.input).unwrap_or(Value::Null);
        let command = input["command"].as_str().unwrap_or_default();
        let settings = settings();
        let denied = if settings["deny"].is_array() { strings(&settings["deny"]) } else { DEFAULT_DENY.iter().map(|s| s.to_string()).collect() };
        match denied.iter().find(|refused| command.contains(refused.as_str())) {
            Some(refused) => BeforeTool::Deny(format!("`{refused}` rewrites history; ask the user to run it")),
            None => BeforeTool::Allow,
        }
    }

    fn after_tool(outcome: ToolResult) -> AfterTool {
        if outcome.tool == "bash" && outcome.failed {
            return AfterTool::Note("saw this command fail".into());
        }
        AfterTool::Keep
    }

    fn prompt_submit(_prompt: Prompt) -> PromptSubmit {
        PromptSubmit::Keep
    }

    fn turn_end(reply: Reply) -> TurnEnd {
        if !reply.text.contains(TRIGGER) {
            return TurnEnd::Accept;
        }
        let command = strings(&settings()["test"]);
        let Some((program, args)) = command.split_first() else {
            log(Level::Warn, "no test command configured; set config.test to a program and its arguments");
            return TurnEnd::Accept;
        };
        let timeout = settings()["timeoutMs"].as_u64().map(|ms| ms as u32).unwrap_or(TEST_TIMEOUT_MS);
        match run(program, args, timeout) {
            Ok(output) if output.code == 0 => TurnEnd::Note("tests passed".into()),
            Ok(output) => {
                show("Tests failed", &format!("exit code {}; the agent is fixing them", output.code), Tone::Warning);
                TurnEnd::Continue(format!("The tests failed with exit code {}. Fix them before finishing.\n\n{}", output.code, tail(&output.stderr, &output.stdout)))
            }
            Err(error) => TurnEnd::Continue(format!("The tests could not run: {error}")),
        }
    }

    fn permission(_ask: PermissionAsk) -> Permission {
        Permission::Pass
    }

    fn compaction(_compaction: Compaction) -> Compacting {
        Compacting::Instruct("Keep the list of commands the guard plugin refused in this conversation, if any.".into())
    }

    fn session(_session: Session, _kind: SessionKind) {}
}

/// The last lines of a run's output: where a test runner says what failed.
fn tail(stderr: &str, stdout: &str) -> String {
    let text = if stderr.trim().is_empty() { stdout } else { stderr };
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(40)..].join("\n")
}

export!(Guard);
