export function ConfigRow({ icon: Icon, label, value }: {
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  label: string;
  value: string;
}) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: "10px",
        padding: "10px 16px",
        borderBottom: "1px solid var(--border-1)",
      }}
    >
      <span style={{ color: "var(--text-3)", display: "flex", flexShrink: 0 }}>
        <Icon size={14} strokeWidth={2} />
      </span>
      <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", width: "120px", flexShrink: 0 }}>
        {label}
      </span>
      <span style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--text-1)" }}>
        {value}
      </span>
    </div>
  );
}
