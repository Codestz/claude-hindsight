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
          <div className="mono text-sm" style={{ color: "var(--text-3)" }}>No session ID provided</div>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex flex-col flex-1">
        <Header title="Session" />
        <div className="flex-1 flex items-center justify-center">
          <div className="mono text-sm animate-pulse" style={{ color: "var(--text-2)" }}>Loading session…</div>
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
            <div className="text-sm mb-2" style={{ color: "var(--red)" }}>{error ?? "Session not found"}</div>
            <Link href="/projects" className="mono text-xs hover:underline" style={{ color: "var(--accent)" }}>← Back to projects</Link>
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
              className="mono text-xs px-3 py-1 transition-colors"
              style={{
                background: "rgba(0,255,136,0.08)",
                color: "var(--accent)",
                border: "1px solid rgba(0,255,136,0.2)",
                fontSize: "10px",
                letterSpacing: "0.06em",
                textTransform: "uppercase",
              }}
            >
              ⟳ Live watch
            </Link>
            <Link
              href={`/projects/detail?name=${encodeURIComponent(session.project_name)}`}
              className="mono text-xs"
              style={{ color: "var(--text-2)", fontSize: "11px" }}
            >
              ← {session.project_name}
            </Link>
          </div>
        }
      />

      <div className="flex-1 p-4 space-y-1">
        {/* Session meta — bento */}
        <div
          className="bento"
          style={{ gridTemplateColumns: "repeat(5, 1fr)" }}
        >
          {[
            { label: "Session ID", value: shortId(session.session_id), color: "var(--text-1)" },
            { label: "Tokens", value: formatTokens(session.total_tokens), color: "var(--cyan)" },
            { label: "Cost", value: formatCost(session.estimated_cost), color: "var(--amber)" },
            { label: "Size", value: formatBytes(session.file_size), color: "var(--text-1)" },
            { label: "Last active", value: timeAgo(session.modified_at), color: "var(--text-2)" },
          ].map((s) => (
            <div key={s.label} className="px-4 py-3" style={{ background: "var(--bg)" }}>
              <div className="label mb-1" style={{ color: "var(--text-3)" }}>{s.label}</div>
              <div className="mono font-medium text-sm tabular" style={{ color: s.color }}>{s.value}</div>
            </div>
          ))}
        </div>

        {/* Model / subagent / error row */}
        <div
          className="px-4 py-3 flex items-center gap-3 flex-wrap"
          style={{ background: "var(--bg-2)", border: "1px solid var(--border)" }}
        >
          <span className="label" style={{ color: "var(--text-3)" }}>Model</span>
          <span className="mono text-xs" style={{ color: "var(--text-2)" }}>{session.model ?? "unknown"}</span>
          {session.has_subagents && (
            <span className="tbadge tbadge-sub">has subagents</span>
          )}
          {session.error_count > 0 && (
            <span className="tbadge tbadge-err">{session.error_count} error{session.error_count !== 1 ? "s" : ""}</span>
          )}
          {session.first_message && (
            <span className="text-sm italic truncate" style={{ color: "var(--text-2)", marginLeft: "auto" }}>&ldquo;{session.first_message}&rdquo;</span>
          )}
        </div>

        {/* Execution tree */}
        <div style={{ background: "var(--bg-2)", border: "1px solid var(--border)" }}>
          <div
            className="flex items-center justify-between px-5 py-3"
            style={{ borderBottom: "1px solid var(--border)" }}
          >
            <div className="label" style={{ color: "var(--text-2)" }}>Execution Tree</div>
            {tree && (
              <div className="mono text-2xs" style={{ color: "var(--text-3)" }}>
                {tree.total_nodes} nodes · depth {tree.max_depth}
              </div>
            )}
          </div>
          <div className="py-2">
            {tree ? <NodeTree tree={tree} /> : <div className="mono text-center py-8 text-sm" style={{ color: "var(--text-3)" }}>No tree data</div>}
          </div>
        </div>
      </div>
    </div>
  );
}

export default function SessionPage() {
  return (
    <Suspense fallback={
      <div className="flex flex-col flex-1">
        <Header title="Session" />
        <div className="flex-1 flex items-center justify-center">
          <div className="mono text-sm animate-pulse" style={{ color: "var(--text-2)" }}>Loading…</div>
        </div>
      </div>
    }>
      <SessionDetail />
    </Suspense>
  );
}
