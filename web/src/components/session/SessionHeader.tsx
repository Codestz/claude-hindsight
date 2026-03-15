import React from "react";
import { Link } from "react-router-dom";
import { formatBytes, formatCost, formatTokens, shortId, shortModel, timeAgo } from "@/lib/utils";
import type { SessionHeaderProps } from "./types";

export const SessionHeader = React.memo(function SessionHeader({ session, otelSummary, stats }: SessionHeaderProps) {
  return (
    <div style={{ flexShrink: 0, marginBottom: "6px" }}>
      <div style={{
        display: "flex", alignItems: "center", gap: "8px", flexWrap: "wrap",
      }}>
        {/* Back + ID */}
        <Link
          to={`/projects/${encodeURIComponent(session.project_name)}`}
          style={{
            fontSize: "13px", color: "var(--text-3)", textDecoration: "none",
            fontFamily: "var(--font-mono)", transition: "color 0.12s", flexShrink: 0,
          }}
          onMouseEnter={(e) => { e.currentTarget.style.color = "var(--text-2)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.color = "var(--text-3)"; }}
        >
          &larr; {session.project_name}
        </Link>
        <span style={{ color: "var(--border-3)", fontSize: "13px" }}>/</span>
        <span style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--text-1)", fontWeight: 500 }}>
          {shortId(session.session_id)}
        </span>

        {/* All pills — meta + stats in one row */}
        <div style={{ display: "flex", gap: "5px", marginLeft: "auto", alignItems: "center", flexWrap: "wrap" }}>
          <Pill label={shortModel(session.model)} color="var(--indigo)" />
          <Pill label={timeAgo(session.created_at)} />
          <Pill label={formatBytes(session.file_size)} />

          {/* Stats inline */}
          <Pill label={`${stats.totalTurns} turns`} />
          <Pill label={`${stats.toolCalls} tools`} color="var(--amber)" />
          {stats.errorCount > 0 && <Pill label={`${stats.errorCount} err`} color="var(--rose)" />}
          {stats.totalTokens > 0 && <Pill label={formatTokens(stats.totalTokens)} color="var(--sky)" />}
          {stats.costUsd != null && stats.costUsd > 0 && <Pill label={formatCost(stats.costUsd)} color="var(--amber)" />}
          {stats.durationMs != null && <Pill label={formatDuration(stats.durationMs)} />}

          {session.subagent_models?.map((m) => (
            <Pill key={m} label={shortModel(m)} color="var(--violet)" />
          ))}
        </div>
      </div>
    </div>
  );
});

function Pill({ label, color }: { label: string; color?: string }) {
  return (
    <span style={{
      fontFamily: "var(--font-mono)", fontSize: "10px", fontWeight: 500,
      color: color ?? "var(--text-3)",
      background: color ? `color-mix(in srgb, ${color} 6%, transparent)` : "var(--bg-2)",
      border: `1px solid ${color ? `color-mix(in srgb, ${color} 12%, transparent)` : "var(--border-1)"}`,
      borderRadius: "10px", padding: "2px 7px",
      whiteSpace: "nowrap", fontVariantNumeric: "tabular-nums",
    }}>
      {label}
    </span>
  );
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const s = ms / 1000;
  if (s < 60) return `${s.toFixed(0)}s`;
  const m = Math.floor(s / 60);
  const rem = Math.round(s % 60);
  return `${m}m ${rem}s`;
}
