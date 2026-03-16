// Pure utility functions for flattening the node tree into a conversation timeline.
// No React dependency — unit-testable.

import type { NodeResponse, Turn, TurnCost, SessionStats, OtelSessionSummary } from "./types";
import { isTaskNotification } from "./node-meta";

/** Filter configuration for the node tree / execution list */
export interface NodeFilter {
  /** Active type chips (e.g., "user", "assistant", "tool", "error", "thinking", "prompt") */
  types: Set<string>;
  /** Keyword search string */
  keyword: string;
}

/** Depth-first chronological flat list of all nodes */
export function flattenTree(roots: NodeResponse[]): NodeResponse[] {
  const result: NodeResponse[] = [];
  const walk = (nodes: NodeResponse[]) => {
    for (const n of nodes) {
      result.push(n);
      if (n.children?.length) walk(n.children);
    }
  };
  walk(roots);
  return result;
}

/** Determine the role of a node for turn grouping */
function nodeRole(node: NodeResponse): "user" | "assistant" | "system" {
  const t = node.node_type;
  if (t === "user") {
    // Tool results (color=blue) are part of the assistant flow
    if (node.color === "blue") return "assistant";
    return "user";
  }
  if (t === "assistant") return "assistant";
  return "system";
}

/** Group a flat list of nodes into conversation turns by role boundaries */
export function groupIntoTurns(flat: NodeResponse[]): Turn[] {
  const turns: Turn[] = [];
  let current: Turn | null = null;

  for (const node of flat) {
    const role = nodeRole(node);

    // System/progress/snapshot nodes go into the current turn if one exists,
    // or start their own
    if (role === "system") {
      if (current && current.role !== "user") {
        current.nodes.push(node);
      } else {
        current = {
          id: node.uuid ?? `turn-${turns.length}`,
          role: "system",
          nodes: [node],
          timestamp: node.timestamp,
        };
        turns.push(current);
      }
      continue;
    }

    if (!current || current.role !== role) {
      current = {
        id: node.uuid ?? `turn-${turns.length}`,
        role,
        nodes: [node],
        timestamp: node.timestamp,
      };
      turns.push(current);
    } else {
      current.nodes.push(node);
    }
  }

  return turns;
}

/** Aggregate stats for the session header */
export function computeSessionStats(
  flat: NodeResponse[],
  otel: OtelSessionSummary | null,
): SessionStats {
  let toolCalls = 0;
  let errorCount = 0;
  let totalTokens = 0;

  for (const node of flat) {
    if (node.node_type === "assistant" && node.color === "yellow") toolCalls++;
    if (node.has_error) errorCount++;
    const usage = node.token_usage ?? node.message?.usage;
    if (usage) {
      totalTokens +=
        (usage.input_tokens ?? 0) +
        (usage.output_tokens ?? 0) +
        (usage.cache_creation_input_tokens ?? 0) +
        (usage.cache_read_input_tokens ?? 0);
    }
  }

  // Use otel tokens if available (more accurate)
  if (otel && otel.api_requests > 0) {
    totalTokens =
      otel.input_tokens +
      otel.output_tokens +
      otel.cache_read_tokens +
      otel.cache_creation_tokens;
  }

  const turns = groupIntoTurns(flat);
  const userTurns = turns.filter((t) => t.role === "user");

  // Duration from first to last timestamp
  const timestamps = flat
    .map((n) => n.timestamp)
    .filter((t): t is number => t != null);
  const durationMs =
    timestamps.length >= 2
      ? Math.max(...timestamps) - Math.min(...timestamps)
      : null;

  // Cost-per-turn breakdown
  // Each "turn" = user turn + the assistant turn that follows it
  const turnCosts: TurnCost[] = [];
  for (let i = 0; i < turns.length; i++) {
    const turn = turns[i];
    if (turn.role !== "user") continue;

    const turnNodes = [...turn.nodes];
    // Include the next assistant turn's nodes (the response to this user turn)
    if (i + 1 < turns.length && turns[i + 1].role === "assistant") {
      turnNodes.push(...turns[i + 1].nodes);
    }

    let inp = 0, out = 0, cacheRead = 0, cacheWrite = 0, tools = 0;
    for (const n of turnNodes) {
      if (n.node_type === "assistant" && n.color === "yellow") tools++;
      const u = n.token_usage ?? n.message?.usage;
      if (u) {
        inp += u.input_tokens ?? 0;
        out += u.output_tokens ?? 0;
        cacheRead += u.cache_read_input_tokens ?? 0;
        cacheWrite += u.cache_creation_input_tokens ?? 0;
      }
    }

    turnCosts.push({
      turnIndex: turnCosts.length,
      inputTokens: inp,
      outputTokens: out,
      cacheReadTokens: cacheRead,
      cacheWriteTokens: cacheWrite,
      totalTokens: inp + out + cacheRead + cacheWrite,
      toolCalls: tools,
    });
  }

  return {
    totalTurns: userTurns.length,
    toolCalls,
    errorCount,
    totalTokens,
    costUsd: otel?.cost_usd ?? null,
    durationMs,
    turnCosts,
  };
}

/** Filter a flat node list using the same logic as NodeTree */
export function filterNodes(
  flat: NodeResponse[],
  filter: NodeFilter,
  activeFilePaths?: Set<string>,
): NodeResponse[] {
  const hasTypes = filter.types.size > 0;
  const hasKeyword = filter.keyword.trim().length > 0;

  const hasPathFilter = activeFilePaths != null && activeFilePaths.size > 0;

  if (!hasTypes && !hasKeyword && !hasPathFilter) return flat;

  return flat.filter((node) => {
    // Path filter: node must reference at least one matching path
    if (hasPathFilter) {
      const nodePaths = node.file_paths ?? [];
      if (!nodePaths.some((p) => activeFilePaths!.has(p))) return false;
    }
    let typeMatch = !hasTypes;
    let keywordMatch = !hasKeyword;

    if (hasTypes) {
      for (const t of filter.types) {
        const isTaskNotif = isTaskNotification(node);
        switch (t) {
          case "user":
            if (node.node_type === "user" && node.color !== "blue" && !isTaskNotif) typeMatch = true;
            break;
          case "assistant":
            if (node.node_type === "assistant" && node.color === "green") typeMatch = true;
            break;
          case "tool":
            if (node.node_type === "assistant" && node.color === "yellow") typeMatch = true;
            if (node.node_type === "user" && node.color === "blue") typeMatch = true;
            break;
          case "task":
            if (isTaskNotif) typeMatch = true;
            break;
          case "error":
            if (node.has_error) typeMatch = true;
            break;
          case "thinking":
            if (node.node_type === "assistant" && node.color === "magenta") typeMatch = true;
            break;
          case "prompt":
            if (node.prompt_score != null && node.prompt_score >= 40) typeMatch = true;
            break;
        }
      }
    }

    if (hasKeyword) {
      const kw = filter.keyword.toLowerCase();
      const searchable = [
        node.label,
        node.summary,
        node.thinking,
        node.tool_use?.name,
        node.tool_name,
        node.tool_result?.content,
        typeof node.message?.content === "string" ? node.message.content : null,
        Array.isArray(node.message?.content)
          ? node.message!.content
              .filter((b) => b.type === "text")
              .map((b) => (b as { type: "text"; text: string }).text)
              .join(" ")
          : null,
        node.file_paths?.join(" "),
      ]
        .filter(Boolean)
        .join(" ")
        .toLowerCase();

      keywordMatch = searchable.includes(kw);
    }

    return typeMatch && keywordMatch;
  });
}

/** Filter turns — a turn is visible if any of its nodes pass the filter */
export function filterTurns(
  turns: Turn[],
  filter: NodeFilter,
  activeFilePaths: Set<string>,
): Turn[] {
  const hasFilter = filter.types.size > 0 || filter.keyword.trim().length > 0;
  const hasPathFilter = activeFilePaths.size > 0;

  if (!hasFilter && !hasPathFilter) return turns;

  return turns.filter((turn) => {
    // Path filter: at least one node must reference a filtered path
    if (hasPathFilter) {
      const turnPaths = turn.nodes.flatMap((n) => n.file_paths ?? []);
      const hasMatchingPath = turnPaths.some((p) => activeFilePaths.has(p));
      if (!hasMatchingPath) return false;
    }

    if (!hasFilter) return true;

    return turn.nodes.some((node) => {
      const filtered = filterNodes([node], filter);
      return filtered.length > 0;
    });
  });
}
