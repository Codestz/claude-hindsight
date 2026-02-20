"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import type { GlobalAnalytics, SessionFile } from "@/lib/types";
import { formatCost, formatTokens } from "@/lib/utils";
import { StatCard } from "@/components/ui/StatCard";
import { SectionHeader } from "@/components/ui/SectionHeader";
import { DayChart } from "@/components/charts/DayChart";
import { BarChart } from "@/components/charts/BarChart";
import { SessionTable } from "@/components/sessions/SessionTable";

// ── Layout constants ──────────────────────────────────────────
const MAX_W = "1400px";
const PAGE_PAD = "0 28px";
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
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      api.globalAnalytics(),
      api.globalSparkline(14),
      api.sessions({ limit: 10 }),
      api.globalTopFiles().catch(() => [] as [string, number][]),
    ])
      .then(([a, spark, sessions, files]) => {
        setAnalytics(a);
        setSparkline(spark);
        setRecent(sessions);
        setTopFiles(files);
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
  const nativeTols = analytics.top_tools.filter(([n]) => !n.startsWith("mcp__"));
  const topFilesForChart = topFiles.map(([p, c]) => [shortPath(p), c] as [string, number]);

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
              label="Total Tokens"
              value={formatTokens(analytics.total_tokens)}
              sub="across all sessions"
              valueColor="var(--cyan)"
            />
          </div>
          <StatCard
            label="Estimated Cost"
            value={formatCost(analytics.total_cost)}
            sub="based on model pricing"
            valueColor="var(--amber)"
          />
        </div>
      </Card>

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
            <BarChart data={nativeTols} limit={6} color="var(--cyan)" />
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
        padding: `36px ${PAGE_PAD.split(" ")[1]}`,
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
          hindsight serve
        </code>{" "}
        running on :7227?
      </div>
    </div>
  );
}
