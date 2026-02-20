"use client";

import { useState } from "react";

interface DayChartProps {
  // 14 values ordered oldest → newest (index 0 = 14 days ago)
  data: number[];
  color?: string;
}

function Bar({
  value,
  max,
  label,
  color,
}: {
  value: number;
  max: number;
  label: string;
  color: string;
}) {
  const [hovered, setHovered] = useState(false);
  const heightPct = max > 0 ? (value / max) * 100 : 0;

  return (
    <div
      style={{
        flex: 1,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: "6px",
      }}
    >
      {/* Value on hover */}
      <div
        style={{
          fontSize: "10px",
          fontFamily: "var(--font-mono)",
          color: "var(--text-2)",
          height: "14px",
          opacity: hovered && value > 0 ? 1 : 0,
          transition: "opacity 0.1s",
        }}
      >
        {value}
      </div>

      {/* Bar track */}
      <div
        style={{
          flex: 1,
          width: "100%",
          display: "flex",
          alignItems: "flex-end",
        }}
      >
        <div
          onMouseEnter={() => setHovered(true)}
          onMouseLeave={() => setHovered(false)}
          title={`${value} sessions`}
          style={{
            width: "100%",
            height: `${Math.max(heightPct, value > 0 ? 3 : 0)}%`,
            minHeight: value > 0 ? "3px" : "0",
            background: color,
            borderRadius: "2px 2px 0 0",
            opacity: hovered ? 1 : 0.5,
            transition: "opacity 0.12s, height 0.2s",
            cursor: "default",
          }}
        />
      </div>

      {/* Day label */}
      <div
        style={{
          fontSize: "9px",
          fontFamily: "var(--font-mono)",
          color: "var(--text-3)",
          height: "12px",
          lineHeight: "12px",
        }}
      >
        {label}
      </div>
    </div>
  );
}

export function DayChart({ data, color = "var(--green)" }: DayChartProps) {
  const max = Math.max(...data, 1);

  // Build day labels: date numbers for every other bar (avoids crowding)
  const today = new Date();
  const labels = data.map((_, i) => {
    const d = new Date(today);
    d.setDate(d.getDate() - (data.length - 1 - i));
    // Show date number on every even index
    return i % 2 === 0 ? String(d.getDate()) : "";
  });

  const total = data.reduce((a, b) => a + b, 0);
  const peak = Math.max(...data);

  return (
    <div>
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "baseline",
          justifyContent: "space-between",
          marginBottom: "16px",
        }}
      >
        <div>
          <div
            style={{
              fontSize: "13px",
              fontWeight: 600,
              color: "var(--text-1)",
              marginBottom: "2px",
            }}
          >
            Conversations / day
          </div>
          <div
            style={{ fontSize: "12px", color: "var(--text-3)" }}
          >
            Last {data.length} days
          </div>
        </div>

        <div style={{ textAlign: "right" }}>
          <div
            style={{
              fontSize: "20px",
              fontWeight: 700,
              fontFamily: "var(--font-mono)",
              color: "var(--text-1)",
              letterSpacing: "-0.03em",
              lineHeight: 1,
            }}
          >
            {total}
          </div>
          <div style={{ fontSize: "11px", color: "var(--text-3)", marginTop: "2px" }}>
            peak {peak}
          </div>
        </div>
      </div>

      {/* Bars */}
      <div
        style={{
          display: "flex",
          alignItems: "stretch",
          gap: "3px",
          height: "100px",
        }}
      >
        {data.map((v, i) => (
          <Bar key={i} value={v} max={max} label={labels[i]} color={color} />
        ))}
      </div>
    </div>
  );
}
