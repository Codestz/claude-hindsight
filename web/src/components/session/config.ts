/**
 * Configuration constants for session components.
 *
 * All magic values, color maps, and extensible config live here.
 */

import type { NodeResponse } from "@/lib/types";
import { isTaskNotification } from "@/lib/node-meta";

// ── Graph colors ─────────────────────────────────────────────

/** Raw hex colors for Three.js rendering (CSS vars don't work in WebGL). */
export const GRAPH_COLORS: Record<string, string> = {
  cyan: "#38BDF8", green: "#34D399", magenta: "#A78BFA",
  yellow: "#F59E0B", amber: "#F59E0B", blue: "#38BDF8",
  gray: "#4a4a5a", white: "#ECECF1",
};

export const GRAPH_SELECTED = "#818CF8";
export const GRAPH_ERROR = "#FB7185";
export const GRAPH_TASK_COLOR = "#c084fc";
export const GRAPH_BG = "#0a0a0e";

// ── Graph node sizing ────────────────────────────────────────

/** Get hex color for a node in the 3D graph. */
export function graphNodeColor(n: NodeResponse): string {
  if (isTaskNotification(n)) return GRAPH_TASK_COLOR;
  return GRAPH_COLORS[n.color] ?? "#A1A1B5";
}

/** Get radius for a node in the 3D graph (scales by importance). */
export function graphNodeRadius(n: NodeResponse): number {
  if (isTaskNotification(n)) return 4;
  if (n.node_type === "user" && n.color !== "blue") return 5;
  if (n.node_type === "assistant" && n.color === "green") return 4.5;
  if (n.color === "yellow") return 3.5;
  if (n.color === "blue") return 3;
  if (n.color === "magenta") return 3;
  if (n.node_type === "progress") return 1.5;
  return 1.5;
}

/** Performance tier based on node count. */
export function graphPerfTier(count: number): "high" | "medium" | "low" {
  if (count < 300) return "high";
  if (count < 1000) return "medium";
  return "low";
}

// ── Filter bar ───────────────────────────────────────────────

/** Available node type filter chips. */
export const FILTER_OPTIONS: string[] = ["User", "Assistant", "Tool", "Task", "Error", "Thinking", "Prompt"];

// ── Collapsible node detection ───────────────────────────────

/** Node types that should be collapsed in groups when consecutive. */
export function isCollapsibleNode(node: NodeResponse): boolean {
  if (node.node_type === "progress") return true;
  if (node.node_type === "system") return true;
  if (node.node_type === "queue-operation") return true;
  if (node.node_type === "file-history-snapshot") return true;
  return false;
}
