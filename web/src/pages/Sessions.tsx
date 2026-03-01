import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import type { SessionFile, ProjectStats } from "@/lib/types";
import { PageShell } from "@/components/ui/PageShell";
import { Card } from "@/components/ui/Card";
import { LoadingState } from "@/components/ui/LoadingState";
import { ErrorState } from "@/components/ui/ErrorState";
import { EmptyState } from "@/components/ui/EmptyState";
import { SessionTable } from "@/components/sessions/SessionTable";

export default function SessionsPage() {
  const [sessions, setSessions] = useState<SessionFile[]>([]);
  const [projects, setProjects] = useState<ProjectStats[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [projectFilter, setProjectFilter] = useState<string>("all");
  const [errorsOnly, setErrorsOnly] = useState(false);

  useEffect(() => {
    Promise.all([
      api.sessions({ limit: 200 }),
      api.projects(),
    ])
      .then(([s, p]) => {
        setSessions(s);
        setProjects(p);
      })
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <PageShell><LoadingState /></PageShell>;
  if (error) return <PageShell><ErrorState message={error} /></PageShell>;

  let filtered = sessions;
  if (projectFilter !== "all") {
    filtered = filtered.filter((s) => s.project_name === projectFilter);
  }
  if (errorsOnly) {
    filtered = filtered.filter((s) => s.error_count > 0);
  }

  return (
    <PageShell>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <h1
            style={{
              fontSize: "20px",
              fontWeight: 600,
              color: "var(--text-1)",
              fontFamily: "var(--font-sans)",
              margin: 0,
            }}
          >
            Sessions
          </h1>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)" }}>
            {sessions.length}
          </span>
        </div>

        {/* Filters */}
        <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
          {/* Project dropdown */}
          <select
            value={projectFilter}
            onChange={(e) => setProjectFilter(e.target.value)}
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "12px",
              padding: "4px 8px",
              borderRadius: "var(--radius-sm)",
              border: "1px solid var(--border-2)",
              background: "var(--bg-2)",
              color: "var(--text-1)",
              cursor: "pointer",
              outline: "none",
            }}
          >
            <option value="all">All projects</option>
            {projects.map((p) => (
              <option key={p.project_name} value={p.project_name}>
                {p.project_name}
              </option>
            ))}
          </select>

          {/* Errors toggle */}
          <button
            onClick={() => setErrorsOnly(!errorsOnly)}
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "11px",
              fontWeight: 600,
              padding: "4px 10px",
              borderRadius: "var(--radius-sm)",
              border: errorsOnly ? "1px solid var(--red)" : "1px solid var(--border-2)",
              background: errorsOnly ? "rgba(255,69,69,0.08)" : "transparent",
              color: errorsOnly ? "var(--red)" : "var(--text-3)",
              cursor: "pointer",
              transition: "all 0.12s",
            }}
          >
            Errors only
          </button>
        </div>
      </div>

      {/* Session table */}
      {filtered.length === 0 ? (
        <Card>
          <EmptyState
            title="No sessions found"
            description={errorsOnly ? "No sessions with errors" : "No sessions match the current filter"}
          />
        </Card>
      ) : (
        <Card>
          <SessionTable
            sessions={filtered}
            title={`${filtered.length} sessions`}
          />
        </Card>
      )}
    </PageShell>
  );
}
