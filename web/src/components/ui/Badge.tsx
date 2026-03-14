import type { CSSProperties, ReactNode } from "react";

export type BadgeVariant =
  | "success"   // emerald — healthy, positive
  | "error"     // rose    — errors, failures
  | "warn"      // amber   — warnings, cost
  | "info"      // sky     — tokens, informational
  | "purple"    // violet  — AI nodes, thinking
  | "muted"     // dim     — neutral labels, model names
  | "default";  // subtle  — general purpose

const VARIANT_STYLES: Record<BadgeVariant, CSSProperties> = {
  success: {
    background: "rgba(52, 211, 153, 0.10)",
    color: "var(--emerald)",
    border: "1px solid rgba(52, 211, 153, 0.18)",
  },
  error: {
    background: "rgba(251, 113, 133, 0.10)",
    color: "var(--rose)",
    border: "1px solid rgba(251, 113, 133, 0.18)",
  },
  warn: {
    background: "rgba(245, 158, 11, 0.10)",
    color: "var(--amber)",
    border: "1px solid rgba(245, 158, 11, 0.18)",
  },
  info: {
    background: "rgba(56, 189, 248, 0.10)",
    color: "var(--sky)",
    border: "1px solid rgba(56, 189, 248, 0.18)",
  },
  purple: {
    background: "rgba(167, 139, 250, 0.10)",
    color: "var(--violet)",
    border: "1px solid rgba(167, 139, 250, 0.18)",
  },
  muted: {
    background: "rgba(255, 255, 255, 0.03)",
    color: "var(--text-3)",
    border: "1px solid var(--border-1)",
  },
  default: {
    background: "var(--bg-2)",
    color: "var(--text-2)",
    border: "1px solid var(--border-2)",
  },
};

interface BadgeProps {
  children: ReactNode;
  variant?: BadgeVariant;
}

export function Badge({ children, variant = "default" }: BadgeProps) {
  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        padding: "3px 9px",
        borderRadius: "var(--radius-md)",
        fontFamily: "var(--font-mono)",
        fontSize: "11px",
        fontWeight: 500,
        letterSpacing: "0.02em",
        whiteSpace: "nowrap",
        lineHeight: "18px",
        ...VARIANT_STYLES[variant],
      }}
    >
      {children}
    </span>
  );
}
