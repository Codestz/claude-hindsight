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
  const selectedBg = "rgba(0, 255, 136, 0.06)";

  // Prompt badge color intensity scales with score
  const promptBadgeOpacity = promptScore != null && promptScore >= 40
    ? 0.5 + (promptScore / 100) * 0.5
    : 0;

  return (
    <div
      onClick={onSelect}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") onSelect(); }}
      style={{
        display: "flex",
        alignItems: "center",
        height: "40px",
        paddingLeft: `${10 + depth * 18}px`,
        paddingRight: "14px",
        gap: "6px",
        cursor: "pointer",
        background: isSelected ? selectedBg : "transparent",
        borderLeft: isSelected
          ? "2px solid var(--accent)"
          : "2px solid transparent",
        transition: "background 0.1s",
        userSelect: "none",
      }}
      onMouseEnter={(e) => {
        const el = e.currentTarget as HTMLElement;
        el.style.background = isSelected ? selectedBg : "var(--bg-2)";
      }}
      onMouseLeave={(e) => {
        const el = e.currentTarget as HTMLElement;
        el.style.background = isSelected ? selectedBg : "transparent";
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
          width: "16px",
          height: "16px",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          background: "none",
          border: "none",
          cursor: hasChildren ? "pointer" : "default",
          color: hasChildren ? "var(--text-3)" : "transparent",
          fontSize: "10px",
          padding: 0,
          flexShrink: 0,
          lineHeight: 1,
        }}
      >
        {hasChildren ? (isExpanded ? "▾" : "▸") : "·"}
      </button>

      {/* Node type icon */}
      <span
        style={{
          color: meta.color,
          fontSize: "14px",
          lineHeight: 1,
          flexShrink: 0,
        }}
      >
        {meta.icon}
      </span>

      {/* Badge */}
      <span
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "10px",
          fontWeight: 700,
          letterSpacing: "0.07em",
          textTransform: "uppercase",
          color: meta.color,
          flexShrink: 0,
          opacity: 0.85,
        }}
      >
        {meta.badge}
      </span>

      {/* Summary or label */}
      <span
        style={{
          fontSize: "13px",
          color: "var(--text-2)",
          fontFamily: "var(--font-sans)",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          flex: 1,
        }}
      >
        {node.summary || node.label}
      </span>

      {/* Prompt score badge */}
      {promptScore != null && promptScore >= 40 && (
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "10px",
            fontWeight: 600,
            color: "var(--green)",
            opacity: promptBadgeOpacity,
            flexShrink: 0,
            padding: "1px 5px",
            borderRadius: "var(--radius-sm)",
            background: "rgba(0, 255, 136, 0.08)",
            border: "1px solid rgba(0, 255, 136, 0.15)",
          }}
        >
          P:{promptScore}%
        </span>
      )}

      {/* Timestamp */}
      {node.timestamp != null && (
        <span
          style={{
            fontSize: "11px",
            color: "var(--text-3)",
            fontFamily: "var(--font-mono)",
            fontVariantNumeric: "tabular-nums",
            flexShrink: 0,
          }}
        >
          {formatMs(node.timestamp)}
        </span>
      )}
    </div>
  );
}
