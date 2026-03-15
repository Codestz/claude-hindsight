// ============================================================
// Node Display Metadata
//
// Single source of truth for how each node type is rendered.
// Every component that shows a node imports from here —
// no node display logic lives in individual components.
//
// Node taxonomy comes from src/analyzer/smart_label.rs:
//   user       → user_message | user_tool_result
//   assistant  → assistant_thinking | assistant_text | assistant_tool_call
//   progress   → progress_bash | progress_hook | progress_agent | progress_other
//   file-history-snapshot
//   system
//   queue-operation
// ============================================================

import type { NodeColor, NodeResponse } from "./types";

// ─────────────────────────────────────────────────────────────
// § 1 — Node color → CSS variable map
//       The server sends a color token string; we map it to
//       the actual CSS custom property for use in inline styles.
// ─────────────────────────────────────────────────────────────

export const NODE_COLOR_CSS: Record<string, string> = {
  cyan:    "var(--cyan)",
  green:   "var(--green)",
  magenta: "var(--purple)",  // design system uses --purple for magenta
  yellow:  "var(--amber)",
  amber:   "var(--amber)",
  blue:    "var(--cyan)",    // tool results share cyan with info
  gray:    "var(--text-3)",
  white:   "var(--text-1)",
};

export function nodeColorToCss(color: NodeColor | string): string {
  return NODE_COLOR_CSS[color] ?? "var(--text-2)";
}

// ─────────────────────────────────────────────────────────────
// § 2 — NodeMeta: the display shape every XNode component uses
// ─────────────────────────────────────────────────────────────

export interface NodeMeta {
  /** Short display label — used in badges */
  badge: string;
  /** Unicode glyph — shown as an icon inline with the badge */
  icon: string;
  /** CSS color for the icon and badge accent */
  color: string;
  /** Human-readable description — used in detail panels and tooltips */
  description: string;
}

// ─────────────────────────────────────────────────────────────
// § 3 — Lookup table: node_type → NodeMeta
//       We derive semantic type from node_type + content,
//       but we also need a fallback for each raw node_type.
// ─────────────────────────────────────────────────────────────

export const NODE_META: Record<string, NodeMeta> = {
  // User messages and tool returns
  user: {
    badge: "User",
    icon: "◉",
    color: "var(--cyan)",
    description: "User message or tool result",
  },

  // Assistant responses: thinking, text, tool calls
  assistant: {
    badge: "Asst",
    icon: "◈",
    color: "var(--green)",
    description: "Assistant response",
  },

  // Thinking block inside an assistant message
  thinking: {
    badge: "Think",
    icon: "◌",
    color: "var(--purple)",
    description: "Extended thinking block",
  },

  // Tool invocation (assistant calling a tool)
  tool_use: {
    badge: "Tool",
    icon: "⊕",
    color: "var(--amber)",
    description: "Tool invocation",
  },

  // Tool result returned to the model
  tool_result: {
    badge: "Result",
    icon: "⊗",
    color: "var(--cyan)",
    description: "Tool result",
  },

  // Progress events from long-running operations
  progress: {
    badge: "Prog",
    icon: "⟳",
    color: "var(--amber)",
    description: "Progress update",
  },

  // Sub-agent spawned by the assistant
  subagent: {
    badge: "Agent",
    icon: "⟳",
    color: "var(--purple)",
    description: "Sub-agent session",
  },

  // Task notification — sub-agent completion
  task: {
    badge: "Task",
    icon: "↻",
    color: "var(--purple)",
    description: "Task completion",
  },

  // Error node
  error: {
    badge: "Error",
    icon: "✗",
    color: "var(--red)",
    description: "Error",
  },

  // Compaction boundary — context was compressed
  compact_boundary: {
    badge: "Compact",
    icon: "⟐",
    color: "var(--amber)",
    description: "Context compacted",
  },

  // System events (turn_duration, etc.)
  system: {
    badge: "Sys",
    icon: "◦",
    color: "var(--text-3)",
    description: "System event",
  },

  // File history snapshot
  "file-history-snapshot": {
    badge: "Snap",
    icon: "◫",
    color: "var(--purple)",
    description: "File snapshot",
  },

  // Queue operation
  "queue-operation": {
    badge: "Queue",
    icon: "≡",
    color: "var(--text-3)",
    description: "Queued operation",
  },
};

/** Fallback metadata for unknown node types */
const FALLBACK_META: NodeMeta = {
  badge: "Node",
  icon: "◦",
  color: "var(--text-2)",
  description: "Unknown node type",
};

/**
 * Get display metadata for a node.
 *
 * Uses the server's `color` field to detect semantic sub-types
 * (e.g. distinguishes "thinking" from "assistant" even though
 * both have node_type = "assistant").
 */
export function getNodeMeta(node: NodeResponse): NodeMeta {
  const t = node.node_type;

  // ── Semantic refinement for assistant nodes ─────────────────
  // The server sets color = "magenta" for thinking-only messages,
  // "yellow" for tool calls, "green" for text responses.
  if (t === "assistant") {
    if (node.color === "magenta") return NODE_META.thinking ?? FALLBACK_META;
    if (node.color === "yellow") return NODE_META.tool_use ?? FALLBACK_META;
    return NODE_META.assistant ?? FALLBACK_META;
  }

  // ── Semantic refinement for user nodes ─────────────────────
  // color = "blue" means tool result (user node that carries tool output)
  if (t === "user") {
    if (node.color === "blue") return NODE_META.tool_result ?? FALLBACK_META;
    // Task notification from sub-agent completion
    if (isTaskNotification(node)) {
      return NODE_META.task ?? FALLBACK_META;
    }
    return NODE_META.user ?? FALLBACK_META;
  }

  // ── Semantic refinement for system nodes ────────────────────
  // color = "amber" means compact_boundary (context compacted)
  if (t === "system") {
    if (node.color === "amber") return NODE_META.compact_boundary ?? FALLBACK_META;
  }

  // ── Error detection ─────────────────────────────────────────
  if (node.has_error) return NODE_META.error ?? FALLBACK_META;

  return NODE_META[t] ?? FALLBACK_META;
}

// ─────────────────────────────────────────────────────────────
// § 4 — Node content helpers
//       Pure functions for extracting content from a NodeResponse.
//       Components call these instead of re-implementing extraction.
// ─────────────────────────────────────────────────────────────

/**
 * Returns all text content from a node as a plain string.
 * Handles: thinking, message content (string or blocks), tool summaries.
 */
export function getNodeText(node: NodeResponse): string {
  // Summary is the server's pre-computed short description
  if (node.summary) return node.summary;

  // Thinking node
  if (node.thinking) return node.thinking;

  // Message content
  if (node.message?.content) {
    const c = node.message.content;
    if (typeof c === "string") return c;
    if (Array.isArray(c)) {
      return c
        .filter((b) => b.type === "text")
        .map((b) => (b as { type: "text"; text: string }).text)
        .join("\n\n");
    }
  }

  // Tool result
  if (node.tool_result?.content) return node.tool_result.content;

  return "";
}

/**
 * Returns thinking text if the node contains a thinking block.
 * Returns null if there is no thinking content.
 */
export function getThinkingText(node: NodeResponse): string | null {
  if (node.thinking) return node.thinking;

  if (node.message?.content && Array.isArray(node.message.content)) {
    const block = node.message.content.find((b) => b.type === "thinking");
    if (block && block.type === "thinking") {
      return (block as { type: "thinking"; thinking: string }).thinking;
    }
  }

  return null;
}

/**
 * Returns token usage from whichever field has it.
 * The server stores usage inside message.usage for assistant nodes.
 */
export function getTokenUsage(node: NodeResponse) {
  return node.token_usage ?? node.message?.usage ?? null;
}

/**
 * Returns the prompt score for a node, or null if not applicable.
 * Scores >= 40 indicate a meaningful prompt.
 */
export function getPromptScore(node: NodeResponse): number | null {
  return node.prompt_score != null && node.prompt_score >= 40
    ? node.prompt_score
    : null;
}

/**
 * True if the node is a task notification from a completed sub-agent.
 * These are user nodes containing <task-notification> XML.
 */
export function isTaskNotification(node: NodeResponse): boolean {
  return node.node_type === "user"
    && typeof node.message?.content === "string"
    && node.message.content.includes("<task-notification>");
}

/**
 * True if the node is purely decorative / internal (no user-facing content).
 * These can be collapsed by default in the tree view.
 */
export function isInternalNode(node: NodeResponse): boolean {
  // compact_boundary system nodes are user-facing (show compaction events)
  if (node.node_type === "system" && node.color === "amber") return false;

  return (
    node.node_type === "system" ||
    node.node_type === "queue-operation" ||
    node.node_type === "file-history-snapshot"
  );
}
