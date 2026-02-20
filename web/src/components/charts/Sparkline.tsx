"use client";

import { AreaChart, Area, ResponsiveContainer, Tooltip, XAxis, CartesianGrid } from "recharts";

interface SparklineProps {
  data: number[];
  color?: string;
  height?: number;
  showLabels?: boolean;
}

const DAY_ABBR = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

function getDayLabels(count: number): string[] {
  const today = new Date();
  const labels: string[] = [];
  for (let i = count - 1; i >= 0; i--) {
    const d = new Date(today);
    d.setDate(today.getDate() - i);
    labels.push(DAY_ABBR[d.getDay()]);
  }
  return labels;
}

export function Sparkline({
  data,
  color = "#00FF88",
  height = 80,
  showLabels = false,
}: SparklineProps) {
  const labels = showLabels ? getDayLabels(data.length) : [];
  const chartData = data.map((value, i) => ({ i, value, day: labels[i] ?? "" }));
  const gradientId = `sg-${color.replace("#", "")}`;

  return (
    <ResponsiveContainer width="100%" height={height}>
      <AreaChart data={chartData} margin={{ top: 4, right: 0, bottom: 0, left: 0 }}>
        <defs>
          <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor={color} stopOpacity={0.06} />
            <stop offset="100%" stopColor={color} stopOpacity={0} />
          </linearGradient>
        </defs>

        <CartesianGrid
          vertical={false}
          stroke="#1C1C1C"
          strokeDasharray="0"
        />

        {showLabels && (
          <XAxis
            dataKey="day"
            tick={{ fill: "#606060", fontSize: 9, fontFamily: "JetBrains Mono" }}
            tickLine={false}
            axisLine={false}
            interval={1}
            height={18}
          />
        )}

        <Tooltip
          content={({ active, payload }) =>
            active && payload?.[0] ? (
              <div
                className="mono text-xs px-2 py-1"
                style={{
                  background: "var(--bg-3)",
                  border: "1px solid var(--border)",
                  borderRadius: 0,
                  color: "var(--text-1)",
                  fontVariantNumeric: "tabular-nums",
                }}
              >
                {payload[0].value} sessions
              </div>
            ) : null
          }
        />

        <Area
          type="monotone"
          dataKey="value"
          stroke={color}
          strokeWidth={1.5}
          fill={`url(#${gradientId})`}
          dot={false}
          isAnimationActive={false}
        />
      </AreaChart>
    </ResponsiveContainer>
  );
}
