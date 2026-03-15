/**
 * Utility functions for session components.
 *
 * Pure functions with no React dependency — testable in isolation.
 */

import type { NodeResponse } from "@/lib/types";
import type { DisplayItem, GraphNode, GraphLink, TaskNotification } from "./types";
import { isCollapsibleNode, graphNodeColor, graphNodeRadius } from "./config";

// ── ExecutionList display items ──────────────────────────────

/** Collapse consecutive collapsible nodes into groups. */
export function buildDisplayItems(nodes: NodeResponse[]): DisplayItem[] {
  const items: DisplayItem[] = [];
  let i = 0;

  while (i < nodes.length) {
    if (isCollapsibleNode(nodes[i])) {
      const group: NodeResponse[] = [];
      while (i < nodes.length && isCollapsibleNode(nodes[i])) {
        group.push(nodes[i]);
        i++;
      }
      if (group.length <= 2) {
        for (const n of group) items.push({ kind: "node", node: n });
      } else {
        const types = new Map<string, number>();
        for (const n of group) {
          const t = n.node_type;
          types.set(t, (types.get(t) ?? 0) + 1);
        }
        const dominant = [...types.entries()].sort((a, b) => b[1] - a[1])[0];
        const label = dominant[0] === "progress" ? "Hook / progress events"
          : dominant[0] === "system" ? "System events"
          : dominant[0] === "queue-operation" ? "Queue operations"
          : "Internal events";
        items.push({ kind: "group", nodes: group, label });
      }
    } else {
      items.push({ kind: "node", node: nodes[i] });
      i++;
    }
  }

  return items;
}

// ── ExecutionGraph data builder ──────────────────────────────

/** Build graph nodes and links from a tree of NodeResponses. */
export function buildGraph(roots: NodeResponse[]): { nodes: GraphNode[]; links: GraphLink[] } {
  const nodes: GraphNode[] = [];
  const links: GraphLink[] = [];

  const walk = (n: NodeResponse, pid: string | null) => {
    const id = n.uuid ?? `n${nodes.length}`;
    nodes.push({ id, node: n, color: graphNodeColor(n), r: graphNodeRadius(n) });
    if (pid) links.push({ source: pid, target: id });
    for (const c of n.children ?? []) walk(c, id);
  };

  for (const root of roots) walk(root, null);
  return { nodes, links };
}

// ── Task notification parsing ────────────────────────────────

/** Pre-compiled regexes for XML tag extraction. */
const TASK_RE = {
  taskId: /<task-id>([\s\S]*?)<\/task-id>/,
  status: /<status>([\s\S]*?)<\/status>/,
  summary: /<summary>([\s\S]*?)<\/summary>/,
  result: /<result>([\s\S]*?)<\/result>/,
  totalTokens: /<total_tokens>([\s\S]*?)<\/total_tokens>/,
  toolUses: /<tool_uses>([\s\S]*?)<\/tool_uses>/,
  durationMs: /<duration_ms>([\s\S]*?)<\/duration_ms>/,
} as const;

/** Parse <task-notification> XML from a user node's content. */
export function parseTaskNotification(content: string): TaskNotification | null {
  if (!content.includes("<task-notification>")) return null;
  return {
    taskId: content.match(TASK_RE.taskId)?.[1]?.trim() ?? null,
    status: content.match(TASK_RE.status)?.[1]?.trim() ?? null,
    summary: content.match(TASK_RE.summary)?.[1]?.trim() ?? null,
    result: content.match(TASK_RE.result)?.[1]?.trim() ?? null,
    totalTokens: content.match(TASK_RE.totalTokens)?.[1]?.trim() ?? null,
    toolUses: content.match(TASK_RE.toolUses)?.[1]?.trim() ?? null,
    durationMs: content.match(TASK_RE.durationMs)?.[1]?.trim() ?? null,
  };
}
