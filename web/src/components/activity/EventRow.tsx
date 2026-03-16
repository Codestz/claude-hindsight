/**
 * Single event row in the activity timeline.
 *
 * Displays a hook event with time, icon, label, session link,
 * and expandable detail/error content.
 */

import { useState } from "react";
import { Link } from "react-router-dom";
import { ChevronDown, ChevronRight } from "lucide-react";
import { shortId, relativeTime } from "@/lib/utils";
import { KIND_CONFIG } from "./config";
import type { UnifiedEvent } from "./types";

export type { UnifiedEvent } from "./types";

interface EventRowProps {
  event: UnifiedEvent;
}

export function EventRow({ event }: EventRowProps) {
  const [expanded, setExpanded] = useState(false);
  const cfg = KIND_CONFIG[event.kind];
  const Icon = cfg.icon;
  const hasDetail = !!(event.detail || event.error);

  return (
    <div style={{ borderBottom: "1px solid var(--border-1)" }}>
      <div
        onClick={() => hasDetail && setExpanded(!expanded)}
        style={{
          display: "flex",
          alignItems: "center",
          gap: "10px",
          padding: "10px 16px",
          cursor: hasDetail ? "pointer" : "default",
          transition: "background 0.1s",
        }}
        onMouseEnter={(e) => {
          if (hasDetail) (e.currentTarget as HTMLElement).style.background = "var(--bg-2)";
        }}
        onMouseLeave={(e) => {
          (e.currentTarget as HTMLElement).style.background = "transparent";
        }}
      >
        {/* Time */}
        <span style={{
          fontFamily: "var(--font-mono)", fontSize: "11px",
          color: "var(--text-3)", width: "56px", flexShrink: 0, textAlign: "right",
        }}>
          {relativeTime(event.occurred_at)}
        </span>

        {/* Timeline dot */}
        <span style={{
          position: "relative", flexShrink: 0,
          display: "flex", alignItems: "center", justifyContent: "center",
          width: "20px", height: "20px",
        }}>
          <span style={{
            width: "8px", height: "8px", borderRadius: "50%",
            background: cfg.color, boxShadow: `0 0 6px ${cfg.color}`,
          }} />
        </span>

        {/* Label */}
        <span style={{
          fontSize: "13px",
          color: event.kind === "tool_failure" ? "var(--red)" : "var(--text-1)",
          fontFamily: "var(--font-sans)", flex: 1,
          overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
        }}>
          {event.label}
        </span>

        {/* Session link */}
        <Link
          to={`/sessions/${encodeURIComponent(event.session_id)}`}
          onClick={(e) => e.stopPropagation()}
          style={{
            fontFamily: "var(--font-mono)", fontSize: "11px",
            color: "var(--text-3)", flexShrink: 0, textDecoration: "none",
          }}
          onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--accent)"; }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-3)"; }}
        >
          {shortId(event.session_id)}
        </Link>

        {/* Expand arrow */}
        {hasDetail && (
          <span style={{ color: "var(--text-3)", display: "flex", flexShrink: 0 }}>
            {expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
          </span>
        )}
      </div>

      {/* Expanded detail */}
      {expanded && hasDetail && (
        <div style={{
          padding: "0 16px 12px 92px",
          fontSize: "12px", fontFamily: "var(--font-mono)",
          lineHeight: 1.5, whiteSpace: "pre-wrap", wordBreak: "break-all",
          color: event.error ? "var(--red)" : "var(--text-3)",
          maxHeight: "200px", overflowY: "auto",
        }}>
          {event.error || event.detail}
        </div>
      )}
    </div>
  );
}
