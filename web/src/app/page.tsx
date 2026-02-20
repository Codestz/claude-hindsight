"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { ArrowRight } from "lucide-react";
import { api } from "@/lib/api";
import type { GlobalAnalytics, SessionFile } from "@/lib/types";
import { formatCost, formatTokens, formatBytes, shortId, timeAgo } from "@/lib/utils";
import { Header } from "@/components/layout/Header";
import { Sparkline } from "@/components/charts/Sparkline";
import { ToolBarChart } from "@/components/charts/ToolBarChart";

export default function DashboardPage() {
  const [analytics, setAnalytics] = useState<GlobalAnalytics | null>(null);
  const [sparkline, setSparkline] = useState<number[]>([]);
  const [recent, setRecent] = useState<SessionFile[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      api.globalAnalytics(),
      api.globalSparkline(14),
      api.sessions({ limit: 10 }),
    ])
      .then(([a, spark, sessions]) => {
        setAnalytics(a);
        setSparkline(spark);
        setRecent(sessions);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <LoadingState />;
  if (!analytics) return <ErrorState />;

  return (
    <div className="flex flex-col flex-1" style={{ background: "var(--bg)" }}>
      <Header title="Dashboard" subtitle="Global session overview" />

      <div className="flex-1 p-4 space-y-1">

        {/* ── Top stats bento (3 cols) ── */}
        <div
          className="bento"
          style={{ gridTemplateColumns: "repeat(3, 1fr)" }}
        >
          <BentoStat
            label="Total Sessions"
            value={analytics.total_sessions.toLocaleString()}
            sub={`${analytics.sessions_today} today · ${analytics.sessions_this_week} this week`}
            accent
          />
          <BentoStat
            label="Total Tokens"
            value={formatTokens(analytics.total_tokens)}
            sub="cumulative across all sessions"
          />
          <BentoStat
            label="Estimated Cost"
            value={formatCost(analytics.total_cost)}
            sub="based on model pricing"
            valueColor="var(--amber)"
          />
        </div>

        {/* ── Secondary bento (4 cols) ── */}
        <div
          className="bento"
          style={{ gridTemplateColumns: "repeat(4, 1fr)" }}
        >
          <BentoMini label="Projects" value={analytics.total_projects.toLocaleString()} />
          <BentoMini label="Today" value={analytics.sessions_today.toLocaleString()} />
          <BentoMini label="Errors" value={analytics.total_errors.toLocaleString()} alert={analytics.total_errors > 0} />
          <BentoMini label="Avg Size" value={formatBytes(analytics.avg_session_size)} />
        </div>

        {/* ── Activity + Tools bento ── */}
        <div
          className="bento"
          style={{ gridTemplateColumns: "2fr 1fr" }}
        >
          <div className="p-5" style={{ background: "var(--bg)" }}>
            <div className="flex items-center justify-between mb-4">
              <div>
                <div style={{ fontWeight: 600, fontSize: "13px", color: "var(--text-1)" }}>
                  Session Activity
                </div>
                <div className="text-xs mt-0.5" style={{ color: "var(--text-2)" }}>
                  Last 14 days
                </div>
              </div>
              <div className="flex items-center gap-2">
                <span className="dot dot--pulse" aria-hidden="true" />
                <span className="label" style={{ fontSize: "9px" }}>LIVE</span>
              </div>
            </div>
            <Sparkline data={sparkline} color="#00FF88" height={90} showLabels />
          </div>

          <div className="p-5" style={{ background: "var(--bg)" }}>
            <div style={{ fontWeight: 600, fontSize: "13px", color: "var(--text-1)", marginBottom: "1rem" }}>
              Top Tools
            </div>
            <ToolBarChart tools={analytics.top_tools} />
          </div>
        </div>

        {/* ── Recent Sessions bento ── */}
        <div style={{ background: "var(--border)" }}>
          {/* Table header bar */}
          <div
            className="flex items-center justify-between px-5 py-3"
            style={{ background: "var(--bg)", borderBottom: "1px solid var(--border)" }}
          >
            <div className="flex items-center gap-2">
              <span style={{ fontWeight: 600, fontSize: "13px", color: "var(--text-1)" }}>
                Recent Sessions
              </span>
              <span
                className="mono"
                style={{
                  fontSize: "10px",
                  padding: "1px 6px",
                  background: "var(--bg-2)",
                  color: "var(--text-2)",
                  border: "1px solid var(--border)",
                }}
              >
                {recent.length}
              </span>
            </div>
            <Link
              href="/projects"
              className="flex items-center gap-1 text-xs transition-colors duration-100 outline-none"
              style={{ color: "var(--accent)", fontSize: "11px" }}
            >
              View all <ArrowRight size={11} aria-hidden="true" />
            </Link>
          </div>

          {/* Column headers */}
          <div
            className="grid px-5 py-2 mono"
            style={{
              gridTemplateColumns: "5.5rem 1fr 8rem 6rem 5rem 5rem 8rem 5rem",
              borderBottom: "1px solid var(--border)",
              color: "var(--text-3)",
              fontSize: "9px",
              textTransform: "uppercase",
              letterSpacing: "0.12em",
              background: "var(--bg)",
            }}
          >
            <span>Session</span>
            <span>Message</span>
            <span>Project</span>
            <span>Tokens</span>
            <span>Cost</span>
            <span>Errors</span>
            <span>Model</span>
            <span className="text-right">When</span>
          </div>

          <div role="list" aria-label="Recent sessions" style={{ background: "var(--bg)" }}>
            {recent.map((s) => (
              <SessionRow key={s.session_id} session={s} />
            ))}
          </div>
        </div>

      </div>
    </div>
  );
}

/* ── Bento stat cells ── */

function BentoStat({
  label, value, sub, accent, valueColor,
}: {
  label: string; value: string; sub: string; accent?: boolean; valueColor?: string;
}) {
  return (
    <div
      className="p-5 relative"
      style={{ background: "var(--bg)" }}
    >
      {accent && (
        <div
          style={{ position: "absolute", top: 0, left: 0, right: 0, height: "1px", background: "var(--accent)" }}
          aria-hidden="true"
        />
      )}
      <div
        className="label mb-3"
        style={{ color: accent ? "var(--accent)" : "var(--text-3)" }}
      >
        {label}
      </div>
      <div
        className="mono tabular"
        style={{
          fontSize: "1.875rem",
          fontWeight: 700,
          color: valueColor ?? "var(--text-1)",
          letterSpacing: "-0.03em",
          lineHeight: 1,
          marginBottom: "0.375rem",
        }}
      >
        {value}
      </div>
      <div style={{ fontSize: "11px", color: "var(--text-3)" }}>{sub}</div>
    </div>
  );
}

function BentoMini({
  label, value, alert,
}: {
  label: string; value: string; alert?: boolean;
}) {
  return (
    <div
      className="px-4 py-3"
      style={{ background: "var(--bg)" }}
    >
      <div className="label mb-1" style={{ color: "var(--text-3)" }}>{label}</div>
      <div
        className="mono tabular"
        style={{
          fontSize: "1.125rem",
          fontWeight: 700,
          color: alert ? "var(--red)" : "var(--text-1)",
        }}
      >
        {value}
      </div>
    </div>
  );
}

function SessionRow({ session }: { session: SessionFile }) {
  return (
    <div role="listitem">
      <Link href={`/sessions?id=${session.session_id}`}>
        <div
          className="grid items-center px-5 mono text-xs tabular cursor-pointer transition-colors duration-75"
          style={{
            gridTemplateColumns: "5.5rem 1fr 8rem 6rem 5rem 5rem 8rem 5rem",
            height: "40px",
            borderBottom: "1px solid var(--border)",
            color: "var(--text-2)",
          }}
          onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.background = "var(--bg-2)"; }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.background = "transparent"; }}
        >
          {/* Session ID */}
          <span style={{ color: "var(--accent)" }}>{shortId(session.session_id)}</span>

          {/* Message */}
          <span
            className="truncate pr-4"
            style={{ fontFamily: "var(--font-sans, Inter)", color: "var(--text-1)", fontSize: "0.8125rem", fontVariantNumeric: "normal" }}
          >
            {session.first_message ?? <em style={{ color: "var(--text-3)" }}>No message</em>}
          </span>

          {/* Project */}
          <span className="truncate pr-3">{session.project_name}</span>

          {/* Tokens */}
          <span style={{ color: "var(--cyan)" }}>{formatTokens(session.total_tokens)}</span>

          {/* Cost */}
          <span style={{ color: "var(--amber)" }}>{formatCost(session.estimated_cost)}</span>

          {/* Errors */}
          <span>
            {session.error_count > 0 ? (
              <span className="tbadge tbadge-err">{session.error_count}</span>
            ) : (
              <span style={{ color: "var(--text-3)" }}>—</span>
            )}
          </span>

          {/* Model */}
          <span>
            {session.model ? (
              <span className="tbadge tbadge-asst">
                {session.model.replace("claude-", "").replace(/-\d{8}$/, "")}
              </span>
            ) : (
              <span style={{ color: "var(--text-3)" }}>—</span>
            )}
          </span>

          {/* Time */}
          <span className="text-right" style={{ color: "var(--text-3)" }}>
            {timeAgo(session.modified_at)}
          </span>
        </div>
      </Link>
    </div>
  );
}

function LoadingState() {
  return (
    <div className="flex-1 flex items-center justify-center" style={{ background: "var(--bg)" }}>
      <span className="mono text-sm" style={{ color: "var(--text-2)" }}>Loading…</span>
    </div>
  );
}

function ErrorState() {
  return (
    <div className="flex-1 flex items-center justify-center" style={{ background: "var(--bg)" }}>
      <div className="text-center">
        <div className="text-sm mb-2" style={{ color: "var(--red)" }}>
          Failed to load analytics
        </div>
        <div className="mono text-xs" style={{ color: "var(--text-2)" }}>
          Is <code style={{ color: "var(--accent)" }}>hindsight serve</code> running on :7227?
        </div>
      </div>
    </div>
  );
}
