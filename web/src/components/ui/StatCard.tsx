import type { StatCardProps } from "./types";

export function StatCard({ label, value, sub, accent, valueColor }: StatCardProps) {
  const accentColor = accent ? "var(--indigo)" : undefined;

  return (
    <div
      style={{
        padding: "22px 24px",
        background: "var(--bg-1)",
        position: "relative",
        borderRight: "1px solid var(--border-1)",
        boxShadow: accent ? "0 0 20px rgba(129, 140, 248, 0.06)" : undefined,
      }}
    >
      {/* Top accent glow */}
      {accent && (
        <div
          aria-hidden="true"
          style={{
            position: "absolute",
            top: 0,
            left: "15%",
            right: "15%",
            height: "1px",
            background: `linear-gradient(90deg, transparent, var(--indigo), transparent)`,
            opacity: 0.6,
          }}
        />
      )}

      <div
        style={{
          fontSize: "11px",
          fontWeight: 300,
          letterSpacing: "0.06em",
          textTransform: "uppercase",
          color: accentColor ?? "var(--text-3)",
          marginBottom: "10px",
          fontFamily: "var(--font-mono)",
        }}
      >
        {label}
      </div>

      <div
        style={{
          fontSize: "28px",
          fontWeight: 600,
          fontFamily: "var(--font-sans)",
          fontVariantNumeric: "tabular-nums",
          letterSpacing: "-0.03em",
          lineHeight: 1,
          color: valueColor ?? "var(--text-1)",
          marginBottom: sub ? "8px" : 0,
        }}
      >
        {value}
      </div>

      {sub && (
        <div
          style={{
            fontSize: "12px",
            color: "var(--text-3)",
            fontFamily: "var(--font-sans)",
          }}
        >
          {sub}
        </div>
      )}
    </div>
  );
}
