//! Puts the repository's state beside each prompt: the branch, what is changed, and the last few
//! commits, so the model knows where it is without running git first.

wit_bindgen::generate!({ world: "plugin", path: "../../wit" });

use drift::plugin::host::config;
use drift::plugin::process::run;
use serde_json::Value;

struct Context;

const TIMEOUT_MS: u32 = 10_000;
const MAX_STATUS_LINES: usize = 30;

fn settings() -> Value {
    serde_json::from_str(&config()).unwrap_or(Value::Null)
}

fn enabled(setting: &str) -> bool {
    settings()[setting].as_bool().unwrap_or(true)
}

fn git(args: &[&str]) -> Option<String> {
    let owned: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    let output = run("git", &owned, TIMEOUT_MS).ok()?;
    (output.code == 0).then(|| output.stdout.trim_end().to_owned())
}

impl Guest for Context {
    fn name() -> String {
        "git-context".into()
    }

    fn before_tool(_call: ToolCall) -> BeforeTool {
        BeforeTool::Allow
    }

    fn after_tool(_outcome: ToolResult) -> AfterTool {
        AfterTool::Keep
    }

    fn prompt_submit(_prompt: Prompt) -> PromptSubmit {
        let Some(branch) = git(&["branch", "--show-current"]) else { return PromptSubmit::Keep };
        let mut lines = vec![format!("Branch: {}", if branch.is_empty() { "detached HEAD".to_owned() } else { branch })];
        if enabled("status") {
            let status = git(&["status", "--short"]).unwrap_or_default();
            let changed: Vec<&str> = status.lines().collect();
            if changed.is_empty() {
                lines.push("Working tree: clean".into());
            } else {
                lines.push(format!("Changed files ({}):", changed.len()));
                lines.extend(changed.iter().take(MAX_STATUS_LINES).map(|line| format!("  {line}")));
                if changed.len() > MAX_STATUS_LINES {
                    lines.push(format!("  ... {} more", changed.len() - MAX_STATUS_LINES));
                }
            }
        }
        if enabled("log") {
            if let Some(log) = git(&["log", "--oneline", "-5"]) {
                lines.push("Recent commits:".into());
                lines.extend(log.lines().map(|line| format!("  {line}")));
            }
        }
        PromptSubmit::AddContext(lines.join("\n"))
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

export!(Context);
