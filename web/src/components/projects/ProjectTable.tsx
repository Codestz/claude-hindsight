import type { ProjectStats } from "@/lib/types";
import { SectionHeader } from "@/components/ui/SectionHeader";
import { ProjectRow, PROJECT_COLS } from "./ProjectRow";

const HEADERS = [
  { label: "Project",  span: "1fr" },
  { label: "Sessions", span: "80px" },
  { label: "Size",     span: "100px" },
  { label: "Active",   span: "100px", alignRight: true },
];

interface ProjectTableProps {
  projects: ProjectStats[];
  title?: string;
  emptyMessage?: string;
}

export function ProjectTable({
  projects,
  title = "Projects",
  emptyMessage = "No projects found",
}: ProjectTableProps) {
  return (
    <div>
      {/* Header bar */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "16px 24px",
          borderBottom: "1px solid var(--border-1)",
        }}
      >
        <SectionHeader title={title} count={projects.length} />
      </div>

      {/* Column labels */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: PROJECT_COLS,
          alignItems: "center",
          padding: "0 24px",
          height: "36px",
          borderBottom: "1px solid var(--border-1)",
        }}
      >
        {HEADERS.map((h) => (
          <span
            key={h.label}
            style={{
              fontSize: "11px",
              fontFamily: "var(--font-mono)",
              fontWeight: 600,
              letterSpacing: "0.08em",
              textTransform: "uppercase",
              color: "var(--text-3)",
              textAlign: h.alignRight ? "right" : "left",
            }}
          >
            {h.label}
          </span>
        ))}
      </div>

      {/* Rows */}
      {projects.length === 0 ? (
        <div
          style={{
            padding: "48px 24px",
            textAlign: "center",
            fontSize: "14px",
            color: "var(--text-3)",
            fontFamily: "var(--font-sans)",
          }}
        >
          {emptyMessage}
        </div>
      ) : (
        projects.map((p) => <ProjectRow key={p.project_name} project={p} />)
      )}
    </div>
  );
}
