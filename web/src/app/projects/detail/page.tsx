"use client";

import { useEffect, useState, Suspense } from "react";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { api } from "@/lib/api";
import type { ProjectAnalytics, SessionFile } from "@/lib/types";
import { formatCost, formatTokens, formatBytes } from "@/lib/utils";
import { Header } from "@/components/layout/Header";
import { SessionsTable } from "@/components/sessions/SessionsTable";
import { ToolBarChart } from "@/components/charts/ToolBarChart";

function ProjectDetail() {
  const searchParams = useSearchParams();
  const name = searchParams.get("name") ?? "";
  const projectName = decodeURIComponent(name);

  const [analytics, setAnalytics] = useState<ProjectAnalytics | null>(null);
  const [sessions, setSessions] = useState<SessionFile[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!projectName) { setLoading(false); return; }
    Promise.all([api.projectAnalytics(projectName), api.sessions({ project: projectName })])
      .then(([a, s]) => { setAnalytics(a); setSessions(s); })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [projectName]);

  return (
    <div className="flex flex-col flex-1">
      <Header
        title={projectName || "Project"}
        subtitle={`${sessions.length} session${sessions.length !== 1 ? "s" : ""}`}
        actions={
          <Link href="/projects" className="mono text-xs" style={{ color: "var(--text-2)", fontSize: "11px" }}>← Projects</Link>
        }
      />

      <div className="flex-1 p-4 space-y-1">
        {analytics && (
          <>
            {/* Stats bento */}
            <div
              className="bento"
              style={{ gridTemplateColumns: "repeat(4, 1fr)" }}
            >
              {[
                { label: "Sessions",   value: analytics.total_sessions.toLocaleString(), color: "var(--text-1)" },
                { label: "Tokens",     value: formatTokens(analytics.total_tokens),      color: "var(--cyan)" },
                { label: "Total Cost", value: formatCost(analytics.total_cost),           color: "var(--amber)" },
                { label: "Total Size", value: formatBytes(analytics.total_size),          color: "var(--text-2)" },
              ].map((s) => (
                <div key={s.label} className="px-4 py-3" style={{ background: "var(--bg)" }}>
                  <div className="label mb-1" style={{ color: "var(--text-3)" }}>{s.label}</div>
                  <div className="mono font-bold tabular" style={{ fontSize: "1.25rem", color: s.color }}>{s.value}</div>
                </div>
              ))}
            </div>

            {analytics.top_tools.length > 0 && (
              <div className="p-5" style={{ background: "var(--bg-2)", border: "1px solid var(--border)", maxWidth: "360px" }}>
                <div className="label mb-4" style={{ color: "var(--text-2)" }}>Top Tools</div>
                <ToolBarChart tools={analytics.top_tools} />
              </div>
            )}
          </>
        )}

        {/* Sessions table */}
        <div style={{ background: "var(--border)" }}>
          <div
            className="px-5 py-3 label"
            style={{ background: "var(--bg)", borderBottom: "1px solid var(--border)", color: "var(--text-2)" }}
          >
            Sessions
          </div>
          <div style={{ background: "var(--bg)" }}>
            {loading ? (
              <div className="mono text-center py-16 animate-pulse" style={{ color: "var(--text-3)" }}>Loading sessions…</div>
            ) : (
              <SessionsTable sessions={sessions} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default function ProjectDetailPage() {
  return (
    <Suspense fallback={
      <div className="flex flex-col flex-1">
        <Header title="Project" />
        <div className="flex-1 flex items-center justify-center">
          <div className="mono text-sm animate-pulse" style={{ color: "var(--text-2)" }}>Loading…</div>
        </div>
      </div>
    }>
      <ProjectDetail />
    </Suspense>
  );
}
