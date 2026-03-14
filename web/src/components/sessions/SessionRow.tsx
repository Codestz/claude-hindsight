import { Link } from "react-router-dom";
import type { SessionFile } from "@/lib/types";
import { shortId, shortModel, timeAgo } from "@/lib/utils";
import { Badge } from "@/components/ui/Badge";

// Column layout — shared with SessionTable header so they always stay in sync.
// 6 cols: dot | id | message | project | model | when
export const SESSION_COLS = "28px 96px 1fr 140px 110px 80px";

interface SessionRowProps {
  session: SessionFile;
}

/** Derive a short label from a source_dir path.
 *  "~/.claudep/projects" → "claudep"
 *  "~/.claude/projects"  → "claude" (omitted — it's the default, no tag)
 */
function sourceLabel(sourceDir: string): string | null {
  if (!sourceDir || sourceDir === "~/.claude/projects") return null;
  // Take the second path segment after ~ (e.g. "~/.claudep/projects" → ".claudep" → "claudep")
  const parts = sourceDir.replace(/^~\//, "").split("/");
  const segment = parts[0]?.replace(/^\./, "") ?? null;
  return segment || null;
}

export function SessionRow({ session: s }: SessionRowProps) {
  const hasErrors = s.error_count > 0;
  const model = shortModel(s.model);
  const srcLabel = sourceLabel(s.source_dir ?? "");

  return (
    <Link
      to={`/sessions/${encodeURIComponent(s.session_id)}`}
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
        borderLeft: "2px solid transparent",
        transition: "background 0.1s, border-color 0.1s",
        cursor: "pointer",
      }}
      onMouseEnter={(e) => {
        (e.currentTarget as HTMLElement).style.background = "var(--bg-2)";
        (e.currentTarget as HTMLElement).style.borderLeft = "2px solid var(--indigo)";
      }}
      onMouseLeave={(e) => {
        (e.currentTarget as HTMLElement).style.background = "transparent";
        (e.currentTarget as HTMLElement).style.borderLeft = "2px solid transparent";
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
            boxShadow: hasErrors ? "0 0 6px var(--rose)" : undefined,
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

      {/* Project + optional source label */}
      <span
        style={{
          display: "flex",
          alignItems: "center",
          gap: "6px",
          overflow: "hidden",
          paddingRight: "12px",
        }}
      >
        <span
          style={{
            color: "var(--text-2)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            fontSize: "13px",
          }}
        >
          {s.project_name}
        </span>
        {srcLabel && (
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "10px",
              color: "var(--amber)",
              opacity: 0.75,
              flexShrink: 0,
              letterSpacing: "0.04em",
            }}
          >
            [{srcLabel}]
          </span>
        )}
      </span>

      {/* Model */}
      <span>
        {model !== "—" ? (
          <Badge variant="muted">{model}</Badge>
        ) : (
          <span style={{ color: "var(--text-3)" }}>—</span>
        )}
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
