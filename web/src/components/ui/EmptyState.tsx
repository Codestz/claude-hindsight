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
      <span
        className="animate-in"
        style={{
          width: "48px",
          height: "48px",
          borderRadius: "50%",
          background: "var(--bg-2)",
          border: "1px solid var(--border-1)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          fontSize: "22px",
          color: "var(--text-3)",
        }}
      >
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
