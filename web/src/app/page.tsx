"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import type { GlobalAnalytics, OtelLogDto, SessionFile, TelemetrySummary } from "@/lib/types";
import { formatBytes, formatCost, formatTokens } from "@/lib/utils";
import { StatCard } from "@/components/ui/StatCard";
import { SectionHeader } from "@/components/ui/SectionHeader";
import { DayChart } from "@/components/charts/DayChart";
import { BarChart } from "@/components/charts/BarChart";
import { TokenBreakdownBar } from "@/components/charts/TokenBreakdownBar";
import { SessionTable } from "@/components/sessions/SessionTable";

// ── Layout constants ──────────────────────────────────────────
const MAX_W = "1400px";
const PAGE_PAD_X = "28px";
const SECTION_GAP = "20px";

// ── MCP extraction: mcp__<server>__<tool> → group by server ──
function extractMcpServers(topTools: [string, number][]): [string, number][] {
  const servers: Record<string, number> = {};
  for (const [name, count] of topTools) {
    if (name.startsWith("mcp__")) {
      const parts = name.split("__");
      const server = parts[1] ?? name;
      servers[server] = (servers[server] ?? 0) + count;
    }
  }
  return Object.entries(servers)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 6) as [string, number][];
}

// ── File path shortener: show last 2 segments ────────────────
function shortPath(path: string): string {
  const parts = path.replace(/\\/g, "/").split("/").filter(Boolean);
  if (parts.length <= 2) return parts.join("/");
  return `.../${parts.slice(-2).join("/")}`;
}

export default function DashboardPage() {
  const [analytics, setAnalytics] = useState<GlobalAnalytics | null>(null);
  const [sparkline, setSparkline] = useState<number[]>([]);
  const [recent, setRecent] = useState<SessionFile[]>([]);
  const [topFiles, setTopFiles] = useState<[string, number][]>([]);
  const [telemetry, setTelemetry] = useState<TelemetrySummary | null>(null);
  const [otelLogs, setOtelLogs] = useState<OtelLogDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      api.globalAnalytics(),
      api.globalSparkline(14),
      api.sessions({ limit: 10 }),
      api.globalTopFiles().catch(() => [] as [string, number][]),
      api.telemetrySummary().catch(() => null),
      api.otelLogs({ event: "api_request" }).catch(() => [] as OtelLogDto[]),
    ])
      .then(([a, spark, sessions, files, telem, logs]) => {
        setAnalytics(a);
        setSparkline(spark);
        setRecent(sessions);
        setTopFiles(files);
        setTelemetry(telem);
        setOtelLogs(logs);
      })
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <PageShell><LoadingState /></PageShell>;
  if (error || !analytics) return <PageShell><ErrorState message={error} /></PageShell>;

  const sessionsSub = [
    analytics.sessions_today > 0 && `+${analytics.sessions_today} today`,
    `${analytics.sessions_this_week} this week`,
  ]
    .filter(Boolean)
    .join(" · ");

  const mcpServers = extractMcpServers(analytics.top_tools);
  const nativeTools = analytics.top_tools.filter(([n]) => !n.startsWith("mcp__"));
  const topFilesForChart = topFiles.map(([p, c]) => [shortPath(p), c] as [string, number]);

  // Compute model cost breakdown from OTEL logs
  const hasTelemetry = telemetry && telemetry.cost_usd > 0;
  const modelCosts: [string, number][] = [];
  if (otelLogs.length > 0) {
    const byCost: Record<string, number> = {};
    for (const log of otelLogs) {
      if (log.model && log.cost_usd) {
        const short = log.model.replace(/^claude-/, "").replace(/-\d{8}$/, "");
        byCost[short] = (byCost[short] ?? 0) + log.cost_usd;
      }
    }
    modelCosts.push(
      ...Object.entries(byCost)
        .sort((a, b) => b[1] - a[1])
        .map(([m, c]) => [m, Math.round(c * 100) / 100] as [string, number]),
    );
  }

  return (
    <PageShell>
      {/* ── Hero stats ───────────────────────────────────── */}
      <Card>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)" }}>
          <div style={{ borderRight: "1px solid var(--border-1)" }}>
            <StatCard
              label="Total Sessions"
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
          <StatCard
            label="Errors"
            value={analytics.total_errors.toLocaleString()}
            sub={analytics.total_errors === 0 ? "clean sessions" : "across sessions"}
            valueColor={analytics.total_errors > 0 ? "var(--red)" : undefined}
          />
        </div>
      </Card>

      {/* ── Cost/Token KPI row ─────────────────────────────── */}
      {hasTelemetry && (
        <Card>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)" }}>
            <div style={{ borderRight: "1px solid var(--border-1)" }}>
              <StatCard
                label="Total Cost"
                value={formatCost(telemetry.cost_usd)}
                sub={`${telemetry.total_sessions} sessions with telemetry`}
                valueColor="var(--amber)"
              />
            </div>
            <div style={{ borderRight: "1px solid var(--border-1)" }}>
              <StatCard
                label="Input Tokens"
                value={formatTokens(telemetry.input_tokens)}
              />
            </div>
            <div style={{ borderRight: "1px solid var(--border-1)" }}>
              <StatCard
                label="Output Tokens"
                value={formatTokens(telemetry.output_tokens)}
              />
            </div>
            <StatCard
              label="Cache Tokens"
              value={formatTokens(telemetry.cache_read_tokens + telemetry.cache_creation_tokens)}
              sub={`${formatTokens(telemetry.cache_read_tokens)} read · ${formatTokens(telemetry.cache_creation_tokens)} created`}
            />
          </div>
        </Card>
      )}

      {/* ── Token Breakdown + Cost by Model ────────────────── */}
      {hasTelemetry && (
        <Card>
          <div style={{ display: "grid", gridTemplateColumns: modelCosts.length > 0 ? "2fr 1fr" : "1fr" }}>
            <div style={{ padding: "24px 28px", borderRight: modelCosts.length > 0 ? "1px solid var(--border-1)" : "none" }}>
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
            {modelCosts.length > 0 && (
              <div style={{ padding: "24px 28px" }}>
                <div style={{ marginBottom: "18px" }}>
                  <SectionHeader title="Cost by Model" />
                </div>
                <BarChart data={modelCosts} limit={6} color="var(--amber)" countLabel="USD" />
              </div>
            )}
          </div>
        </Card>
      )}

      {/* ── Charts row 1: sparkline + top tools ──────────── */}
      <Card>
        <div style={{ display: "grid", gridTemplateColumns: "2fr 1fr" }}>
          <div style={{ padding: "24px 28px", borderRight: "1px solid var(--border-1)" }}>
            <DayChart data={sparkline} />
          </div>
          <div style={{ padding: "24px 28px" }}>
            <div style={{ marginBottom: "18px" }}>
              <SectionHeader title="Top Tools" />
            </div>
            <BarChart data={nativeTools} limit={6} color="var(--cyan)" />
          </div>
        </div>
      </Card>

      {/* ── Charts row 2: MCPs + Top Files ───────────────── */}
      {(mcpServers.length > 0 || topFilesForChart.length > 0) && (
        <Card>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 2fr" }}>
            {/* Top MCPs */}
            {mcpServers.length > 0 ? (
              <div style={{ padding: "24px 28px", borderRight: "1px solid var(--border-1)" }}>
                <div style={{ marginBottom: "18px" }}>
                  <SectionHeader title="Top MCP Servers" />
                </div>
                <BarChart data={mcpServers} limit={6} color="var(--purple)" />
              </div>
            ) : (
              <div style={{ padding: "24px 28px", borderRight: "1px solid var(--border-1)" }}>
                <div style={{ marginBottom: "18px" }}>
                  <SectionHeader title="Top MCP Servers" />
                </div>
                <div style={{ fontSize: "13px", color: "var(--text-3)", fontFamily: "var(--font-sans)" }}>
                  No MCP tools detected
                </div>
              </div>
            )}

            {/* Top Files */}
            {topFilesForChart.length > 0 && (
              <div style={{ padding: "24px 28px" }}>
                <div style={{ marginBottom: "18px" }}>
                  <SectionHeader title="Most Accessed Files" />
                </div>
                <BarChart data={topFilesForChart} limit={8} color="var(--amber)" countLabel="accesses" />
              </div>
            )}
          </div>
        </Card>
      )}

      {/* ── Recent sessions ──────────────────────────────── */}
      <Card>
        <SessionTable
          sessions={recent}
          action={{ label: "All projects", href: "/projects" }}
        />
      </Card>
    </PageShell>
  );
}

// ── Local layout components ───────────────────────────────────

function PageShell({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        maxWidth: MAX_W,
        margin: "0 auto",
        padding: `36px ${PAGE_PAD_X}`,
        display: "flex",
        flexDirection: "column",
        gap: SECTION_GAP,
      }}
    >
      {children}
    </div>
  );
}

function Card({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        background: "var(--bg-1)",
        border: "1px solid var(--border-1)",
        borderRadius: "var(--radius-lg)",
        overflow: "hidden",
      }}
    >
      {children}
    </div>
  );
}

function LoadingState() {
  return (
    <div style={{
      height: "60vh", display: "flex", alignItems: "center",
      justifyContent: "center", fontSize: "14px",
      color: "var(--text-3)", fontFamily: "var(--font-sans)",
    }}>
      Loading…
    </div>
  );
}

function ErrorState({ message }: { message: string | null }) {
  return (
    <div style={{
      height: "60vh", display: "flex", alignItems: "center",
      justifyContent: "center", flexDirection: "column",
      gap: "10px", textAlign: "center",
    }}>
      <div style={{ fontSize: "14px", color: "var(--red)" }}>
        {message ?? "Failed to load analytics"}
      </div>
      <div style={{ fontSize: "13px", color: "var(--text-3)" }}>
        Is{" "}
        <code style={{ fontFamily: "var(--font-mono)", color: "var(--accent)" }}>
          claude-hindsight serve
        </code>{" "}
        running on :7227?
      </div>
    </div>
  );
}
