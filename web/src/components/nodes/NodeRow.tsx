import type { NodeResponse } from "@/lib/types";
import type { NodeMeta } from "@/lib/node-meta";

interface NodeRowProps {
  node: NodeResponse;
  depth: number;
  meta: NodeMeta;
  isSelected: boolean;
  hasChildren: boolean;
  isExpanded: boolean;
  onSelect: () => void;
  onToggle: () => void;
  promptScore?: number;
}

// Milliseconds → HH:MM:SS display
function formatMs(ms: number): string {
  const d = new Date(ms);
  return d.toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export function NodeRow({
  node,
  depth,
  meta,
  isSelected,
  hasChildren,
  isExpanded,
  onSelect,
  onToggle,
  promptScore,
}: NodeRowProps) {
  const selectedBg = "rgba(129, 140, 248, 0.06)";

  return (
    <div
      onClick={onSelect}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") onSelect(); }}
      style={{
        display: "flex",
        alignItems: "center",
        minHeight: "34px",
        paddingLeft: `${8 + depth * 14}px`,
        paddingRight: "12px",
        paddingTop: "4px",
        paddingBottom: "4px",
        gap: "8px",
        cursor: "pointer",
        background: isSelected ? selectedBg : "transparent",
        borderLeft: `3px solid ${isSelected ? meta.color : "transparent"}`,
        transition: "background 0.1s, border-color 0.1s",
        userSelect: "none",
      }}
      onMouseEnter={(e) => {
        if (!isSelected) (e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.02)";
      }}
      onMouseLeave={(e) => {
        if (!isSelected) (e.currentTarget as HTMLElement).style.background = "transparent";
      }}
    >
      {/* Expand/collapse toggle */}
      <button
        aria-label={isExpanded ? "Collapse" : "Expand"}
        onClick={(e) => {
          e.stopPropagation();
          if (hasChildren) onToggle();
        }}
        style={{
          width: "12px",
          height: "12px",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          background: "none",
          border: "none",
          cursor: hasChildren ? "pointer" : "default",
          color: hasChildren ? "var(--text-3)" : "transparent",
          fontSize: "8px",
          padding: 0,
          flexShrink: 0,
          lineHeight: 1,
          transition: "transform 0.12s",
          transform: hasChildren && isExpanded ? "rotate(0deg)" : "rotate(-90deg)",
        }}
      >
        {hasChildren ? "▾" : ""}
      </button>

      {/* Color dot — tiny semantic indicator */}
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

      {/* Type — minimal text label */}
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

      {/* Summary or label */}
      <span
        style={{
          fontSize: "13px",
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
        {node.summary || node.label}
      </span>

      {/* Prompt score */}
      {promptScore != null && promptScore >= 40 && (
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "9px",
            fontWeight: 500,
            color: "var(--emerald)",
            opacity: 0.7,
            flexShrink: 0,
          }}
        >
          {promptScore}%
        </span>
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
          {formatMs(node.timestamp)}
        </span>
      )}
    </div>
  );
}
