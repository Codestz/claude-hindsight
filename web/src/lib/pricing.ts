// Model-aware pricing for Claude API usage.
// Mirrors src/pricing.rs — calculates costs from token counts per model.

import type { TokenUsage } from "./types";

interface ModelPricing {
  input: number;
  output: number;
  cacheCreation: number;
  cacheRead: number;
}

// Per-million-token rates. Source: https://docs.anthropic.com/en/docs/about-claude/models
const PRICING: [string, ModelPricing][] = [
  ["claude-opus-4",   { input: 15,   output: 75,   cacheCreation: 18.75, cacheRead: 1.50 }],
  ["claude-sonnet-4", { input: 3,    output: 15,   cacheCreation: 3.75,  cacheRead: 0.30 }],
  ["claude-haiku-4",  { input: 0.80, output: 4,    cacheCreation: 1.0,   cacheRead: 0.08 }],
  ["claude-haiku-3-5",{ input: 0.80, output: 4,    cacheCreation: 1.0,   cacheRead: 0.08 }],
  ["claude-sonnet-3-5",{input: 3,    output: 15,   cacheCreation: 3.75,  cacheRead: 0.30 }],
  ["claude-sonnet-3", { input: 3,    output: 15,   cacheCreation: 3.75,  cacheRead: 0.30 }],
  ["claude-opus-3",   { input: 15,   output: 75,   cacheCreation: 18.75, cacheRead: 1.50 }],
  ["claude-haiku-3",  { input: 0.25, output: 1.25, cacheCreation: 0.30,  cacheRead: 0.03 }],
];

const DEFAULT_PRICING: ModelPricing = { input: 3, output: 15, cacheCreation: 3.75, cacheRead: 0.30 };

/** Strip 8-digit date suffix: "claude-sonnet-4-5-20250929" → "claude-sonnet-4-5" */
function stripDateSuffix(model: string): string {
  return model.replace(/-\d{8}$/, "");
}

/** Look up pricing by model name (prefix match, falls back to Sonnet rates). */
export function lookupPricing(model: string | null | undefined): ModelPricing {
  if (!model) return DEFAULT_PRICING;
  const stripped = stripDateSuffix(model);
  for (const [prefix, pricing] of PRICING) {
    if (stripped.startsWith(prefix)) return pricing;
  }
  return DEFAULT_PRICING;
}

/** Calculate cost in USD from token usage and model name. */
export function calculateCost(usage: TokenUsage | null | undefined, model: string | null | undefined): number {
  if (!usage) return 0;
  const p = lookupPricing(model);
  const inp = usage.input_tokens ?? 0;
  const out = usage.output_tokens ?? 0;
  const cacheCreate = usage.cache_creation_input_tokens ?? 0;
  const cacheRead = usage.cache_read_input_tokens ?? 0;

  return (inp * p.input + out * p.output + cacheCreate * p.cacheCreation + cacheRead * p.cacheRead) / 1_000_000;
}
