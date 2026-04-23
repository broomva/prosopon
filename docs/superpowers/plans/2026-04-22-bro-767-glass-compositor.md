# BRO-767 · prosopon-compositor-glass Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a 2D web compositor for Prosopon — Arcan Glass-styled, total over every `Intent` variant, served by an embeddable Rust crate that fans live envelopes over WebSocket + SSE — plus a broomva.tech demo route that renders text + glass side-by-side.

**Architecture:** Hybrid Rust + TypeScript crate at `core/prosopon/crates/prosopon-compositor-glass/`. The Rust side is an `axum` HTTP + WS server that subscribes to a `prosopon_runtime::Runtime`, fans envelopes as JSONL frames over WebSocket (primary) and as `text/event-stream` (read-only fallback), and embeds the built `web/` assets via `include_dir!`. The TS side (bun workspace under `web/`) is a Preact app using `@preact/signals-core`. A `SceneStore` + `SignalBus` mirror the Rust runtime semantics; an `IntentRegistry` dispatches each Intent variant to a per-surface Preact component (Arcan Glass-tokened); Pretext handles text measurement and Yoga handles flex layout for composite intents. Golden-file vitest snapshots against the same fixture scenes the text compositor uses lock cross-surface parity.

**Tech Stack:**
- Rust: `axum` 0.8, `tokio` workspace, `tower-http`, `include_dir` 0.7, `futures`, `tokio-stream`, reusing `prosopon-core` / `prosopon-protocol` / `prosopon-runtime`.
- TS: bun, TypeScript 5.9, Preact 10.x, `@preact/signals-core` 1.x, `@chenglou/pretext`, `yoga-layout` 3.x, `vite` 6.x, `vitest` 2.x, `biome` 2.x.
- Style: Tailwind v4 tokens inlined (so the bundle is standalone; no runtime Tailwind import); tokens copied from `apps/arcan-glass/arcan-glass/` references.
- Demo route: added to `broomva.tech/apps/docs/` Next.js 16 app under `/app/prosopon/demo/page.tsx`.

**RFC impact:** Extends RFC-0002 (new compositor following the totality contract). No IR changes; no `IR_SCHEMA_VERSION` or `PROTOCOL_VERSION` bumps. Adds one well-known attr key (`glass.variant`) documented in RFC-0001's attribute table.

**Blast radius guardrails:**
- `prosopon-core` MUST NOT gain dependencies (CONTROL.md S6.2, S7.1).
- No new crate in the workspace may import from another compositor (S7.2).
- `Compositor` trait stays object-safe (S7.3).
- Every new `Intent` match in Rust and TS includes a `_` wildcard arm — `Intent` is `#[non_exhaustive]`.

---

## File structure

### New Rust crate

```
core/prosopon/crates/prosopon-compositor-glass/
├── Cargo.toml
├── README.md                          # crate-level doc with the embed flow
├── build.rs                           # optional: emit a cargo::rerun-if-changed for web/dist
├── src/
│   ├── lib.rs                         # GlassCompositor + public API + include_dir! assets
│   ├── compositor.rs                  # impl Compositor for GlassCompositor (totality guard)
│   ├── fanout.rs                      # tokio::sync::broadcast of Envelope
│   ├── server.rs                      # axum Router: / (index), /ws, /events, /schema
│   ├── assets.rs                      # ServeDir fallback + include_dir static handler
│   └── bin/
│       └── prosopon-glass.rs          # thin CLI: `serve --port --fixture <path>`
├── tests/
│   ├── apply_totality.rs              # every Intent + every Event variant round-trips
│   ├── server_handshake.rs            # WS handshake + envelope fanout integration test
│   └── fixtures/                      # shared JSON fixtures used by goldens on both sides
│       ├── demo_scene.json
│       ├── tool_flow.json
│       └── streaming_tokens.json
└── web/                               # TS sub-package, bun workspace
    ├── package.json
    ├── tsconfig.json
    ├── biome.json
    ├── vite.config.ts
    ├── index.html
    ├── public/                        # served as-is (arcan-glass logo, favicon)
    ├── src/
    │   ├── index.tsx                  # mounts App
    │   ├── app.tsx                    # layout: scene | envelopes | status
    │   ├── runtime/
    │   │   ├── scene-store.ts         # applies ProsoponEvent to local Scene
    │   │   ├── signal-bus.ts          # Topic → Signal<SignalValue>
    │   │   ├── transport.ts           # WS client + SSE fallback, envelope validator
    │   │   └── types.ts               # generated-shape types mirroring prosopon-core
    │   ├── registry/
    │   │   ├── intents.ts             # dispatcher: Intent.type → Component
    │   │   ├── context.ts             # RegistryContext provider + hooks
    │   │   └── fallback.tsx           # placeholder for unknown/unimplemented intents
    │   ├── components/
    │   │   ├── Node.tsx               # generic Node wrapper (lifecycle, actions, children)
    │   │   ├── Prose.tsx
    │   │   ├── Code.tsx               # Shiki lazy-loaded
    │   │   ├── Section.tsx
    │   │   ├── Divider.tsx
    │   │   ├── Progress.tsx
    │   │   ├── ToolCall.tsx
    │   │   ├── ToolResult.tsx
    │   │   ├── Choice.tsx
    │   │   ├── Confirm.tsx
    │   │   ├── Input.tsx
    │   │   ├── Signal.tsx
    │   │   ├── Stream.tsx              # Pretext-measured token tail
    │   │   ├── EntityRef.tsx
    │   │   ├── Link.tsx
    │   │   ├── Citation.tsx
    │   │   ├── Math.tsx
    │   │   ├── Image.tsx
    │   │   ├── Audio.tsx
    │   │   ├── Video.tsx
    │   │   ├── Empty.tsx
    │   │   └── Custom.tsx
    │   ├── layout/
    │   │   ├── measure.ts              # Pretext wrapper (lazy-init WASM)
    │   │   └── group.ts                # GroupKind → Tailwind flex/grid classes
    │   ├── tokens/
    │   │   ├── glass.css               # Arcan Glass variables (bundled, not via Tailwind)
    │   │   └── index.ts                # JS access to token names
    │   ├── actions/
    │   │   └── emit.ts                 # POST fallback + WS send for ActionEmitted
    │   └── util/
    │       ├── binding.ts              # resolve Bindings against signal cache
    │       └── format.ts               # SignalValue formatter (mirrors Rust::preview)
    └── tests/
        ├── setup.ts
        ├── goldens.test.ts            # renders each fixture, snapshots outerHTML
        ├── apply.test.ts              # scene-store applies each event variant
        ├── binding.test.ts            # Progress.pct hydration round-trip
        └── totality.test.ts           # every Intent renders something non-empty
```

### broomva.tech demo route

```
broomva.tech/apps/docs/app/prosopon/demo/
├── page.tsx                           # client component, side-by-side text + glass panels
├── layout.tsx                         # full-bleed no-sidebar override
└── lib/
    ├── fixture-envelopes.ts           # 3 canned envelope streams (reuses same JSON)
    └── glass-embed.tsx                # thin import of the web module as an ES module
```

The web package is **not** published to npm for v0.2. The demo route imports the built ES module directly from `core/prosopon/crates/prosopon-compositor-glass/web/dist/` via a path alias, the same way `core/life/interface/` is consumed by other TS apps in the workspace.

### Documentation touchpoints

- `core/prosopon/crates/prosopon-compositor-glass/README.md` — module purpose, quickstart.
- `core/prosopon/docs/surfaces/glass.md` — promote from "planned" to "shipped" with the final module sketch.
- `core/prosopon/docs/rfcs/0001-ir-schema.md` — add `glass.variant` well-known attr key.
- `core/prosopon/CONTROL.md` — update S2.3 line (text compositor goldens → text + glass goldens).
- `core/prosopon/CHANGELOG.md` — add v0.2.0-alpha entry.
- `core/prosopon/PLANS.md` — check BRO-767.
- Linear BRO-767 — post summary comment linking to this plan path and PR.

---

## Sequencing

Tasks 0–2 unblock everything else. Task 3 is the Rust trait impl. Tasks 4–6 build the TS compositor foundations. Tasks 7–9 implement every Intent variant group. Tasks 10–11 wire live transport. Task 12 is goldens. Task 13 is the broomva.tech demo. Task 14 is release hygiene.

Each task ends with a commit. Commits are conventional. Push after Task 2 so CI runs against the branch early; don't open the PR until Task 14.

---

## Task 0: Unblock `make smoke` (baseline repair)

**Why first:** `make smoke` is currently red with two clippy errors in `prosopon-runtime`. CONTROL.md S1.2 requires zero clippy errors with `-D warnings`. All downstream verification depends on smoke being green.

**Files:**
- Modify: `core/prosopon/crates/prosopon-runtime/src/store.rs:93-100` (StoreEvent::Reset)
- Modify: `core/prosopon/crates/prosopon-runtime/src/store.rs:154-157` (test construction style)

- [ ] **Step 0.1: Reproduce the failure**

Run: `cd core/prosopon && make smoke 2>&1 | grep -E "error|warning: unused"`
Expected: exactly two clippy diagnostics —
- `clippy::large_enum_variant` on `StoreEvent::Reset { previous: Scene }`
- `clippy::field_reassign_with_default` on the `NodePatch::default()` + assign pattern

- [ ] **Step 0.2: Box the large variant in `StoreEvent`**

Edit `core/prosopon/crates/prosopon-runtime/src/store.rs`. Change:

```rust
pub enum StoreEvent {
    Reset { previous: Scene },
    Added { parent: NodeId, id: NodeId },
    Updated { id: NodeId },
    Removed { id: NodeId },
    SignalUpdated { topic: prosopon_core::Topic },
    Passthrough,
}
```

to:

```rust
pub enum StoreEvent {
    /// The scene was wholesale replaced. `previous` is boxed to keep the
    /// enum tag-word small; `StoreEvent::Reset` is rare.
    Reset { previous: Box<Scene> },
    Added { parent: NodeId, id: NodeId },
    Updated { id: NodeId },
    Removed { id: NodeId },
    SignalUpdated { topic: prosopon_core::Topic },
    Passthrough,
}
```

And update the single emitter inside `SceneStore::apply`:

```rust
ProsoponEvent::SceneReset { scene } => {
    let prev = std::mem::replace(&mut self.scene, scene);
    Ok(StoreEvent::Reset { previous: Box::new(prev) })
}
```

- [ ] **Step 0.3: Fix the `patches_update_attrs` test style**

In `crates/prosopon-runtime/src/store.rs`, replace the `patches_update_attrs` test body's patch construction:

```rust
let mut patch = NodePatch::default();
patch.intent = Some(Intent::Prose { text: "updated".into() });
```

with struct-literal form:

```rust
let patch = NodePatch {
    intent: Some(Intent::Prose { text: "updated".into() }),
    ..Default::default()
};
```

- [ ] **Step 0.4: Verify smoke is green**

Run: `cd core/prosopon && make smoke`
Expected: exit 0. No clippy errors. All 49 tests pass.

- [ ] **Step 0.5: Commit**

```bash
cd core/prosopon
git checkout -b bro-767-glass-compositor
git add crates/prosopon-runtime/src/store.rs
git commit -m "fix(runtime): box StoreEvent::Reset + use struct literal

Two clippy regressions appeared with rustc/clippy 1.93 — large_enum_variant
on StoreEvent::Reset and field_reassign_with_default on a test. Boxing the
rare Reset variant shrinks the enum tag to 48B; struct-literal construction
is idiomatic. No behavioural change; all 49 tests still pass.

Unblocks BRO-767."
```

---

## Task 1: Scaffold the `prosopon-compositor-glass` crate shell (Rust)

**Files:**
- Create: `core/prosopon/crates/prosopon-compositor-glass/Cargo.toml`
- Create: `core/prosopon/crates/prosopon-compositor-glass/src/lib.rs`
- Create: `core/prosopon/crates/prosopon-compositor-glass/README.md`
- Modify: `core/prosopon/Cargo.toml` (add to workspace members + workspace.dependencies)

- [ ] **Step 1.1: Add the member to the workspace**

Edit `core/prosopon/Cargo.toml`:

```toml
[workspace]
members = [
    "crates/prosopon-core",
    "crates/prosopon-protocol",
    "crates/prosopon-runtime",
    "crates/prosopon-compositor-text",
    "crates/prosopon-compositor-glass",
    "crates/prosopon-sdk",
    "crates/prosopon-cli",
    "crates/prosopon-pneuma",
    "examples/hello-prosopon",
]
```

And inside `[workspace.dependencies]`:

```toml
# --- Internal crates (path + version for crates.io publish) ---
prosopon-core = { path = "crates/prosopon-core", version = "0.1.0" }
prosopon-protocol = { path = "crates/prosopon-protocol", version = "0.1.0" }
prosopon-runtime = { path = "crates/prosopon-runtime", version = "0.1.0" }
prosopon-compositor-text = { path = "crates/prosopon-compositor-text", version = "0.1.0" }
prosopon-compositor-glass = { path = "crates/prosopon-compositor-glass", version = "0.2.0" }
prosopon-sdk = { path = "crates/prosopon-sdk", version = "0.1.0" }
prosopon-pneuma = { path = "crates/prosopon-pneuma", version = "0.1.0" }
```

And append (under Serialization):

```toml
# --- Web server (compositor-glass, prosopon-daemon) ---
axum = { version = "0.8", features = ["ws", "macros"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
include_dir = "0.7"
mime_guess = "2"
bytes = "1"
```

- [ ] **Step 1.2: Write the crate `Cargo.toml`**

Create `core/prosopon/crates/prosopon-compositor-glass/Cargo.toml`:

```toml
[package]
name = "prosopon-compositor-glass"
version = "0.2.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "Prosopon 2D web compositor — Arcan Glass-styled, axum + embedded Preact bundle"
repository.workspace = true
keywords = ["prosopon", "agents", "ui", "compositor", "web"]
categories.workspace = true
readme = "README.md"

[lib]
path = "src/lib.rs"

[[bin]]
name = "prosopon-glass"
path = "src/bin/prosopon-glass.rs"

[dependencies]
prosopon-core = { workspace = true }
prosopon-protocol = { workspace = true }
prosopon-runtime = { workspace = true }

axum = { workspace = true }
tokio = { workspace = true }
tokio-stream = { workspace = true }
futures = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }

serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

include_dir = { workspace = true }
mime_guess = { workspace = true }
bytes = { workspace = true }

clap = { workspace = true }

[dev-dependencies]
tokio = { workspace = true }
tracing-subscriber = { workspace = true }
pretty_assertions = { workspace = true }
tokio-tungstenite = "0.24"
```

- [ ] **Step 1.3: Write a minimal `lib.rs` that compiles + exports an empty struct**

Create `core/prosopon/crates/prosopon-compositor-glass/src/lib.rs`:

```rust
//! # prosopon-compositor-glass
//!
//! 2D web compositor for Prosopon — serves an embedded Preact bundle over HTTP,
//! streams [`prosopon_protocol::Envelope`] over WebSocket + SSE, and implements
//! [`prosopon_runtime::Compositor`] for in-process use.
//!
//! The web bundle under `web/dist/` is embedded at build time via `include_dir`.
//! Agents can consume this crate three ways:
//!
//! 1. **In-process compositor.** Register a [`GlassCompositor`] on a
//!    [`prosopon_runtime::Runtime`]; envelopes fan out to any connected browser.
//! 2. **Standalone server.** Use the `prosopon-glass` binary (`serve --port 4321`).
//! 3. **Consumed bundle.** Import the `@prosopon/compositor-glass` TS package from
//!    a downstream web app and connect it to any Prosopon-speaking endpoint.
//!
//! See `docs/surfaces/glass.md` for the design note.

#![forbid(unsafe_code)]

pub mod compositor;
pub mod fanout;
pub mod server;
pub mod assets;

pub use compositor::{GlassCompositor, GlassCompositorBuilder};
pub use fanout::{EnvelopeFanout, EnvelopeReceiver};
pub use server::{GlassServer, GlassServerConfig};

/// Version of this compositor crate. Distinct from `PROTOCOL_VERSION` and
/// `IR_SCHEMA_VERSION` — bumps independently.
pub const COMPOSITOR_VERSION: &str = env!("CARGO_PKG_VERSION");
```

Create stub files so `cargo check` succeeds:

```rust
// src/compositor.rs
//! `GlassCompositor` — stub, implemented in Task 3.
pub struct GlassCompositor;
pub struct GlassCompositorBuilder;
```

```rust
// src/fanout.rs
//! Envelope fanout — stub, implemented in Task 11.
pub struct EnvelopeFanout;
pub struct EnvelopeReceiver;
```

```rust
// src/server.rs
//! HTTP + WebSocket server — stub, implemented in Task 10.
pub struct GlassServer;
pub struct GlassServerConfig;
```

```rust
// src/assets.rs
//! Embedded web bundle — stub, implemented in Task 10.
```

- [ ] **Step 1.4: Write `README.md`**

Create `core/prosopon/crates/prosopon-compositor-glass/README.md`:

```markdown
# prosopon-compositor-glass

2D web compositor for the Prosopon display server. Renders `prosopon_core::Intent`
into an Arcan Glass-styled Preact UI, served from an embedded bundle or
consumable as a TypeScript package.

## Run the dev server

    cargo run -p prosopon-compositor-glass --bin prosopon-glass -- serve \
        --port 4321 --fixture tests/fixtures/demo_scene.json

Open http://localhost:4321/.

## Use as a Compositor

    use prosopon_compositor_glass::{GlassCompositor, GlassServer};
    use prosopon_runtime::Runtime;

    let mut server = GlassServer::bind("127.0.0.1:4321").await?;
    let compositor = GlassCompositor::new(server.fanout());
    runtime.register_compositor(Box::new(compositor)).await;
    server.serve().await?;

## Architecture

See `docs/surfaces/glass.md` and `docs/rfcs/0002-compositor-contract.md`.
```

- [ ] **Step 1.5: Verify the crate compiles**

Run: `cd core/prosopon && cargo check -p prosopon-compositor-glass`
Expected: Compiles clean. No tests yet.

- [ ] **Step 1.6: Commit**

```bash
git add core/prosopon/Cargo.toml \
        core/prosopon/crates/prosopon-compositor-glass/
git commit -m "feat(glass): scaffold prosopon-compositor-glass crate (BRO-767)

Empty crate shell with lib.rs module tree and README. Wires the crate into
the workspace and adds axum/tower-http/include_dir as workspace-level deps.
No behaviour yet — subsequent commits fill in the compositor trait,
envelope fanout, server, and embedded web bundle."
```

---

## Task 2: Scaffold the `web/` TypeScript sub-package

**Files:**
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/package.json`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/tsconfig.json`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/biome.json`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/vite.config.ts`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/index.html`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/index.tsx`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/app.tsx`

- [ ] **Step 2.1: Write `package.json`**

```json
{
  "name": "@prosopon/compositor-glass",
  "version": "0.2.0",
  "private": false,
  "license": "Apache-2.0",
  "description": "Prosopon 2D web compositor — Arcan Glass-styled Preact renderer",
  "type": "module",
  "main": "./dist/index.js",
  "module": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "test": "vitest run",
    "test:watch": "vitest",
    "lint": "biome check src tests",
    "format": "biome format --write src tests",
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
    "preact": "^10.24.0",
    "@preact/signals": "^2.0.0",
    "@preact/signals-core": "^1.8.0",
    "@chenglou/pretext": "^0.3.0",
    "yoga-layout": "^3.2.1"
  },
  "devDependencies": {
    "@biomejs/biome": "^2.0.0",
    "@testing-library/preact": "^3.2.4",
    "@types/node": "^22.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "happy-dom": "^15.0.0",
    "typescript": "^5.9.0",
    "vite": "^6.0.0",
    "vitest": "^2.0.0"
  }
}
```

- [ ] **Step 2.2: Write `tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "jsx": "react-jsx",
    "jsxImportSource": "preact",
    "esModuleInterop": true,
    "skipLibCheck": true,
    "isolatedModules": true,
    "allowImportingTsExtensions": true,
    "noEmit": true,
    "paths": {
      "react": ["./node_modules/preact/compat"],
      "react-dom": ["./node_modules/preact/compat"]
    }
  },
  "include": ["src", "tests"],
  "exclude": ["node_modules", "dist"]
}
```

- [ ] **Step 2.3: Write `biome.json`**

```json
{
  "$schema": "https://biomejs.dev/schemas/2.0.0/schema.json",
  "files": {
    "ignoreUnknown": true,
    "include": ["src/**", "tests/**"]
  },
  "formatter": {
    "enabled": true,
    "indentStyle": "space",
    "indentWidth": 2,
    "lineWidth": 100
  },
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true,
      "style": {
        "noUselessElse": "error",
        "useConsistentArrayType": "error"
      }
    }
  }
}
```

- [ ] **Step 2.4: Write `vite.config.ts`**

```ts
import { defineConfig } from "vite";
import preact from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [preact()],
  build: {
    outDir: "dist",
    emptyOutDir: true,
    lib: {
      entry: "src/index.tsx",
      name: "ProsoponGlass",
      fileName: "index",
      formats: ["es"],
    },
    rollupOptions: {
      external: [],
      output: {
        inlineDynamicImports: true,
        assetFileNames: "assets/[name][extname]",
      },
    },
    sourcemap: true,
  },
  test: {
    environment: "happy-dom",
    setupFiles: ["./tests/setup.ts"],
    globals: false,
  },
});
```

- [ ] **Step 2.5: Write `index.html`**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width,initial-scale=1" />
    <title>Prosopon — Glass Compositor</title>
    <link rel="stylesheet" href="/src/tokens/glass.css" />
  </head>
  <body class="pgl-root">
    <div id="app"></div>
    <script type="module" src="/src/index.tsx"></script>
  </body>
</html>
```

- [ ] **Step 2.6: Write minimal `src/index.tsx` + `src/app.tsx`**

```tsx
// src/index.tsx
import { render } from "preact";
import { App } from "./app";

const root = document.getElementById("app");
if (!root) throw new Error("Prosopon glass: #app mount point missing");
render(<App />, root);
```

```tsx
// src/app.tsx
export function App() {
  return (
    <div className="pgl-shell">
      <h1>Prosopon — Glass Compositor</h1>
      <p>Scaffolding live. No scene connected yet.</p>
    </div>
  );
}
```

- [ ] **Step 2.7: Install and verify**

Run:
```
cd core/prosopon/crates/prosopon-compositor-glass/web
bun install
bun run typecheck
bun run build
```

Expected: `bun install` completes; `typecheck` exits 0; `build` produces `dist/index.js` under ~30 KB (Preact + empty app).

- [ ] **Step 2.8: Gitignore `dist` but track the shape**

Create `core/prosopon/crates/prosopon-compositor-glass/web/.gitignore`:

```
node_modules
dist
*.log
.vite
.turbo
```

And add a placeholder `core/prosopon/crates/prosopon-compositor-glass/web/dist/.gitkeep` so Cargo's `include_dir!` always has a directory to embed (even before the first build):

```
# placeholder — run `bun run build` in web/ to populate
```

- [ ] **Step 2.9: Commit**

```bash
git add core/prosopon/crates/prosopon-compositor-glass/web/
git commit -m "feat(glass-web): scaffold Preact + signals bun package (BRO-767)

Empty app mounts successfully. Pulls in preact, @preact/signals-core,
@chenglou/pretext, yoga-layout as runtime deps; vitest + happy-dom for
tests; biome for lint/format. Output bundle lives in web/dist/ and will
be embedded by the Rust crate via include_dir! in Task 10."
```

---

## Task 3: Implement `GlassCompositor` Compositor trait (TDD)

**Files:**
- Modify: `core/prosopon/crates/prosopon-compositor-glass/src/compositor.rs`
- Create: `core/prosopon/crates/prosopon-compositor-glass/tests/apply_totality.rs`

- [ ] **Step 3.1: Write the totality test FIRST**

Create `core/prosopon/crates/prosopon-compositor-glass/tests/apply_totality.rs`:

```rust
//! The compositor MUST accept every ProsoponEvent variant without erroring or
//! panicking. It MAY no-op for events it surfaces downstream (no browser connected).

use prosopon_compositor_glass::GlassCompositor;
use prosopon_core::{
    ChunkPayload, Intent, Node, NodePatch, ProsoponEvent, Scene, SignalValue, StreamChunk, Topic,
};
use prosopon_runtime::Compositor;

fn scene() -> Scene {
    Scene::new(Node::new(Intent::Prose { text: "hi".into() }).with_id("root"))
}

fn all_events() -> Vec<ProsoponEvent> {
    vec![
        ProsoponEvent::SceneReset { scene: scene() },
        ProsoponEvent::NodeAdded {
            parent: prosopon_core::NodeId::from_raw("root"),
            node: Node::new(Intent::Prose { text: "x".into() }),
        },
        ProsoponEvent::NodeUpdated {
            id: prosopon_core::NodeId::from_raw("root"),
            patch: NodePatch::default(),
        },
        ProsoponEvent::NodeRemoved {
            id: prosopon_core::NodeId::from_raw("root"),
        },
        ProsoponEvent::SignalChanged {
            topic: Topic::new("t"),
            value: SignalValue::Scalar(serde_json::json!(1.0)),
            ts: chrono::Utc::now(),
        },
        ProsoponEvent::StreamChunk {
            id: prosopon_core::StreamId::from_raw("s"),
            chunk: StreamChunk {
                seq: 1,
                payload: ChunkPayload::Text { text: "tok".into() },
                final_: true,
            },
        },
        ProsoponEvent::Heartbeat {
            ts: chrono::Utc::now(),
        },
    ]
}

#[tokio::test]
async fn apply_is_total_over_events() {
    let mut c = GlassCompositor::detached();
    for e in all_events() {
        c.apply(&e).expect("every event must be accepted");
    }
    c.flush().expect("flush never errors");
}

#[tokio::test]
async fn capabilities_advertise_twod() {
    let c = GlassCompositor::detached();
    let caps = c.capabilities();
    assert!(caps.surfaces.contains(&prosopon_core::SurfaceKind::TwoD));
    assert!(caps.supports_streaming);
    assert!(caps.supports_signal_push);
}
```

- [ ] **Step 3.2: Run the test — expect failure**

Run: `cd core/prosopon && cargo test -p prosopon-compositor-glass --test apply_totality`
Expected: FAIL — `GlassCompositor::detached` doesn't exist.

- [ ] **Step 3.3: Implement `compositor.rs`**

Replace `core/prosopon/crates/prosopon-compositor-glass/src/compositor.rs`:

```rust
//! `GlassCompositor` — implements `prosopon_runtime::Compositor` by forwarding
//! every envelope into an [`EnvelopeFanout`] for browser clients. In "detached"
//! mode (no fanout) all events are accepted and dropped; this mode exists for
//! tests and for embedding the compositor before an HTTP listener is bound.

use prosopon_core::{ProsoponEvent, SurfaceKind};
use prosopon_protocol::{Envelope, SessionId};
use prosopon_runtime::{Capabilities, Compositor, CompositorError, CompositorId};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::fanout::EnvelopeFanout;

/// A glass-surface compositor.
pub struct GlassCompositor {
    id: CompositorId,
    session: SessionId,
    seq: AtomicU64,
    fanout: Option<EnvelopeFanout>,
}

impl GlassCompositor {
    /// Create a compositor wired to an [`EnvelopeFanout`].
    #[must_use]
    pub fn new(fanout: EnvelopeFanout) -> Self {
        Self {
            id: CompositorId::new("prosopon-compositor-glass"),
            session: SessionId::new(),
            seq: AtomicU64::new(1),
            fanout: Some(fanout),
        }
    }

    /// Create a compositor with no connected fanout. Useful for tests and for
    /// programmatic registration before the server has been bound.
    #[must_use]
    pub fn detached() -> Self {
        Self {
            id: CompositorId::new("prosopon-compositor-glass"),
            session: SessionId::new(),
            seq: AtomicU64::new(1),
            fanout: None,
        }
    }

    fn next_seq(&self) -> u64 {
        self.seq.fetch_add(1, Ordering::Relaxed)
    }
}

impl Compositor for GlassCompositor {
    fn id(&self) -> CompositorId {
        self.id.clone()
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            surfaces: vec![SurfaceKind::TwoD],
            max_fps: Some(60),
            supports_signal_push: true,
            supports_streaming: true,
        }
    }

    fn apply(&mut self, event: &ProsoponEvent) -> Result<(), CompositorError> {
        let Some(fanout) = &self.fanout else {
            return Ok(());
        };
        let envelope = Envelope::new(self.session.clone(), self.next_seq(), event.clone());
        // send() returns Err only if there are no subscribers — not a failure
        // mode we surface upward. Log for observability and continue.
        if let Err(e) = fanout.send(envelope) {
            tracing::debug!(target: "prosopon::glass", "no subscribers: {e}");
        }
        Ok(())
    }
}

/// Builder for `GlassCompositor` configured with a specific session id. Reserved
/// for future multi-session use; v0.2 always mints a fresh id.
pub struct GlassCompositorBuilder;
```

- [ ] **Step 3.4: Implement a minimal `EnvelopeFanout` just enough for the test**

Replace `core/prosopon/crates/prosopon-compositor-glass/src/fanout.rs`:

```rust
//! `EnvelopeFanout` — a cloneable tokio broadcast sender that duplicates each
//! [`Envelope`] to every connected browser client.
//!
//! Receivers lag silently; the fanout uses a bounded channel (capacity 1024)
//! and slow clients observe `RecvError::Lagged` — they reconnect via SSE or
//! re-subscribe via WS.

use prosopon_protocol::Envelope;
use thiserror::Error;
use tokio::sync::broadcast;

const FANOUT_CAPACITY: usize = 1024;

/// Publisher side of the fanout — held by the compositor.
#[derive(Clone)]
pub struct EnvelopeFanout {
    tx: broadcast::Sender<Envelope>,
}

impl Default for EnvelopeFanout {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvelopeFanout {
    #[must_use]
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(FANOUT_CAPACITY);
        Self { tx }
    }

    /// Create a new receiver. Receivers see envelopes sent after they
    /// subscribe; they do NOT replay the historical scene. The HTTP handler
    /// snapshots the current scene from the runtime and sends it as the first
    /// frame to bridge the gap.
    pub fn subscribe(&self) -> EnvelopeReceiver {
        EnvelopeReceiver {
            rx: self.tx.subscribe(),
        }
    }

    /// Publish an envelope. Returns the number of currently connected
    /// subscribers; a value of 0 is NOT an error.
    pub fn send(&self, envelope: Envelope) -> Result<usize, FanoutError> {
        // tokio `send` returns Err only when there are zero subscribers.
        match self.tx.send(envelope) {
            Ok(n) => Ok(n),
            Err(_) => Ok(0),
        }
    }
}

/// Subscriber side — one per HTTP connection.
pub struct EnvelopeReceiver {
    rx: broadcast::Receiver<Envelope>,
}

impl EnvelopeReceiver {
    /// Await the next envelope.
    pub async fn recv(&mut self) -> Result<Envelope, FanoutError> {
        self.rx.recv().await.map_err(|e| match e {
            broadcast::error::RecvError::Closed => FanoutError::Closed,
            broadcast::error::RecvError::Lagged(n) => FanoutError::Lagged(n),
        })
    }
}

/// Errors surfaced by the fanout.
#[derive(Debug, Error)]
pub enum FanoutError {
    #[error("fanout channel closed")]
    Closed,
    #[error("subscriber lagged by {0} envelopes — reconnect required")]
    Lagged(u64),
}
```

- [ ] **Step 3.5: Run the totality test — expect pass**

Run: `cd core/prosopon && cargo test -p prosopon-compositor-glass --test apply_totality`
Expected: 2 tests pass.

- [ ] **Step 3.6: Run the full workspace tests**

Run: `cd core/prosopon && cargo test --workspace`
Expected: 49 + 2 = 51 tests pass.

- [ ] **Step 3.7: Commit**

```bash
git add core/prosopon/crates/prosopon-compositor-glass/src/compositor.rs \
        core/prosopon/crates/prosopon-compositor-glass/src/fanout.rs \
        core/prosopon/crates/prosopon-compositor-glass/tests/apply_totality.rs
git commit -m "feat(glass): implement Compositor trait + EnvelopeFanout (BRO-767)

GlassCompositor accepts every ProsoponEvent variant, forwards each as an
Envelope with monotonic seq into a tokio broadcast channel. Two new
integration tests cover totality + capabilities. No subscribers = not an
error; the server will snapshot-then-subscribe on WS connect (Task 10)."
```

---

## Task 4: TS `types.ts` mirroring prosopon-core shape (hand-written, minimal)

**Files:**
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/runtime/types.ts`

The future plan is to auto-generate these from the published JSON schema (separate ticket). For v0.2, we hand-write the shape. We do NOT model every supporting enum — just what rendering needs.

- [ ] **Step 4.1: Write the types**

```ts
// src/runtime/types.ts
// Mirror of prosopon-core's JSON shape. Hand-written for v0.2; will be
// auto-generated from scene_schema_json() in a future ticket.

export type Topic = string;
export type NodeId = string;
export type StreamId = string;
export type ActionId = string;
export type SceneId = string;

export type SignalValue =
  | { type: "scalar"; value: number | string | boolean | null }
  | { type: "series"; points: Array<{ ts: string; value: number }> }
  | { type: "categorical"; label: string }
  | { type: "vector"; components: number[] };

export type Intent =
  | { type: "prose"; text: string }
  | { type: "code"; lang: string; source: string }
  | { type: "math"; source: string }
  | { type: "entity_ref"; kind: string; id: string; label?: string }
  | { type: "link"; href: string; label?: string }
  | { type: "citation"; source: string; anchor?: string }
  | { type: "signal"; topic: Topic; display?: "inline" | "sparkline" | "badge" }
  | { type: "stream"; id: StreamId; kind: "text" | "audio" | "binary" | "jsonl" }
  | { type: "choice"; prompt: string; options: Array<ChoiceOption> }
  | { type: "confirm"; message: string; severity?: Severity }
  | { type: "input"; prompt: string; input: InputKind; default?: unknown }
  | { type: "tool_call"; name: string; args: unknown; stream?: StreamId }
  | { type: "tool_result"; success: boolean; payload: unknown }
  | { type: "progress"; pct?: number; label?: string }
  | { type: "group"; layout: GroupKind }
  | { type: "section"; title?: string; collapsible?: boolean }
  | { type: "divider" }
  | { type: "field"; topic: Topic; projection: Projection }
  | { type: "locus"; frame: SpatialFrame; position: [number, number, number] }
  | { type: "formation"; topic: Topic; kind: FormationKind }
  | { type: "image"; uri: string; alt?: string }
  | { type: "audio"; uri?: string; stream?: StreamId; voice?: string }
  | { type: "video"; uri: string; poster?: string }
  | { type: "empty" }
  | { type: "custom"; kind: string; payload: unknown };

export type GroupKind = "list" | "grid" | "sequence" | "parallel" | "stack";
export type Severity = "info" | "notice" | "warning" | "danger";
export type Projection = "heatmap" | "contour" | "volume" | "summary";
export type SpatialFrame = "viewer" | "world" | "geo";
export type FormationKind = "quorum" | "swarm" | "geometric" | "stigmergy";

export type InputKind =
  | { kind: "text"; multiline?: boolean }
  | { kind: "number"; min?: number; max?: number }
  | { kind: "boolean" }
  | { kind: "date" }
  | { kind: "json" };

export interface ChoiceOption {
  id: string;
  label: string;
  description?: string;
  default?: boolean;
}

export interface Node {
  id: NodeId;
  intent: Intent;
  children: Node[];
  bindings: Binding[];
  actions: ActionSlot[];
  attrs: Record<string, unknown>;
  lifecycle: Lifecycle;
}

export interface Binding {
  source: { topic: Topic; path?: string };
  target:
    | { type: "attr"; key: string }
    | { type: "intent_slot"; path: string }
    | { type: "child_content"; id: NodeId };
  transform?:
    | { type: "identity" }
    | { type: "format"; template: string }
    | { type: "clamp"; min: number; max: number }
    | { type: "round"; places: number }
    | { type: "percent" };
}

export interface ActionSlot {
  id: ActionId;
  kind: ActionKind;
  label?: string;
  enabled: boolean;
  visibility: "hidden" | "visible" | "primary";
}

export type ActionKind =
  | { type: "submit"; form_id?: string }
  | { type: "inspect"; node: NodeId }
  | { type: "focus"; node: NodeId }
  | { type: "invoke"; command: string; args?: unknown }
  | { type: "feedback"; kind: "positive" | "negative" | "neutral" }
  | { type: "choose"; option_id: string }
  | { type: "input"; value: unknown }
  | { type: "confirm"; accepted: boolean };

export interface Lifecycle {
  created_at?: string;
  expires_at?: string;
  priority: "ambient" | "normal" | "urgent" | "blocking";
  status:
    | { type: "active" }
    | { type: "pending" }
    | { type: "resolved" }
    | { type: "failed"; reason: string }
    | { type: "decaying"; half_life_ms: number };
}

export interface Scene {
  id: SceneId;
  root: Node;
  signals: Record<Topic, SignalValue>;
  hints: SceneHints;
}

export interface SceneHints {
  preferred_surfaces?: Array<"text" | "two_d" | "three_d" | "shader" | "audio" | "spatial" | "tactile">;
  intent_profile?: "balanced" | "dense_technical" | "ambient_monitor" | "cinematic" | "conversational";
  locale?: string;
  density?: "compact" | "comfortable" | "spacious";
  viewport?: { cols: number; rows: number };
}

export type StreamChunk = {
  seq: number;
  payload:
    | { encoding: "text"; text: string }
    | { encoding: "b64"; data: string; mime?: string }
    | { encoding: "json"; value: unknown };
  final_?: boolean;
};

export type ProsoponEvent =
  | { type: "scene_reset"; scene: Scene }
  | { type: "node_added"; parent: NodeId; node: Node }
  | { type: "node_updated"; id: NodeId; patch: NodePatch }
  | { type: "node_removed"; id: NodeId }
  | { type: "signal_changed"; topic: Topic; value: SignalValue; ts: string }
  | { type: "stream_chunk"; id: StreamId; chunk: StreamChunk }
  | { type: "action_emitted"; slot: ActionId; source: NodeId; kind: ActionKind }
  | { type: "heartbeat"; ts: string };

export interface NodePatch {
  intent?: Intent;
  attrs?: Record<string, unknown>;
  lifecycle?: Partial<Lifecycle>;
  children?: { op: "replace"; children: Node[] } | { op: "append"; child: Node } | { op: "remove"; id: NodeId };
}

export interface Envelope {
  version: number;
  session_id: string;
  seq: number;
  ts: string;
  event: ProsoponEvent;
}
```

- [ ] **Step 4.2: Typecheck**

Run: `cd core/prosopon/crates/prosopon-compositor-glass/web && bun run typecheck`
Expected: exits 0.

- [ ] **Step 4.3: Commit**

```bash
git add core/prosopon/crates/prosopon-compositor-glass/web/src/runtime/types.ts
git commit -m "feat(glass-web): mirror prosopon-core IR as hand-written TS types

v0.2 hand-writes the shape; a follow-up ticket will auto-generate from
scene_schema_json() and event_schema_json(). Shape is locked to the JSON
serde output of prosopon-core (snake_case tags, 'type' discriminator)."
```

---

## Task 5: TS `SceneStore` — applies events to a local Scene (TDD)

**Files:**
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/runtime/scene-store.ts`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/tests/apply.test.ts`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/tests/setup.ts`

- [ ] **Step 5.1: Write the setup file**

```ts
// tests/setup.ts
// happy-dom already loaded by vitest config. No further globals needed.
```

- [ ] **Step 5.2: Write the test FIRST**

```ts
// tests/apply.test.ts
import { describe, expect, it } from "vitest";
import { createSceneStore } from "../src/runtime/scene-store";
import type { ProsoponEvent, Scene } from "../src/runtime/types";

const root: Scene = {
  id: "s1",
  root: {
    id: "root",
    intent: { type: "section", title: "Hello" },
    children: [],
    bindings: [],
    actions: [],
    attrs: {},
    lifecycle: { priority: "normal", status: { type: "active" } },
  },
  signals: {},
  hints: {},
};

describe("scene store", () => {
  it("applies scene_reset by replacing the scene", () => {
    const store = createSceneStore(root);
    const next: Scene = { ...root, id: "s2" };
    store.apply({ type: "scene_reset", scene: next });
    expect(store.scene().id).toBe("s2");
  });

  it("applies node_added under the given parent", () => {
    const store = createSceneStore(root);
    store.apply({
      type: "node_added",
      parent: "root",
      node: {
        id: "c1",
        intent: { type: "prose", text: "child" },
        children: [],
        bindings: [],
        actions: [],
        attrs: {},
        lifecycle: { priority: "normal", status: { type: "active" } },
      },
    });
    expect(store.scene().root.children).toHaveLength(1);
    expect(store.scene().root.children[0]?.id).toBe("c1");
  });

  it("applies node_removed by id", () => {
    const store = createSceneStore(root);
    store.apply({
      type: "node_added",
      parent: "root",
      node: {
        id: "c1",
        intent: { type: "prose", text: "child" },
        children: [],
        bindings: [],
        actions: [],
        attrs: {},
        lifecycle: { priority: "normal", status: { type: "active" } },
      },
    });
    store.apply({ type: "node_removed", id: "c1" });
    expect(store.scene().root.children).toHaveLength(0);
  });

  it("signal_changed updates the signals cache", () => {
    const store = createSceneStore(root);
    store.apply({
      type: "signal_changed",
      topic: "t",
      value: { type: "scalar", value: 42 },
      ts: new Date().toISOString(),
    });
    expect(store.scene().signals.t).toEqual({ type: "scalar", value: 42 });
  });

  it("unknown event variant is a silent no-op", () => {
    const store = createSceneStore(root);
    const unknown = { type: "future_variant", foo: "bar" } as unknown as ProsoponEvent;
    expect(() => store.apply(unknown)).not.toThrow();
    expect(store.scene().id).toBe("s1");
  });
});
```

- [ ] **Step 5.3: Run the test — expect failure**

Run: `cd core/prosopon/crates/prosopon-compositor-glass/web && bun run test`
Expected: FAIL — `createSceneStore` undefined.

- [ ] **Step 5.4: Implement `scene-store.ts`**

```ts
// src/runtime/scene-store.ts
// Minimal SceneStore: applies events to a local Scene and exposes a subscribe
// hook. Mirrors prosopon-runtime::SceneStore. Forward-compatible: unknown
// event variants are no-op rather than throw.

import { signal, type Signal } from "@preact/signals-core";
import type { Node, NodeId, ProsoponEvent, Scene } from "./types";

export interface SceneStore {
  /** Current scene (reactive). */
  readonly scene: () => Scene;
  /** Reactive signal — subscribe in Preact components via .value. */
  readonly signal: Signal<Scene>;
  /** Apply a single event. Unknown variants are silent no-ops. */
  apply(event: ProsoponEvent): void;
}

export function createSceneStore(initial: Scene): SceneStore {
  const s = signal<Scene>(initial);

  const apply = (event: ProsoponEvent) => {
    switch (event.type) {
      case "scene_reset":
        s.value = event.scene;
        return;
      case "node_added": {
        const next = deepClone(s.value);
        const parent = findNode(next.root, event.parent);
        if (parent) parent.children.push(event.node);
        s.value = next;
        return;
      }
      case "node_updated": {
        const next = deepClone(s.value);
        const target = findNode(next.root, event.id);
        if (target) applyPatch(target, event.patch);
        s.value = next;
        return;
      }
      case "node_removed": {
        const next = deepClone(s.value);
        removeNode(next.root, event.id);
        s.value = next;
        return;
      }
      case "signal_changed": {
        const next = deepClone(s.value);
        next.signals[event.topic] = event.value;
        s.value = next;
        return;
      }
      case "stream_chunk":
      case "action_emitted":
      case "heartbeat":
        // Pass-through — compositors handle directly via the transport layer.
        return;
      default:
        // Forward-compatible: future variants become no-ops.
        return;
    }
  };

  return {
    scene: () => s.value,
    signal: s,
    apply,
  };
}

function findNode(root: Node, id: NodeId): Node | undefined {
  if (root.id === id) return root;
  for (const child of root.children) {
    const found = findNode(child, id);
    if (found) return found;
  }
  return undefined;
}

function removeNode(root: Node, id: NodeId): boolean {
  const idx = root.children.findIndex((c) => c.id === id);
  if (idx >= 0) {
    root.children.splice(idx, 1);
    return true;
  }
  for (const child of root.children) {
    if (removeNode(child, id)) return true;
  }
  return false;
}

function applyPatch(node: Node, patch: import("./types").NodePatch): void {
  if (patch.intent) node.intent = patch.intent;
  if (patch.attrs) Object.assign(node.attrs, patch.attrs);
  if (patch.lifecycle) Object.assign(node.lifecycle, patch.lifecycle);
  if (patch.children) {
    if (patch.children.op === "replace") node.children = patch.children.children;
    else if (patch.children.op === "append") node.children.push(patch.children.child);
    else if (patch.children.op === "remove") removeNode(node, patch.children.id);
  }
}

function deepClone<T>(v: T): T {
  return structuredClone(v);
}
```

- [ ] **Step 5.5: Run — expect pass**

Run: `bun run test -- scene-store`
Expected: all 5 tests pass.

- [ ] **Step 5.6: Commit**

```bash
git add core/prosopon/crates/prosopon-compositor-glass/web/src/runtime/scene-store.ts \
        core/prosopon/crates/prosopon-compositor-glass/web/tests/apply.test.ts \
        core/prosopon/crates/prosopon-compositor-glass/web/tests/setup.ts
git commit -m "feat(glass-web): SceneStore applies ProsoponEvents reactively

Mirrors prosopon-runtime::SceneStore. Uses @preact/signals-core so any
Preact component reading store.signal.value automatically re-renders when
events apply. Unknown event variants are silent no-ops (forward compat)."
```

---

## Task 6: TS `SignalBus` + `binding.ts` — live signal hydration (TDD)

**Files:**
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/runtime/signal-bus.ts`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/util/binding.ts`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/tests/binding.test.ts`

- [ ] **Step 6.1: Write the binding test FIRST**

```ts
// tests/binding.test.ts
import { describe, expect, it } from "vitest";
import { hydrateIntent } from "../src/util/binding";
import type { Binding, Intent, SignalValue } from "../src/runtime/types";

describe("binding hydration", () => {
  it("resolves Progress.pct from attr:pct binding", () => {
    const intent: Intent = { type: "progress", pct: 0, label: "scoring" };
    const bindings: Binding[] = [
      {
        source: { topic: "job.pct" },
        target: { type: "attr", key: "pct" },
      },
    ];
    const signals: Record<string, SignalValue> = {
      "job.pct": { type: "scalar", value: 0.42 },
    };
    const out = hydrateIntent(intent, bindings, signals);
    expect(out).toEqual({ type: "progress", pct: 0.42, label: "scoring" });
  });

  it("intent_slot binding resolves via path", () => {
    const intent: Intent = { type: "prose", text: "placeholder" };
    const bindings: Binding[] = [
      {
        source: { topic: "greeting" },
        target: { type: "intent_slot", path: "text" },
      },
    ];
    const signals: Record<string, SignalValue> = {
      greeting: { type: "scalar", value: "hello" },
    };
    const out = hydrateIntent(intent, bindings, signals);
    expect(out).toEqual({ type: "prose", text: "hello" });
  });

  it("missing signal leaves intent unchanged", () => {
    const intent: Intent = { type: "progress", pct: 0.1 };
    const bindings: Binding[] = [
      { source: { topic: "missing" }, target: { type: "attr", key: "pct" } },
    ];
    const out = hydrateIntent(intent, bindings, {});
    expect(out).toEqual(intent);
  });
});
```

- [ ] **Step 6.2: Run — expect failure**

Run: `bun run test -- binding`
Expected: FAIL — `hydrateIntent` undefined.

- [ ] **Step 6.3: Implement `signal-bus.ts`**

```ts
// src/runtime/signal-bus.ts
import { signal, type Signal } from "@preact/signals-core";
import type { SignalValue, Topic } from "./types";

/** Reactive mapping of Topic → SignalValue. Backed by one Signal per topic. */
export class SignalBus {
  private readonly topics = new Map<Topic, Signal<SignalValue | undefined>>();

  get(topic: Topic): Signal<SignalValue | undefined> {
    let s = this.topics.get(topic);
    if (!s) {
      s = signal<SignalValue | undefined>(undefined);
      this.topics.set(topic, s);
    }
    return s;
  }

  publish(topic: Topic, value: SignalValue): void {
    this.get(topic).value = value;
  }

  snapshot(): Record<Topic, SignalValue> {
    const out: Record<Topic, SignalValue> = {};
    for (const [k, v] of this.topics) {
      if (v.value !== undefined) out[k] = v.value;
    }
    return out;
  }
}
```

- [ ] **Step 6.4: Implement `binding.ts`**

```ts
// src/util/binding.ts
// Resolve Bindings against a signal cache and return a hydrated Intent clone.
// Mirrors prosopon-compositor-text::hydrate_intent (RFC-0001 §Binding resolution).

import type { Binding, Intent, SignalValue } from "../runtime/types";

export function hydrateIntent(
  intent: Intent,
  bindings: readonly Binding[],
  signals: Readonly<Record<string, SignalValue>>,
): Intent {
  if (bindings.length === 0) return intent;
  let out: Intent = structuredClone(intent);
  for (const b of bindings) {
    const value = signals[b.source.topic];
    if (!value) continue;
    if (value.type !== "scalar") continue;
    const raw = value.value;
    if (b.target.type === "attr") {
      out = applyAttrBinding(out, b.target.key, raw);
    } else if (b.target.type === "intent_slot") {
      out = applyIntentSlotBinding(out, b.target.path, raw);
    }
    // child_content is handled at component level — not here.
  }
  return out;
}

function applyAttrBinding(intent: Intent, key: string, raw: unknown): Intent {
  if (intent.type === "progress" && key === "pct" && typeof raw === "number") {
    return { ...intent, pct: raw };
  }
  // Generic fallback: unhandled attr bindings leave the intent alone.
  return intent;
}

function applyIntentSlotBinding(intent: Intent, path: string, raw: unknown): Intent {
  if (intent.type === "prose" && path === "text" && typeof raw === "string") {
    return { ...intent, text: raw };
  }
  if (intent.type === "progress" && path === "pct" && typeof raw === "number") {
    return { ...intent, pct: raw };
  }
  return intent;
}
```

- [ ] **Step 6.5: Run — expect pass**

Run: `bun run test -- binding`
Expected: all 3 tests pass.

- [ ] **Step 6.6: Commit**

```bash
git add core/prosopon/crates/prosopon-compositor-glass/web/src/runtime/signal-bus.ts \
        core/prosopon/crates/prosopon-compositor-glass/web/src/util/binding.ts \
        core/prosopon/crates/prosopon-compositor-glass/web/tests/binding.test.ts
git commit -m "feat(glass-web): SignalBus + binding hydration (parity with text)

Glass hydrates the same bindings the Rust text compositor does:
attr:pct and intent_slot:text/pct, scalar values only. Signal cache is
keyed by topic; each topic is a Preact Signal so components re-render
autonomously."
```

---

## Task 7: Intent registry + Node wrapper + tokens

**Files:**
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/tokens/glass.css`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/tokens/index.ts`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/registry/context.ts`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/registry/intents.ts`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/registry/fallback.tsx`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/components/Node.tsx`

- [ ] **Step 7.1: Write Arcan Glass tokens (inlined — no Tailwind runtime)**

```css
/* src/tokens/glass.css */
/* Arcan Glass tokens — subset inlined for bundle-standalone compositor.
   Pulled from apps/arcan-glass/arcan-glass/references on 2026-04-22. */
:root {
  --pgl-bg: hsl(240 33% 6%);
  --pgl-surface: hsl(240 20% 10% / 0.72);
  --pgl-surface-elevated: hsl(240 20% 14% / 0.88);
  --pgl-border: hsl(240 30% 30% / 0.3);
  --pgl-text: hsl(0 0% 98%);
  --pgl-text-dim: hsl(0 0% 60%);
  --pgl-accent: hsl(239 84% 67%);
  --pgl-accent-soft: hsl(239 84% 67% / 0.15);
  --pgl-success: hsl(142 71% 45%);
  --pgl-warning: hsl(45 93% 58%);
  --pgl-danger: hsl(0 84% 60%);
  --pgl-font-sans: "Inter", ui-sans-serif, system-ui, sans-serif;
  --pgl-font-mono: "JetBrains Mono", ui-monospace, monospace;
  --pgl-radius: 10px;
  --pgl-radius-sm: 6px;
  --pgl-space-1: 4px;
  --pgl-space-2: 8px;
  --pgl-space-3: 12px;
  --pgl-space-4: 16px;
  --pgl-space-6: 24px;
  --pgl-space-8: 32px;
  --pgl-shadow: 0 1px 2px hsl(240 33% 0% / 0.4), 0 8px 24px hsl(240 33% 0% / 0.3);
  --pgl-glass-blur: 16px;
}
.pgl-root {
  margin: 0;
  padding: 0;
  background: var(--pgl-bg);
  color: var(--pgl-text);
  font-family: var(--pgl-font-sans);
  min-height: 100vh;
  -webkit-font-smoothing: antialiased;
}
.pgl-shell {
  padding: var(--pgl-space-6);
  max-width: 900px;
  margin: 0 auto;
}
.pgl-card {
  background: var(--pgl-surface);
  border: 1px solid var(--pgl-border);
  border-radius: var(--pgl-radius);
  padding: var(--pgl-space-4);
  backdrop-filter: blur(var(--pgl-glass-blur));
}
.pgl-dim { color: var(--pgl-text-dim); }
.pgl-mono { font-family: var(--pgl-font-mono); }
.pgl-flex-col { display: flex; flex-direction: column; gap: var(--pgl-space-3); }
.pgl-flex-row { display: flex; flex-direction: row; gap: var(--pgl-space-3); }
.pgl-divider {
  height: 1px;
  background: var(--pgl-border);
  margin: var(--pgl-space-3) 0;
}
.pgl-progress {
  display: flex;
  align-items: center;
  gap: var(--pgl-space-2);
}
.pgl-progress-track {
  flex: 1;
  height: 6px;
  background: var(--pgl-accent-soft);
  border-radius: 999px;
  overflow: hidden;
}
.pgl-progress-fill {
  height: 100%;
  background: var(--pgl-accent);
  transition: width 120ms linear;
}
.pgl-tool-call {
  font-family: var(--pgl-font-mono);
  color: var(--pgl-text-dim);
}
.pgl-tool-result-ok { color: var(--pgl-success); }
.pgl-tool-result-err { color: var(--pgl-danger); }
.pgl-fallback {
  color: var(--pgl-text-dim);
  font-style: italic;
}
.pgl-section-title {
  font-weight: 700;
  font-size: 1.1em;
  margin: 0 0 var(--pgl-space-2) 0;
}
.pgl-code {
  background: var(--pgl-surface-elevated);
  border-radius: var(--pgl-radius-sm);
  padding: var(--pgl-space-3);
  font-family: var(--pgl-font-mono);
  font-size: 0.9em;
  overflow-x: auto;
}
```

- [ ] **Step 7.2: Write `tokens/index.ts`**

```ts
// src/tokens/index.ts
// Not strictly necessary — most usage is via CSS custom properties. Exported so
// TS components can reference token names symbolically if they need to inject
// inline styles.

export const GLASS_TOKENS = {
  bg: "var(--pgl-bg)",
  surface: "var(--pgl-surface)",
  border: "var(--pgl-border)",
  text: "var(--pgl-text)",
  textDim: "var(--pgl-text-dim)",
  accent: "var(--pgl-accent)",
  success: "var(--pgl-success)",
  warning: "var(--pgl-warning)",
  danger: "var(--pgl-danger)",
} as const;
```

- [ ] **Step 7.3: Write `registry/context.ts`**

```ts
// src/registry/context.ts
import { createContext } from "preact";
import { useContext } from "preact/hooks";
import type { Scene, SignalValue } from "../runtime/types";
import { SignalBus } from "../runtime/signal-bus";

export interface RegistryCtx {
  scene: Scene;
  bus: SignalBus;
  emitAction: (envelope: unknown) => void;
}

// biome-ignore lint/style/noNonNullAssertion: provider asserts this in layout
export const RegistryContext = createContext<RegistryCtx>(null!);

export function useRegistry(): RegistryCtx {
  const ctx = useContext(RegistryContext);
  if (!ctx) throw new Error("useRegistry outside of RegistryContext");
  return ctx;
}

export function useSignalSnapshot(): Record<string, SignalValue> {
  const { scene } = useRegistry();
  return scene.signals;
}
```

- [ ] **Step 7.4: Write `registry/intents.ts`**

```ts
// src/registry/intents.ts
// Dispatcher: Intent.type → Component.
//
// The dispatcher is TOTAL: every Intent variant defined in types.ts MUST map
// to a component. Unknown variants (from future non_exhaustive additions) fall
// through to <Fallback>.

import type { ComponentType } from "preact";
import type { Intent } from "../runtime/types";
import { Audio } from "../components/Audio";
import { Choice } from "../components/Choice";
import { Citation } from "../components/Citation";
import { Code } from "../components/Code";
import { Confirm } from "../components/Confirm";
import { Custom } from "../components/Custom";
import { Divider } from "../components/Divider";
import { Empty } from "../components/Empty";
import { EntityRef } from "../components/EntityRef";
import { Image } from "../components/Image";
import { Input } from "../components/Input";
import { Link } from "../components/Link";
import { Math } from "../components/Math";
import { Progress } from "../components/Progress";
import { Prose } from "../components/Prose";
import { Section } from "../components/Section";
import { Signal } from "../components/Signal";
import { Stream } from "../components/Stream";
import { ToolCall } from "../components/ToolCall";
import { ToolResult } from "../components/ToolResult";
import { Video } from "../components/Video";
import { Fallback } from "./fallback";

export interface IntentProps<T extends Intent = Intent> {
  intent: T;
  nodeId: string;
}

type Dispatch = {
  [K in Intent["type"]]: ComponentType<IntentProps<Extract<Intent, { type: K }>>>;
};

export const INTENT_REGISTRY: Dispatch = {
  prose: Prose,
  code: Code,
  math: Math,
  entity_ref: EntityRef,
  link: Link,
  citation: Citation,
  signal: Signal,
  stream: Stream,
  choice: Choice,
  confirm: Confirm,
  input: Input,
  tool_call: ToolCall,
  tool_result: ToolResult,
  progress: Progress,
  group: Fallback as ComponentType<IntentProps<Extract<Intent, { type: "group" }>>>,
  section: Section,
  divider: Divider,
  field: Fallback as ComponentType<IntentProps<Extract<Intent, { type: "field" }>>>,
  locus: Fallback as ComponentType<IntentProps<Extract<Intent, { type: "locus" }>>>,
  formation: Fallback as ComponentType<IntentProps<Extract<Intent, { type: "formation" }>>>,
  image: Image,
  audio: Audio,
  video: Video,
  empty: Empty,
  custom: Custom,
};

export function renderIntent(intent: Intent, nodeId: string) {
  const Component = INTENT_REGISTRY[intent.type as keyof typeof INTENT_REGISTRY] ?? Fallback;
  // biome-ignore lint/suspicious/noExplicitAny: registry is total by construction
  const Props = Component as any;
  return <Props intent={intent} nodeId={nodeId} />;
}
```

- [ ] **Step 7.5: Write `registry/fallback.tsx`**

```tsx
// src/registry/fallback.tsx
import type { IntentProps } from "./intents";

export function Fallback({ intent }: IntentProps) {
  return (
    <div className="pgl-fallback pgl-mono">
      [{intent.type}] — compositor surface upgrade suggested
    </div>
  );
}
```

- [ ] **Step 7.6: Write `components/Node.tsx`**

```tsx
// src/components/Node.tsx
// Generic Node wrapper — handles lifecycle, actions, children. Each intent
// component handles its own body; this wrapper supplies priority styling and
// action buttons.

import type { Node as ProsoponNode } from "../runtime/types";
import { useRegistry } from "../registry/context";
import { hydrateIntent } from "../util/binding";
import { renderIntent } from "../registry/intents";

export function NodeView({ node }: { node: ProsoponNode }) {
  const { scene } = useRegistry();
  const hydrated = hydrateIntent(node.intent, node.bindings, scene.signals);
  const priorityClass = priorityClassFor(node.lifecycle.priority);

  return (
    <div className={`pgl-node ${priorityClass}`} data-node-id={node.id}>
      {renderIntent(hydrated, node.id)}
      {node.children.map((child) => (
        <NodeView key={child.id} node={child} />
      ))}
      <ActionBar node={node} />
    </div>
  );
}

function ActionBar({ node }: { node: ProsoponNode }) {
  const { emitAction } = useRegistry();
  const visible = node.actions.filter((a) => a.visibility !== "hidden");
  if (visible.length === 0) return null;
  return (
    <div className="pgl-flex-row" style={{ marginTop: "var(--pgl-space-2)" }}>
      {visible.map((a) => (
        <button
          key={a.id}
          disabled={!a.enabled}
          onClick={() => emitAction({ slot: a.id, source: node.id, kind: a.kind })}
        >
          {a.label ?? a.kind.type}
        </button>
      ))}
    </div>
  );
}

function priorityClassFor(p: ProsoponNode["lifecycle"]["priority"]): string {
  switch (p) {
    case "blocking":
      return "pgl-prio-blocking";
    case "urgent":
      return "pgl-prio-urgent";
    case "ambient":
      return "pgl-prio-ambient";
    default:
      return "";
  }
}
```

- [ ] **Step 7.7: Typecheck (will fail — components don't exist yet)**

Run: `bun run typecheck`
Expected: fails with ~24 "cannot find module '../components/X'" errors. That's intentional — next task writes them.

- [ ] **Step 7.8: Commit**

```bash
git add core/prosopon/crates/prosopon-compositor-glass/web/src/tokens \
        core/prosopon/crates/prosopon-compositor-glass/web/src/registry \
        core/prosopon/crates/prosopon-compositor-glass/web/src/components/Node.tsx
git commit -m "feat(glass-web): tokens + registry + NodeView (intent components pending)

Arcan Glass tokens inlined (no Tailwind runtime). INTENT_REGISTRY is a total
map over Intent['type']; missing variants fall through to Fallback. NodeView
applies hydrated intent + renders children + action bar. Components land in
the next commit."
```

---

## Task 8: Intent components — batch A (Textual, Entity, Structural, Meta)

**Files:** one `.tsx` per component, listed below.

- [ ] **Step 8.1: Prose**

```tsx
// src/components/Prose.tsx
import type { Intent } from "../runtime/types";
export function Prose({ intent }: { intent: Extract<Intent, { type: "prose" }> }) {
  return <p className="pgl-prose">{intent.text}</p>;
}
```

- [ ] **Step 8.2: Code**

```tsx
// src/components/Code.tsx
// Lightweight code block — no syntax highlight in v0.2 (Shiki pull-in deferred).
import type { Intent } from "../runtime/types";
export function Code({ intent }: { intent: Extract<Intent, { type: "code" }> }) {
  return (
    <pre className="pgl-code" data-lang={intent.lang}>
      <code>{intent.source}</code>
    </pre>
  );
}
```

- [ ] **Step 8.3: Math**

```tsx
// src/components/Math.tsx
import type { Intent } from "../runtime/types";
export function Math({ intent }: { intent: Extract<Intent, { type: "math" }> }) {
  return <code className="pgl-mono pgl-dim">{intent.source}</code>;
}
```

- [ ] **Step 8.4: EntityRef**

```tsx
// src/components/EntityRef.tsx
import type { Intent } from "../runtime/types";
export function EntityRef({ intent }: { intent: Extract<Intent, { type: "entity_ref" }> }) {
  const label = intent.label ?? `${intent.kind}:${intent.id}`;
  return (
    <span className="pgl-mono" style={{ color: "var(--pgl-accent)" }}>
      → {label}
    </span>
  );
}
```

- [ ] **Step 8.5: Link**

```tsx
// src/components/Link.tsx
import type { Intent } from "../runtime/types";
export function Link({ intent }: { intent: Extract<Intent, { type: "link" }> }) {
  return (
    <a href={intent.href} style={{ color: "var(--pgl-accent)" }} rel="noopener">
      {intent.label ?? intent.href}
    </a>
  );
}
```

- [ ] **Step 8.6: Citation**

```tsx
// src/components/Citation.tsx
import type { Intent } from "../runtime/types";
export function Citation({ intent }: { intent: Extract<Intent, { type: "citation" }> }) {
  return (
    <cite className="pgl-dim pgl-mono">
      cite: {intent.source}
      {intent.anchor ? `#${intent.anchor}` : ""}
    </cite>
  );
}
```

- [ ] **Step 8.7: Section**

```tsx
// src/components/Section.tsx
import type { Intent } from "../runtime/types";
export function Section({ intent }: { intent: Extract<Intent, { type: "section" }> }) {
  if (!intent.title) return null;
  return (
    <div className="pgl-section">
      <h2 className="pgl-section-title">{intent.title}</h2>
      <div className="pgl-divider" />
    </div>
  );
}
```

- [ ] **Step 8.8: Divider**

```tsx
// src/components/Divider.tsx
export function Divider() {
  return <div className="pgl-divider" role="separator" />;
}
```

- [ ] **Step 8.9: Empty**

```tsx
// src/components/Empty.tsx
export function Empty() {
  return null;
}
```

- [ ] **Step 8.10: Custom**

```tsx
// src/components/Custom.tsx
import type { Intent } from "../runtime/types";
export function Custom({ intent }: { intent: Extract<Intent, { type: "custom" }> }) {
  return (
    <div className="pgl-fallback pgl-mono">
      [{intent.kind}] {JSON.stringify(intent.payload)}
    </div>
  );
}
```

- [ ] **Step 8.11: Run typecheck (half of errors should clear)**

Run: `bun run typecheck`
Expected: ~15 remaining "cannot find module" errors (the ones we write in Task 9).

- [ ] **Step 8.12: Commit**

```bash
git add core/prosopon/crates/prosopon-compositor-glass/web/src/components/{Prose,Code,Math,EntityRef,Link,Citation,Section,Divider,Empty,Custom}.tsx
git commit -m "feat(glass-web): batch A intent components (textual/entity/structural/meta)"
```

---

## Task 9: Intent components — batch B (Process, Decision, Live, Media)

- [ ] **Step 9.1: Progress**

```tsx
// src/components/Progress.tsx
import type { Intent } from "../runtime/types";
export function Progress({ intent }: { intent: Extract<Intent, { type: "progress" }> }) {
  const pct = clamp(intent.pct ?? 0, 0, 1);
  return (
    <div className="pgl-progress">
      <div className="pgl-progress-track">
        <div className="pgl-progress-fill" style={{ width: `${(pct * 100).toFixed(0)}%` }} />
      </div>
      <span className="pgl-dim pgl-mono">{(pct * 100).toFixed(0)}%</span>
      {intent.label ? <span className="pgl-dim">{intent.label}</span> : null}
    </div>
  );
}
function clamp(n: number, lo: number, hi: number) { return n < lo ? lo : n > hi ? hi : n; }
```

- [ ] **Step 9.2: ToolCall**

```tsx
// src/components/ToolCall.tsx
import type { Intent } from "../runtime/types";
export function ToolCall({ intent }: { intent: Extract<Intent, { type: "tool_call" }> }) {
  return (
    <div className="pgl-tool-call">
      <span style={{ color: "var(--pgl-accent)" }}>⚙ </span>
      <strong>{intent.name}</strong>({jsonPreview(intent.args)})
    </div>
  );
}
function jsonPreview(v: unknown): string {
  const s = JSON.stringify(v);
  return s.length > 80 ? `${s.slice(0, 79)}…` : s;
}
```

- [ ] **Step 9.3: ToolResult**

```tsx
// src/components/ToolResult.tsx
import type { Intent } from "../runtime/types";
export function ToolResult({ intent }: { intent: Extract<Intent, { type: "tool_result" }> }) {
  const cls = intent.success ? "pgl-tool-result-ok" : "pgl-tool-result-err";
  const marker = intent.success ? "✓" : "✗";
  const s = JSON.stringify(intent.payload);
  return (
    <div className={`pgl-mono ${cls}`}>
      {marker} {s.length > 120 ? `${s.slice(0, 119)}…` : s}
    </div>
  );
}
```

- [ ] **Step 9.4: Choice**

```tsx
// src/components/Choice.tsx
import type { Intent } from "../runtime/types";
import { useRegistry } from "../registry/context";
export function Choice({ intent, nodeId }: { intent: Extract<Intent, { type: "choice" }>; nodeId: string }) {
  const { emitAction } = useRegistry();
  return (
    <div className="pgl-card pgl-flex-col">
      <strong>{intent.prompt}</strong>
      <div className="pgl-flex-col">
        {intent.options.map((o) => (
          <button
            key={o.id}
            onClick={() =>
              emitAction({ slot: o.id, source: nodeId, kind: { type: "choose", option_id: o.id } })
            }
          >
            {o.default ? "● " : "○ "}
            {o.label}
            {o.description ? <span className="pgl-dim"> — {o.description}</span> : null}
          </button>
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 9.5: Confirm**

```tsx
// src/components/Confirm.tsx
import type { Intent } from "../runtime/types";
import { useRegistry } from "../registry/context";
export function Confirm({ intent, nodeId }: { intent: Extract<Intent, { type: "confirm" }>; nodeId: string }) {
  const { emitAction } = useRegistry();
  const color = severityColor(intent.severity ?? "info");
  return (
    <div className="pgl-card" style={{ borderColor: color }}>
      <p style={{ color }}>? {intent.message}</p>
      <div className="pgl-flex-row">
        <button onClick={() => emitAction({ slot: "confirm", source: nodeId, kind: { type: "confirm", accepted: true } })}>
          Confirm
        </button>
        <button onClick={() => emitAction({ slot: "confirm", source: nodeId, kind: { type: "confirm", accepted: false } })}>
          Cancel
        </button>
      </div>
    </div>
  );
}
function severityColor(s: string): string {
  switch (s) {
    case "danger": return "var(--pgl-danger)";
    case "warning": return "var(--pgl-warning)";
    case "notice": return "var(--pgl-accent)";
    default: return "var(--pgl-text-dim)";
  }
}
```

- [ ] **Step 9.6: Input**

```tsx
// src/components/Input.tsx
import type { Intent } from "../runtime/types";
import { useRegistry } from "../registry/context";
import { useState } from "preact/hooks";
export function Input({ intent, nodeId }: { intent: Extract<Intent, { type: "input" }>; nodeId: string }) {
  const { emitAction } = useRegistry();
  const [value, setValue] = useState<string>(String(intent.default ?? ""));
  return (
    <div className="pgl-flex-col">
      <label>{intent.prompt}</label>
      <input
        value={value}
        onInput={(e) => setValue((e.currentTarget as HTMLInputElement).value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") emitAction({ slot: "input", source: nodeId, kind: { type: "input", value } });
        }}
      />
    </div>
  );
}
```

- [ ] **Step 9.7: Signal**

```tsx
// src/components/Signal.tsx
import type { Intent } from "../runtime/types";
import { useRegistry } from "../registry/context";
import { previewSignal } from "../util/format";
export function Signal({ intent }: { intent: Extract<Intent, { type: "signal" }> }) {
  const { scene } = useRegistry();
  const value = scene.signals[intent.topic];
  return (
    <div className="pgl-mono pgl-dim">
      ~ {intent.topic} = {value ? previewSignal(value) : "<pending>"}
    </div>
  );
}
```

- [ ] **Step 9.8: Stream (Pretext-measured tail lives in goldens test — v0.2 ships basic append)**

```tsx
// src/components/Stream.tsx
import type { Intent } from "../runtime/types";
export function Stream({ intent }: { intent: Extract<Intent, { type: "stream" }> }) {
  return (
    <div className="pgl-mono pgl-dim" data-stream-id={intent.id}>
      ⟳ stream:{intent.id} ({intent.kind})
    </div>
  );
}
```

Note: wiring live stream chunks into the `<Stream>` body is Task 11.

- [ ] **Step 9.9: Image / Audio / Video**

```tsx
// src/components/Image.tsx
import type { Intent } from "../runtime/types";
export function Image({ intent }: { intent: Extract<Intent, { type: "image" }> }) {
  return <img src={intent.uri} alt={intent.alt ?? ""} style={{ maxWidth: "100%" }} />;
}
```

```tsx
// src/components/Audio.tsx
import type { Intent } from "../runtime/types";
export function Audio({ intent }: { intent: Extract<Intent, { type: "audio" }> }) {
  if (intent.uri) return <audio src={intent.uri} controls />;
  return <div className="pgl-mono pgl-dim">♪ live audio (voice: {intent.voice ?? "default"})</div>;
}
```

```tsx
// src/components/Video.tsx
import type { Intent } from "../runtime/types";
export function Video({ intent }: { intent: Extract<Intent, { type: "video" }> }) {
  return <video src={intent.uri} poster={intent.poster} controls style={{ maxWidth: "100%" }} />;
}
```

- [ ] **Step 9.10: Write `util/format.ts`**

```ts
// src/util/format.ts
import type { SignalValue } from "../runtime/types";
export function previewSignal(v: SignalValue): string {
  switch (v.type) {
    case "scalar":
      return String(v.value);
    case "series":
      return `[${v.points.length} pts]`;
    case "categorical":
      return v.label;
    case "vector":
      return `[${v.components.join(", ")}]`;
    default:
      return "<?>";
  }
}
```

- [ ] **Step 9.11: Totality test — every intent renders something**

Create `tests/totality.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { render } from "@testing-library/preact";
import { RegistryContext } from "../src/registry/context";
import { SignalBus } from "../src/runtime/signal-bus";
import { renderIntent } from "../src/registry/intents";
import type { Intent } from "../src/runtime/types";

function wrap(ui: preact.ComponentChildren) {
  const bus = new SignalBus();
  const scene = { id: "s", root: {} as never, signals: {}, hints: {} };
  return render(
    <RegistryContext.Provider value={{ scene: scene as never, bus, emitAction: () => {} }}>
      {ui}
    </RegistryContext.Provider>,
  );
}

const INTENTS: Intent[] = [
  { type: "prose", text: "hi" },
  { type: "code", lang: "rust", source: "fn main() {}" },
  { type: "math", source: "E=mc^2" },
  { type: "entity_ref", kind: "concept", id: "x", label: "X" },
  { type: "link", href: "https://example.org" },
  { type: "citation", source: "RFC-0001" },
  { type: "signal", topic: "t" },
  { type: "stream", id: "s", kind: "text" },
  { type: "choice", prompt: "?", options: [{ id: "a", label: "A" }] },
  { type: "confirm", message: "sure?" },
  { type: "input", prompt: "name", input: { kind: "text" } },
  { type: "tool_call", name: "search", args: {} },
  { type: "tool_result", success: true, payload: "ok" },
  { type: "progress", pct: 0.5 },
  { type: "section", title: "S" },
  { type: "divider" },
  { type: "image", uri: "/x.png" },
  { type: "audio", voice: "default" },
  { type: "video", uri: "/x.mp4" },
  { type: "empty" },
  { type: "custom", kind: "x", payload: null },
];

describe("intent registry totality", () => {
  for (const intent of INTENTS) {
    it(`renders ${intent.type} without throwing`, () => {
      expect(() => wrap(renderIntent(intent, "n1"))).not.toThrow();
    });
  }
});
```

- [ ] **Step 9.12: Run tests**

Run: `bun run test`
Expected: all prior tests + 21 new totality tests pass.

- [ ] **Step 9.13: Typecheck**

Run: `bun run typecheck`
Expected: exits 0.

- [ ] **Step 9.14: Commit**

```bash
git add core/prosopon/crates/prosopon-compositor-glass/web/src/components \
        core/prosopon/crates/prosopon-compositor-glass/web/src/util/format.ts \
        core/prosopon/crates/prosopon-compositor-glass/web/tests/totality.test.ts
git commit -m "feat(glass-web): batch B intent components + totality test

Progress / ToolCall / ToolResult / Choice / Confirm / Input / Signal /
Stream / Image / Audio / Video now render. Totality test asserts every
listed Intent variant renders without throwing. Field / Locus / Formation
fall through to Fallback until the field compositor (BRO-774)."
```

---

## Task 10: Axum server + embedded assets (Rust)

**Files:**
- Modify: `core/prosopon/crates/prosopon-compositor-glass/src/server.rs`
- Modify: `core/prosopon/crates/prosopon-compositor-glass/src/assets.rs`
- Create: `core/prosopon/crates/prosopon-compositor-glass/src/bin/prosopon-glass.rs`
- Create: `core/prosopon/crates/prosopon-compositor-glass/tests/server_handshake.rs`

- [ ] **Step 10.1: Write the server handshake test FIRST**

```rust
// tests/server_handshake.rs
//! Integration test: start a GlassServer on 127.0.0.1:0, connect via WS, send
//! a SceneReset through the compositor, and assert it arrives on the wire.

use futures::StreamExt;
use prosopon_compositor_glass::{GlassCompositor, GlassServer, GlassServerConfig};
use prosopon_core::{Intent, Node, ProsoponEvent, Scene};
use prosopon_runtime::Compositor;
use tokio_tungstenite::connect_async;

#[tokio::test]
async fn ws_client_receives_envelopes() {
    let server = GlassServer::bind(GlassServerConfig {
        addr: "127.0.0.1:0".parse().unwrap(),
    })
    .await
    .expect("bind succeeds");
    let url = format!("ws://{}/ws", server.local_addr());
    let mut compositor = GlassCompositor::new(server.fanout());
    tokio::spawn(server.serve());

    // Allow the server to start accepting.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let (mut ws, _resp) = connect_async(&url).await.expect("ws connect");

    let scene = Scene::new(Node::new(Intent::Prose { text: "hello".into() }));
    compositor
        .apply(&ProsoponEvent::SceneReset { scene })
        .unwrap();

    let msg = tokio::time::timeout(std::time::Duration::from_secs(2), ws.next())
        .await
        .expect("got a message in time")
        .expect("stream not closed")
        .expect("ws frame");
    let text = msg.into_text().unwrap();
    assert!(text.contains("\"scene_reset\""), "frame was: {text}");
}
```

- [ ] **Step 10.2: Run — expect failure (linker, undefined symbols)**

Run: `cd core/prosopon && cargo test -p prosopon-compositor-glass --test server_handshake`
Expected: compilation error — `GlassServer`/`GlassServerConfig` not implemented.

- [ ] **Step 10.3: Implement `assets.rs`**

```rust
// src/assets.rs
//! Embeds the compiled TypeScript bundle from `web/dist/` via `include_dir!`.
//! At runtime, HTTP GETs for any path inside the bundle are served with the
//! appropriate MIME type. Missing `web/dist/` (pre-`bun run build`) is tolerated
//! — the server returns a helpful HTML shell telling the operator to build.

use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, Response, StatusCode};
use include_dir::{Dir, include_dir};

static WEB: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/web/dist");

const FALLBACK_HTML: &str = r#"<!doctype html>
<html><head><meta charset="utf-8"><title>prosopon-glass</title></head>
<body><h1>prosopon-compositor-glass</h1>
<p>The web bundle is missing. From the crate root, run:</p>
<pre>cd web &amp;&amp; bun install &amp;&amp; bun run build</pre>
<p>Then restart this server.</p></body></html>"#;

pub fn serve_asset(path: &str) -> Response<Body> {
    let normalized = path.trim_start_matches('/');
    let normalized = if normalized.is_empty() { "index.html" } else { normalized };
    match WEB.get_file(normalized) {
        Some(file) => {
            let mime = mime_guess::from_path(normalized).first_or_octet_stream();
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_str(mime.as_ref()).unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            let body = Body::from(file.contents());
            let mut resp = Response::new(body);
            *resp.headers_mut() = headers;
            resp
        }
        None if normalized == "index.html" => {
            let mut resp = Response::new(Body::from(FALLBACK_HTML));
            resp.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            resp
        }
        None => {
            let mut resp = Response::new(Body::empty());
            *resp.status_mut() = StatusCode::NOT_FOUND;
            resp
        }
    }
}
```

- [ ] **Step 10.4: Implement `server.rs`**

```rust
// src/server.rs
//! axum server exposing:
//!   GET  /                 — embedded index.html
//!   GET  /assets/...       — embedded static assets
//!   GET  /ws               — WebSocket, bidi envelope stream
//!   GET  /events           — SSE, one-way JSONL envelope stream
//!   GET  /schema/scene     — prosopon-core scene schema (application/json)
//!   GET  /schema/event     — prosopon-core event schema
//!
//! The server is designed so the downstream `prosopon-daemon` (BRO-768) can
//! reuse it verbatim by constructing the router and adding its own middleware.

use axum::body::Body;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::header;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use futures::stream::{Stream, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::assets::serve_asset;
use crate::fanout::EnvelopeFanout;

/// Server configuration.
pub struct GlassServerConfig {
    pub addr: SocketAddr,
}

/// Runtime state shared by every request handler.
#[derive(Clone)]
struct AppState {
    fanout: Arc<EnvelopeFanout>,
}

/// Bound-but-not-yet-serving HTTP server.
pub struct GlassServer {
    listener: TcpListener,
    fanout: Arc<EnvelopeFanout>,
    local_addr: SocketAddr,
}

impl GlassServer {
    /// Bind a TCP listener. Returns immediately; the caller drives `serve()`.
    pub async fn bind(config: GlassServerConfig) -> std::io::Result<Self> {
        let listener = TcpListener::bind(config.addr).await?;
        let local_addr = listener.local_addr()?;
        Ok(Self {
            listener,
            fanout: Arc::new(EnvelopeFanout::new()),
            local_addr,
        })
    }

    /// Clone a fanout handle for registering a compositor.
    pub fn fanout(&self) -> EnvelopeFanout {
        (*self.fanout).clone()
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Run the server until the task is cancelled.
    pub async fn serve(self) -> std::io::Result<()> {
        let state = AppState { fanout: self.fanout };
        let router = Router::new()
            .route("/", get(index))
            .route("/assets/{*path}", get(asset))
            .route("/ws", get(ws_upgrade))
            .route("/events", get(sse_stream))
            .route("/schema/scene", get(schema_scene))
            .route("/schema/event", get(schema_event))
            .layer(TraceLayer::new_for_http())
            .with_state(state);
        axum::serve(self.listener, router).await
    }
}

async fn index() -> Response {
    serve_asset("index.html").into_response()
}

async fn asset(axum::extract::Path(path): axum::extract::Path<String>) -> Response {
    serve_asset(&path).into_response()
}

async fn ws_upgrade(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| ws_session(state, socket))
}

async fn ws_session(state: AppState, mut socket: WebSocket) {
    let mut rx = state.fanout.subscribe();
    loop {
        tokio::select! {
            maybe_env = rx.recv() => {
                match maybe_env {
                    Ok(envelope) => {
                        let frame = match serde_json::to_string(&envelope) {
                            Ok(s) => s,
                            Err(e) => {
                                tracing::warn!(target: "prosopon::glass", "encode failed: {e}");
                                continue;
                            }
                        };
                        if socket.send(Message::Text(frame.into())).await.is_err() {
                            return;
                        }
                    }
                    Err(crate::fanout::FanoutError::Lagged(n)) => {
                        tracing::warn!(target: "prosopon::glass", "ws client lagged {n}; closing");
                        return;
                    }
                    Err(_) => return,
                }
            }
            msg = socket.recv() => {
                // Client→server: ActionEmitted envelopes. Not implemented in v0.2;
                // we ACK by ignoring. BRO-778 will wire these into the runtime.
                match msg {
                    None | Some(Err(_)) => return,
                    Some(Ok(Message::Close(_))) => return,
                    Some(Ok(_)) => continue,
                }
            }
        }
    }
}

async fn sse_stream(State(state): State<AppState>) -> Sse<impl Stream<Item = Result<SseEvent, std::convert::Infallible>>> {
    let mut rx = state.fanout.subscribe();
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(envelope) => {
                    let data = serde_json::to_string(&envelope).unwrap_or_default();
                    yield Ok(SseEvent::default().event("envelope").data(data));
                }
                Err(_) => break,
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn schema_scene() -> Response {
    let body = prosopon_core::scene_schema_json();
    ([(header::CONTENT_TYPE, "application/json")], body).into_response()
}

async fn schema_event() -> Response {
    let body = prosopon_core::event_schema_json();
    ([(header::CONTENT_TYPE, "application/json")], body).into_response()
}
```

- [ ] **Step 10.5: Add `async-stream` to crate deps**

Edit `core/prosopon/crates/prosopon-compositor-glass/Cargo.toml`, append:

```toml
async-stream = "0.3"
```

- [ ] **Step 10.6: Re-export from `lib.rs`**

Edit `src/lib.rs`:

```rust
pub use server::{GlassServer, GlassServerConfig};
```

is already there; leave as-is. If `cargo check` complains about `assets` not being `pub`, that's fine.

- [ ] **Step 10.7: Build web bundle so `include_dir!` has something to embed**

Run:
```
cd core/prosopon/crates/prosopon-compositor-glass/web
bun run build
```

Expected: `dist/index.js` + `dist/assets/glass.css` produced.

- [ ] **Step 10.8: Implement the `prosopon-glass` CLI**

Create `src/bin/prosopon-glass.rs`:

```rust
//! `prosopon-glass` — stand up a local glass compositor server against a
//! fixture envelope stream. Useful for dev, goldens, and demo pages.

use clap::{Parser, Subcommand};
use prosopon_compositor_glass::{GlassCompositor, GlassServer, GlassServerConfig};
use prosopon_core::ProsoponEvent;
use prosopon_runtime::Compositor;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "prosopon-glass", about = "Prosopon glass compositor server")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Serve the compositor on an HTTP port, optionally replaying a fixture.
    Serve {
        #[arg(long, default_value = "127.0.0.1:4321")]
        addr: String,
        /// Path to a JSONL fixture of Envelope events to replay on start.
        #[arg(long)]
        fixture: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Serve { addr, fixture } => {
            let config = GlassServerConfig { addr: addr.parse()? };
            let server = GlassServer::bind(config).await?;
            println!("prosopon-glass serving at http://{}/", server.local_addr());
            let mut compositor = GlassCompositor::new(server.fanout());
            if let Some(path) = fixture {
                replay_fixture(&mut compositor, &path)?;
            }
            server.serve().await?;
            Ok(())
        }
    }
}

fn replay_fixture(c: &mut GlassCompositor, path: &std::path::Path) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(path)?;
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() { continue; }
        let event: ProsoponEvent = serde_json::from_str(line)
            .map_err(|e| anyhow::anyhow!("line {}: {e}", n + 1))?;
        c.apply(&event)?;
    }
    Ok(())
}
```

Add `anyhow` to crate deps (under workspace):

```toml
anyhow = { workspace = true }
```

- [ ] **Step 10.9: Run the handshake test**

Run: `cd core/prosopon && cargo test -p prosopon-compositor-glass --test server_handshake`
Expected: PASS.

- [ ] **Step 10.10: Run full workspace tests**

Run: `cargo test --workspace`
Expected: 52+ tests green.

- [ ] **Step 10.11: Smoke-check the CLI manually**

Run: `cargo run -p prosopon-compositor-glass --bin prosopon-glass -- serve --addr 127.0.0.1:4321`
Then open http://localhost:4321/ in a browser. The empty Preact app from Task 2 should load with Arcan Glass tokens. Ctrl-C to stop.

- [ ] **Step 10.12: Commit**

```bash
git add core/prosopon/crates/prosopon-compositor-glass/src \
        core/prosopon/crates/prosopon-compositor-glass/Cargo.toml \
        core/prosopon/crates/prosopon-compositor-glass/tests/server_handshake.rs
git commit -m "feat(glass): axum server + embedded assets + CLI (BRO-767)

GlassServer bounds an HTTP listener, fans envelopes over WS + SSE, serves
the embedded Preact bundle from include_dir!, and publishes the IR schema.
WS client→server messages are ignored in v0.2 (action round-trip is BRO-778).
The prosopon-glass binary supports fixture replay for demos."
```

---

## Task 11: Wire the web transport layer (WS client + SSE fallback)

**Files:**
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/runtime/transport.ts`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/src/actions/emit.ts`
- Modify: `core/prosopon/crates/prosopon-compositor-glass/web/src/app.tsx`

- [ ] **Step 11.1: Write `transport.ts`**

```ts
// src/runtime/transport.ts
// WebSocket client with SSE fallback. Both deliver Envelope frames, one per
// message. On WS open, re-emit a "connected" signal; on close, fall back to
// SSE after a short backoff. Subscribers get the decoded Envelope.

import type { Envelope } from "./types";

export type TransportState = "connecting" | "open" | "reconnecting" | "closed";

export interface Transport {
  onEnvelope(cb: (env: Envelope) => void): () => void;
  onState(cb: (state: TransportState) => void): () => void;
  send(frame: unknown): void;
  close(): void;
}

const BACKOFF_MS = [250, 500, 1000, 2000, 5000] as const;

export function connectTransport(baseUrl: string): Transport {
  const envCbs = new Set<(env: Envelope) => void>();
  const stateCbs = new Set<(s: TransportState) => void>();
  let ws: WebSocket | null = null;
  let state: TransportState = "connecting";
  let attempt = 0;
  let closed = false;

  function setState(s: TransportState) {
    state = s;
    for (const cb of stateCbs) cb(s);
  }

  function connect() {
    if (closed) return;
    ws = new WebSocket(`${baseUrl.replace(/^http/, "ws")}/ws`);
    ws.onopen = () => {
      attempt = 0;
      setState("open");
    };
    ws.onmessage = (ev) => {
      try {
        const envelope = JSON.parse(ev.data) as Envelope;
        for (const cb of envCbs) cb(envelope);
      } catch (e) {
        console.warn("prosopon-glass: dropping malformed envelope", e);
      }
    };
    ws.onclose = () => {
      ws = null;
      if (closed) return;
      setState("reconnecting");
      const delay = BACKOFF_MS[Math.min(attempt, BACKOFF_MS.length - 1)];
      attempt += 1;
      setTimeout(connect, delay);
    };
    ws.onerror = () => ws?.close();
  }

  connect();

  return {
    onEnvelope(cb) { envCbs.add(cb); return () => envCbs.delete(cb); },
    onState(cb) { stateCbs.add(cb); cb(state); return () => stateCbs.delete(cb); },
    send(frame) {
      if (ws?.readyState === WebSocket.OPEN) ws.send(JSON.stringify(frame));
    },
    close() { closed = true; ws?.close(); setState("closed"); },
  };
}
```

- [ ] **Step 11.2: Write `actions/emit.ts`**

```ts
// src/actions/emit.ts
// Helper for components to build and dispatch ActionEmitted frames.
import type { Transport } from "../runtime/transport";

export function makeActionEmitter(transport: Transport, sessionId: () => string) {
  let seq = 1;
  return (slot: string, source: string, kind: unknown) => {
    transport.send({
      version: 1,
      session_id: sessionId(),
      seq: seq++,
      ts: new Date().toISOString(),
      event: { type: "action_emitted", slot, source, kind },
    });
  };
}
```

- [ ] **Step 11.3: Wire `app.tsx` end-to-end**

Replace `src/app.tsx`:

```tsx
// src/app.tsx
import { useEffect, useState } from "preact/hooks";
import { RegistryContext } from "./registry/context";
import { createSceneStore } from "./runtime/scene-store";
import { SignalBus } from "./runtime/signal-bus";
import { connectTransport, type TransportState } from "./runtime/transport";
import { makeActionEmitter } from "./actions/emit";
import { NodeView } from "./components/Node";
import type { Scene } from "./runtime/types";

const EMPTY_SCENE: Scene = {
  id: "empty",
  root: {
    id: "root",
    intent: { type: "prose", text: "Waiting for envelopes…" },
    children: [],
    bindings: [],
    actions: [],
    attrs: {},
    lifecycle: { priority: "normal", status: { type: "active" } },
  },
  signals: {},
  hints: {},
};

export function App() {
  const [store] = useState(() => createSceneStore(EMPTY_SCENE));
  const [bus] = useState(() => new SignalBus());
  const [state, setState] = useState<TransportState>("connecting");
  const [lastSession, setLastSession] = useState("");

  useEffect(() => {
    const base = typeof window !== "undefined" ? window.location.origin : "";
    const t = connectTransport(base);
    const offState = t.onState(setState);
    const offEnv = t.onEnvelope((env) => {
      setLastSession(env.session_id);
      store.apply(env.event);
      if (env.event.type === "signal_changed") bus.publish(env.event.topic, env.event.value);
    });
    return () => {
      offState();
      offEnv();
      t.close();
    };
  }, [store, bus]);

  const emitAction = makeActionEmitter(
    // biome-ignore lint/style/noNonNullAssertion: transport is always present
    null!,
    () => lastSession,
  );

  return (
    <div className="pgl-shell">
      <header className="pgl-flex-row" style={{ justifyContent: "space-between" }}>
        <strong>Prosopon · Glass</strong>
        <span className="pgl-dim pgl-mono">ws: {state}</span>
      </header>
      <main>
        <RegistryContext.Provider value={{ scene: store.signal.value, bus, emitAction }}>
          <NodeView node={store.signal.value.root} />
        </RegistryContext.Provider>
      </main>
    </div>
  );
}
```

(The `null!` is flagged: resolve by threading the transport handle. Step 11.4 fixes this.)

- [ ] **Step 11.4: Thread the transport into RegistryContext correctly**

Replace `useEffect`'s body + return so the `emitAction` uses the live transport:

```tsx
  const [transport, setTransport] = useState<ReturnType<typeof connectTransport> | null>(null);

  useEffect(() => {
    const base = typeof window !== "undefined" ? window.location.origin : "";
    const t = connectTransport(base);
    setTransport(t);
    const offState = t.onState(setState);
    const offEnv = t.onEnvelope((env) => {
      setLastSession(env.session_id);
      store.apply(env.event);
      if (env.event.type === "signal_changed") bus.publish(env.event.topic, env.event.value);
    });
    return () => {
      offState();
      offEnv();
      t.close();
    };
  }, [store, bus]);

  const emitAction = transport ? makeActionEmitter(transport, () => lastSession) : () => {};
```

- [ ] **Step 11.5: Re-build & typecheck**

Run:
```
cd core/prosopon/crates/prosopon-compositor-glass/web
bun run typecheck
bun run build
```

Expected: both clean.

- [ ] **Step 11.6: Manual smoke: fixture replay**

Create a fixture JSONL (inline, one event per line) at
`core/prosopon/crates/prosopon-compositor-glass/tests/fixtures/demo_scene.jsonl`:

```jsonl
{"type":"scene_reset","scene":{"id":"s1","root":{"id":"root","intent":{"type":"section","title":"Prosopon demo"},"children":[{"id":"a","intent":{"type":"prose","text":"A prose child."},"children":[],"bindings":[],"actions":[],"attrs":{},"lifecycle":{"priority":"normal","status":{"type":"active"}}},{"id":"b","intent":{"type":"tool_call","name":"search","args":{"q":"prosopon"}},"children":[],"bindings":[],"actions":[],"attrs":{},"lifecycle":{"priority":"normal","status":{"type":"active"}}},{"id":"c","intent":{"type":"progress","pct":0.66,"label":"scoring"},"children":[],"bindings":[],"actions":[],"attrs":{},"lifecycle":{"priority":"normal","status":{"type":"active"}}}],"bindings":[],"actions":[],"attrs":{},"lifecycle":{"priority":"normal","status":{"type":"active"}}},"signals":{},"hints":{}}}
```

Then run:

```
cd core/prosopon
cargo run -p prosopon-compositor-glass --bin prosopon-glass -- \
  serve --addr 127.0.0.1:4321 \
  --fixture crates/prosopon-compositor-glass/tests/fixtures/demo_scene.jsonl
```

Open http://localhost:4321/. You should see the demo scene rendered via glass tokens: a section header, prose, a tool_call line, and a progress bar.

- [ ] **Step 11.7: Commit**

```bash
git add core/prosopon/crates/prosopon-compositor-glass/web/src/runtime/transport.ts \
        core/prosopon/crates/prosopon-compositor-glass/web/src/actions/emit.ts \
        core/prosopon/crates/prosopon-compositor-glass/web/src/app.tsx \
        core/prosopon/crates/prosopon-compositor-glass/tests/fixtures
git commit -m "feat(glass-web): WS transport + action emitter + end-to-end demo

App connects to /ws, falls back to reconnect backoff, and applies every
envelope through the scene store. Actions round-trip through the same
WebSocket. Fixture replay verified manually against the canonical demo
scene."
```

---

## Task 12: Golden-file snapshot tests (parity with text compositor)

**Files:**
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/tests/goldens.test.ts`
- Create: `core/prosopon/crates/prosopon-compositor-glass/web/tests/fixtures/*.json`

- [ ] **Step 12.1: Write fixtures (JSON — unified across Rust + TS sides)**

Create `tests/fixtures/demo_scene.json`:

```json
{
  "id": "demo",
  "root": {
    "id": "root",
    "intent": { "type": "section", "title": "Prosopon demo" },
    "children": [
      {
        "id": "p1",
        "intent": { "type": "prose", "text": "A prose child." },
        "children": [], "bindings": [], "actions": [], "attrs": {},
        "lifecycle": { "priority": "normal", "status": { "type": "active" } }
      },
      {
        "id": "tc",
        "intent": {
          "type": "tool_call",
          "name": "search",
          "args": { "q": "prosopon" }
        },
        "children": [], "bindings": [], "actions": [], "attrs": {},
        "lifecycle": { "priority": "normal", "status": { "type": "active" } }
      },
      {
        "id": "pr",
        "intent": { "type": "progress", "pct": 0.66, "label": "scoring" },
        "children": [], "bindings": [], "actions": [], "attrs": {},
        "lifecycle": { "priority": "normal", "status": { "type": "active" } }
      }
    ],
    "bindings": [], "actions": [], "attrs": {},
    "lifecycle": { "priority": "normal", "status": { "type": "active" } }
  },
  "signals": {},
  "hints": {}
}
```

Create `tests/fixtures/tool_flow.json` and `tests/fixtures/streaming_tokens.json` with similar shape. (Exact contents: reuse the demo scene minus the progress bar + a Stream intent for `streaming_tokens.json`.)

- [ ] **Step 12.2: Write goldens test**

```ts
// tests/goldens.test.ts
import { describe, expect, it } from "vitest";
import { render } from "@testing-library/preact";
import { RegistryContext } from "../src/registry/context";
import { SignalBus } from "../src/runtime/signal-bus";
import { NodeView } from "../src/components/Node";
import type { Scene } from "../src/runtime/types";

import demo from "./fixtures/demo_scene.json";
import toolFlow from "./fixtures/tool_flow.json";
import stream from "./fixtures/streaming_tokens.json";

function renderScene(scene: Scene): string {
  const bus = new SignalBus();
  const { container } = render(
    <RegistryContext.Provider value={{ scene, bus, emitAction: () => {} }}>
      <NodeView node={scene.root} />
    </RegistryContext.Provider>,
  );
  return container.outerHTML;
}

describe("glass compositor goldens", () => {
  it.each([
    ["demo_scene", demo as unknown as Scene],
    ["tool_flow", toolFlow as unknown as Scene],
    ["streaming_tokens", stream as unknown as Scene],
  ])("%s matches snapshot", (_name, scene) => {
    expect(renderScene(scene)).toMatchSnapshot();
  });
});
```

- [ ] **Step 12.3: Also mirror the same fixtures on the Rust side for the text compositor**

Create `core/prosopon/crates/prosopon-compositor-text/tests/goldens.rs`:

```rust
//! Golden-file parity: the same fixture scenes used by the glass goldens are
//! rendered through the text compositor with snapshot testing. Ensures
//! cross-surface coherence.

use prosopon_compositor_text::{RenderOptions, render_scene};
use prosopon_core::Scene;

fn load(name: &str) -> Scene {
    let path = format!(
        "{}/../prosopon-compositor-glass/web/tests/fixtures/{}.json",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    let text = std::fs::read_to_string(&path).expect("fixture exists");
    serde_json::from_str(&text).expect("fixture parses")
}

#[test]
fn demo_scene_snapshot() {
    let scene = load("demo_scene");
    let out = render_scene(&scene, 80, &RenderOptions::plain());
    insta::assert_snapshot!(out);
}

#[test]
fn tool_flow_snapshot() {
    let scene = load("tool_flow");
    let out = render_scene(&scene, 80, &RenderOptions::plain());
    insta::assert_snapshot!(out);
}

#[test]
fn streaming_tokens_snapshot() {
    let scene = load("streaming_tokens");
    let out = render_scene(&scene, 80, &RenderOptions::plain());
    insta::assert_snapshot!(out);
}
```

Append `[dev-dependencies] insta = { workspace = true }` to `core/prosopon/crates/prosopon-compositor-text/Cargo.toml`.

- [ ] **Step 12.4: Run tests & review snapshots**

Run:
```
cd core/prosopon/crates/prosopon-compositor-glass/web && bun run test
cd core/prosopon && cargo test -p prosopon-compositor-text
```

Expected: First run fails — snapshots don't exist yet. Review and accept with `bun run test -- -u` (vitest) and `cargo insta accept` (after `cargo install cargo-insta` if needed). Re-run; expected: PASS.

- [ ] **Step 12.5: Commit**

```bash
git add core/prosopon/crates/prosopon-compositor-glass/web/tests \
        core/prosopon/crates/prosopon-compositor-text/tests/goldens.rs \
        core/prosopon/crates/prosopon-compositor-text/Cargo.toml \
        core/prosopon/crates/prosopon-compositor-text/tests/snapshots
git commit -m "test(glass,text): cross-surface goldens against shared fixtures

Three canonical fixture scenes (demo, tool_flow, streaming_tokens) live in
the glass web/tests/fixtures and are consumed by both vitest snapshots and
the Rust text-compositor insta snapshots. Any IR change that breaks parity
shows up in both places."
```

---

## Task 13: broomva.tech/prosopon/demo — public demo route

**Files:**
- Create: `broomva.tech/apps/docs/app/prosopon/demo/layout.tsx`
- Create: `broomva.tech/apps/docs/app/prosopon/demo/page.tsx`
- Create: `broomva.tech/apps/docs/app/prosopon/demo/lib/fixture-envelopes.ts`

The demo is **client-only** and does not depend on a live Rust daemon. It replays the same fixture envelopes the goldens use, through the same web module, so anyone can see the compositor without spinning up Rust. Live mode is BRO-773.

- [ ] **Step 13.1: Copy the published web bundle into docs app**

Since the web package isn't published to npm for v0.2, we import from the monorepo via a path alias. Edit `broomva.tech/apps/docs/next.config.mjs` (or equivalent) to add:

```js
export default {
  // existing config...
  transpilePackages: ["@prosopon/compositor-glass"],
};
```

And add a path alias in `broomva.tech/tsconfig.json` or the docs app's tsconfig:

```json
"paths": {
  "@prosopon/compositor-glass": ["../../core/prosopon/crates/prosopon-compositor-glass/web/src/index.tsx"],
  "@prosopon/compositor-glass/*": ["../../core/prosopon/crates/prosopon-compositor-glass/web/src/*"]
}
```

(Exact paths depend on the docs app's resolved tsconfig; adjust during implementation.)

- [ ] **Step 13.2: Write `fixture-envelopes.ts`**

```ts
// broomva.tech/apps/docs/app/prosopon/demo/lib/fixture-envelopes.ts
import type { Envelope } from "@prosopon/compositor-glass/runtime/types";
import demoScene from "../../../../../../core/prosopon/crates/prosopon-compositor-glass/web/tests/fixtures/demo_scene.json";

export function canned(): Envelope[] {
  return [
    {
      version: 1,
      session_id: "demo",
      seq: 1,
      ts: new Date().toISOString(),
      event: { type: "scene_reset", scene: demoScene as unknown as Envelope["event"] extends { scene: infer S } ? S : never },
    },
  ];
}
```

(The `Envelope` type ergonomics may need tightening — the engineer can simplify with a local type.)

- [ ] **Step 13.3: Write `layout.tsx` (full-bleed)**

```tsx
// broomva.tech/apps/docs/app/prosopon/demo/layout.tsx
export default function DemoLayout({ children }: { children: React.ReactNode }) {
  return <div style={{ minHeight: "100vh", background: "hsl(240 33% 6%)" }}>{children}</div>;
}
```

- [ ] **Step 13.4: Write `page.tsx`**

```tsx
// broomva.tech/apps/docs/app/prosopon/demo/page.tsx
"use client";

import { useEffect, useState } from "react";

export default function ProsoponDemo() {
  const [mounted, setMounted] = useState(false);
  useEffect(() => { setMounted(true); }, []);
  if (!mounted) return null;
  return (
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, padding: 24, color: "#eee" }}>
      <section>
        <h2>Glass compositor</h2>
        <GlassPanel />
      </section>
      <section>
        <h2>Envelope stream (raw)</h2>
        <pre style={{ fontFamily: "ui-monospace, monospace", background: "#111", padding: 12, borderRadius: 8, overflow: "auto" }}>
          <EnvelopePanel />
        </pre>
      </section>
    </div>
  );
}

function GlassPanel() {
  // Dynamic import so SSR doesn't try to evaluate the bundle.
  const [App, setApp] = useState<null | (() => JSX.Element)>(null);
  useEffect(() => {
    (async () => {
      const mod = await import("@prosopon/compositor-glass/app");
      setApp(() => mod.App);
    })();
  }, []);
  if (!App) return <p>Loading…</p>;
  return <App />;
}

function EnvelopePanel() {
  const [lines, setLines] = useState<string[]>([]);
  useEffect(() => {
    (async () => {
      const { canned } = await import("./lib/fixture-envelopes");
      setLines(canned().map((e) => JSON.stringify(e)));
    })();
  }, []);
  return <>{lines.join("\n")}</>;
}
```

- [ ] **Step 13.5: Test the docs app locally**

```
cd broomva.tech/apps/docs
bun install
bun dev
```

Navigate to `http://localhost:3003/prosopon/demo`. Expected: glass panel renders the demo scene; envelope panel shows the raw JSON.

- [ ] **Step 13.6: Commit**

```bash
git add broomva.tech/apps/docs/app/prosopon
git commit -m "feat(broomva.tech): /prosopon/demo route renders glass + envelopes

Client-only demo reuses the same fixture envelopes as the goldens; no
backend required. Live WebSocket mode arrives with BRO-773 (arcan session
emit)."
```

---

## Task 14: Docs, CHANGELOG, surface note promotion, final verification

- [ ] **Step 14.1: Promote `docs/surfaces/glass.md`**

Replace `**Status:** planned for v0.2.0` with `**Status:** shipped in v0.2.0-alpha (BRO-767)`. Replace the Module Sketch block with a link to the actual crate tree. Remove "tentative" from design choices that landed. Keep open questions that remain.

- [ ] **Step 14.2: Update RFC-0001 with the `glass.variant` attr key**

In `docs/rfcs/0001-ir-schema.md`'s "Well-known attribute keys" table, add:

```
| `glass.variant` | `"card"` \| `"inline"` \| `"ambient"` | structural | Glass compositor override for the presentation density of the bearing node |
```

- [ ] **Step 14.3: Update `PLANS.md`**

Check the BRO-767 box with a link to the PR once it exists.

- [ ] **Step 14.4: Write a CHANGELOG entry**

Create or prepend to `core/prosopon/CHANGELOG.md`:

```markdown
## [0.2.0-alpha] — 2026-04-22

### Added
- `prosopon-compositor-glass` crate — 2D web compositor, Arcan Glass-styled,
  serves an embedded Preact bundle over HTTP + WebSocket + SSE.
- `@prosopon/compositor-glass` TypeScript package under the same crate's `web/`.
- Cross-surface golden tests: shared fixtures drive both text (insta) and glass
  (vitest snapshot) renders.
- `broomva.tech/prosopon/demo` — public client-only demo.
- Well-known attr key `glass.variant` (RFC-0001).

### Fixed
- `prosopon-runtime::StoreEvent::Reset` now boxes its `Scene` payload
  (large_enum_variant clippy regression).
```

- [ ] **Step 14.5: Update `CONTROL.md`**

Mark S2.3 as partially satisfied: text + glass have golden tests.

- [ ] **Step 14.6: Final `make smoke`**

Run: `cd core/prosopon && make smoke`
Expected: clean. All workspace tests pass. Clippy passes.

- [ ] **Step 14.7: Final TS pass**

```
cd core/prosopon/crates/prosopon-compositor-glass/web
bun run lint
bun run typecheck
bun run build
bun run test
```

Expected: all clean.

- [ ] **Step 14.8: Commit docs**

```bash
git add core/prosopon/docs core/prosopon/CHANGELOG.md core/prosopon/PLANS.md core/prosopon/CONTROL.md
git commit -m "docs(prosopon): promote glass surface, RFC-0001 glass.variant, CHANGELOG

v0.2.0-alpha entry covers BRO-767 deliverables. Glass surface note promoted
from planned → shipped. RFC-0001 gains glass.variant as the first
compositor-specific well-known attr key."
```

- [ ] **Step 14.9: Open the PR**

```bash
git push -u origin bro-767-glass-compositor
gh pr create --title "BRO-767: prosopon-compositor-glass — 2D web compositor" \
  --body "$(cat <<'EOF'
## Summary
- New crate `prosopon-compositor-glass` ships the Prosopon 2D web compositor (Arcan Glass-styled Preact bundle served over HTTP/WS/SSE, embedded via include_dir!).
- Totality over every current `Intent` variant; `Field`/`Locus`/`Formation` fall through to `Fallback` (implemented by BRO-774).
- Golden-file parity: shared JSON fixtures drive vitest snapshots (glass) AND insta snapshots (text).
- `broomva.tech/prosopon/demo` route renders the same fixtures client-only — no backend required.
- Baseline repair: boxed `StoreEvent::Reset` to silence `large_enum_variant` so `make smoke` is green again.

## RFC impact
- RFC-0001: adds `glass.variant` as a well-known attr key.
- RFC-0002: no change — compositor trait unchanged, totality contract honored.
- No `IR_SCHEMA_VERSION` or `PROTOCOL_VERSION` bump.

## Test plan
- [x] `make smoke` in `core/prosopon/`
- [x] `cargo test --workspace` (includes new handshake + totality tests)
- [x] `bun run test` in `web/` (scene-store, binding, totality, goldens)
- [x] Manual: `prosopon-glass serve --fixture ...` + browser load at :4321
- [x] Manual: broomva.tech `/prosopon/demo` in local docs app

Plan: `core/prosopon/docs/superpowers/plans/2026-04-22-bro-767-glass-compositor.md`
Closes BRO-767.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 14.10: Post a comment on Linear BRO-767**

Use the MCP Linear server (`mcp__linear-server__save_comment`) to post a short summary + PR URL + plan path.

---

## Self-review

**Spec coverage:**
- Scaffold Rust crate (hybrid with TS): Tasks 1, 2 ✓
- axum HTTP + WS server subscribed to a Runtime, JSONL envelope fanout over WS + SSE: Task 10 ✓
- Embedded web/ via include_dir!: Task 10 ✓
- Scene store + signal bus mirroring Rust runtime: Tasks 5, 6 ✓
- registry/intents.ts mapping each Intent variant: Task 7 ✓
- @chenglou/pretext for text measurement: declared dep (Task 2); stream tail measurement deferred to BRO-760 follow-up (documented in Stream component, Task 9)
- Yoga for layout: declared dep (Task 2); currently used only for GroupKind at component level, full layout engine integration deferred — no task introduces Yoga usage yet. **Action:** adjust expectations — Task 7 tokens + CSS flex handle every layout need for v0.2; Yoga lands in a follow-up when Group variants require flex-internal behavior the browser can't express. Dep declaration stays so adopters can tree-shake themselves.
- Design tokens from arcan-glass: Task 7 (inlined, copied) ✓
- Cover every Intent variant, starting with Prose/Code/Section/Divider/Progress/ToolCall/ToolResult/Choice/Confirm/Signal/Stream: Tasks 8, 9 ✓. Field/Locus/Formation → Fallback (Task 7) ✓.
- Golden files (vitest snapshot against same fixtures as text): Task 12 ✓
- broomva.tech/prosopon/demo showing text + glass side-by-side with envelope stream in a third panel: Task 13 — **GAP**: plan only shows glass + envelopes (two panels). **Action:** add a "Text compositor panel" step under Task 13 that imports `prosopon_compositor_text` behavior via a WASM bridge OR (pragmatic) renders pre-computed text output captured from `cargo run -p prosopon-cli -- demo` at build time. Fix below.
- Commit + PR + CI per AGENTS.md + CONTROL.md: Task 14 ✓
- BRO-773 follow-on: explicitly deferred to the next plan; this plan stops at BRO-767 ✓

**Addition to Task 13 to close the text-panel gap:**

Add step 13.2b between 13.2 and 13.3:

```
- [ ] Step 13.2b: Capture text compositor output as a static fixture

Run once during development:
cd core/prosopon && cargo run -p prosopon-cli -- demo > apps/docs/app/prosopon/demo/lib/text-output.txt

Then add to page.tsx a third panel that imports the captured text via
`?raw` import:

import textOutput from "./lib/text-output.txt?raw";

<section><h2>Text compositor</h2><pre>{textOutput}</pre></section>

Grid becomes three columns. If the fixtures change, the engineer regenerates
text-output.txt; a future task (BRO-779) can auto-sync via a postbuild hook.
```

**Placeholder scan:** All code blocks are concrete. No "TBD" / "implement later" / "similar to Task N". The one "adjust during implementation" line in Step 13.1 (path alias) is necessary — the engineer must inspect the actual tsconfig path resolution to write a correct alias, and exact paths vary between Next app layouts. Acceptable.

**Type consistency:** `createSceneStore` / `SceneStore` consistent across Tasks 5, 7, 11. `SignalBus` consistent. `GlassCompositor.detached()` introduced in Task 3 and used in tests (Task 3). `GlassServer::bind(GlassServerConfig{...})` consistent between Task 10 tests and implementation. `EnvelopeFanout` derives `Clone` — confirmed in Task 3.5 code. `Transport` type declared in Task 11 and used in Task 11.4 under the same name. `RegistryCtx.scene` is a snapshot `Scene`, not a `Signal<Scene>`, and `app.tsx` passes `store.signal.value` (Task 11.3) — matches Task 7.3 provider typing. ✓

**One inconsistency caught and to be fixed during implementation:** Step 11.3's `<RegistryContext.Provider value={{ scene: store.signal.value, ... }}>` reads `store.signal.value` but won't re-render when the signal updates (React-style vs Preact-signals-style). Two valid fixes; the engineer picks one at implementation time:
- (a) Wrap with the Preact signals `computed` + `useComputed` hook so the Provider re-evaluates on store.signal.value change.
- (b) Use `useState` in `App` to mirror `store.signal.value` and call `setState` from inside `onEnvelope`.

This is flagged in the plan by adding a brief note:

**Task 11.3 note** — Provider value must re-evaluate on scene change. Simplest: `const scene = store.signal.value;` subscribed via `useSignalEffect` from `@preact/signals`, OR mirror into useState. Confirm correct re-renders before committing Task 11.

All other types resolve cleanly.

---

**End of plan.**
