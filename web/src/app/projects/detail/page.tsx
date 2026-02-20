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
          <Link href="/projects" className="text-text-muted text-sm hover:text-text-primary">← Projects</Link>
        }
      />

      <div className="flex-1 p-6 space-y-6">
        {analytics && (
          <>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {[
                { label: "Sessions", value: analytics.total_sessions.toLocaleString(), color: "#22d3ee" },
                { label: "Tokens", value: formatTokens(analytics.total_tokens), color: "#4ade80" },
                { label: "Total Cost", value: formatCost(analytics.total_cost), color: "#facc15" },
                { label: "Total Size", value: formatBytes(analytics.total_size), color: "#64748b" },
              ].map((s) => (
                <div key={s.label} className="card p-4 rounded-lg">
                  <div className="text-text-muted text-xs mb-1">{s.label}</div>
                  <div className="mono font-semibold text-lg" style={{ color: s.color }}>{s.value}</div>
                </div>
              ))}
            </div>

            {analytics.top_tools.length > 0 && (
              <div className="card p-5 rounded-lg max-w-sm">
                <div className="text-text-primary font-medium text-sm mb-4">Top Tools</div>
                <ToolBarChart tools={analytics.top_tools} />
              </div>
            )}
          </>
        )}

        <div className="card rounded-lg overflow-hidden">
          <div className="px-5 py-4 border-b text-text-primary font-medium text-sm" style={{ borderColor: "rgba(255,255,255,0.06)" }}>
            Sessions
          </div>
          {loading ? (
            <div className="text-center py-16 text-text-muted animate-pulse">Loading sessions…</div>
          ) : (
            <SessionsTable sessions={sessions} />
          )}
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
          <div className="text-text-muted animate-pulse">Loading…</div>
        </div>
      </div>
    }>
      <ProjectDetail />
    </Suspense>
  );
}
