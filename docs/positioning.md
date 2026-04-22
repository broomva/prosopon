# Positioning — Prosopon vs. the "Windows moment for agents" thesis

**Audience:** engineers, investors, and partners thinking about agent UX as a product category.

**Length target:** one screen. Nothing more.

---

## The thesis (a16z, paraphrased)

Agents today feel like MS-DOS: command lines, manual skill discovery, workflows that break awkwardly. The next unlock is the "Windows moment" — a visual abstraction over agents, strategy-game-inspired, safe by default, usable by non-technical humans. The big wins will come from *making agent systems legible and manageable*, not from smarter base models alone.

The thesis is right. The question is what level of the stack you build at.

## Where most entrants are going

Three product shapes have crystallized in the last six months:

1. **Desktop orchestrators** — Mission Control (this workspace), Conductor, Superconductor, Polyscope, Omnara, Cursor 3, Codex App. Tiled grids, checkpoints, auth gateways. Great products. Each is one GUI.
2. **Declarative agent-UI protocols** — Google's **A2UI** (Apache 2.0), the official **MCP Apps** extension (Anthropic + OpenAI, Jan 2026), CopilotKit's **AG-UI**, Vercel's AI SDK 5.0 `message.parts[]`. Agents emit structured UI; hosts render via a widget registry or sandboxed iframe.
3. **Generative-UI research** — Google's *Generative UI* paper (Leviathan et al., 2025) with the PAGEN dataset; Shadcn `ai-elements`; assistant-ui (YC W25).

Products (1) compete at the GUI layer. Protocols (2) compete at the *wire* layer. Research (3) experiments with both.

## Prosopon's bet

**Build at the display-server layer** — one step *below* every product in (1), interoperable with every protocol in (2). Agents emit semantic intent; compositors render it. The IR is the product; GUIs are compositors on top.

Why this works specifically inside the Broomva ecosystem:

- **Life already has the substrate.** The Pneuma trait family formalizes inter-boundary flow; Sensorium implements `Pneuma<ExternalToL0>` (world → plant). Prosopon is literally its mirror — `Pneuma<L0ToExternal>` (plant → observer). It's not a new silo; it's the missing outward twin.
- **Arcan Glass, Mission Control, Prompter, Relay are all already compositors in waiting.** Each has UI work that's currently duplicated; each can consume the same Prosopon IR instead of reinventing a mini-protocol.
- **The research validates the level.** Flutter's Widget/Element/RenderObject separation is exactly the abstraction prosopon needs. Wayland's "tiny core + versioned extensions" is the exact posture for *never* breaking compatibility as new Intents land. Dioxus, Leptos, Iced prove the pattern in Rust at production quality.

## The differentiator vs. existing declarative protocols

| Protocol | Scope | Multi-surface | Rust native | Substrate-level |
|---|---|---|---|---|
| MCP Apps | Sandboxed iframe (HTML/JS) | No (web only) | Bridged | No |
| A2UI | Declarative JSON + client registry | Flutter / Angular / Lit / Markdown | Bridged | No |
| AG-UI | Event-based transport | Framework-agnostic | Bridged | No |
| AI SDK parts | Typed React components | No (React only) | No | No |
| **Prosopon** | **Semantic intent IR** | **Text / 2D / 3D / shader / audio / spatial / tactile** | **Yes** | **Yes (Pneuma family)** |

Prosopon does not replace any of these. It adopts the same declarative-with-client-registry pattern that A2UI pioneered, ships a Wayland-minimal core over it, and crucially **bridges** to MCP Apps / A2UI / AG-UI / AI SDK as first-class compositors. That's the "XWayland moment": the day prosopon ships, every agent already emitting one of those formats keeps working, rendered into the native multi-surface pipeline.

The two wedge surfaces chosen for v0.1:

- **Text (in-repo reference).** The universal fallback — every agent is already producing text. Upgrading it from markdown-in-a-box to reactive, intent-typed living prose is the smallest change that makes the polymorphism thesis real.
- **Glass (planned next).** Web canvas / Arcan Glass. This is where the screenshots live; this is what a16z will see. Mission Control, Prompter, Relay all consume it.

Later compositors (shader, spatial, audio) are where the category-definition happens: they're the evidence that the IR is genuinely polymorphic, not just "HTML with extra steps."

## What to measure

- **Schema adoption** — number of out-of-repo agents emitting Prosopon envelopes.
- **Compositor diversity** — number of surfaces rendering the same IR (target: four by 2026-07, the four named above).
- **Bridge coverage** — percentage of MCP Apps / A2UI / AG-UI examples that render unchanged through prosopon-compat.
- **Agent OS integration** — Pneuma trait landed, Prosopon registered as `Pneuma<L0ToExternal>`, arcan emitting its session view through Prosopon.

## The wager

The next decade of agent UX will be won by whoever owns the *IR*, not whoever ships the prettiest GUI. GUIs churn every 18 months; a well-chosen IR outlives them. Wayland is still here; the Linux desktop fashion of 2013 is not.

If prosopon is wrong about the level, the worst case is that we end up with a genuinely nice text and web compositor for the Broomva stack. If prosopon is right, we own the substrate that every agent-UX product in the ecosystem builds on.

---

## References

- Cheng Lou, [pretext.cool](https://www.pretext.cool/) — the text measurement engine that gave this repo its philosophy (pure-function layout, decouple intent from resolution).
- [MCP Apps spec (Jan 2026)](https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/)
- [Google A2UI v0.9](https://a2ui.org/specification/v0.9-a2ui/)
- [CopilotKit AG-UI](https://www.copilotkit.ai/ag-ui)
- [Inside Flutter — Widget/Element/RenderObject](https://docs.flutter.dev/resources/inside-flutter)
- [Wayland Book — wl_compositor](https://wayland-book.com/surfaces/compositor.html)
- [Leviathan et al., *Generative UI*, 2025](https://generativeui.github.io/static/pdfs/paper.pdf)
