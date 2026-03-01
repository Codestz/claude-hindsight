interface EmptyStateProps {
  icon?: string;
  title: string;
  description?: string;
}

export function EmptyState({ icon = "∅", title, description }: EmptyStateProps) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        padding: "60px 20px",
        gap: "12px",
        textAlign: "center",
      }}
    >
      <span style={{ fontSize: "28px", color: "var(--text-3)", opacity: 0.5 }}>
        {icon}
      </span>
      <div
        style={{
          fontSize: "15px",
          fontWeight: 500,
          color: "var(--text-2)",
          fontFamily: "var(--font-sans)",
        }}
      >
        {title}
      </div>
      {description && (
        <div
          style={{
            fontSize: "13px",
            color: "var(--text-3)",
            fontFamily: "var(--font-sans)",
            maxWidth: "320px",
          }}
        >
          {description}
        </div>
      )}
    </div>
  );
}
