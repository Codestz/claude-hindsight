// Stacked horizontal bar showing input/output/cache-read/cache-creation
// token proportions with a color-coded legend.

import { formatTokens } from "@/lib/utils";

interface TokenBreakdownBarProps {
  input: number;
  output: number;
  cacheRead: number;
  cacheCreation: number;
}

const SEGMENTS: { key: keyof TokenBreakdownBarProps; label: string; color: string }[] = [
  { key: "input",         label: "Input",          color: "var(--cyan)" },
  { key: "output",        label: "Output",         color: "var(--green)" },
  { key: "cacheRead",     label: "Cache Read",     color: "var(--purple)" },
  { key: "cacheCreation", label: "Cache Creation", color: "var(--amber)" },
];

export function TokenBreakdownBar({ input, output, cacheRead, cacheCreation }: TokenBreakdownBarProps) {
  const values: Record<string, number> = { input, output, cacheRead, cacheCreation };
  const total = input + output + cacheRead + cacheCreation;

  if (total === 0) return null;

  return (
    <div>
      {/* Stacked bar */}
      <div
        style={{
          display: "flex",
          height: "10px",
          borderRadius: "5px",
          overflow: "hidden",
          background: "var(--border-2)",
        }}
      >
        {SEGMENTS.map(({ key, color }) => {
          const pct = (values[key] / total) * 100;
          if (pct === 0) return null;
          return (
            <div
              key={key}
              style={{
                width: `${pct}%`,
                background: color,
                opacity: 0.75,
                transition: "width 0.3s ease",
              }}
            />
          );
        })}
      </div>

      {/* Legend */}
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          gap: "16px",
          marginTop: "12px",
        }}
      >
        {SEGMENTS.map(({ key, label, color }) => {
          const v = values[key];
          if (v === 0) return null;
          return (
            <div
              key={key}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "6px",
                fontSize: "12px",
                fontFamily: "var(--font-sans)",
                color: "var(--text-2)",
              }}
            >
              <div
                style={{
                  width: "8px",
                  height: "8px",
                  borderRadius: "2px",
                  background: color,
                  opacity: 0.75,
                  flexShrink: 0,
                }}
              />
              <span>{label}</span>
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "11px",
                  color: "var(--text-3)",
                }}
              >
                {formatTokens(v)}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
