export function ScopeBadge({ scope }: { scope: string }) {
  const isGlobal = scope === "global";
  const isPlugin = scope.startsWith("plugin:");
  const label = isPlugin ? scope.replace("plugin:", "") : scope;
  const color = isGlobal ? "var(--info)" : isPlugin ? "var(--violet)" : "var(--amber)";

  return (
    <span
      style={{
        fontFamily: "var(--font-mono)",
        fontSize: "10px",
        fontWeight: 600,
        color,
        background: `color-mix(in srgb, ${color} 12%, transparent)`,
        border: `1px solid color-mix(in srgb, ${color} 25%, transparent)`,
        borderRadius: "var(--radius-md)",
        padding: "1px 6px",
      }}
    >
      {label}
    </span>
  );
}
