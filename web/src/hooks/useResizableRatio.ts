/**
 * Hook for managing a resizable panel ratio with localStorage persistence.
 *
 * Handles: initial load from storage, clamping by min widths,
 * preset snapping, and persist-on-change.
 */

import { useCallback, useRef, useState } from "react";

interface UseResizableRatioOptions {
  storageKey: string;
  defaultRatio: number;
  minLeftPx: number;
  minRightPx: number;
}

interface UseResizableRatioResult {
  ratio: number;
  setRatio: (r: number) => void;
  clamp: (r: number) => number;
  persist: (r: number) => void;
  applyPreset: (p: number) => void;
  containerRef: React.RefObject<HTMLDivElement | null>;
}

export function useResizableRatio({
  storageKey,
  defaultRatio,
  minLeftPx,
  minRightPx,
}: UseResizableRatioOptions): UseResizableRatioResult {
  const containerRef = useRef<HTMLDivElement>(null);

  const [ratio, setRatio] = useState(() => {
    try {
      const saved = localStorage.getItem(storageKey);
      if (saved) return parseFloat(saved);
    } catch { /* ignore */ }
    return defaultRatio;
  });

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

  const applyPreset = useCallback(
    (p: number) => {
      const clamped = clamp(p);
      setRatio(clamped);
      persist(clamped);
    },
    [clamp, persist],
  );

  return { ratio, setRatio, clamp, persist, applyPreset, containerRef };
}
