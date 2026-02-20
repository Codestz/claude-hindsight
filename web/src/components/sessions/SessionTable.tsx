import type { SessionFile } from "@/lib/types";
import { SectionHeader } from "@/components/ui/SectionHeader";
import { SessionRow, SESSION_COLS } from "./SessionRow";

// Column header labels — order must match SESSION_COLS in SessionRow.tsx
const HEADERS = [
  { label: "",        span: "28px" },    // status dot (no label)
  { label: "Session", span: "96px" },
  { label: "Message", span: "1fr" },
  { label: "Project", span: "140px" },
  { label: "Model",   span: "110px" },
  { label: "Tokens",  span: "76px" },
  { label: "Cost",    span: "76px" },
  { label: "When",    span: "80px", alignRight: true },
];

interface SessionTableProps {
  sessions: SessionFile[];
  title?: string;
  action?: { label: string; href: string };
  emptyMessage?: string;
}

export function SessionTable({
  sessions,
  title = "Recent Sessions",
  action,
  emptyMessage = "No sessions found",
}: SessionTableProps) {
  return (
    <div>
      {/* Table header bar */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "16px 24px",
          borderBottom: "1px solid var(--border-1)",
        }}
      >
        <SectionHeader
          title={title}
          count={sessions.length}
          action={action}
        />
      </div>

      {/* Column labels */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: SESSION_COLS,
          alignItems: "center",
          padding: "0 24px",
          height: "36px",
          borderBottom: "1px solid var(--border-1)",
        }}
      >
        {HEADERS.map((h) => (
          <span
            key={h.label}
            style={{
              fontSize: "11px",
              fontFamily: "var(--font-mono)",
              fontWeight: 600,
              letterSpacing: "0.08em",
              textTransform: "uppercase",
              color: "var(--text-3)",
              textAlign: h.alignRight ? "right" : "left",
            }}
          >
            {h.label}
          </span>
        ))}
      </div>

      {/* Rows */}
      {sessions.length === 0 ? (
        <div
          style={{
            padding: "48px 24px",
            textAlign: "center",
            fontSize: "14px",
            color: "var(--text-3)",
            fontFamily: "var(--font-sans)",
          }}
        >
          {emptyMessage}
        </div>
      ) : (
        sessions.map((s) => <SessionRow key={s.session_id} session={s} />)
      )}
    </div>
  );
}
