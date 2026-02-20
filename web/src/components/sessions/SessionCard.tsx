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
        className="card p-4 rounded-lg cursor-pointer transition-all duration-150 hover:border-accent-cyan/30"
        style={{ borderColor: "rgba(255,255,255,0.08)" }}
      >
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2 mb-1">
              <span className="text-accent-cyan mono text-xs">{shortId(session.session_id)}</span>
              {session.has_subagents && (
                <span className="text-accent-magenta text-xs px-1.5 py-0.5 rounded" style={{ background: "rgba(232,121,249,0.1)" }}>
                  subagent
                </span>
              )}
              {session.error_count > 0 && (
                <span className="text-accent-red text-xs px-1.5 py-0.5 rounded" style={{ background: "rgba(248,113,113,0.1)" }}>
                  {session.error_count} err
                </span>
              )}
            </div>
            <p className="text-text-primary text-sm truncate">
              {session.first_message ?? "(no message)"}
            </p>
            <div className="text-text-muted text-xs mt-1">{session.project_name}</div>
          </div>
          <div className="text-right flex-shrink-0">
            <div className="text-accent-yellow mono text-sm">{formatCost(session.estimated_cost)}</div>
            <div className="text-text-muted text-xs mono">{formatTokens(session.total_tokens)}tok</div>
            <div className="text-text-muted text-xs mt-1">{timeAgo(session.modified_at)}</div>
          </div>
        </div>
      </div>
    </Link>
  );
}
