import type { CSSProperties, ReactNode } from "react";

export type BadgeVariant =
  | "success"   // green  — healthy, positive
  | "error"     // red    — errors, failures
  | "warn"      // amber  — warnings, cost
  | "info"      // cyan   — tokens, informational
  | "purple"    // purple — AI nodes, thinking
  | "muted"     // dim    — neutral labels, model names
  | "default";  // subtle — general purpose

const VARIANT_STYLES: Record<BadgeVariant, CSSProperties> = {
  success: {
    background: "rgba(0, 255, 136, 0.1)",
    color: "var(--green)",
    border: "1px solid rgba(0, 255, 136, 0.2)",
  },
  error: {
    background: "rgba(255, 69, 69, 0.1)",
    color: "var(--red)",
    border: "1px solid rgba(255, 69, 69, 0.2)",
  },
  warn: {
    background: "rgba(255, 181, 71, 0.1)",
    color: "var(--amber)",
    border: "1px solid rgba(255, 181, 71, 0.2)",
  },
  info: {
    background: "rgba(0, 200, 255, 0.1)",
    color: "var(--cyan)",
    border: "1px solid rgba(0, 200, 255, 0.2)",
  },
  purple: {
    background: "rgba(167, 139, 250, 0.1)",
    color: "var(--purple)",
    border: "1px solid rgba(167, 139, 250, 0.2)",
  },
  muted: {
    background: "rgba(255, 255, 255, 0.04)",
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
        padding: "1px 6px",
        borderRadius: "var(--radius-sm)",
        fontFamily: "var(--font-mono)",
        fontSize: "11px",
        fontWeight: 500,
        letterSpacing: "0.03em",
        whiteSpace: "nowrap",
        lineHeight: "18px",
        ...VARIANT_STYLES[variant],
      }}
    >
      {children}
    </span>
  );
}
