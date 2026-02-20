"use client";

import { useState } from "react";
import type { NodeResponse } from "@/lib/types";
import { timeAgo } from "@/lib/utils";

const TYPE_BADGES: Record<string, { label: string; cls: string }> = {
  user:        { label: "USER",   cls: "tbadge-user" },
  assistant:   { label: "ASST",  cls: "tbadge-asst" },
  tool_use:    { label: "TOOL",  cls: "tbadge-tool" },
  tool_result: { label: "RESULT",cls: "tbadge-result" },
  thinking:    { label: "THINK", cls: "tbadge-think" },
  error:       { label: "ERR",   cls: "tbadge-err" },
  subagent:    { label: "SUB",   cls: "tbadge-sub" },
};

interface NodeRowProps {
  node: NodeResponse;
  defaultExpanded?: boolean;
}

export function NodeRow({ node, defaultExpanded = false }: NodeRowProps) {
  const [expanded, setExpanded] = useState(defaultExpanded || node.depth < 2);

  const hasChildren = node.children.length > 0;
  const badge = TYPE_BADGES[node.node_type] ?? { label: node.node_type.toUpperCase(), cls: "tbadge-asst" };

  return (
    <div>
      <div
        className="flex items-start gap-2 py-1.5 px-3 transition-colors cursor-default"
        style={{
          paddingLeft: `${node.depth * 20 + 12}px`,
          background: node.has_error ? "rgba(255,69,69,0.04)" : "transparent",
        }}
        onMouseEnter={(e) => {
          if (!node.has_error) (e.currentTarget as HTMLElement).style.background = "var(--bg-2)";
        }}
        onMouseLeave={(e) => {
          (e.currentTarget as HTMLElement).style.background = node.has_error ? "rgba(255,69,69,0.04)" : "transparent";
        }}
        onClick={() => hasChildren && setExpanded((e) => !e)}
      >
        {/* Expand toggle */}
        <span
          className="mono flex-shrink-0 mt-0.5 w-4 text-center"
          style={{
            fontSize: "10px",
            color: "var(--text-3)",
            opacity: hasChildren ? 1 : 0,
            cursor: hasChildren ? "pointer" : "default",
          }}
        >
          {expanded ? "▾" : "▸"}
        </span>

        {/* Badge */}
        <span className={`tbadge ${badge.cls} flex-shrink-0 mt-0.5`}>
          {badge.label}
        </span>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <span style={{ color: "var(--text-1)", fontSize: "13px" }}>{node.label}</span>
            {node.has_error && (
              <span className="tbadge tbadge-err">error</span>
            )}
          </div>
          {node.timestamp && (
            <div className="mono text-2xs mt-0.5" style={{ color: "var(--text-3)" }}>
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
