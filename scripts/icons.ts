// Icon artwork for every plugin: a rounded tile with a two-colour gradient and a white glyph.
import { mkdirSync } from "node:fs"
import { join } from "node:path"

const root = join(import.meta.dir, "..")

const tile = (from: string, to: string, glyph: string) => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64">
  <defs>
    <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="${from}"/>
      <stop offset="1" stop-color="${to}"/>
    </linearGradient>
  </defs>
  <rect width="64" height="64" rx="14" fill="url(#g)"/>
  <g fill="none" stroke="#fff" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round">
${glyph}
  </g>
</svg>
`

const icons: Record<string, string> = {
  guard: tile(
    "#5b7cfa",
    "#2f3d9e",
    `    <path d="M32 12 17 18v12c0 10.5 6.4 18.3 15 22 8.6-3.7 15-11.5 15-22V18Z"/>
    <path d="m25 32 5 5 10-11"/>`,
  ),
  "protect-files": tile(
    "#f08a4b",
    "#b3471e",
    `    <rect x="18" y="28" width="28" height="22" rx="4"/>
    <path d="M24 28v-6a8 8 0 0 1 16 0v6"/>
    <circle cx="32" cy="39" r="2.6" fill="#fff" stroke="none"/>`,
  ),
  "auto-format": tile(
    "#c56bf0",
    "#6e2ea8",
    `    <path d="M30 14l3.4 9.6L43 27l-9.6 3.4L30 40l-3.4-9.6L17 27l9.6-3.4Z"/>
    <path d="M45 38l1.6 4.4L51 44l-4.4 1.6L45 50l-1.6-4.4L39 44l4.4-1.6Z"/>`,
  ),
  "lint-check": tile(
    "#34c48a",
    "#157a56",
    `    <path d="M18 34h8M18 42h12M18 26h20"/>
    <path d="m34 42 5 5 9-11"/>`,
  ),
  "test-gate": tile(
    "#4fc3e8",
    "#1d6f9e",
    `    <path d="M26 14h12"/>
    <path d="M28 14v12L17.5 44.5A3 3 0 0 0 20.2 49h23.6a3 3 0 0 0 2.7-4.5L36 26V14"/>
    <path d="M22 38h20"/>`,
  ),
  notify: tile(
    "#ffb547",
    "#c76a0a",
    `    <path d="M43 28a11 11 0 0 0-22 0c0 12-5 13-5 16h32c0-3-5-4-5-16"/>
    <path d="M28 49a4 4 0 0 0 8 0"/>`,
  ),
  "git-autocommit": tile(
    "#ff7b72",
    "#b3261e",
    `    <circle cx="32" cy="32" r="7"/>
    <path d="M14 32h11M39 32h11"/>`,
  ),
  "git-context": tile(
    "#8b5cf6",
    "#4c1d95",
    `    <circle cx="22" cy="18" r="4.5"/>
    <circle cx="22" cy="46" r="4.5"/>
    <circle cx="42" cy="26" r="4.5"/>
    <path d="M22 22.5v19M42 30.5c0 7-8 7-14 10"/>`,
  ),
  "tool-log": tile(
    "#64748b",
    "#1e293b",
    `    <path d="M26 20h20M26 32h20M26 44h20"/>
    <circle cx="18" cy="20" r="1.6" fill="#fff" stroke="none"/>
    <circle cx="18" cy="32" r="1.6" fill="#fff" stroke="none"/>
    <circle cx="18" cy="44" r="1.6" fill="#fff" stroke="none"/>`,
  ),
}

for (const [id, svg] of Object.entries(icons)) {
  mkdirSync(join(root, "plugins", id), { recursive: true })
  await Bun.write(join(root, "plugins", id, "icon.svg"), svg)
}
console.log("wrote", Object.keys(icons).length)
