"use client";

interface ToolBarChartProps {
  tools: [string, number][];
}

export function ToolBarChart({ tools }: ToolBarChartProps) {
  if (!tools.length) {
    return <div className="mono text-xs" style={{ color: "var(--text-3)" }}>No tool data</div>;
  }

  const max = tools[0][1];
  const total = tools.slice(0, 6).reduce((sum, [, n]) => sum + n, 0);

  return (
    <div className="space-y-3">
      {tools.slice(0, 6).map(([name, count]) => {
        const pct = total > 0 ? Math.round((count / total) * 100) : 0;

        return (
          <div key={name} className="flex items-center gap-2.5">
            <span
              className="tbadge tbadge-tool flex-shrink-0"
              style={{ width: "52px", justifyContent: "center" }}
            >
              {name.length > 7 ? name.slice(0, 6) + "…" : name}
            </span>

            <div
              className="flex-1 relative"
              style={{ height: "2px", background: "var(--border)" }}
              role="meter"
              aria-valuenow={pct}
              aria-valuemin={0}
              aria-valuemax={100}
              aria-label={`${name}: ${pct}%`}
            >
              <div
                className="absolute left-0 top-0 h-full transition-all duration-500"
                style={{
                  width: `${(count / max) * 100}%`,
                  background: "var(--amber)",
                }}
              />
            </div>

            <div className="flex items-center gap-2 flex-shrink-0">
              <span className="mono tabular text-xs" style={{ color: "var(--text-2)", minWidth: "28px", textAlign: "right" }}>
                {count}
              </span>
              <span className="mono tabular text-2xs" style={{ color: "var(--text-3)", minWidth: "28px", textAlign: "right" }}>
                {pct}%
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
