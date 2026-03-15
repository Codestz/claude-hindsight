import React from "react";
import type { NodeResponse } from "@/lib/types";
import { getNodeMeta } from "@/lib/node-meta";
import { formatTimestamp } from "@/lib/utils";

interface ExecutionRowProps {
  node: NodeResponse;
  isSelected: boolean;
  onSelect: () => void;
}

export const ExecutionRow = React.memo(function ExecutionRow({ node, isSelected, onSelect }: ExecutionRowProps) {
  const meta = getNodeMeta(node);
  const isTask = meta.badge === "Task";
  const toolName = node.tool_name ?? node.tool_use?.name ?? null;
  const firstFile = node.file_paths?.[0];
  const fileSegment = firstFile ? firstFile.split("/").filter(Boolean).pop() : null;

  return (
    <div
      onClick={onSelect}
      role="button"
      tabIndex={-1}
      style={{
        display: "flex",
        alignItems: "center",
        height: "36px",
        paddingRight: "10px",
        gap: "6px",
        cursor: "pointer",
        background: isSelected ? "rgba(129, 140, 248, 0.06)" : "transparent",
        transition: "background 0.1s",
        userSelect: "none",
        flexShrink: 0,
        position: "relative",
      }}
      onMouseEnter={(e) => {
        if (!isSelected) e.currentTarget.style.background = "rgba(255,255,255,0.02)";
      }}
      onMouseLeave={(e) => {
        if (!isSelected) e.currentTarget.style.background = "transparent";
      }}
    >
      {/* Left color border strip */}
      <span
        style={{
          width: "3px",
          alignSelf: "stretch",
          background: meta.color,
          opacity: isSelected ? 1 : 0.4,
          flexShrink: 0,
          transition: "opacity 0.1s",
        }}
      />

      <span style={{ width: "5px", flexShrink: 0 }} />

      {/* Color dot */}
      <span
        style={{
          width: "6px",
          height: "6px",
          borderRadius: "50%",
          background: meta.color,
          flexShrink: 0,
          opacity: isSelected ? 1 : 0.6,
        }}
      />

      {/* Type badge */}
      <span
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "10px",
          fontWeight: 500,
          letterSpacing: "0.03em",
          color: isSelected ? meta.color : "var(--text-3)",
          flexShrink: 0,
          width: "44px",
          textTransform: "lowercase",
        }}
      >
        {meta.badge}
      </span>

      {/* Tool name badge */}
      {toolName && (
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "10px",
            fontWeight: 500,
            color: "var(--amber)",
            background: "rgba(245, 158, 11, 0.08)",
            padding: "1px 5px",
            borderRadius: "var(--radius-sm)",
            flexShrink: 0,
          }}
        >
          {toolName}
        </span>
      )}

      {/* Summary */}
      <span
        style={{
          fontSize: "12px",
          color: isSelected ? "var(--text-1)" : "var(--text-2)",
          fontFamily: "var(--font-sans)",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          flex: 1,
          fontWeight: isSelected ? 500 : 400,
          lineHeight: 1.3,
        }}
      >
        {isTask
          ? (typeof node.message?.content === "string"
              ? node.message.content.match(/<summary>([\s\S]*?)<\/summary>/)?.[1] ?? node.label
              : node.label)
          : (node.summary || node.label)}
      </span>

      {/* File chip */}
      {fileSegment && (
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "9px",
            color: "var(--emerald)",
            background: "rgba(52, 211, 153, 0.08)",
            padding: "1px 5px",
            borderRadius: "var(--radius-sm)",
            flexShrink: 0,
            maxWidth: "100px",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {fileSegment}
        </span>
      )}

      {/* Error indicator */}
      {node.has_error && (
        <span
          style={{
            width: "6px",
            height: "6px",
            borderRadius: "50%",
            background: "var(--rose)",
            flexShrink: 0,
          }}
        />
      )}

      {/* Timestamp */}
      {node.timestamp != null && (
        <span
          style={{
            fontSize: "10px",
            color: "var(--text-3)",
            fontFamily: "var(--font-mono)",
            fontVariantNumeric: "tabular-nums",
            flexShrink: 0,
            opacity: 0.7,
          }}
        >
          {formatTimestamp(node.timestamp)}
        </span>
      )}
    </div>
  );
});
