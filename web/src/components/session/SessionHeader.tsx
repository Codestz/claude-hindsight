import React from "react";
import { Link } from "react-router-dom";
import { formatBytes, formatCost, formatDuration, formatTokens, shortId, shortModel, timeAgo } from "@/lib/utils";
import type { SessionHeaderProps } from "./types";

export const SessionHeader = React.memo(function SessionHeader({ session, otelSummary, stats }: SessionHeaderProps) {
  const hasOtel = otelSummary && otelSummary.api_requests > 0;

  return (
    <div style={{ flexShrink: 0, marginBottom: "8px" }}>
      {/* Navigation breadcrumb */}
      <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "8px" }}>
        <Link
          to={`/projects/${encodeURIComponent(session.project_name)}`}
          style={{
            fontSize: "13px", color: "var(--text-3)", textDecoration: "none",
            fontFamily: "var(--font-mono)", transition: "color 0.12s",
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
        {session.subagent_models?.map((m) => (
          <Pill key={m} label={shortModel(m)} color="var(--violet)" />
        ))}
      </div>

      {/* Stats card */}
      <div style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(100px, 1fr))",
        gap: "1px",
        background: "var(--border-1)",
        borderRadius: "var(--radius-md)",
        overflow: "hidden",
        border: "1px solid var(--border-1)",
      }}>
        <StatCell label="Model" value={shortModel(session.model)} color="var(--indigo)" />
        <StatCell label="Turns" value={String(stats.totalTurns)} />
        <StatCell label="Tools" value={String(stats.toolCalls)} color="var(--amber)" />
        {stats.errorCount > 0 && <StatCell label="Errors" value={String(stats.errorCount)} color="var(--rose)" />}
        {stats.durationMs != null && <StatCell label="Duration" value={formatDuration(stats.durationMs)} />}
        <StatCell label="Size" value={formatBytes(session.file_size)} />
        <StatCell label="Age" value={timeAgo(session.created_at)} />

        {/* Token breakdown */}
        {hasOtel && (
          <>
            <StatCell label="Input" value={formatTokens(otelSummary.input_tokens)} color="var(--sky)" />
            <StatCell label="Output" value={formatTokens(otelSummary.output_tokens)} color="var(--emerald)" />
            <StatCell label="Cache Read" value={formatTokens(otelSummary.cache_read_tokens)} color="var(--violet)" />
            <StatCell label="Cache Write" value={formatTokens(otelSummary.cache_creation_tokens)} color="var(--amber)" />
          </>
        )}
        {!hasOtel && stats.totalTokens > 0 && (
          <StatCell label="Tokens" value={formatTokens(stats.totalTokens)} color="var(--sky)" />
        )}

        {stats.costUsd != null && stats.costUsd > 0 && (
          <StatCell label="Cost" value={formatCost(stats.costUsd)} color="var(--amber)" highlight />
        )}
      </div>
    </div>
  );
});

function StatCell({ label, value, color, highlight }: {
  label: string;
  value: string;
  color?: string;
  highlight?: boolean;
}) {
  return (
    <div style={{
      background: highlight ? "rgba(245, 158, 11, 0.04)" : "var(--bg-1)",
      padding: "6px 10px",
      display: "flex", flexDirection: "column", gap: "1px",
    }}>
      <span style={{
        fontFamily: "var(--font-mono)", fontSize: "9px", fontWeight: 600,
        letterSpacing: "0.06em", textTransform: "uppercase",
        color: "var(--text-3)",
      }}>
        {label}
      </span>
      <span style={{
        fontFamily: "var(--font-mono)", fontSize: "13px", fontWeight: 600,
        color: color ?? "var(--text-1)",
        fontVariantNumeric: "tabular-nums",
      }}>
        {value}
      </span>
    </div>
  );
}

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
