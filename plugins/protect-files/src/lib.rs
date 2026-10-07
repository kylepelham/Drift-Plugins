//! Keeps the model out of files that should never be edited by an agent: secrets, lockfiles,
//! the git directory. Edits, writes and patches to a protected path are refused, and so is a
//! shell line that names one.

wit_bindgen::generate!({ world: "plugin", path: "../../wit" });

use drift::plugin::host::config;
use serde_json::Value;

struct Protect;

const DEFAULT_PATTERNS: &[&str] = &[".env", ".env.*", "*.pem", "*.key", "*.p12", "*.pfx", "id_rsa*", "package-lock.json", "pnpm-lock.yaml", "yarn.lock", "bun.lock", "bun.lockb", "Cargo.lock", "poetry.lock", "Gemfile.lock", ".git/**"];
const WRITE_TOOLS: &[&str] = &["edit", "write", "apply_patch"];

fn patterns() -> Vec<String> {
    let settings: Value = serde_json::from_str(&config()).unwrap_or(Value::Null);
    match settings["patterns"].as_array() {
        Some(items) => items.iter().filter_map(|item| item.as_str().map(str::to_owned)).collect(),
        None => DEFAULT_PATTERNS.iter().map(|s| s.to_string()).collect(),
    }
}

/// Whether a path matches a pattern: `*` within a segment, `**` across segments, `?` one character.
/// A pattern without a slash matches the file name wherever it is.
fn matches(pattern: &str, path: &str) -> bool {
    let path = path.replace('\\', "/");
    if !pattern.contains('/') {
        return path.rsplit('/').next().is_some_and(|name| glob(pattern, name));
    }
    let trimmed = path.trim_start_matches("./");
    glob(pattern, trimmed) || suffixes(trimmed).any(|tail| glob(pattern, tail))
}

fn suffixes(path: &str) -> impl Iterator<Item = &str> {
    path.match_indices('/').map(move |(at, _)| &path[at + 1..])
}

fn glob(pattern: &str, text: &str) -> bool {
    fn walk(p: &[char], t: &[char]) -> bool {
        match (p.first(), t.first()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some('*'), _) if p.get(1) == Some(&'*') => {
                let rest = if p.get(2) == Some(&'/') { &p[3..] } else { &p[2..] };
                (0..=t.len()).any(|skip| walk(rest, &t[skip..]))
            }
            (Some('*'), _) => (0..=t.len()).take_while(|&skip| skip == 0 || t[skip - 1] != '/').any(|skip| walk(&p[1..], &t[skip..])),
            (Some('?'), Some(c)) if *c != '/' => walk(&p[1..], &t[1..]),
            (Some(a), Some(b)) if a == b => walk(&p[1..], &t[1..]),
            _ => false,
        }
    }
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    walk(&p, &t)
}

fn protected_in(text: &str, patterns: &[String]) -> Option<String> {
    text.split(|ch: char| ch.is_whitespace() || matches!(ch, '"' | '\'' | '>' | '<' | '|' | ';' | '&' | '(' | ')'))
        .filter(|word| !word.is_empty() && !word.starts_with('-'))
        .find(|word| patterns.iter().any(|pattern| matches(pattern, word)))
        .map(str::to_owned)
}

impl Guest for Protect {
    fn name() -> String {
        "protect-files".into()
    }

    fn before_tool(call: ToolCall) -> BeforeTool {
        let input: Value = serde_json::from_str(&call.input).unwrap_or(Value::Null);
        let patterns = patterns();
        if WRITE_TOOLS.contains(&call.tool.as_str()) {
            let paths = ["path", "filePath", "file_path"].iter().filter_map(|key| input[*key].as_str());
            let patch = input["patch"].as_str().unwrap_or_default();
            let hit = paths.map(str::to_owned).find(|path| patterns.iter().any(|pattern| matches(pattern, path))).or_else(|| protected_in(patch, &patterns));
            if let Some(path) = hit {
                return BeforeTool::Deny(format!("{path} is protected; it holds secrets or is generated, so a person edits it"));
            }
        }
        if call.tool == "bash" {
            let command = input["command"].as_str().unwrap_or_default();
            if let Some(path) = protected_in(command, &patterns) {
                return BeforeTool::Deny(format!("{path} is protected; a shell line may not touch it"));
            }
        }
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

export!(Protect);
