import { useCallback, useEffect, useRef, useState } from "react";

interface ResizablePanelProps {
  left: React.ReactNode;
  right: React.ReactNode;
  storageKey?: string;
  defaultRatio?: number;
  minLeftPx?: number;
  minRightPx?: number;
  presets?: number[];
}

export function ResizablePanel({
  left,
  right,
  storageKey = "resizable-panel",
  defaultRatio = 0.35,
  minLeftPx = 250,
  minRightPx = 300,
  presets = [0.3, 0.5, 0.7],
}: ResizablePanelProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [ratio, setRatio] = useState(() => {
    try {
      const saved = localStorage.getItem(storageKey);
      if (saved) return parseFloat(saved);
    } catch { /* ignore */ }
    return defaultRatio;
  });
  const [dragging, setDragging] = useState(false);
  const [hovering, setHovering] = useState(false);

  const clamp = useCallback(
    (r: number) => {
      const el = containerRef.current;
      if (!el) return r;
      const w = el.offsetWidth;
      const minL = minLeftPx / w;
      const maxL = 1 - minRightPx / w;
      return Math.min(Math.max(r, minL), maxL);
    },
    [minLeftPx, minRightPx],
  );

  const persist = useCallback(
    (r: number) => {
      try { localStorage.setItem(storageKey, String(r)); } catch { /* ignore */ }
    },
    [storageKey],
  );

  const applyPreset = (p: number) => {
    const clamped = clamp(p);
    setRatio(clamped);
    persist(clamped);
  };

  // Drag handling via pointer events
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
    [dragging, clamp],
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
      <div
        style={{
          width: `${ratio * 100}%`,
          overflowY: "auto",
          flexShrink: 0,
        }}
      >
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
          <div
            style={{
              position: "absolute",
              top: "8px",
              left: "50%",
              transform: "translateX(-50%)",
              display: "flex",
              flexDirection: "column",
              gap: "4px",
              zIndex: 10,
            }}
          >
            {presets.map((p) => (
              <button
                key={p}
                onPointerDown={(e) => e.stopPropagation()}
                onClick={(e) => { e.stopPropagation(); applyPreset(p); }}
                style={{
                  width: "24px",
                  height: "18px",
                  fontSize: "8px",
                  fontFamily: "var(--font-mono)",
                  fontWeight: 600,
                  color: "var(--text-1)",
                  background: "var(--bg-3)",
                  border: "1px solid var(--border-2)",
                  borderRadius: "var(--radius-sm)",
                  cursor: "pointer",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
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
      <div
        style={{
          flex: 1,
          overflowY: "auto",
          borderLeft: "none", // handle serves as visual border
        }}
      >
        {right}
      </div>
    </div>
  );
}
