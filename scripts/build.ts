// Builds every plugin, copies the components into dist/ and writes registry.json with their hashes.
// Run with bun after `rustup target add wasm32-wasip2`.
import { createHash } from "node:crypto"
import { readdirSync, readFileSync, mkdirSync, copyFileSync, existsSync } from "node:fs"
import { join } from "node:path"
import { spawnSync } from "node:child_process"

const root = join(import.meta.dir, "..")
const base = "https://raw.githubusercontent.com/kylepelham/Drift-Plugins/main"

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
    download: `${base}/dist/${id}.wasm`,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    size: bytes.length,
  })
}

function workspaceVersion() {
  const text = readFileSync(join(root, "Cargo.toml"), "utf8")
  return /\[workspace\.package\][^[]*?version\s*=\s*"([^"]+)"/.exec(text)?.[1] ?? "0.0.0"
}

const registry = { version: 1, wit: "drift:plugin@0.2.0", plugins }
await Bun.write(join(root, "registry.json"), JSON.stringify(registry, null, 2) + "\n")
console.log(`registry.json: ${plugins.length} plugins`)
