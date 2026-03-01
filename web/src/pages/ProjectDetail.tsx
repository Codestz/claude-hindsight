import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "@/lib/api";
import type { ProjectAnalytics, SessionFile, SessionTelemetry } from "@/lib/types";
import { formatBytes, formatCost, formatTokens, extractMcpServers, shortPath } from "@/lib/utils";
import { PageShell } from "@/components/ui/PageShell";
import { Card } from "@/components/ui/Card";
import { LoadingState } from "@/components/ui/LoadingState";
import { ErrorState } from "@/components/ui/ErrorState";
import { StatCard } from "@/components/ui/StatCard";
import { SectionHeader } from "@/components/ui/SectionHeader";
import { BarChart } from "@/components/charts/BarChart";
import { TokenBreakdownBar } from "@/components/charts/TokenBreakdownBar";
import { SessionTable } from "@/components/sessions/SessionTable";

export default function ProjectDetailPage() {
  const { name } = useParams<{ name: string }>();

  if (!name) {
    return <PageShell><LoadingState /></PageShell>;
  }

  return <ProjectDetail key={name} name={decodeURIComponent(name)} />;
}

function ProjectDetail({ name }: { name: string }) {
  const [analytics, setAnalytics] = useState<ProjectAnalytics | null>(null);
  const [sessions, setSessions] = useState<SessionFile[]>([]);
  const [topFiles, setTopFiles] = useState<[string, number][]>([]);
  const [projectTelemetry, setProjectTelemetry] = useState<{ cost: number; input: number; output: number; cacheRead: number; cacheCreation: number } | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      api.projectAnalytics(name),
      api.sessions({ project: name }),
      api.projectTopFiles(name).catch(() => [] as [string, number][]),
      api.telemetrySessions().catch(() => [] as SessionTelemetry[]),
    ])
      .then(([a, s, files, allTelem]) => {
        setAnalytics(a);
        setSessions(s);
        setTopFiles(files);
        const projectSessions = allTelem.filter((t) => t.project_name === name);
        if (projectSessions.length > 0) {
          const agg = { cost: 0, input: 0, output: 0, cacheRead: 0, cacheCreation: 0 };
          for (const t of projectSessions) {
            agg.cost += t.cost_usd;
            agg.input += t.input_tokens;
            agg.output += t.output_tokens;
            agg.cacheRead += t.cache_read_tokens;
            agg.cacheCreation += t.cache_creation_tokens;
          }
          if (agg.cost > 0) setProjectTelemetry(agg);
        }
      })
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, [name]);

  if (loading) return <PageShell><LoadingState /></PageShell>;
  if (error || !analytics) return <PageShell><ErrorState message={error} /></PageShell>;

  const sessionsSub = [
    analytics.sessions_today > 0 && `+${analytics.sessions_today} today`,
    `${analytics.sessions_this_week} this week`,
  ]
    .filter(Boolean)
    .join(" · ");

  const mcpServers  = extractMcpServers(analytics.top_tools);
  const nativeTools = analytics.top_tools.filter(([n]) => !n.startsWith("mcp__"));
  const topFilesForChart = topFiles.map(([p, c]) => [shortPath(p), c] as [string, number]);

  return (
    <PageShell>
      {/* Breadcrumb */}
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <BackLink to="/projects">← Projects</BackLink>
        <span style={{ color: "var(--border-3)", fontSize: "13px" }}>/</span>
        <span style={{ fontSize: "13px", fontFamily: "var(--font-mono)", color: "var(--text-2)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {name}
        </span>
      </div>

      {/* Stats bento */}
      <Card>
        <div style={{ display: "grid", gridTemplateColumns: projectTelemetry ? "repeat(4, 1fr)" : "repeat(3, 1fr)" }}>
          <div style={{ borderRight: "1px solid var(--border-1)" }}>
            <StatCard label="Sessions" value={analytics.total_sessions.toLocaleString()} sub={sessionsSub || undefined} accent />
          </div>
          <div style={{ borderRight: "1px solid var(--border-1)" }}>
            <StatCard label="Size" value={formatBytes(analytics.total_size)} sub={`avg ${formatBytes(analytics.avg_session_size)}/session`} />
          </div>
          {projectTelemetry && (
            <div style={{ borderRight: "1px solid var(--border-1)" }}>
              <StatCard
                label="Cost"
                value={formatCost(projectTelemetry.cost)}
                sub={`${formatTokens(projectTelemetry.input + projectTelemetry.output)} tokens`}
                valueColor="var(--amber)"
              />
            </div>
          )}
          <StatCard
            label="Errors"
            value={analytics.total_errors.toLocaleString()}
            sub={analytics.total_errors === 0 ? "clean sessions" : "across sessions"}
            valueColor={analytics.total_errors > 0 ? "var(--red)" : undefined}
          />
        </div>
      </Card>

      {/* Charts — 2-column grid */}
      {(projectTelemetry || nativeTools.length > 0 || mcpServers.length > 0 || topFilesForChart.length > 0) && (
        <div style={{
          display: "grid",
          gridTemplateColumns: (nativeTools.length > 0 || mcpServers.length > 0) ? "1fr 1fr" : "1fr",
          gap: "16px",
          alignItems: "start",
        }}>
          {/* Left column */}
          <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
            {projectTelemetry && (
              <Card>
                <div style={{ padding: "20px 24px" }}>
                  <div style={{ marginBottom: "14px" }}><SectionHeader title="Token Breakdown" /></div>
                  <TokenBreakdownBar
                    input={projectTelemetry.input}
                    output={projectTelemetry.output}
                    cacheRead={projectTelemetry.cacheRead}
                    cacheCreation={projectTelemetry.cacheCreation}
                  />
                </div>
              </Card>
            )}
            {topFilesForChart.length > 0 && (
              <Card>
                <div style={{ padding: "20px 24px" }}>
                  <div style={{ marginBottom: "14px" }}><SectionHeader title="Most Accessed Files" /></div>
                  <BarChart data={topFilesForChart} limit={10} color="var(--amber)" countLabel="accesses" />
                </div>
              </Card>
            )}
          </div>

          {/* Right column */}
          {(nativeTools.length > 0 || mcpServers.length > 0) && (
            <Card>
              {nativeTools.length > 0 && (
                <div style={{ padding: "20px 24px", borderBottom: mcpServers.length > 0 ? "1px solid var(--border-1)" : "none" }}>
                  <div style={{ marginBottom: "14px" }}><SectionHeader title="Top Tools" /></div>
                  <BarChart data={nativeTools} limit={8} color="var(--cyan)" />
                </div>
              )}
              {mcpServers.length > 0 && (
                <div style={{ padding: "20px 24px" }}>
                  <div style={{ marginBottom: "14px" }}><SectionHeader title="Top MCP Servers" /></div>
                  <BarChart data={mcpServers} limit={6} color="var(--purple)" />
                </div>
              )}
            </Card>
          )}
        </div>
      )}

      {/* Sessions */}
      <Card>
        <SessionTable sessions={sessions} title="Sessions" emptyMessage="No sessions for this project" />
      </Card>
    </PageShell>
  );
}

function BackLink({ to, children }: { to: string; children: React.ReactNode }) {
  return (
    <Link
      to={to}
      style={{ fontSize: "13px", color: "var(--text-3)", textDecoration: "none", fontFamily: "var(--font-mono)", transition: "color 0.12s", flexShrink: 0 }}
      onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-2)"; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-3)"; }}
    >
      {children}
    </Link>
  );
}
