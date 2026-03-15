import React, { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { api } from "@/lib/api";
import type {
  GlobalAnalytics,
  HookActivitySummary,
  HookToolEvent,
  OtelLogDto,
  SessionFile,
  TelemetrySummary,
} from "@/lib/types";
import { formatBytes, formatCost, formatTokens, shortId, extractMcpServers, shortPath, greeting } from "@/lib/utils";
import { MiniKpi } from "@/components/ui/MiniKpi";
import { StatCard } from "@/components/ui/StatCard";
import { SectionHeader } from "@/components/ui/SectionHeader";
import { Card } from "@/components/ui/Card";
import { PageShell } from "@/components/ui/PageShell";
import { LoadingState } from "@/components/ui/LoadingState";
import { ErrorState } from "@/components/ui/ErrorState";
import { DayChart } from "@/components/charts/DayChart";
import { BarChart } from "@/components/charts/BarChart";
import { TokenBreakdownBar } from "@/components/charts/TokenBreakdownBar";
import { SessionTable } from "@/components/sessions/SessionTable";
import {
  Wrench,
  AlertTriangle,
  ChevronRight,
} from "lucide-react";

export default function DashboardPage() {
  const [analytics, setAnalytics] = useState<GlobalAnalytics | null>(null);
  const [sparkline, setSparkline] = useState<number[]>([]);
  const [recent, setRecent] = useState<SessionFile[]>([]);
  const [topFiles, setTopFiles] = useState<[string, number][]>([]);
  const [telemetry, setTelemetry] = useState<TelemetrySummary | null>(null);
  const [otelLogs, setOtelLogs] = useState<OtelLogDto[]>([]);
  const [hookSummary, setHookSummary] = useState<HookActivitySummary | null>(null);
  const [recentErrors, setRecentErrors] = useState<HookToolEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      api.globalAnalytics(),
      api.globalSparkline(14),
      api.sessions({ limit: 8 }),
      api.globalTopFiles().catch(() => [] as [string, number][]),
      api.telemetrySummary().catch(() => null),
      api.otelLogs({ event: "api_request" }).catch(() => [] as OtelLogDto[]),
      api.hookActivitySummary().catch(() => null),
      api.hookToolFailures({ limit: 5 }).catch(() => [] as HookToolEvent[]),
    ])
      .then(([a, spark, sessions, files, telem, logs, hooks, errors]) => {
        setAnalytics(a);
        setSparkline(spark);
        setRecent(sessions);
        setTopFiles(files);
        setTelemetry(telem);
        setOtelLogs(logs);
        setHookSummary(hooks);
        setRecentErrors(errors);
      })
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  // All hooks MUST be called before any early returns (Rules of Hooks)
  const mcpServers = useMemo(() => analytics ? extractMcpServers(analytics.top_tools) : [], [analytics]);
  const nativeTools = useMemo(() => analytics ? analytics.top_tools.filter(([n]) => !n.startsWith("mcp__")) : [], [analytics]);
  const topFilesForChart = useMemo(() => topFiles.map(([p, c]) => [shortPath(p), c] as [string, number]), [topFiles]);
  const modelCosts = useMemo(() => {
    if (otelLogs.length === 0) return [] as [string, number][];
    const byCost: Record<string, number> = {};
    for (const log of otelLogs) {
      if (log.model && log.cost_usd) {
        const short = log.model.replace(/^claude-/, "").replace(/-\d{8}$/, "");
        byCost[short] = (byCost[short] ?? 0) + log.cost_usd;
      }
    }
    return Object.entries(byCost)
      .sort((a, b) => b[1] - a[1])
      .map(([m, c]) => [m, Math.round(c * 100) / 100] as [string, number]);
  }, [otelLogs]);

  if (loading) return <PageShell maxWidth="1320px"><LoadingState /></PageShell>;
  if (error || !analytics) return <PageShell maxWidth="1320px"><ErrorState message={error} /></PageShell>;

  const sessionsSub = [
    analytics.sessions_today > 0 && `+${analytics.sessions_today} today`,
    `${analytics.sessions_this_week} this week`,
  ]
    .filter(Boolean)
    .join(" · ");

  const hasTelemetry = telemetry && telemetry.cost_usd > 0;
  const today = new Date().toLocaleDateString("en-US", { weekday: "long", month: "long", day: "numeric" });

  return (
    <PageShell maxWidth="1320px">
      {/* ── Greeting header ──────────────────────────────── */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", borderBottom: "1px solid var(--border-1)", paddingBottom: "16px" }}>
        <h1 style={{ fontSize: "20px", fontWeight: 600, color: "var(--text-1)", fontFamily: "var(--font-sans)", margin: 0 }}>
          {greeting()}
        </h1>
        <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)" }}>
          {today}
        </span>
      </div>

      {/* ── KPI Row ──────────────────────────────────────── */}
      <div className="animate-in" style={{ "--delay": "0s" } as React.CSSProperties}>
      <Card glow="var(--indigo)">
        <div style={{ display: "grid", gridTemplateColumns: hasTelemetry ? "repeat(4, 1fr)" : "repeat(3, 1fr)" }}>
          <div style={{ borderRight: "1px solid var(--border-1)" }}>
            <StatCard
              label="Sessions"
              value={analytics.total_sessions.toLocaleString()}
              sub={sessionsSub}
              accent
            />
          </div>
          <div style={{ borderRight: "1px solid var(--border-1)" }}>
            <StatCard
              label="Projects"
              value={analytics.total_projects.toLocaleString()}
              sub={formatBytes(analytics.total_size)}
            />
          </div>
          {hasTelemetry && (
            <div style={{ borderRight: "1px solid var(--border-1)" }}>
              <StatCard
                label="Total Cost"
                value={formatCost(telemetry.cost_usd)}
                sub={`${telemetry.total_sessions} sessions`}
                valueColor="var(--amber)"
              />
            </div>
          )}
          <StatCard
            label="Errors"
            value={analytics.total_errors.toLocaleString()}
            sub={analytics.total_errors === 0 ? "clean" : "across sessions"}
            valueColor={analytics.total_errors > 0 ? "var(--red)" : undefined}
          />
        </div>
      </Card>
      </div>

      {/* Quick insight */}
      <div style={{
        display: "flex", alignItems: "center", gap: "8px",
        padding: "0 4px",
        fontSize: "12px", fontFamily: "var(--font-sans)", color: "var(--text-3)",
      }}>
        <span style={{ width: "4px", height: "4px", borderRadius: "50%", background: "var(--indigo)", flexShrink: 0 }} />
        {analytics.sessions_today > 0
          ? `${analytics.sessions_today} session${analytics.sessions_today > 1 ? "s" : ""} today across ${analytics.total_projects} projects`
          : `${analytics.sessions_this_week} sessions this week`
        }
      </div>

      {/* ── Sparkline + Token Breakdown ──────────────────── */}
      <div className="animate-in" style={{ "--delay": "0.06s" } as React.CSSProperties}>
      <Card glow="var(--accent)">
        <div style={{ display: "grid", gridTemplateColumns: hasTelemetry ? "2fr 1fr" : "1fr" }}>
          <div style={{ padding: "24px 28px", borderRight: hasTelemetry ? "1px solid var(--border-1)" : "none" }}>
            <DayChart data={sparkline} />
          </div>
          {hasTelemetry && (
            <div style={{ padding: "24px 28px" }}>
              <div style={{ marginBottom: "18px" }}>
                <SectionHeader title="Token Breakdown" />
              </div>
              <TokenBreakdownBar
                input={telemetry.input_tokens}
                output={telemetry.output_tokens}
                cacheRead={telemetry.cache_read_tokens}
                cacheCreation={telemetry.cache_creation_tokens}
              />
            </div>
          )}
        </div>
      </Card>
      </div>

      {/* ── Recent Sessions + Top Tools ──────────────────── */}
      <div className="animate-in" style={{ "--delay": "0.12s" } as React.CSSProperties}>
      <Card>
        <div style={{ display: "grid", gridTemplateColumns: "2fr 1fr" }}>
          <div style={{ borderRight: "1px solid var(--border-1)" }}>
            <SessionTable
              sessions={recent}
              title="Recent Sessions"
              action={{ label: "All sessions", href: "/sessions" }}
            />
          </div>
          <div style={{ padding: "24px 28px" }}>
            <div style={{ marginBottom: "18px" }}>
              <SectionHeader title="Top Tools" />
            </div>
            <div style={{ fontSize: "11px", color: "var(--text-3)", marginBottom: "12px" }}>
              Most used tools across all sessions
            </div>
            <BarChart data={nativeTools} limit={6} color="var(--cyan)" />

            {/* Cost by model (if data exists) */}
            {modelCosts.length > 0 && (
              <>
                <div style={{ marginTop: "28px", marginBottom: "18px" }}>
                  <SectionHeader title="Cost by Model" />
                </div>
                <BarChart data={modelCosts} limit={4} color="var(--amber)" countLabel="USD" />
              </>
            )}
          </div>
        </div>
      </Card>
      </div>

      {/* ── Recent Activity + Top Files ──────────────────── */}
      <div className="animate-in" style={{ "--delay": "0.18s" } as React.CSSProperties}>
      <Card>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 2fr" }}>
          {/* Recent Activity preview */}
          <div style={{ borderRight: "1px solid var(--border-1)" }}>
            <div style={{ padding: "16px 24px", borderBottom: "1px solid var(--border-1)", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
              <SectionHeader title="Recent Activity" />
              <Link
                to="/activity"
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "2px",
                  fontSize: "12px",
                  color: "var(--text-3)",
                  textDecoration: "none",
                }}
              >
                View all <ChevronRight size={12} />
              </Link>
            </div>
            {hookSummary && hookSummary.total_tool_events > 0 ? (
              <div>
                {/* Summary row */}
                <div style={{ display: "flex", gap: "16px", padding: "14px 24px", borderBottom: "1px solid var(--border-1)" }}>
                  <MiniKpi icon={Wrench} label="Tools" value={hookSummary.total_tool_events} />
                  <MiniKpi icon={AlertTriangle} label="Errors" value={hookSummary.recent_errors} color="var(--red)" />
                </div>
                {/* Recent errors preview */}
                {recentErrors.length > 0 ? (
                  recentErrors.slice(0, 4).map((e) => (
                    <Link
                      key={e.id}
                      to={`/sessions/${encodeURIComponent(e.session_id)}`}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: "8px",
                        padding: "8px 24px",
                        borderBottom: "1px solid var(--border-1)",
                        fontSize: "12px",
                        fontFamily: "var(--font-mono)",
                        textDecoration: "none",
                        color: "var(--text-2)",
                      }}
                    >
                      <span style={{ color: "var(--red)", flexShrink: 0 }}>
                        <AlertTriangle size={12} />
                      </span>
                      <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        {e.tool_name ?? e.hook_event}
                      </span>
                      <span style={{ color: "var(--text-3)", flexShrink: 0 }}>
                        {shortId(e.session_id)}
                      </span>
                    </Link>
                  ))
                ) : (
                  <div style={{ padding: "24px", textAlign: "center", fontSize: "13px", color: "var(--text-3)" }}>
                    No recent errors
                  </div>
                )}
              </div>
            ) : (
              <div style={{ padding: "32px 24px", textAlign: "center", fontSize: "13px", color: "var(--text-3)" }}>
                No hook activity yet
              </div>
            )}
          </div>

          {/* Top Files + MCPs */}
          <div>
            {topFilesForChart.length > 0 && (
              <div style={{ padding: "24px 28px", borderBottom: mcpServers.length > 0 ? "1px solid var(--border-1)" : "none" }}>
                <div style={{ marginBottom: "18px" }}>
                  <SectionHeader title="Most Accessed Files" />
                </div>
                <BarChart data={topFilesForChart} limit={8} color="var(--amber)" countLabel="accesses" />
              </div>
            )}
            {mcpServers.length > 0 && (
              <div style={{ padding: "24px 28px" }}>
                <div style={{ marginBottom: "18px" }}>
                  <SectionHeader title="Top MCP Servers" />
                </div>
                <BarChart data={mcpServers} limit={6} color="var(--purple)" />
              </div>
            )}
            {topFilesForChart.length === 0 && mcpServers.length === 0 && (
              <div style={{ padding: "48px 28px", textAlign: "center", fontSize: "13px", color: "var(--text-3)" }}>
                No file access or MCP data available
              </div>
            )}
          </div>
        </div>
      </Card>
      </div>
    </PageShell>
  );
}

