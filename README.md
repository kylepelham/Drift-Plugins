# Drift Plugins

Plugins for [Drift](https://github.com/kylepelham/Drift), the desktop coding agent. Each is a
WebAssembly component built against Drift's plugin contract (`wit/drift.wit`), sandboxed by the
engine: a plugin reaches only what its imports name (files under the workspace, a process in it,
the network, a notice to you), and Settings shows those as its capabilities.

Install from Settings > Plugins > Registry in Drift, which reads `registry.json` here and
downloads the component from `dist/`, checking its hash.

## Plugins

| Plugin | What it does | Hooks |
| --- | --- | --- |
| guard | Refuses history-rewriting shell commands; `@guard test` runs your tests before a turn ends | before-tool, after-tool, turn-end |
| protect-files | Keeps the agent out of secrets, lockfiles and `.git` | before-tool |
| auto-format | Formats every written file by extension | after-tool |
| lint-check | Runs a type checker or linter after writes and feeds the problems to the model | after-tool |
| test-gate | Runs the test suite before a turn that changed files may end | after-tool, turn-end |
| notify | In-app notices and a webhook when Drift needs you or finishes | permission, turn-end, session |
| git-autocommit | Commits after every written file | after-tool |
| git-context | Branch, changed files and recent commits beside each prompt | prompt-submit |
| tool-log | One JSON line per tool call, in a file or to an endpoint | after-tool |

## Skill packs

`packs.json` lists Markdown skill collections from other repositories (Superpowers, Skills for Real
Engineers, Agent Skills, Karpathy Guidelines, SQL Server Query Plans). Drift installs one by
unpacking the pinned ref's archive into `~/.config/drift/skills/<id>`, where it reads skills from
already; a pack runs no code. A single skill is an entry with `kind: "skill"`, a collection `kind: "skills"`; both install the
same way, and Drift lets you pick which of a pack's skills to take.

Every plugin's config is in its `plugin.json`; Drift's install sheet offers the same fields and
writes them to your `~/.config/drift/drift.json` entry.

## Writing one

A plugin implements the `plugin` world: `name`, `before-tool`, `after-tool`, `prompt-submit`,
`turn-end`, `permission`, `compaction` and `session`, each returning the do-nothing answer when it
does not care. Copy any directory under `plugins/`, keep `plugin.json` honest, and build with
`cargo build --release --target wasm32-wasip2`. Any language with a component toolchain works the
same way: the contract is the WIT file, not Rust.

`bun scripts/build.ts` builds everything, copies the components to `dist/` and rewrites
`registry.json` with their hashes.
