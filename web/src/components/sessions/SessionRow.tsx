"use client";

import Link from "next/link";
import type { SessionFile } from "@/lib/types";
import { formatCost, formatTokens, shortId, shortModel, timeAgo } from "@/lib/utils";
import { Badge } from "@/components/ui/Badge";

// Column layout — shared with SessionTable header so they always stay in sync.
// 8 cols: dot | id | message | project | model | tokens | cost | when
export const SESSION_COLS = "28px 96px 1fr 140px 110px 76px 76px 80px";

interface SessionRowProps {
  session: SessionFile;
}

export function SessionRow({ session: s }: SessionRowProps) {
  const hasErrors = s.error_count > 0;
  const model = shortModel(s.model);

  return (
    <Link
      href={`/sessions/${s.session_id}/`}
      style={{
        display: "grid",
        gridTemplateColumns: SESSION_COLS,
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
      }}
      onMouseEnter={(e) => {
        (e.currentTarget as HTMLElement).style.background = "var(--bg-2)";
      }}
      onMouseLeave={(e) => {
        (e.currentTarget as HTMLElement).style.background = "transparent";
      }}
    >
      {/* Status dot */}
      <span>
        <span
          title={hasErrors ? `${s.error_count} error(s)` : "No errors"}
          style={{
            display: "inline-block",
            width: "7px",
            height: "7px",
            borderRadius: "50%",
            background: hasErrors ? "var(--red)" : "var(--green)",
            opacity: hasErrors ? 1 : 0.5,
            flexShrink: 0,
          }}
        />
      </span>

      {/* Session ID */}
      <span style={{ color: "var(--accent)", letterSpacing: "0.02em" }}>
        {shortId(s.session_id)}
      </span>

      {/* First message */}
      <span
        style={{
          fontFamily: "var(--font-sans)",
          fontVariantNumeric: "normal",
          color: s.first_message ? "var(--text-1)" : "var(--text-3)",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          paddingRight: "20px",
          fontSize: "14px",
        }}
      >
        {s.first_message ?? "No message"}
      </span>

      {/* Project */}
      <span
        style={{
          color: "var(--text-2)",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          paddingRight: "12px",
          fontSize: "13px",
        }}
      >
        {s.project_name}
      </span>

      {/* Model */}
      <span>
        {model !== "—" ? (
          <Badge variant="muted">{model}</Badge>
        ) : (
          <span style={{ color: "var(--text-3)" }}>—</span>
        )}
      </span>

      {/* Tokens */}
      <span style={{ color: "var(--cyan)" }}>
        {formatTokens(s.total_tokens)}
      </span>

      {/* Cost */}
      <span style={{ color: "var(--amber)" }}>
        {formatCost(s.estimated_cost)}
      </span>

      {/* When */}
      <span
        style={{ color: "var(--text-3)", textAlign: "right", fontSize: "13px" }}
      >
        {timeAgo(s.modified_at)}
      </span>
    </Link>
  );
}
