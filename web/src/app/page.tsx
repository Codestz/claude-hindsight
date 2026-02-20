"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { api } from "@/lib/api";
import type { GlobalAnalytics, SessionFile } from "@/lib/types";
import { formatCost, formatTokens, formatBytes, shortId, timeAgo } from "@/lib/utils";
import { Header } from "@/components/layout/Header";
import { Sparkline } from "@/components/charts/Sparkline";
import { ToolBarChart } from "@/components/charts/ToolBarChart";

export default function DashboardPage() {
  const [analytics, setAnalytics] = useState<GlobalAnalytics | null>(null);
  const [sparkline, setSparkline] = useState<number[]>([]);
  const [recent, setRecent] = useState<SessionFile[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      api.globalAnalytics(),
      api.globalSparkline(14),
      api.sessions({ limit: 8 }),
    ])
      .then(([analytics, spark, sessions]) => {
        setAnalytics(analytics);
        setSparkline(spark);
        setRecent(sessions);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <LoadingState />;
  if (!analytics) return <ErrorState />;

  const stats = [
    { label: "Total Sessions", value: analytics.total_sessions.toLocaleString(), color: "#22d3ee" },
    { label: "Total Tokens", value: formatTokens(analytics.total_tokens), color: "#4ade80" },
    { label: "Total Cost", value: formatCost(analytics.total_cost), color: "#facc15" },
    { label: "Projects", value: analytics.total_projects.toLocaleString(), color: "#e879f9" },
    { label: "Today", value: analytics.sessions_today.toLocaleString(), color: "#22d3ee" },
    { label: "This Week", value: analytics.sessions_this_week.toLocaleString(), color: "#4ade80" },
    { label: "With Errors", value: analytics.total_errors.toLocaleString(), color: "#f87171" },
    { label: "Avg Size", value: formatBytes(analytics.avg_session_size), color: "#64748b" },
  ];

  return (
    <div className="flex flex-col flex-1">
      <Header title="Dashboard" subtitle="Global session overview" />

      <div className="flex-1 p-6 space-y-6">
        {/* Stats grid */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {stats.map((stat) => (
            <StatCard key={stat.label} {...stat} />
          ))}
        </div>

        {/* Sparkline + Tools row */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Sessions sparkline */}
          <div className="md:col-span-2 card p-5 rounded-lg">
            <div className="flex items-center justify-between mb-4">
              <div>
                <div className="text-text-primary font-medium text-sm">Session Activity</div>
                <div className="text-text-muted text-xs">Last 14 days</div>
              </div>
              <div className="text-accent-cyan mono text-sm">
                {analytics.sessions_this_week} this week
              </div>
            </div>
            <Sparkline data={sparkline} color="#22d3ee" height={64} />
          </div>

          {/* Top tools */}
          <div className="card p-5 rounded-lg">
            <div className="text-text-primary font-medium text-sm mb-4">Top Tools</div>
            <ToolBarChart tools={analytics.top_tools} />
          </div>
        </div>

        {/* Recent sessions */}
        <div className="card rounded-lg overflow-hidden">
          <div className="flex items-center justify-between px-5 py-4 border-b" style={{ borderColor: "rgba(255,255,255,0.06)" }}>
            <div className="text-text-primary font-medium text-sm">Recent Sessions</div>
            <Link href="/projects" className="text-accent-cyan text-xs hover:underline">
              View all →
            </Link>
          </div>

          <div className="divide-y" style={{ borderColor: "rgba(255,255,255,0.04)" }}>
            {recent.map((s) => (
              <RecentRow key={s.session_id} session={s} />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

function StatCard({ label, value, color }: { label: string; value: string; color: string }) {
  return (
    <div className="card p-4 rounded-lg">
      <div className="text-text-muted text-xs mb-1">{label}</div>
      <div className="mono font-semibold text-xl" style={{ color }}>
        {value}
      </div>
    </div>
  );
}

function RecentRow({ session }: { session: SessionFile }) {
  return (
    <Link href={`/sessions?id=${session.session_id}`}>
      <div className="flex items-center gap-4 px-5 py-3 hover:bg-white/[0.02] transition-colors cursor-pointer">
        <span className="text-accent-cyan mono text-xs w-20 flex-shrink-0">
          {shortId(session.session_id)}
        </span>
        <div className="flex-1 min-w-0">
          <div className="text-text-primary text-sm truncate">
            {session.first_message ?? <span className="text-text-muted italic">No message</span>}
          </div>
          <div className="text-text-muted text-xs">{session.project_name}</div>
        </div>
        <div className="text-right flex-shrink-0">
          <div className="text-accent-yellow mono text-sm">{formatCost(session.estimated_cost)}</div>
          <div className="text-text-muted text-xs">{timeAgo(session.modified_at)}</div>
        </div>
      </div>
    </Link>
  );
}

function LoadingState() {
  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="text-text-muted text-sm animate-pulse">Loading dashboard…</div>
    </div>
  );
}

function ErrorState() {
  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="text-center">
        <div className="text-accent-red text-sm mb-2">Failed to load analytics</div>
        <div className="text-text-muted text-xs">Make sure the Rust server is running on :7227</div>
      </div>
    </div>
  );
}
