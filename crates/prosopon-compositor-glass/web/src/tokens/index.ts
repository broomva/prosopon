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
