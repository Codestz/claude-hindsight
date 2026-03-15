// Horizontal ranked bar chart.
// Works for: top tools, top MCPs, top files, any [label, count] dataset.

interface BarChartProps {
  data: [string, number][];
  color?: string;
  limit?: number;
  countLabel?: string;
  // Label column width — wider for file paths
  labelWidth?: string;
}

export function BarChart({
  data,
  color = "var(--green)",
  limit = 6,
  countLabel,
  labelWidth = "130px",
}: BarChartProps) {
  const rows = data.slice(0, limit);
  const max = Math.max(...rows.map(([, n]) => n), 1);

  if (rows.length === 0) {
    return (
      <div style={{ fontSize: "14px", color: "var(--text-3)", paddingTop: "8px" }}>
        No data
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
      {countLabel && (
        <div style={{
          fontSize: "11px", fontFamily: "var(--font-mono)", color: "var(--text-3)",
          letterSpacing: "0.06em", textTransform: "uppercase", marginBottom: "2px",
        }}>
          {countLabel}
        </div>
      )}

      {rows.map(([label, count]) => {
        const pct = (count / max) * 100;
        return (
          <div key={label} style={{ display: "flex", alignItems: "center", gap: "12px" }}>
            {/* Label */}
            <div
              style={{
                width: labelWidth,
                flexShrink: 0,
                fontSize: "13px",
                color: "var(--text-1)",
                fontFamily: "var(--font-sans)",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
              }}
              title={label}
            >
              {label}
            </div>

            {/* Bar track */}
            <div style={{
              flex: 1, height: "8px", background: "var(--border-2)",
              borderRadius: "4px", overflow: "hidden",
            }}>
              <div style={{
                width: `${pct}%`, height: "100%",
                background: color, borderRadius: "4px",
                opacity: 0.8, transition: "width 0.3s ease, opacity 0.15s",
              }} />
            </div>

            {/* Count */}
            <div style={{
              width: "36px", flexShrink: 0,
              fontSize: "12px", fontFamily: "var(--font-mono)",
              color: "var(--text-3)", textAlign: "right",
              fontVariantNumeric: "tabular-nums",
            }}>
              {count.toLocaleString()}
            </div>
          </div>
        );
      })}
    </div>
  );
}
