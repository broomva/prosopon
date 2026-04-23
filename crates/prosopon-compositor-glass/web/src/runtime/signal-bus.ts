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
