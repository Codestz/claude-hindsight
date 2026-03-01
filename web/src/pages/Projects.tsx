import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import type { ProjectStats } from "@/lib/types";
import { formatBytes } from "@/lib/utils";
import { StatCard } from "@/components/ui/StatCard";
import { ProjectTable } from "@/components/projects/ProjectTable";

const MAX_W = "1400px";
const PAGE_PAD = "28px";
const SECTION_GAP = "20px";

export default function ProjectsPage() {
  const [projects, setProjects] = useState<ProjectStats[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .projects()
      .then(setProjects)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <PageShell><LoadingState /></PageShell>;
  if (error)   return <PageShell><ErrorState message={error} /></PageShell>;

  const totalSessions = projects.reduce((s, p) => s + p.session_count, 0);
  const totalSize     = projects.reduce((s, p) => s + p.total_size, 0);

  return (
    <PageShell>
      <Card>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)" }}>
          <div style={{ borderRight: "1px solid var(--border-1)" }}>
            <StatCard label="Projects" value={projects.length.toLocaleString()} sub="tracked workspaces" accent />
          </div>
          <div style={{ borderRight: "1px solid var(--border-1)" }}>
            <StatCard label="Total Sessions" value={totalSessions.toLocaleString()} sub="across all projects" valueColor="var(--cyan)" />
          </div>
          <StatCard label="Total Size" value={formatBytes(totalSize)} sub="JSONL transcript data" valueColor="var(--amber)" />
        </div>
      </Card>
      <Card>
        <ProjectTable projects={projects} />
      </Card>
    </PageShell>
  );
}

function PageShell({ children }: { children: React.ReactNode }) {
  return (
    <div style={{
      maxWidth: MAX_W, margin: "0 auto",
      padding: `36px ${PAGE_PAD}`,
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
      <div style={{ fontSize: "14px", color: "var(--red)" }}>{message ?? "Failed to load projects"}</div>
      <div style={{ fontSize: "13px", color: "var(--text-3)" }}>
        Is <code style={{ fontFamily: "var(--font-mono)", color: "var(--accent)" }}>claude-hindsight serve</code> running on :7227?
      </div>
    </div>
  );
}
