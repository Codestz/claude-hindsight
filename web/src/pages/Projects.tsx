import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { api } from "@/lib/api";
import type { ProjectStats } from "@/lib/types";
import { formatBytes, timeAgo } from "@/lib/utils";
import { PageShell } from "@/components/ui/PageShell";
import { LoadingState } from "@/components/ui/LoadingState";
import { ErrorState } from "@/components/ui/ErrorState";
import { EmptyState } from "@/components/ui/EmptyState";
import { Search } from "lucide-react";

// ─── Project Card ─────────────────────────────────────────────────────────────

interface ProjectCardProps {
  project: ProjectStats;
  maxSize: number;
}

function ProjectCard({ project, maxSize }: ProjectCardProps) {
  const navigate = useNavigate();
  const [hovered, setHovered] = useState(false);

  const sizeRatio = maxSize > 0 ? project.total_size / maxSize : 0;

  return (
    <div
      onClick={() => navigate(`/projects/${encodeURIComponent(project.project_name)}`)}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        background: "var(--bg-1)",
        border: `1px solid ${hovered ? "var(--indigo)" : "var(--border-1)"}`,
        borderLeft: `3px solid ${hovered ? "var(--indigo)" : "transparent"}`,
        borderRadius: "var(--radius-lg)",
        padding: "20px 20px 18px 18px",
        cursor: "pointer",
        boxShadow: hovered ? "var(--shadow-md)" : "var(--shadow-sm)",
        transition: "border-color 0.15s ease, box-shadow 0.15s ease, transform 0.15s ease",
        transform: hovered ? "translateY(-2px)" : "translateY(0)",
        display: "flex",
        flexDirection: "column",
        gap: "14px",
      }}
    >
      {/* Project name */}
      <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: "8px" }}>
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "15px",
            fontWeight: 600,
            color: "var(--indigo)",
            lineHeight: 1.3,
            wordBreak: "break-all",
          }}
        >
          {project.project_name}
        </span>
      </div>

      {/* Stats row */}
      <div style={{ display: "flex", gap: "20px", flexWrap: "wrap" }}>
        <StatPill label="Sessions" value={project.session_count.toLocaleString()} />
        <StatPill label="Size" value={formatBytes(project.total_size)} />
        <StatPill
          label="Last active"
          value={project.last_activity ? timeAgo(project.last_activity) : "—"}
        />
      </div>

      {/* Mini activity bar */}
      <div>
        <div
          style={{
            height: "4px",
            borderRadius: "2px",
            background: "var(--bg-2)",
            overflow: "hidden",
          }}
        >
          <div
            style={{
              height: "100%",
              width: `${Math.max(sizeRatio * 100, sizeRatio > 0 ? 2 : 0)}%`,
              background: hovered
                ? "var(--indigo)"
                : "color-mix(in srgb, var(--indigo) 60%, transparent)",
              borderRadius: "2px",
              transition: "background 0.15s ease, width 0.3s ease",
            }}
          />
        </div>
        <div
          style={{
            marginTop: "4px",
            fontSize: "11px",
            color: "var(--text-3)",
            fontFamily: "var(--font-mono)",
          }}
        >
          {Math.round(sizeRatio * 100)}% of largest
        </div>
      </div>
    </div>
  );
}

function StatPill({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1px" }}>
      <span style={{ fontSize: "11px", color: "var(--text-3)", fontFamily: "var(--font-sans)", textTransform: "uppercase", letterSpacing: "0.05em" }}>
        {label}
      </span>
      <span style={{ fontSize: "13px", fontWeight: 600, color: "var(--text-1)", fontFamily: "var(--font-mono)" }}>
        {value}
      </span>
    </div>
  );
}

// ─── Summary stat pill (inline) ───────────────────────────────────────────────

function SummaryPill({ label, value }: { label: string; value: string }) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: "6px",
        padding: "5px 12px",
        background: "var(--bg-1)",
        border: "1px solid var(--border-1)",
        borderRadius: "999px",
        fontSize: "13px",
        fontFamily: "var(--font-sans)",
      }}
    >
      <span style={{ color: "var(--text-3)" }}>{label}</span>
      <span style={{ color: "var(--text-1)", fontWeight: 600, fontFamily: "var(--font-mono)" }}>{value}</span>
    </div>
  );
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export default function ProjectsPage() {
  const [projects, setProjects] = useState<ProjectStats[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState("");

  useEffect(() => {
    api
      .projects()
      .then(setProjects)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <PageShell><LoadingState /></PageShell>;
  if (error) return <PageShell><ErrorState message={error} /></PageShell>;

  const totalSessions = projects.reduce((s, p) => s + p.session_count, 0);
  const totalSize = projects.reduce((s, p) => s + p.total_size, 0);
  const maxSize = Math.max(...projects.map((p) => p.total_size), 0);

  const filtered = search.trim()
    ? projects.filter((p) =>
        p.project_name.toLowerCase().includes(search.trim().toLowerCase())
      )
    : projects;

  return (
    <PageShell>
      {/* Page header */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "16px", flexWrap: "wrap" }}>
        <h1
          style={{
            fontSize: "20px",
            fontWeight: 600,
            color: "var(--text-1)",
            fontFamily: "var(--font-sans)",
            margin: 0,
          }}
        >
          Projects
        </h1>

        {/* Search input */}
        <div style={{ position: "relative", flexShrink: 0 }}>
          <Search
            size={14}
            style={{
              position: "absolute",
              left: "10px",
              top: "50%",
              transform: "translateY(-50%)",
              color: "var(--text-3)",
              pointerEvents: "none",
            }}
          />
          <input
            type="text"
            placeholder="Search projects…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            style={{
              paddingLeft: "30px",
              paddingRight: "12px",
              paddingTop: "7px",
              paddingBottom: "7px",
              background: "var(--bg-1)",
              border: "1px solid var(--border-1)",
              borderRadius: "var(--radius-lg)",
              color: "var(--text-1)",
              fontFamily: "var(--font-sans)",
              fontSize: "13px",
              outline: "none",
              width: "220px",
              transition: "border-color 0.15s ease",
            }}
            onFocus={(e) => { e.currentTarget.style.borderColor = "var(--indigo)"; }}
            onBlur={(e) => { e.currentTarget.style.borderColor = "var(--border-1)"; }}
          />
        </div>
      </div>

      {/* Summary stats row */}
      <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
        <SummaryPill label="Projects" value={projects.length.toLocaleString()} />
        <SummaryPill label="Sessions" value={totalSessions.toLocaleString()} />
        <SummaryPill label="Total size" value={formatBytes(totalSize)} />
      </div>

      {/* Card grid */}
      {filtered.length === 0 ? (
        <EmptyState title={search ? `No projects matching "${search}"` : "No projects found"} />
      ) : (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fill, minmax(340px, 1fr))",
            gap: "16px",
          }}
        >
          {filtered.map((project, i) => (
            <div
              key={project.project_name}
              className="animate-in"
              style={{ "--delay": `${i * 0.04}s` } as React.CSSProperties}
            >
              <ProjectCard project={project} maxSize={maxSize} />
            </div>
          ))}
        </div>
      )}
    </PageShell>
  );
}
