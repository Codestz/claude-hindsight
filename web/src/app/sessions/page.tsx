"use client";

import { useEffect, useState, Suspense } from "react";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { api } from "@/lib/api";
import type { SessionFile, TreeResponse } from "@/lib/types";
import { formatCost, formatTokens, formatBytes, shortId, timeAgo } from "@/lib/utils";
import { Header } from "@/components/layout/Header";
import { NodeTree } from "@/components/nodes/NodeTree";

function SessionDetail() {
  const searchParams = useSearchParams();
  const id = searchParams.get("id") ?? "";

  const [session, setSession] = useState<SessionFile | null>(null);
  const [tree, setTree] = useState<TreeResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) { setLoading(false); return; }
    Promise.all([api.session(id), api.sessionNodes(id)])
      .then(([s, t]) => { setSession(s); setTree(t); })
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [id]);

  if (!id) {
    return (
      <div className="flex flex-col flex-1">
        <Header title="Session" />
        <div className="flex-1 flex items-center justify-center">
          <div className="text-text-muted">No session ID provided</div>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex flex-col flex-1">
        <Header title="Session" />
        <div className="flex-1 flex items-center justify-center">
          <div className="text-text-muted animate-pulse">Loading session…</div>
        </div>
      </div>
    );
  }

  if (error || !session) {
    return (
      <div className="flex flex-col flex-1">
        <Header title="Session Not Found" />
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <div className="text-accent-red text-sm mb-2">{error ?? "Session not found"}</div>
            <Link href="/projects" className="text-accent-cyan text-xs hover:underline">← Back to projects</Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col flex-1">
      <Header
        title={shortId(session.session_id)}
        subtitle={session.project_name}
        actions={
          <div className="flex items-center gap-3">
            <Link
              href={`/sessions/live?id=${encodeURIComponent(session.session_id)}`}
              className="text-xs px-3 py-1.5 rounded transition-colors"
              style={{ background: "rgba(34,211,238,0.1)", color: "#22d3ee", border: "1px solid rgba(34,211,238,0.2)" }}
            >
              ⟳ Live watch
            </Link>
            <Link
              href={`/projects/detail?name=${encodeURIComponent(session.project_name)}`}
              className="text-text-muted text-sm hover:text-text-primary"
            >
              ← {session.project_name}
            </Link>
          </div>
        }
      />

      <div className="flex-1 p-6 space-y-6">
        <div className="card p-5 rounded-lg">
          <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
            <MetaStat label="Session ID" value={shortId(session.session_id)} mono />
            <MetaStat label="Tokens" value={formatTokens(session.total_tokens)} color="#4ade80" mono />
            <MetaStat label="Cost" value={formatCost(session.estimated_cost)} color="#facc15" mono />
            <MetaStat label="Size" value={formatBytes(session.file_size)} />
            <MetaStat label="Last active" value={timeAgo(session.modified_at)} />
          </div>
          <div className="mt-4 pt-4 border-t" style={{ borderColor: "rgba(255,255,255,0.06)" }}>
            <div className="flex items-center gap-3 flex-wrap">
              <span className="text-text-muted text-xs">Model:</span>
              <span className="text-text-primary text-xs mono">{session.model ?? "unknown"}</span>
              {session.has_subagents && (
                <span className="text-accent-magenta text-xs px-2 py-0.5 rounded" style={{ background: "rgba(232,121,249,0.1)" }}>has subagents</span>
              )}
              {session.error_count > 0 && (
                <span className="text-accent-red text-xs px-2 py-0.5 rounded" style={{ background: "rgba(248,113,113,0.1)" }}>
                  {session.error_count} error{session.error_count !== 1 ? "s" : ""}
                </span>
              )}
            </div>
            {session.first_message && (
              <div className="mt-2 text-text-muted text-sm italic truncate">&ldquo;{session.first_message}&rdquo;</div>
            )}
          </div>
        </div>

        <div className="card rounded-lg overflow-hidden">
          <div className="flex items-center justify-between px-5 py-4 border-b" style={{ borderColor: "rgba(255,255,255,0.06)" }}>
            <div className="text-text-primary font-medium text-sm">Execution Tree</div>
            {tree && <div className="text-text-muted text-xs mono">{tree.total_nodes} nodes · depth {tree.max_depth}</div>}
          </div>
          <div className="py-2">
            {tree ? <NodeTree tree={tree} /> : <div className="text-center py-8 text-text-muted text-sm">No tree data</div>}
          </div>
        </div>
      </div>
    </div>
  );
}

function MetaStat({ label, value, color, mono }: { label: string; value: string; color?: string; mono?: boolean }) {
  return (
    <div>
      <div className="text-text-muted text-xs mb-1">{label}</div>
      <div className={`font-medium text-sm ${mono ? "mono" : ""}`} style={{ color: color ?? "var(--text-primary)" }}>{value}</div>
    </div>
  );
}

export default function SessionPage() {
  return (
    <Suspense fallback={
      <div className="flex flex-col flex-1">
        <Header title="Session" />
        <div className="flex-1 flex items-center justify-center">
          <div className="text-text-muted animate-pulse">Loading…</div>
        </div>
      </div>
    }>
      <SessionDetail />
    </Suspense>
  );
}
