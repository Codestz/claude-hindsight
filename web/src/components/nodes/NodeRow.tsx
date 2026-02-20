"use client";

import { useState } from "react";
import type { NodeResponse } from "@/lib/types";
import { nodeTypeColor, timeAgo } from "@/lib/utils";

const TYPE_ICONS: Record<string, string> = {
  user: "◉",
  assistant: "◈",
  tool_use: "⊕",
  tool_result: "⊗",
  thinking: "◌",
  error: "✗",
};

const TYPE_BADGES: Record<string, { label: string; cls: string }> = {
  user: { label: "user", cls: "text-accent-green bg-accent-green/10" },
  assistant: { label: "assistant", cls: "text-text-muted bg-white/5" },
  tool_use: { label: "tool", cls: "text-accent-cyan bg-accent-cyan/10" },
  tool_result: { label: "result", cls: "text-accent-green bg-accent-green/10" },
  thinking: { label: "think", cls: "text-accent-magenta bg-accent-magenta/10" },
  error: { label: "error", cls: "text-accent-red bg-accent-red/10" },
};

interface NodeRowProps {
  node: NodeResponse;
  defaultExpanded?: boolean;
}

export function NodeRow({ node, defaultExpanded = false }: NodeRowProps) {
  const [expanded, setExpanded] = useState(defaultExpanded || node.depth < 2);

  const hasChildren = node.children.length > 0;
  const badge = TYPE_BADGES[node.node_type] ?? { label: node.node_type, cls: "text-text-muted bg-white/5" };
  const icon = TYPE_ICONS[node.node_type] ?? "·";
  const colorCls = nodeTypeColor(node.node_type, node.color);

  return (
    <div>
      <div
        className={`flex items-start gap-2 py-1.5 px-3 rounded transition-colors cursor-default ${
          node.has_error ? "bg-accent-red/5" : "hover:bg-white/[0.02]"
        }`}
        style={{ paddingLeft: `${node.depth * 20 + 12}px` }}
        onClick={() => hasChildren && setExpanded((e) => !e)}
      >
        {/* Expand toggle */}
        <span
          className={`text-xs flex-shrink-0 mt-0.5 w-4 text-center ${hasChildren ? "cursor-pointer text-text-muted hover:text-text-primary" : "opacity-0"}`}
        >
          {expanded ? "▾" : "▸"}
        </span>

        {/* Icon */}
        <span className={`flex-shrink-0 text-sm mt-0.5 ${colorCls}`}>{icon}</span>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <span
              className={`text-xs px-1.5 py-0.5 rounded font-mono font-medium ${badge.cls}`}
            >
              {badge.label}
            </span>
            <span className="text-text-primary text-sm truncate">{node.label}</span>
            {node.has_error && (
              <span className="text-accent-red text-xs px-1 py-0.5 rounded bg-accent-red/10">error</span>
            )}
          </div>
          {node.timestamp && (
            <div className="text-text-muted text-xs mt-0.5 mono">
              {timeAgo(Math.floor(node.timestamp / 1000))}
            </div>
          )}
        </div>
      </div>

      {/* Children */}
      {hasChildren && expanded && (
        <div>
          {node.children.map((child, i) => (
            <NodeRow key={child.uuid ?? i} node={child} />
          ))}
        </div>
      )}
    </div>
  );
}
