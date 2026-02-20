"use client";

import Link from "next/link";
import type { SessionFile } from "@/lib/types";
import { formatCost, formatTokens, shortId, timeAgo } from "@/lib/utils";

interface SessionCardProps {
  session: SessionFile;
}

export function SessionCard({ session }: SessionCardProps) {
  return (
    <Link href={`/sessions?id=${session.session_id}`}>
      <div
        className="cursor-pointer transition-colors duration-150"
        style={{
          background: "var(--bg-2)",
          border: "1px solid var(--border)",
          padding: "1rem",
        }}
        onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.background = "var(--bg-3)"; }}
        onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.background = "var(--bg-2)"; }}
      >
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2 mb-1 flex-wrap">
              <span className="mono text-xs" style={{ color: "var(--accent)" }}>{shortId(session.session_id)}</span>
              {session.has_subagents && (
                <span className="tbadge tbadge-sub">subagent</span>
              )}
              {session.error_count > 0 && (
                <span className="tbadge tbadge-err">{session.error_count} err</span>
              )}
            </div>
            <p className="text-sm truncate" style={{ color: "var(--text-1)" }}>
              {session.first_message ?? "(no message)"}
            </p>
            <div className="mono text-xs mt-1" style={{ color: "var(--text-3)" }}>{session.project_name}</div>
          </div>
          <div className="text-right flex-shrink-0">
            <div className="mono text-sm tabular" style={{ color: "var(--amber)" }}>{formatCost(session.estimated_cost)}</div>
            <div className="mono text-xs tabular" style={{ color: "var(--cyan)" }}>{formatTokens(session.total_tokens)}tok</div>
            <div className="mono text-xs mt-1" style={{ color: "var(--text-3)" }}>{timeAgo(session.modified_at)}</div>
          </div>
        </div>
      </div>
    </Link>
  );
}
