import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import type { ProjectStats } from "@/lib/types";
import { formatBytes } from "@/lib/utils";
import { PageShell } from "@/components/ui/PageShell";
import { Card } from "@/components/ui/Card";
import { LoadingState } from "@/components/ui/LoadingState";
import { ErrorState } from "@/components/ui/ErrorState";
import { StatCard } from "@/components/ui/StatCard";
import { ProjectTable } from "@/components/projects/ProjectTable";

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
