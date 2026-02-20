import Link from "next/link";

interface SectionHeaderProps {
  title: string;
  // Pill count shown next to the title
  count?: number;
  // Right-aligned action link
  action?: { label: string; href: string };
}

export function SectionHeader({ title, count, action }: SectionHeaderProps) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <span
          style={{
            fontSize: "15px",
            fontWeight: 600,
            color: "var(--text-1)",
            fontFamily: "var(--font-sans)",
          }}
        >
          {title}
        </span>

        {count !== undefined && (
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "11px",
              color: "var(--text-3)",
              background: "var(--bg-2)",
              border: "1px solid var(--border-2)",
              borderRadius: "var(--radius-sm)",
              padding: "0 6px",
              lineHeight: "18px",
            }}
          >
            {count}
          </span>
        )}
      </div>

      {action && (
        <Link
          href={action.href}
          style={{
            fontSize: "12px",
            color: "var(--text-3)",
            fontFamily: "var(--font-sans)",
            transition: "color 0.12s",
            display: "flex",
            alignItems: "center",
            gap: "3px",
          }}
          onMouseEnter={(e) => {
            (e.currentTarget as HTMLElement).style.color = "var(--accent)";
          }}
          onMouseLeave={(e) => {
            (e.currentTarget as HTMLElement).style.color = "var(--text-3)";
          }}
        >
          {action.label}
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
        </Link>
      )}
    </div>
  );
}
