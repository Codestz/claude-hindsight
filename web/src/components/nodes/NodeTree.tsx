"use client";

import { useState, useMemo } from "react";
import type { NodeResponse } from "@/lib/types";
import { getNodeMeta, isInternalNode } from "@/lib/node-meta";
import { NodeRow } from "./NodeRow";

/** Filter configuration for the node tree */
export interface NodeFilter {
  /** Active type chips (e.g., "user", "assistant", "tool", "error", "thinking", "prompt") */
  types: Set<string>;
  /** Keyword search string */
  keyword: string;
}

interface NodeTreeProps {
  nodes: NodeResponse[];
  onSelect: (node: NodeResponse) => void;
  selectedId: string | null;
  filter?: NodeFilter;
}

/** Check if a node matches the given filter */
function nodeMatchesFilter(node: NodeResponse, filter: NodeFilter): boolean {
  const hasTypes = filter.types.size > 0;
  const hasKeyword = filter.keyword.trim().length > 0;

  if (!hasTypes && !hasKeyword) return true;

  let typeMatch = !hasTypes; // If no type filter, pass
  let keywordMatch = !hasKeyword; // If no keyword, pass

  // Type matching
  if (hasTypes) {
    for (const t of filter.types) {
      switch (t) {
        case "user":
          if (node.node_type === "user" && node.color !== "blue") { typeMatch = true; }
          break;
        case "assistant":
          if (node.node_type === "assistant" && node.color === "green") { typeMatch = true; }
          break;
        case "tool":
          if (node.node_type === "assistant" && node.color === "yellow") { typeMatch = true; }
          if (node.node_type === "user" && node.color === "blue") { typeMatch = true; }
          break;
        case "error":
          if (node.has_error) { typeMatch = true; }
          break;
        case "thinking":
          if (node.node_type === "assistant" && node.color === "magenta") { typeMatch = true; }
          break;
        case "prompt":
          if ((node as NodeResponse & { prompt_score?: number }).prompt_score != null &&
              (node as NodeResponse & { prompt_score?: number }).prompt_score! >= 40) {
            typeMatch = true;
          }
          break;
      }
    }
  }

  // Keyword matching
  if (hasKeyword) {
    const kw = filter.keyword.toLowerCase();
    const searchable = [
      node.label,
      node.summary,
      node.thinking,
      node.tool_use?.name,
      node.tool_result?.content,
      typeof node.message?.content === "string" ? node.message.content : null,
      Array.isArray(node.message?.content)
        ? node.message!.content
            .filter((b) => b.type === "text")
            .map((b) => (b as { type: "text"; text: string }).text)
            .join(" ")
        : null,
    ]
      .filter(Boolean)
      .join(" ")
      .toLowerCase();

    keywordMatch = searchable.includes(kw);
  }

  return typeMatch && keywordMatch;
}

/** Check if a node or any descendant matches the filter */
function nodeOrDescendantMatches(node: NodeResponse, filter: NodeFilter): boolean {
  if (nodeMatchesFilter(node, filter)) return true;
  return node.children.some((child) => nodeOrDescendantMatches(child, filter));
}

export function NodeTree({ nodes, onSelect, selectedId, filter }: NodeTreeProps) {
  const hasFilter = filter && (filter.types.size > 0 || filter.keyword.trim().length > 0);

  const visibleNodes = useMemo(() => {
    if (!hasFilter || !filter) return nodes;
    return nodes.filter((node) => nodeOrDescendantMatches(node, filter));
  }, [nodes, filter, hasFilter]);

  return (
    <div>
      {visibleNodes.map((node, i) => (
        <NodeTreeItem
          key={node.uuid ?? `root-${i}`}
          node={node}
          depth={0}
          onSelect={onSelect}
          selectedId={selectedId}
          filter={filter}
        />
      ))}
    </div>
  );
}

function NodeTreeItem({
  node,
  depth,
  onSelect,
  selectedId,
  filter,
}: {
  node: NodeResponse;
  depth: number;
  onSelect: (node: NodeResponse) => void;
  selectedId: string | null;
  filter?: NodeFilter;
}) {
  // Internal/decorative nodes start collapsed to reduce noise.
  const [expanded, setExpanded] = useState(!isInternalNode(node));

  const hasChildren = node.children.length > 0;
  const isSelected = node.uuid != null && node.uuid === selectedId;
  const meta = getNodeMeta(node);

  // Filter children
  const hasFilter = filter && (filter.types.size > 0 || filter.keyword.trim().length > 0);
  const visibleChildren = useMemo(() => {
    if (!hasFilter || !filter) return node.children;
    return node.children.filter((child) => nodeOrDescendantMatches(child, filter));
  }, [node.children, filter, hasFilter]);

  const hasVisibleChildren = visibleChildren.length > 0;

  // Get prompt score for badge display
  const promptScore = (node as NodeResponse & { prompt_score?: number }).prompt_score;

  return (
    <>
      <NodeRow
        node={node}
        depth={depth}
        meta={meta}
        isSelected={isSelected}
        hasChildren={hasVisibleChildren}
        isExpanded={expanded}
        onSelect={() => onSelect(node)}
        onToggle={() => setExpanded((e) => !e)}
        promptScore={promptScore}
      />
      {hasVisibleChildren &&
        expanded &&
        visibleChildren.map((child, i) => (
          <NodeTreeItem
            key={child.uuid ?? `${depth}-${i}`}
            node={child}
            depth={depth + 1}
            onSelect={onSelect}
            selectedId={selectedId}
            filter={filter}
          />
        ))}
    </>
  );
}
