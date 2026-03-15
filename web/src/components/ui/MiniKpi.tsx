/**
 * Compact KPI display with icon, value, and label.
 */

interface MiniKpiProps {
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  label: string;
  value: number;
  color?: string;
}

export function MiniKpi({ icon: Icon, label, value, color = "var(--text-1)" }: MiniKpiProps) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
      <span style={{ color: "var(--text-3)", display: "flex" }}>
        <Icon size={13} strokeWidth={2} />
      </span>
      <span style={{ fontFamily: "var(--font-mono)", fontSize: "16px", fontWeight: 700, color, fontVariantNumeric: "tabular-nums" }}>
        {value.toLocaleString()}
      </span>
      <span style={{ fontSize: "11px", color: "var(--text-3)" }}>{label}</span>
    </div>
  );
}
