/**
 * Resizable two-panel layout with drag handle and preset buttons.
 *
 * Features:
 * - Drag divider to resize
 * - Preset buttons (30/50/70) visible on hover
 * - localStorage persistence
 * - Min width constraints
 */

import { useCallback, useEffect, useState } from "react";
import { useResizableRatio } from "@/hooks/useResizableRatio";
import type { ResizablePanelProps } from "./types";

export function ResizablePanel({
  left,
  right,
  storageKey = "resizable-panel",
  defaultRatio = 0.35,
  minLeftPx = 250,
  minRightPx = 300,
  presets = [0.3, 0.5, 0.7],
}: ResizablePanelProps) {
  const { ratio, setRatio, clamp, persist, applyPreset, containerRef } =
    useResizableRatio({ storageKey, defaultRatio, minLeftPx, minRightPx });

  const [dragging, setDragging] = useState(false);
  const [hovering, setHovering] = useState(false);

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      e.preventDefault();
      (e.target as HTMLElement).setPointerCapture(e.pointerId);
      setDragging(true);
    },
    [],
  );

  const onPointerMove = useCallback(
    (e: React.PointerEvent) => {
      if (!dragging) return;
      const el = containerRef.current;
      if (!el) return;
      const rect = el.getBoundingClientRect();
      const raw = (e.clientX - rect.left) / rect.width;
      setRatio(clamp(raw));
    },
    [dragging, clamp, containerRef, setRatio],
  );

  const onPointerUp = useCallback(
    () => {
      if (dragging) {
        setDragging(false);
        persist(ratio);
      }
    },
    [dragging, ratio, persist],
  );

  // Prevent text selection while dragging
  useEffect(() => {
    if (!dragging) return;
    document.body.style.userSelect = "none";
    document.body.style.cursor = "col-resize";
    return () => {
      document.body.style.userSelect = "";
      document.body.style.cursor = "";
    };
  }, [dragging]);

  return (
    <div
      ref={containerRef}
      style={{ display: "flex", height: "100%", position: "relative" }}
    >
      {/* Left panel */}
      <div style={{ width: `${ratio * 100}%`, overflowY: "auto", flexShrink: 0 }}>
        {left}
      </div>

      {/* Drag handle */}
      <div
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onMouseEnter={() => setHovering(true)}
        onMouseLeave={() => setHovering(false)}
        style={{
          width: "5px",
          cursor: "col-resize",
          background: dragging ? "var(--indigo)" : hovering ? "var(--border-2)" : "var(--border-1)",
          transition: dragging ? "none" : "background 0.15s",
          position: "relative",
          flexShrink: 0,
        }}
      >
        {/* Preset buttons */}
        {(hovering || dragging) && (
          <div style={{
            position: "absolute", top: "8px", left: "50%",
            transform: "translateX(-50%)",
            display: "flex", flexDirection: "column", gap: "4px", zIndex: 10,
          }}>
            {presets.map((p) => (
              <button
                key={p}
                onPointerDown={(e) => e.stopPropagation()}
                onClick={(e) => { e.stopPropagation(); applyPreset(p); }}
                style={{
                  width: "24px", height: "18px", fontSize: "8px",
                  fontFamily: "var(--font-mono)", fontWeight: 600,
                  color: "var(--text-1)", background: "var(--bg-3)",
                  border: "1px solid var(--border-2)",
                  borderRadius: "var(--radius-sm)",
                  cursor: "pointer",
                  display: "flex", alignItems: "center", justifyContent: "center",
                  padding: 0,
                }}
              >
                {Math.round(p * 100)}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Right panel */}
      <div style={{ flex: 1, overflowY: "auto" }}>
        {right}
      </div>
    </div>
  );
}
