"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { api } from "@/lib/api";
import type { ProjectAnalytics, SessionFile } from "@/lib/types";
import { formatBytes, formatCost, formatTokens } from "@/lib/utils";
import { StatCard } from "@/components/ui/StatCard";
import { SectionHeader } from "@/components/ui/SectionHeader";
import { BarChart } from "@/components/charts/BarChart";
import { SessionTable } from "@/components/sessions/SessionTable";

// ── Layout constants ──────────────────────────────────────────
const MAX_W = "1400px";
const PAGE_PAD = "0 28px";
const SECTION_GAP = "20px";

// ── Helpers ───────────────────────────────────────────────────
function extractMcpServers(topTools: [string, number][]): [string, number][] {
  const servers: Record<string, number> = {};
  for (const [name, count] of topTools) {
    if (name.startsWith("mcp__")) {
      const parts = name.split("__");
      const server = parts[1] ?? name;
      servers[server] = (servers[server] ?? 0) + count;
    }
  }
  return Object.entries(servers).sort((a, b) => b[1] - a[1]) as [string, number][];
}

function shortPath(path: string): string {
  const parts = path.replace(/\\/g, "/").split("/").filter(Boolean);
  if (parts.length <= 2) return parts.join("/");
  return `.../${parts.slice(-2).join("/")}`;
}

// ── Root export ───────────────────────────────────────────────
// Reads the project name from window.location.pathname at runtime.
// The Rust server serves this page for any /projects/:name/ path.
export default function ProjectDetailPage() {
  const [name, setName] = useState<string | null>(null);

  useEffect(() => {
    // pathname looks like /projects/MyProject/ — extract the second segment
    const segments = window.location.pathname.split("/").filter(Boolean);
    // segments[0] = "projects", segments[1] = project name
    if (segments.length >= 2) {
      setName(decodeURIComponent(segments[1]));
    }
  }, []);

  if (!name) {
    return (
      <PageShell>
        <LoadingState />
      </PageShell>
    );
  }

  return <ProjectDetail key={name} name={name} />;
}

// ── Project detail ────────────────────────────────────────────
function ProjectDetail({ name }: { name: string }) {
  const [analytics, setAnalytics] = useState<ProjectAnalytics | null>(null);
  const [sessions, setSessions] = useState<SessionFile[]>([]);
  const [topFiles, setTopFiles] = useState<[string, number][]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      api.projectAnalytics(name),
      api.sessions({ project: name }),
      api.projectTopFiles(name).catch(() => [] as [string, number][]),
    ])
      .then(([a, s, files]) => {
        setAnalytics(a);
        setSessions(s);
        setTopFiles(files);
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
        <BackLink href="/projects/">← Projects</BackLink>
        <span style={{ color: "var(--border-3)", fontSize: "13px" }}>/</span>
        <span style={{ fontSize: "13px", fontFamily: "var(--font-mono)", color: "var(--text-2)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {name}
        </span>
      </div>

      {/* Stats bento */}
      <Card>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)" }}>
          <div style={{ borderRight: "1px solid var(--border-1)" }}>
            <StatCard label="Sessions" value={analytics.total_sessions.toLocaleString()} sub={sessionsSub || undefined} accent />
          </div>
          <div style={{ borderRight: "1px solid var(--border-1)" }}>
            <StatCard label="Tokens" value={formatTokens(analytics.total_tokens)} sub="total consumed" valueColor="var(--cyan)" />
          </div>
          <div style={{ borderRight: "1px solid var(--border-1)" }}>
            <StatCard label="Estimated Cost" value={formatCost(analytics.total_cost)} sub="based on model pricing" valueColor="var(--amber)" />
          </div>
          <StatCard
            label="Errors"
            value={analytics.total_errors.toLocaleString()}
            sub={analytics.total_errors === 0 ? "clean sessions" : "across sessions"}
            valueColor={analytics.total_errors > 0 ? "var(--red)" : undefined}
          />
        </div>
      </Card>

      {/* Tools + MCPs */}
      {(nativeTools.length > 0 || mcpServers.length > 0) && (
        <Card>
          <div style={{ display: "grid", gridTemplateColumns: mcpServers.length > 0 ? "1fr 1fr" : "1fr" }}>
            {nativeTools.length > 0 && (
              <div style={{ padding: "24px 28px", borderRight: mcpServers.length > 0 ? "1px solid var(--border-1)" : "none" }}>
                <div style={{ marginBottom: "18px" }}><SectionHeader title="Top Tools" /></div>
                <BarChart data={nativeTools} limit={8} color="var(--cyan)" />
              </div>
            )}
            {mcpServers.length > 0 && (
              <div style={{ padding: "24px 28px" }}>
                <div style={{ marginBottom: "18px" }}><SectionHeader title="Top MCP Servers" /></div>
                <BarChart data={mcpServers} limit={6} color="var(--purple)" />
              </div>
            )}
          </div>
        </Card>
      )}

      {/* Top Files */}
      {topFilesForChart.length > 0 && (
        <Card>
          <div style={{ padding: "24px 28px" }}>
            <div style={{ marginBottom: "18px" }}><SectionHeader title="Most Accessed Files" /></div>
            <BarChart data={topFilesForChart} limit={10} color="var(--amber)" countLabel="accesses" />
          </div>
        </Card>
      )}

      {/* Sessions */}
      <Card>
        <SessionTable sessions={sessions} title="Sessions" emptyMessage="No sessions for this project" />
      </Card>
    </PageShell>
  );
}

// ── Shared layout helpers ─────────────────────────────────────

function PageShell({ children }: { children: React.ReactNode }) {
  return (
    <div style={{
      maxWidth: MAX_W, margin: "0 auto",
      padding: `36px ${PAGE_PAD.split(" ")[1]}`,
      display: "flex", flexDirection: "column", gap: SECTION_GAP,
    }}>
      {children}
    </div>
  );
}

function Card({ children }: { children: React.ReactNode }) {
  return (
    <div style={{
      background: "var(--bg-1)", border: "1px solid var(--border-1)",
      borderRadius: "var(--radius-lg)", overflow: "hidden",
    }}>
      {children}
    </div>
  );
}

function BackLink({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <Link
      href={href}
      style={{ fontSize: "13px", color: "var(--text-3)", textDecoration: "none", fontFamily: "var(--font-mono)", transition: "color 0.12s", flexShrink: 0 }}
      onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-2)"; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-3)"; }}
    >
      {children}
    </Link>
  );
}

function LoadingState() {
  return (
    <div style={{ height: "60vh", display: "flex", alignItems: "center", justifyContent: "center", fontSize: "14px", color: "var(--text-3)", fontFamily: "var(--font-sans)" }}>
      Loading…
    </div>
  );
}

function ErrorState({ message }: { message: string | null }) {
  return (
    <div style={{ height: "60vh", display: "flex", alignItems: "center", justifyContent: "center", flexDirection: "column", gap: "10px", textAlign: "center" }}>
      <div style={{ fontSize: "14px", color: "var(--red)" }}>{message ?? "Failed to load project"}</div>
      <div style={{ fontSize: "13px", color: "var(--text-3)" }}>
        Is <code style={{ fontFamily: "var(--font-mono)", color: "var(--accent)" }}>hindsight serve</code> running on :7227?
      </div>
    </div>
  );
}
