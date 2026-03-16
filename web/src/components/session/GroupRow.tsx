/**
 * Collapsed group row for consecutive internal nodes.
 *
 * Shows a count badge, group label, and time range.
 * Expandable to reveal individual nodes within the group.
 */

import type { NodeResponse } from "@/lib/types";
import { formatTimestamp } from "@/lib/utils";
import { ExecutionRow } from "./ExecutionRow";

interface GroupRowProps {
  group: { nodes: NodeResponse[]; label: string };
  isExpanded: boolean;
  isSelected: boolean;
  onToggle: () => void;
  onSelect: (node: NodeResponse) => void;
}

export function GroupRow({ group, isExpanded, isSelected, onToggle, onSelect }: GroupRowProps) {
  const first = group.nodes[0];
  const last = group.nodes[group.nodes.length - 1];

  return (
    <div>
      <div
        onClick={onToggle}
        role="button"
        tabIndex={-1}
        style={{
          display: "flex", alignItems: "center", height: "36px",
          paddingRight: "10px", gap: "6px", cursor: "pointer",
          background: isSelected ? "rgba(129, 140, 248, 0.06)" : "rgba(167, 139, 250, 0.03)",
          transition: "background 0.1s", userSelect: "none", flexShrink: 0,
        }}
        onMouseEnter={(e) => {
          if (!isSelected) e.currentTarget.style.background = "rgba(167, 139, 250, 0.06)";
        }}
        onMouseLeave={(e) => {
          if (!isSelected) e.currentTarget.style.background = "rgba(167, 139, 250, 0.03)";
        }}
      >
        <span style={{ width: "3px", alignSelf: "stretch", background: "var(--purple)", opacity: 0.4, flexShrink: 0 }} />
        <span style={{ width: "5px", flexShrink: 0 }} />

        <span style={{
          fontSize: "8px", color: "var(--text-3)", flexShrink: 0, width: "10px", textAlign: "center",
          transition: "transform 0.12s", transform: isExpanded ? "rotate(0deg)" : "rotate(-90deg)",
        }}>
          {"\u25BE"}
        </span>

        <span style={{ width: "6px", height: "6px", borderRadius: "50%", background: "var(--purple)", flexShrink: 0, opacity: 0.6 }} />

        <span style={{ fontFamily: "var(--font-mono)", fontSize: "10px", fontWeight: 500, color: "var(--text-3)", flexShrink: 0, letterSpacing: "0.03em" }}>
          internal
        </span>

        <span style={{
          fontFamily: "var(--font-mono)", fontSize: "9px", fontWeight: 600,
          color: "var(--purple)", background: "rgba(167, 139, 250, 0.12)",
          padding: "1px 6px", borderRadius: "8px", flexShrink: 0,
        }}>
          {group.nodes.length}
        </span>

        <span style={{
          fontSize: "12px", color: "var(--text-2)", fontFamily: "var(--font-sans)",
          overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1,
        }}>
          {group.label}
        </span>

        {first.timestamp != null && last.timestamp != null && (
          <span style={{
            fontSize: "10px", color: "var(--text-3)", fontFamily: "var(--font-mono)",
            fontVariantNumeric: "tabular-nums", flexShrink: 0, opacity: 0.7,
          }}>
            {formatTimestamp(first.timestamp)}{"\u2013"}{formatTimestamp(last.timestamp)}
          </span>
        )}
      </div>

      {isExpanded && (
        <div style={{ borderLeft: "2px solid rgba(167, 139, 250, 0.15)", marginLeft: "10px" }}>
          {group.nodes.map((node, i) => (
            <ExecutionRow
              key={node.uuid ?? `group-${i}`}
              node={node}
              isSelected={false}
              onSelect={() => onSelect(node)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
