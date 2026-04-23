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
import { MathComponent } from "../components/Math";
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
  math: MathComponent,
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
  return (Component as any)({ intent, nodeId });
}
