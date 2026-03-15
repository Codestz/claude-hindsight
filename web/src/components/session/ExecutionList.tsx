import { useEffect, useRef, useCallback, useMemo, useState } from "react";
import type { NodeResponse } from "@/lib/types";
import { ExecutionRow } from "./ExecutionRow";
import { formatTimestamp } from "@/lib/utils";
import type { ExecutionListProps } from "./types";
import { buildDisplayItems } from "./utils";

function GroupRow({
  group,
  isExpanded,
  isSelected,
  onToggle,
  onSelect,
}: {
  group: { nodes: NodeResponse[]; label: string };
  isExpanded: boolean;
  isSelected: boolean;
  onToggle: () => void;
  onSelect: (node: NodeResponse) => void;
}) {
  const first = group.nodes[0];
  const last = group.nodes[group.nodes.length - 1];

  return (
    <div>
      <div
        onClick={onToggle}
        role="button"
        tabIndex={-1}
        style={{
          display: "flex",
          alignItems: "center",
          height: "36px",
          paddingRight: "10px",
          gap: "6px",
          cursor: "pointer",
          background: isSelected ? "rgba(129, 140, 248, 0.06)" : "rgba(167, 139, 250, 0.03)",
          transition: "background 0.1s",
          userSelect: "none",
          flexShrink: 0,
        }}
        onMouseEnter={(e) => {
          if (!isSelected) e.currentTarget.style.background = "rgba(167, 139, 250, 0.06)";
        }}
        onMouseLeave={(e) => {
          if (!isSelected) e.currentTarget.style.background = "rgba(167, 139, 250, 0.03)";
        }}
      >
        {/* Left color border */}
        <span style={{
          width: "3px", alignSelf: "stretch",
          background: "var(--purple)", opacity: 0.4, flexShrink: 0,
        }} />
        <span style={{ width: "5px", flexShrink: 0 }} />

        {/* Expand arrow */}
        <span style={{
          fontSize: "8px", color: "var(--text-3)", flexShrink: 0,
          width: "10px", textAlign: "center",
          transition: "transform 0.12s",
          transform: isExpanded ? "rotate(0deg)" : "rotate(-90deg)",
        }}>
          {"\u25BE"}
        </span>

        {/* Agent icon */}
        <span style={{
          width: "6px", height: "6px", borderRadius: "50%",
          background: "var(--purple)", flexShrink: 0, opacity: 0.6,
        }} />

        {/* Badge */}
        <span style={{
          fontFamily: "var(--font-mono)", fontSize: "10px", fontWeight: 500,
          color: "var(--text-3)", flexShrink: 0, letterSpacing: "0.03em",
        }}>
          internal
        </span>

        {/* Count badge */}
        <span style={{
          fontFamily: "var(--font-mono)", fontSize: "9px", fontWeight: 600,
          color: "var(--purple)", background: "rgba(167, 139, 250, 0.12)",
          padding: "1px 6px", borderRadius: "8px", flexShrink: 0,
        }}>
          {group.nodes.length}
        </span>

        {/* Label */}
        <span style={{
          fontSize: "12px", color: "var(--text-2)",
          fontFamily: "var(--font-sans)",
          overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
          flex: 1,
        }}>
          {group.label}
        </span>

        {/* Time range */}
        {first.timestamp != null && last.timestamp != null && (
          <span style={{
            fontSize: "10px", color: "var(--text-3)",
            fontFamily: "var(--font-mono)", fontVariantNumeric: "tabular-nums",
            flexShrink: 0, opacity: 0.7,
          }}>
            {formatTimestamp(first.timestamp)}{"\u2013"}{formatTimestamp(last.timestamp)}
          </span>
        )}
      </div>

      {/* Expanded children */}
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


export function ExecutionList({ nodes, selectedId, onSelect, autoScroll, newestFirst }: ExecutionListProps) {
  const topRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const [expandedGroups, setExpandedGroups] = useState<Set<number>>(new Set());
  const prevCountRef = useRef(nodes.length);

  const displayItems = useMemo(() => buildDisplayItems(nodes), [nodes]);

  // Auto-scroll only when new nodes are actually added (not on sort/filter changes)
  useEffect(() => {
    const grew = nodes.length > prevCountRef.current;
    prevCountRef.current = nodes.length;
    if (!grew || !autoScroll) return;
    if (newestFirst) {
      // New nodes appear at the top — scroll to top
      topRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
    } else {
      bottomRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
    }
  }, [nodes.length, autoScroll, newestFirst]);

  // Keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key !== "ArrowUp" && e.key !== "ArrowDown") return;
      e.preventDefault();

      const currentIdx = selectedId
        ? nodes.findIndex((n) => n.uuid === selectedId)
        : -1;

      let nextIdx: number;
      if (e.key === "ArrowDown") {
        nextIdx = currentIdx < nodes.length - 1 ? currentIdx + 1 : currentIdx;
      } else {
        nextIdx = currentIdx > 0 ? currentIdx - 1 : 0;
      }

      if (nextIdx >= 0 && nextIdx < nodes.length) {
        onSelect(nodes[nextIdx]);
      }
    },
    [nodes, selectedId, onSelect],
  );

  const toggleGroup = (idx: number) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(idx)) next.delete(idx);
      else next.add(idx);
      return next;
    });
  };

  return (
    <div
      tabIndex={0}
      onKeyDown={handleKeyDown}
      style={{ outline: "none", height: "100%" }}
    >
      <div ref={topRef} />
      {displayItems.map((item, i) => {
        if (item.kind === "group") {
          const groupSelected = item.nodes.some((n) => n.uuid != null && n.uuid === selectedId);
          return (
            <GroupRow
              key={`group-${i}`}
              group={item}
              isExpanded={expandedGroups.has(i)}
              isSelected={groupSelected}
              onToggle={() => toggleGroup(i)}
              onSelect={onSelect}
            />
          );
        }
        return (
          <ExecutionRow
            key={item.node.uuid ?? `node-${i}`}
            node={item.node}
            isSelected={item.node.uuid != null && item.node.uuid === selectedId}
            onSelect={() => onSelect(item.node)}
          />
        );
      })}
      <div ref={bottomRef} />
    </div>
  );
}
