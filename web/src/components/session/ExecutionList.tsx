/**
 * Scrollable list of execution nodes with collapsible groups.
 *
 * Features:
 * - Consecutive internal nodes collapsed into GroupRow
 * - Auto-scroll on new nodes (direction-aware)
 * - Keyboard navigation (arrow keys)
 */

import { useEffect, useRef, useCallback, useMemo, useState } from "react";
import { ExecutionRow } from "./ExecutionRow";
import { GroupRow } from "./GroupRow";
import type { ExecutionListProps } from "./types";
import { buildDisplayItems } from "./utils";

export function ExecutionList({ nodes, selectedId, onSelect, autoScroll, newestFirst }: ExecutionListProps) {
  const topRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const [expandedGroups, setExpandedGroups] = useState<Set<number>>(new Set());
  const prevCountRef = useRef(nodes.length);

  const displayItems = useMemo(() => buildDisplayItems(nodes), [nodes]);

  // Auto-scroll only when new nodes are actually added
  useEffect(() => {
    const grew = nodes.length > prevCountRef.current;
    prevCountRef.current = nodes.length;
    if (!grew || !autoScroll) return;
    if (newestFirst) {
      topRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
    } else {
      bottomRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
    }
  }, [nodes.length, autoScroll, newestFirst]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key !== "ArrowUp" && e.key !== "ArrowDown") return;
      e.preventDefault();
      const currentIdx = selectedId ? nodes.findIndex((n) => n.uuid === selectedId) : -1;
      const nextIdx = e.key === "ArrowDown"
        ? Math.min(currentIdx + 1, nodes.length - 1)
        : Math.max(currentIdx - 1, 0);
      if (nextIdx >= 0 && nextIdx < nodes.length) onSelect(nodes[nextIdx]);
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
    <div tabIndex={0} onKeyDown={handleKeyDown} style={{ outline: "none", height: "100%" }}>
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
