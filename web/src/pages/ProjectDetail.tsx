import React, { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "@/lib/api";
import type { ProjectAnalytics, SessionFile, SessionTelemetry } from "@/lib/types";
import { formatBytes, formatCost, formatTokens, extractMcpServers, shortPath } from "@/lib/utils";
import { PageShell } from "@/components/ui/PageShell";
import { Card } from "@/components/ui/Card";
import { LoadingState } from "@/components/ui/LoadingState";
import { ErrorState } from "@/components/ui/ErrorState";
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

function MetaPill({ label, color }: { label: string; color?: string }) {
  return (
    <span style={{
      fontFamily: "var(--font-mono)", fontSize: "11px", fontWeight: 500,
      color: color ?? "var(--text-2)",
      background: color ? `color-mix(in srgb, ${color} 8%, transparent)` : "var(--bg-2)",
      border: `1px solid ${color ? `color-mix(in srgb, ${color} 15%, transparent)` : "var(--border-1)"}`,
      borderRadius: "12px", padding: "3px 10px", whiteSpace: "nowrap",
      fontVariantNumeric: "tabular-nums",
    }}>
      {label}
    </span>
  );
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

      {/* Hero section */}
      <div className="animate-in" style={{ "--delay": "0s" } as React.CSSProperties}>
        <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
          <h1 style={{
            fontSize: "22px",
            fontWeight: 600,
            color: "var(--text-1)",
            fontFamily: "var(--font-mono)",
            margin: 0,
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}>
            {name}
          </h1>
          <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: "6px" }}>
            <MetaPill label={`${analytics.total_sessions.toLocaleString()} sessions`} color="var(--cyan)" />
            <MetaPill label={formatBytes(analytics.total_size)} />
            {projectTelemetry && (
              <MetaPill label={formatCost(projectTelemetry.cost)} color="var(--amber)" />
            )}
            {projectTelemetry && (
              <MetaPill label={`${formatTokens(projectTelemetry.input + projectTelemetry.output)} tokens`} />
            )}
            {analytics.total_errors > 0 ? (
              <MetaPill label={`${analytics.total_errors} errors`} color="var(--red)" />
            ) : (
              <MetaPill label="no errors" />
            )}
            {analytics.sessions_today > 0 && (
              <MetaPill label={`+${analytics.sessions_today} today`} color="var(--green)" />
            )}
          </div>
        </div>
      </div>

      {/* Two-column content layout */}
      <div style={{ display: "grid", gridTemplateColumns: "3fr 2fr", gap: "16px", alignItems: "start" }}>
        {/* Left column (60%) — Token Breakdown + Sessions */}
        <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
          {projectTelemetry && (
            <div className="animate-in" style={{ "--delay": "0.06s" } as React.CSSProperties}>
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
            </div>
          )}

          <div className="animate-in" style={{ "--delay": "0.12s" } as React.CSSProperties}>
            <Card>
              <SessionTable sessions={sessions} title="Sessions" emptyMessage="No sessions for this project" />
            </Card>
          </div>
        </div>

        {/* Right column (40%) — Top Tools + Most Accessed Files + MCP Servers */}
        <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
          {nativeTools.length > 0 && (
            <div className="animate-in" style={{ "--delay": "0.09s" } as React.CSSProperties}>
              <Card>
                <div style={{ padding: "20px 24px" }}>
                  <div style={{ marginBottom: "14px" }}><SectionHeader title="Top Tools" /></div>
                  <BarChart data={nativeTools} limit={8} color="var(--cyan)" />
                </div>
              </Card>
            </div>
          )}

          {topFilesForChart.length > 0 && (
            <div className="animate-in" style={{ "--delay": "0.12s" } as React.CSSProperties}>
              <Card>
                <div style={{ padding: "20px 24px" }}>
                  <div style={{ marginBottom: "14px" }}><SectionHeader title="Most Accessed Files" /></div>
                  <BarChart data={topFilesForChart} limit={10} color="var(--amber)" countLabel="accesses" />
                </div>
              </Card>
            </div>
          )}

          {mcpServers.length > 0 && (
            <div className="animate-in" style={{ "--delay": "0.15s" } as React.CSSProperties}>
              <Card>
                <div style={{ padding: "20px 24px" }}>
                  <div style={{ marginBottom: "14px" }}><SectionHeader title="MCP Servers" /></div>
                  <BarChart data={mcpServers} limit={6} color="var(--purple)" />
                </div>
              </Card>
            </div>
          )}
        </div>
      </div>
    </PageShell>
  );
}

function BackLink({ to, children }: { to: string; children: React.ReactNode }) {
  return (
    <Link
      to={to}
      style={{ fontSize: "13px", color: "var(--text-3)", textDecoration: "none", fontFamily: "var(--font-mono)", transition: "color 0.12s", flexShrink: 0, borderRadius: "var(--radius-sm)", padding: "2px 6px" }}
      onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-2)"; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-3)"; }}
    >
      {children}
    </Link>
  );
}
