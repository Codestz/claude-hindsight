/**
 * Utility functions for session components.
 *
 * Pure functions with no React dependency — testable in isolation.
 */

import type { NodeResponse } from "@/lib/types";
import type { DisplayItem, GraphNode, GraphLink } from "./types";
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
