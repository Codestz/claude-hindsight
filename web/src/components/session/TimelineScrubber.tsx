import { useCallback, useRef } from "react";
import { getNodeMeta } from "@/lib/node-meta";
import type { TimelineScrubberProps } from "./types";

export function TimelineScrubber({ nodes, selectedId, onSelect, turnCosts }: TimelineScrubberProps) {
  const barRef = useRef<HTMLDivElement>(null);

  if (nodes.length === 0) return null;

  const timestamps = nodes.map((n) => n.timestamp).filter((t): t is number => t != null);
  if (timestamps.length < 2) return null;

  // Loop-based min/max — safe for large arrays (Math.min/max can stack overflow with spread)
  let minTs = timestamps[0];
  let maxTs = timestamps[0];
  for (let i = 1; i < timestamps.length; i++) {
    if (timestamps[i] < minTs) minTs = timestamps[i];
    if (timestamps[i] > maxTs) maxTs = timestamps[i];
  }
  const range = maxTs - minTs || 1;

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      const bar = barRef.current;
      if (!bar) return;
      const rect = bar.getBoundingClientRect();
      const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      const targetTs = minTs + ratio * range;
      let best = nodes[0];
      let bestDist = Infinity;
      for (const n of nodes) {
        if (n.timestamp == null) continue;
        const dist = Math.abs(n.timestamp - targetTs);
        if (dist < bestDist) { bestDist = dist; best = n; }
      }
      onSelect(best);
    },
    [nodes, minTs, range, onSelect],
  );

  const selectedNode = selectedId ? nodes.find((n) => n.uuid === selectedId) : null;
  const selectedPos = selectedNode?.timestamp != null
    ? ((selectedNode.timestamp - minTs) / range) * 100
    : null;

  // Token cost overlay — loop-based max (safe for large arrays)
  let maxTurnTokens = 1;
  if (turnCosts) {
    for (const t of turnCosts) {
      if (t.totalTokens > maxTurnTokens) maxTurnTokens = t.totalTokens;
    }
  }

  return (
    <div style={{ flexShrink: 0, padding: "2px 0 4px" }}>
      <div
        ref={barRef}
        onClick={handleClick}
        style={{
          position: "relative",
          height: "28px",
          background: "var(--bg-2)",
          borderRadius: "var(--radius-sm)",
          border: "1px solid var(--border-1)",
          cursor: "crosshair",
          overflow: "hidden",
        }}
      >
        {/* Token-per-turn bars as background */}
        {turnCosts && turnCosts.length > 1 && (
          <div style={{
            position: "absolute", inset: 0,
            display: "flex", alignItems: "flex-end",
            padding: "0 1px",
            pointerEvents: "none",
          }}>
            {turnCosts.map((turn) => {
              const h = Math.max(1, (turn.totalTokens / maxTurnTokens) * 100);
              return (
                <div
                  key={turn.turnIndex}
                  style={{
                    flex: 1,
                    height: `${h}%`,
                    background: turn.toolCalls > 0 ? "var(--amber)" : "var(--sky)",
                    opacity: 0.08,
                    minWidth: "1px",
                  }}
                />
              );
            })}
          </div>
        )}

        {/* Node tick marks */}
        {nodes.map((node, i) => {
          if (node.timestamp == null) return null;
          const pos = ((node.timestamp - minTs) / range) * 100;
          const meta = getNodeMeta(node);
          const isSel = node.uuid === selectedId;
          return (
            <span
              key={node.uuid ?? i}
              style={{
                position: "absolute",
                left: `${pos}%`,
                bottom: 0,
                width: isSel ? "3px" : "1px",
                height: isSel ? "100%" : "40%",
                background: meta.color,
                opacity: isSel ? 0.9 : 0.3,
                pointerEvents: "none",
              }}
            />
          );
        })}

        {/* Playhead */}
        {selectedPos != null && (
          <div style={{
            position: "absolute",
            left: `${selectedPos}%`,
            top: 0, bottom: 0,
            width: "2px",
            background: "var(--indigo)",
            transform: "translateX(-1px)",
            pointerEvents: "none",
            boxShadow: "0 0 4px rgba(129,140,248,0.6)",
          }} />
        )}

        {/* Time labels inside the bar */}
        <span style={{
          position: "absolute", left: "4px", top: "50%", transform: "translateY(-50%)",
          fontFamily: "var(--font-mono)", fontSize: "8px", color: "var(--text-3)",
          opacity: 0.6, pointerEvents: "none", fontVariantNumeric: "tabular-nums",
        }}>{formatTime(minTs)}</span>
        <span style={{
          position: "absolute", right: "4px", top: "50%", transform: "translateY(-50%)",
          fontFamily: "var(--font-mono)", fontSize: "8px", color: "var(--text-3)",
          opacity: 0.6, pointerEvents: "none", fontVariantNumeric: "tabular-nums",
        }}>{formatTime(maxTs)}</span>
      </div>
    </div>
  );
}

function formatTime(ms: number): string {
  return new Date(ms).toLocaleTimeString("en-US", {
    hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit",
  });
}
