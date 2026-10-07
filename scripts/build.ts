// Builds every plugin, copies the components into dist/ and writes registry.json with their hashes.
// Run with bun after `rustup target add wasm32-wasip2`.
import { createHash } from "node:crypto"
import { readdirSync, readFileSync, mkdirSync, copyFileSync, existsSync } from "node:fs"
import { join } from "node:path"
import { spawnSync } from "node:child_process"

const root = join(import.meta.dir, "..")
const base = "https://raw.githubusercontent.com/kylepelham/Drift-Plugins/main"
// raw.githubusercontent.com serves SVG as text, so images come through jsDelivr, which serves them as images.
const images = "https://cdn.jsdelivr.net/gh/kylepelham/Drift-Plugins@main"

if (!process.argv.includes("--no-build")) {
  const built = spawnSync("cargo", ["build", "--release", "--target", "wasm32-wasip2"], { cwd: root, stdio: "inherit" })
  if (built.status !== 0) process.exit(built.status ?? 1)
}

mkdirSync(join(root, "dist"), { recursive: true })
const plugins = []
for (const id of readdirSync(join(root, "plugins")).sort()) {
  const manifestPath = join(root, "plugins", id, "plugin.json")
  if (!existsSync(manifestPath)) continue
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"))
  const cargo = readFileSync(join(root, "plugins", id, "Cargo.toml"), "utf8")
  const version = /version\.workspace\s*=\s*true/.test(cargo) ? workspaceVersion() : /^version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1]
  const built = join(root, "target", "wasm32-wasip2", "release", `${id.replaceAll("-", "_")}.wasm`)
  const out = join(root, "dist", `${id}.wasm`)
  copyFileSync(built, out)
  const bytes = readFileSync(out)
  plugins.push({
    ...manifest,
    version,
    author: "Drift",
    source: `https://github.com/kylepelham/Drift-Plugins/tree/main/plugins/${id}`,
    image: existsSync(join(root, "plugins", id, "icon.svg")) ? `${images}/plugins/${id}/icon.svg` : undefined,
    download: `${base}/dist/${id}.wasm`,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    size: bytes.length,
  })
}

function workspaceVersion() {
  const text = readFileSync(join(root, "Cargo.toml"), "utf8")
  return /\[workspace\.package\][^[]*?version\s*=\s*"([^"]+)"/.exec(text)?.[1] ?? "0.0.0"
}

// Skill packs: Markdown skills from other repositories, installed by extracting an archive of a pinned ref.
// Each pack's skills are read from the repository so the install sheet can offer them one by one.
async function packSkills(owner: string, name: string, ref: string, subdirs: string[]) {
  const tree = (await (await fetch(`https://api.github.com/repos/${owner}/${name}/git/trees/${ref}?recursive=1`)).json()) as { tree: { path: string }[] }
  const inside = (file: string) => !subdirs.length || subdirs.some((sub) => file.startsWith(sub.replace(/\/$/, "") + "/"))
  const files = tree.tree.map((entry) => entry.path).filter((file) => file.endsWith("/SKILL.md") && inside(file)).sort()
  const skills = []
  for (const file of files) {
    const text = await (await fetch(`https://raw.githubusercontent.com/${owner}/${name}/${ref}/${file}`)).text()
    const front = text.startsWith("---") ? text.slice(3, text.indexOf("\n---", 3)) : ""
    const line = front.split("\n").find((item) => item.trim().startsWith("description:")) ?? ""
    const description = line.replace(/^\s*description:\s*/, "").replace(/^["']|["']$/g, "").trim().slice(0, 200)
    skills.push({ name: file.split("/").at(-2), description })
  }
  return skills
}

const packs = []
for (const pack of JSON.parse(readFileSync(join(root, "packs.json"), "utf8")).packs as Record<string, unknown>[]) {
  const repo = String(pack.repo).replace(/\/$/, "")
  const [owner, name] = repo.replace("https://github.com/", "").split("/")
  const subdirs = (pack.subdirs as string[] | undefined) ?? []
  // A "skill" entry is one skill on its own; a "skills" entry is a pack of them. Either installs the same way.
  packs.push({
    kind: "skills",
    category: "skills",
    hooks: [],
    config: [],
    version: String(pack.ref),
    source: repo,
    archive: `https://codeload.github.com/${owner}/${name}/tar.gz/${pack.ref}`,
    skills: await packSkills(owner!, name!, String(pack.ref), subdirs),
    ...pack,
  })
}

const registry = { version: 1, wit: "drift:plugin@0.2.0", plugins: [...plugins.map((plugin) => ({ kind: "wasm", ...plugin })), ...packs] }
await Bun.write(join(root, "registry.json"), JSON.stringify(registry, null, 2) + "\n")
console.log(`registry.json: ${plugins.length} plugins, ${packs.length} skill entries`)
