interface StatCardProps {
  label: string;
  value: string;
  sub?: string;
  accent?: boolean;
  valueColor?: string;
}

export function StatCard({ label, value, sub, accent, valueColor }: StatCardProps) {
  return (
    <div
      style={{
        padding: "24px 28px",
        background: "var(--bg-1)",
        position: "relative",
      }}
    >
      {/* Accent top line */}
      {accent && (
        <div
          aria-hidden="true"
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            right: 0,
            height: "2px",
            background: "var(--accent)",
          }}
        />
      )}

      <div
        style={{
          fontSize: "12px",
          fontWeight: 600,
          letterSpacing: "0.08em",
          textTransform: "uppercase",
          color: accent ? "var(--accent)" : "var(--text-3)",
          marginBottom: "12px",
          fontFamily: "var(--font-mono)",
        }}
      >
        {label}
      </div>

      <div
        style={{
          fontSize: "34px",
          fontWeight: 700,
          fontFamily: "var(--font-mono)",
          fontVariantNumeric: "tabular-nums",
          letterSpacing: "-0.03em",
          lineHeight: 1,
          color: valueColor ?? "var(--text-1)",
          marginBottom: "8px",
        }}
      >
        {value}
      </div>

      {sub && (
        <div
          style={{
            fontSize: "13px",
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
