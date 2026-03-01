interface FilterChipsProps {
  options: string[];
  active: Set<string>;
  onToggle: (value: string) => void;
}

export function FilterChips({ options, active, onToggle }: FilterChipsProps) {
  return (
    <div style={{ display: "flex", flexWrap: "wrap", gap: "4px" }}>
      {options.map((opt) => {
        const isActive = active.has(opt.toLowerCase());
        return (
          <button
            key={opt}
            onClick={() => onToggle(opt.toLowerCase())}
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "10px",
              fontWeight: 600,
              letterSpacing: "0.05em",
              padding: "2px 8px",
              borderRadius: "var(--radius-sm)",
              border: isActive
                ? "1px solid var(--accent)"
                : "1px solid var(--border-2)",
              background: isActive ? "rgba(0,255,136,0.08)" : "transparent",
              color: isActive ? "var(--accent)" : "var(--text-3)",
              cursor: "pointer",
              transition: "all 0.12s",
            }}
          >
            {opt}
          </button>
        );
      })}
    </div>
  );
}
