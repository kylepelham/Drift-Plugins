//! Runs a check (type checker, linter) after the model writes a matching file and puts the
//! problems in front of the model with the tool's output, so it fixes them in the same turn.

wit_bindgen::generate!({ world: "plugin", path: "../../wit" });

use drift::plugin::host::config;
use drift::plugin::process::run;
use serde_json::Value;

struct Lint;

const WRITE_TOOLS: &[&str] = &["edit", "write", "apply_patch"];
const TIMEOUT_MS: u32 = 120_000;
const MAX_LINES: usize = 60;

/// Checks as `[{ "extensions": ["ts", "tsx"], "run": ["bunx", "tsc", "--noEmit", "-p", "."] }]`; `$FILE` is the written file.
fn defaults() -> Value {
    serde_json::json!([
        { "extensions": ["ts", "tsx"], "run": ["bunx", "tsc", "--noEmit", "-p", "."] },
        { "extensions": ["rs"], "run": ["cargo", "check", "--quiet", "--message-format", "short"] },
        { "extensions": ["py"], "run": ["ruff", "check", "$FILE"] },
        { "extensions": ["go"], "run": ["go", "vet", "./..."] },
    ])
}

fn checks_for(path: &str) -> Vec<Vec<String>> {
    let settings: Value = serde_json::from_str(&config()).unwrap_or(Value::Null);
    let table = if settings["checks"].is_array() { settings["checks"].clone() } else { defaults() };
    let extension = path.rsplit('.').next().unwrap_or_default().to_ascii_lowercase();
    table
        .as_array()
        .into_iter()
        .flatten()
        .filter(|check| check["extensions"].as_array().is_some_and(|exts| exts.iter().any(|ext| ext.as_str().is_some_and(|ext| ext.eq_ignore_ascii_case(&extension)))))
        .filter_map(|check| check["run"].as_array().map(|run| run.iter().filter_map(|item| item.as_str()).map(|arg| arg.replace("$FILE", path)).collect::<Vec<String>>()))
        .filter(|command| !command.is_empty())
        .collect()
}

fn written(input: &Value) -> Vec<String> {
    let direct = ["path", "filePath", "file_path"].iter().filter_map(|key| input[*key].as_str().map(str::to_owned));
    let patched = input["patch"].as_str().into_iter().flat_map(|patch| patch.lines().filter_map(|line| line.strip_prefix("*** Update File: ").or_else(|| line.strip_prefix("*** Add File: "))).map(|path| path.trim().to_owned()));
    direct.chain(patched).collect()
}

fn head(text: &str) -> String {
    let lines: Vec<&str> = text.lines().filter(|line| !line.trim().is_empty()).collect();
    let shown = &lines[..lines.len().min(MAX_LINES)];
    let mut out = shown.join("\n");
    if lines.len() > MAX_LINES {
        out.push_str(&format!("\n... {} more lines", lines.len() - MAX_LINES));
    }
    out
}

impl Guest for Lint {
    fn name() -> String {
        "lint-check".into()
    }

    fn before_tool(_call: ToolCall) -> BeforeTool {
        BeforeTool::Allow
    }

    fn after_tool(outcome: ToolResult) -> AfterTool {
        if outcome.failed || !WRITE_TOOLS.contains(&outcome.tool.as_str()) {
            return AfterTool::Keep;
        }
        let input: Value = serde_json::from_str(&outcome.input).unwrap_or(Value::Null);
        let mut ran: Vec<Vec<String>> = Vec::new();
        let mut problems = Vec::new();
        for path in written(&input) {
            for command in checks_for(&path) {
                if ran.contains(&command) {
                    continue;
                }
                ran.push(command.clone());
                let (program, args) = command.split_first().unwrap();
                match run(program, args, TIMEOUT_MS) {
                    Ok(output) if output.code == 0 => {}
                    Ok(output) => {
                        let text = if output.stderr.trim().is_empty() { output.stdout } else { format!("{}\n{}", output.stdout, output.stderr) };
                        problems.push(format!("`{}` exited with {}:\n{}", command.join(" "), output.code, head(&text)));
                    }
                    Err(error) => problems.push(format!("`{}` could not run: {error}", command.join(" "))),
                }
            }
        }
        if problems.is_empty() {
            return if ran.is_empty() { AfterTool::Keep } else { AfterTool::Note("checks passed".into()) };
        }
        AfterTool::Replace(format!("{}\n\n<lint-check>\nThe write succeeded, but a check found problems. Fix them before moving on.\n\n{}\n</lint-check>", outcome.output, problems.join("\n\n")))
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

export!(Lint);
