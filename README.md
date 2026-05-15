# Path of Ways

A personal Path of Exile 2 companion desktop app — notes, campaign route planning, boss strategies, vendor recipes, and useful links, all in one place.

Built with [Leptos](https://leptos.dev/) (Rust → wasm) and wrapped as a native desktop app via [Tauri 2](https://v2.tauri.app/). All data persists locally in IndexedDB; no server, no account, no telemetry.

<p align="center">
  <img src="src-tauri/icons/source.png" alt="Path of Ways" width="180" />
</p>

## Features

- **Notes** with markdown rendering, image embedding, wiki-style `[[Title]]` links, code blocks with syntax highlighting (including a custom PoE filter grammar), and inline color tags (`#str()` / `#dex()` / `#int()`).
- **Campaign tracker** with editable zones grouped by act, per-zone progress checkboxes (waypoint, side area, quest reward, boss), and one-click jump to a zone-linked strategy note (created on first click).
- **Bosses** — categorized reference with quick-jump to per-boss strategy notes.
- **Vendor recipes** — editable reference of crafting recipes.
- **Links** — collection of external URLs (PoE wikis, trade, tools) that open in your OS default browser.
- **Quick switcher** (`Ctrl+K`) — searches across notes, zones, bosses, recipes, and links with markdown-rendered titles.
- **Three themes**: Light, Dark, and PoE2 — all sharing the [Fontin](https://www.exljbris.com/fontin.html) typeface used by the game.
- **Custom title bar** with embedded File / View menus and window controls (the bar respects PoE2's aesthetic and works as a native window drag region).
- **URL routing**, last-page persistence, and keyboard shortcuts.
- **Persistent storage** in IndexedDB across notes, campaign progress, zones, bosses, recipes, links, and embedded images. Orphan-image garbage collection runs on startup.
- **In-app error toasts** surface wasm panics and unhandled JS errors as red toasts pinned to the corner — no need to crack open devtools to know something went sideways.

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `1` … `5` | Jump to Notes / Campaign / Bosses / Recipes / Links |
| `Ctrl+N` | New blank note |
| `Ctrl+K` | Open quick switcher |
| `?` | Open help & shortcuts modal |
| `Esc` | Close any open modal |
| `Ctrl+Shift+I` | Open devtools (Tauri release builds) |

## Tech stack

- **Frontend:** Rust + [Leptos 0.8](https://leptos.dev/) (CSR mode) + [leptos_router](https://docs.rs/leptos_router/) + Tailwind CSS v4
- **Storage:** IndexedDB via [`rexie`](https://crates.io/crates/rexie)
- **Markdown:** [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark) + self-hosted [highlight.js](https://highlightjs.org/) for syntax highlighting
- **Build pipeline:** [Trunk](https://trunkrs.dev/) (wasm bundling) → [Tauri 2](https://v2.tauri.app/) (desktop wrap, includes opener + log plugins)

## Building from source

### Prerequisites

- Rust toolchain (stable)
- [`trunk`](https://trunkrs.dev/) — `cargo install trunk`
- [`tauri-cli`](https://v2.tauri.app/reference/cli/) — `cargo install tauri-cli --version "^2.0"`
- The `wasm32-unknown-unknown` target — `rustup target add wasm32-unknown-unknown`
- On Windows: [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually preinstalled on Win11)

### Run in browser (dev)

```bash
trunk serve
```

Opens at `http://localhost:8080`. Hot-reloads on save.

### Run as desktop app (dev)

```bash
cargo tauri dev
```

### Build release bundle

```bash
cargo tauri build
```

Produces:
- `src-tauri/target/release/app.exe` — standalone executable
- `src-tauri/target/release/bundle/nsis/Path of Ways_*_x64-setup.exe` — NSIS installer
- `src-tauri/target/release/bundle/msi/Path of Ways_*_x64_en-US.msi` — MSI installer

### Regenerating the app icon

The icon is authored as `src-tauri/icons/source.svg`. To regenerate all platform sizes after editing:

```bash
resvg -w 1024 -h 1024 src-tauri/icons/source.svg src-tauri/icons/source.png
cargo tauri icon src-tauri/icons/source.png
```

(`cargo install resvg` if you don't have it.)

## Project layout

```
src/
  app_state/      Global signals + per-action helpers (open note, navigate)
  bosses/         Bosses tab (model, storage, modals, view)
  campaign/       Campaign tracker (zones, progress, modals, view)
  links/          External links tab
  notes/          Notes tab + markdown rendering + lightbox + sidebar
  recipes/        Vendor recipes tab
  search/         Quick switcher (Ctrl+K)
  db.rs           IndexedDB store config (rexie)
  error_log.rs    Panic hook + in-app error toasts
  external.rs     Open external URLs through tauri-plugin-opener
  help.rs         Help modal
  icons.rs        SVG icon components
  images.rs       Embedded-image storage + orphan GC
  keyboard.rs     Global keyboard shortcuts
  modal.rs        Shared <ModalShell> wrapper
  theme.rs        Theme cycling (Light / Dark / PoE2)
  titlebar.rs     Custom title bar with File / View menus + window controls
  main.rs         App root, routing, root-level <ErrorBanner>

src-tauri/       Tauri config, capabilities, icons, Rust shell
style.css        Theme variables + Fontin @font-face + Tailwind v4 imports
vendor/          Self-hosted highlight.js + custom poefilter grammar
fonts/           Fontin (Regular / Italic / Bold / SmallCaps) WOFFs
```

## Storage notes

- The IndexedDB database is named `path-of-ways` and lives per-user under the WebView2 user data folder (Tauri) or the browser profile (dev). It's never sent anywhere.
- Schema is currently at version 7. Migrations are handled by `rexie` upgrades in `src/db.rs`.
- Default zones, bosses, and recipes are seeded once via localStorage flags (`zones_seeded_v1`, etc.) — deletes won't be re-seeded.

## Why this exists

Path of Exile 2 is a deep game with a lot of nuance worth remembering between sessions — campaign routes, vendor recipes, boss mechanics, your build notes. Existing public tools cover pieces of this, but nothing felt like "a single private notebook I can throw whatever I want into and still have it organized." So I built one.
