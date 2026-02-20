"use client";

import { AreaChart, Area, ResponsiveContainer, Tooltip } from "recharts";

interface SparklineProps {
  data: number[];
  color?: string;
  height?: number;
}

export function Sparkline({ data, color = "#22d3ee", height = 40 }: SparklineProps) {
  const chartData = data.map((value, i) => ({ i, value }));

  return (
    <ResponsiveContainer width="100%" height={height}>
      <AreaChart data={chartData} margin={{ top: 2, right: 2, bottom: 2, left: 2 }}>
        <defs>
          <linearGradient id={`sg-${color.replace("#", "")}`} x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor={color} stopOpacity={0.3} />
            <stop offset="100%" stopColor={color} stopOpacity={0} />
          </linearGradient>
        </defs>
        <Tooltip
          content={({ active, payload }) =>
            active && payload?.[0] ? (
              <div
                className="text-xs mono px-2 py-1 rounded"
                style={{ background: "rgba(13,13,15,0.9)", border: "1px solid rgba(255,255,255,0.1)" }}
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
          fill={`url(#sg-${color.replace("#", "")})`}
          dot={false}
          isAnimationActive={false}
        />
      </AreaChart>
    </ResponsiveContainer>
  );
}
