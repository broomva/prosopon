# Surface: Audio

**Status:** planned for v0.4.0

## What it will be

An audio compositor: speech + ambient sonification of agent activity. Each
agent gets a distinct voice timbre; system health is carried by the background
soundscape; priority is audible without a screen glance.

## Why this matters

Audio is the most *ambient* surface. A well-designed audio compositor lets an
operator run dozens of agents with sound as their peripheral status channel —
no dashboards, no glancing, just hearing the system's shape. It's also the
most naturally polymorphic: no visual metaphor needs translation; the audio
compositor just plays.

## Design sketch

- **Speech synthesis** — Gemini TTS (already wired in Broomva stack,
  `gemini-2.5-flash-preview-tts`, `Kore` voice). Each agent gets a distinct
  voice id.
- **Ambient layer** — CPU audio synthesis (cpal + fundsp or synthrs).
  `SignalChanged` events modulate ambient parameters (pitch = load, density =
  activity, timbre = agent identity).
- **Spatial audio** — `SpatialFrame` semantics apply here too: agents at
  `Locus` positions get panned accordingly.
- **Priority cues** — `Priority::Urgent` triggers a tone; `Priority::Blocking`
  pauses ambient and speaks the confirmation.

## Intent → audio mapping

| Intent | Audio |
|---|---|
| `Prose` | TTS in agent voice. Voice chosen by `attrs.voice` or default. |
| `Stream { Audio }` | Play PCM frames directly. |
| `Audio { uri/stream/voice }` | Play or synthesize as specified. |
| `Progress` | Tonal sweep from low-pitch to high-pitch across the bar. |
| `ToolCall` | Short percussive cue (different per tool family). |
| `ToolResult::Success` | Resolving chord. |
| `ToolResult::Failure` | Dissonant chord. |
| `Confirm` | Interruptive chime + TTS; waits for voice confirmation. |
| `Signal` (non-audio) | Drifts ambient layer parameters continuously. |
| Non-audio intents | Silent (the audio compositor is complementary, not replacement). |

## Module sketch

```
crates/prosopon-compositor-audio/
  ├── src/
  │   ├── lib.rs
  │   ├── tts.rs           # Gemini TTS bridge
  │   ├── ambient.rs       # fundsp-based ambient synthesis
  │   ├── cues.rs          # per-intent audio cue catalog
  │   └── spatial.rs       # panning / HRTF
```

## Performance

- **Latency budget** — 50ms buffer ahead; speech cues ≤200ms from `apply` to
  audible.
- **CPU budget** — ≤8% of a single core at idle; ≤20% during active speech.

## Open questions

- **Voice selection.** Should we auto-assign voices by agent id hash, or let
  agents declare preferences via `attrs.voice`?
- **Localization.** Gemini TTS supports many languages; we honor
  `SceneHints::locale` but need a fallback policy.
- **Barge-in.** Can the user interrupt a TTS speech to issue a command? Yes,
  but requires voice activity detection — a v0.5 concern.
