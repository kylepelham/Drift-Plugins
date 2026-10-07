//! Commits after every file the model writes, so each change is its own revertible step and
//! nothing is lost if the turn goes wrong. Commits are made in the workspace's repository.

wit_bindgen::generate!({ world: "plugin", path: "../../wit" });

use drift::plugin::host::config;
use drift::plugin::process::run;
use serde_json::Value;

struct Commit;

const WRITE_TOOLS: &[&str] = &["edit", "write", "apply_patch"];
const TIMEOUT_MS: u32 = 30_000;

fn settings() -> Value {
    serde_json::from_str(&config()).unwrap_or(Value::Null)
}

fn written(input: &Value) -> Vec<String> {
    let direct = ["path", "filePath", "file_path"].iter().filter_map(|key| input[*key].as_str().map(str::to_owned));
    let patched = input["patch"].as_str().into_iter().flat_map(|patch| {
        patch.lines().filter_map(|line| line.strip_prefix("*** Update File: ").or_else(|| line.strip_prefix("*** Add File: ")).or_else(|| line.strip_prefix("*** Delete File: "))).map(|path| path.trim().to_owned())
    });
    direct.chain(patched).collect()
}

fn git(args: &[&str]) -> Result<drift::plugin::process::Output, String> {
    let owned: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    run("git", &owned, TIMEOUT_MS)
}

impl Guest for Commit {
    fn name() -> String {
        "git-autocommit".into()
    }

    fn before_tool(_call: ToolCall) -> BeforeTool {
        BeforeTool::Allow
    }

    fn after_tool(outcome: ToolResult) -> AfterTool {
        if outcome.failed || !WRITE_TOOLS.contains(&outcome.tool.as_str()) {
            return AfterTool::Keep;
        }
        let input: Value = serde_json::from_str(&outcome.input).unwrap_or(Value::Null);
        let files = written(&input);
        if files.is_empty() {
            return AfterTool::Keep;
        }
        let settings = settings();
        let prefix = settings["prefix"].as_str().unwrap_or("drift:");
        let stage_all = settings["stageAll"].as_bool().unwrap_or(false);
        let staged = if stage_all {
            git(&["add", "-A"])
        } else {
            let mut args = vec!["add", "--"];
            args.extend(files.iter().map(String::as_str));
            git(&args)
        };
        if !staged.is_ok_and(|output| output.code == 0) {
            return AfterTool::Keep;
        }
        if git(&["diff", "--cached", "--quiet"]).is_ok_and(|output| output.code == 0) {
            return AfterTool::Keep;
        }
        let names: Vec<&str> = files.iter().map(|file| file.rsplit(['/', '\\']).next().unwrap_or(file)).collect();
        let message = format!("{prefix} {} {}", outcome.tool, names.join(", "));
        match git(&["commit", "-q", "-m", &message]) {
            Ok(output) if output.code == 0 => AfterTool::Note(format!("committed: {message}")),
            Ok(output) => AfterTool::Note(format!("commit failed: {}", output.stderr.lines().next().unwrap_or("unknown reason"))),
            Err(error) => AfterTool::Note(format!("commit failed: {error}")),
        }
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

export!(Commit);
