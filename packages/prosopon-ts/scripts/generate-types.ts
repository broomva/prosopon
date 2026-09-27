/**
 * Auto-generate TypeScript types from the JSON schemas published by
 * `prosopon-core::scene_schema_json` + `event_schema_json`.
 *
 * Usage:
 *   bun run scripts/generate-types.ts
 *
 * Input:  src/generated/{scene,event}.json (committed snapshots)
 * Output: src/generated/types.ts
 *
 * To refresh the snapshots from the canonical Rust source:
 *   cd ../../../core/prosopon
 *   cargo run -p prosopon-cli -- schema scene > $(pwd)/src/generated/scene.json
 *   cargo run -p prosopon-cli -- schema event > $(pwd)/src/generated/event.json
 */

import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { compile } from "json-schema-to-typescript";

const ROOT = path.resolve(new URL("..", import.meta.url).pathname);
const GEN = path.join(ROOT, "src", "generated");

const BANNER = `/**
 * AUTO-GENERATED — do not edit by hand.
 *
 * Source of truth: core/prosopon/crates/prosopon-core (Rust).
 * Regenerate with \`bun run generate\` from packages/prosopon-ts/.
 *
 * Generated on: ${new Date().toISOString()}
 */
/* eslint-disable */
/* biome-ignore-all */
`;

/**
 * Merge the two published schemas into a single composite schema whose root
 * has two properties: `scene` (Scene) and `event` (ProsoponEvent). Shared
 * definitions (Node, Intent, Lifecycle, etc.) appear once in the merged
 * `definitions` block, so json-schema-to-typescript emits each type exactly
 * once. We then post-process the output to drop the synthetic root wrapper.
 */
async function main(): Promise<void> {
  const [sceneSrc, eventSrc] = await Promise.all([
    readFile(path.join(GEN, "scene.json"), "utf-8"),
    readFile(path.join(GEN, "event.json"), "utf-8"),
  ]);
  const scene = JSON.parse(sceneSrc) as SchemaShape;
  const event = JSON.parse(eventSrc) as SchemaShape;

  const sceneDefs = scene.definitions ?? {};
  const eventDefs = event.definitions ?? {};

  // The top-level schemas (Scene / ProsoponEvent) go into `definitions` so
  // they become named exports alongside shared types. We peel the root-level
  // `oneOf` off the event schema and keep it as the ProsoponEvent definition.
  const stripRoot = (s: SchemaShape): SchemaShape => {
    const { definitions: _d, ...root } = s;
    void _d;
    return root;
  };

  const combined = {
    $schema: "http://json-schema.org/draft-07/schema#",
    title: "Prosopon",
    description: "Combined Scene + ProsoponEvent schema for TS code-gen.",
    type: "object",
    required: ["scene", "event"],
    properties: {
      scene: { $ref: "#/definitions/Scene" },
      event: { $ref: "#/definitions/ProsoponEvent" },
    },
    definitions: {
      Scene: stripRoot(scene),
      ProsoponEvent: stripRoot(event),
      ...sceneDefs,
      ...eventDefs,
    },
  };

  const compiled = await compile(
    combined as Parameters<typeof compile>[0],
    "Prosopon",
    {
      bannerComment: "",
      declareExternallyReferenced: true,
      strictIndexSignatures: true,
      additionalProperties: false,
    },
  );

  // Drop the synthetic `export interface Prosopon { scene, event }` wrapper —
  // callers import Scene + ProsoponEvent directly. Everything else stays.
  const cleaned = compiled.replace(
    /export interface Prosopon \{[\s\S]*?scene: Scene;[\s\S]*?event: ProsoponEvent;[\s\S]*?\}\n+/,
    "",
  );

  const out = `${BANNER}\n${cleaned}\n`;
  await writeFile(path.join(GEN, "types.ts"), out);
  console.log(`✓ wrote src/generated/types.ts (${out.length} bytes)`);
}

interface SchemaShape {
  definitions?: Record<string, unknown>;
  [k: string]: unknown;
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
