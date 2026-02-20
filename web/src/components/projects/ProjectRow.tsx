"use client";

import { useRouter } from "next/navigation";
import type { ProjectStats } from "@/lib/types";
import { formatBytes, timeAgo } from "@/lib/utils";

// Column layout — shared with ProjectTable header so they always stay in sync.
// 4 cols: name | sessions | size | last active
export const PROJECT_COLS = "1fr 80px 100px 100px";

interface ProjectRowProps {
  project: ProjectStats;
}

export function ProjectRow({ project: p }: ProjectRowProps) {
  const router = useRouter();

  function go() {
    router.push(`/projects/${encodeURIComponent(p.project_name)}/`);
  }

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={go}
      onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") go(); }}
      style={{
        display: "grid",
        gridTemplateColumns: PROJECT_COLS,
        alignItems: "center",
        padding: "0 24px",
        height: "52px",
        borderBottom: "1px solid var(--border-1)",
        color: "var(--text-2)",
        fontSize: "14px",
        fontFamily: "var(--font-mono)",
        fontVariantNumeric: "tabular-nums",
        textDecoration: "none",
        transition: "background 0.1s",
        cursor: "pointer",
        userSelect: "none",
      }}
      onMouseEnter={(e) => {
        (e.currentTarget as HTMLElement).style.background = "var(--bg-2)";
      }}
      onMouseLeave={(e) => {
        (e.currentTarget as HTMLElement).style.background = "transparent";
      }}
    >
      {/* Project name */}
      <span
        style={{
          color: "var(--accent)",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          paddingRight: "20px",
        }}
      >
        {p.project_name}
      </span>

      {/* Session count */}
      <span style={{ color: "var(--text-1)" }}>
        {p.session_count.toLocaleString()}
      </span>

      {/* Transcript size */}
      <span style={{ color: "var(--text-2)" }}>
        {formatBytes(p.total_size)}
      </span>

      {/* Last active */}
      <span style={{ color: "var(--text-3)", textAlign: "right" }}>
        {p.last_activity ? timeAgo(p.last_activity) : "—"}
      </span>
    </div>
  );
}
