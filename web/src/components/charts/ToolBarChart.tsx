"use client";

interface ToolBarChartProps {
  tools: [string, number][];
}

export function ToolBarChart({ tools }: ToolBarChartProps) {
  if (!tools.length) return <div className="text-text-muted text-sm">No tool data</div>;

  const max = tools[0][1];

  return (
    <div className="space-y-2">
      {tools.slice(0, 5).map(([name, count]) => (
        <div key={name} className="flex items-center gap-3">
          <div className="text-text-muted text-xs mono w-20 truncate flex-shrink-0">{name}</div>
          <div className="flex-1 h-1.5 rounded-full" style={{ background: "rgba(255,255,255,0.06)" }}>
            <div
              className="h-full rounded-full transition-all duration-500"
              style={{
                width: `${(count / max) * 100}%`,
                background: "linear-gradient(90deg, #22d3ee, #4ade80)",
              }}
            />
          </div>
          <div className="text-text-muted text-xs mono w-10 text-right flex-shrink-0">{count}</div>
        </div>
      ))}
    </div>
  );
}
