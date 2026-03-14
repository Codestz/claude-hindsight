interface ErrorStateProps {
  message?: string | null;
  suggestion?: string;
}

export function ErrorState({ message, suggestion }: ErrorStateProps) {
  return (
    <div
      style={{
        height: "60vh",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <div style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: "12px",
        textAlign: "center",
        padding: "32px 40px",
        background: "color-mix(in srgb, var(--rose) 4%, var(--bg-1))",
        border: "1px solid color-mix(in srgb, var(--rose) 12%, transparent)",
        borderRadius: "var(--radius-lg)",
      }}>
        <span style={{ fontSize: "24px" }}>{"\u26A0"}</span>
        <div style={{ fontSize: "14px", color: "var(--rose)" }}>
          {message ?? "Something went wrong"}
        </div>
        <div style={{ fontSize: "13px", color: "var(--text-3)" }}>
          {suggestion ?? (
            <>
              Is{" "}
              <code style={{ fontFamily: "var(--font-mono)", color: "var(--accent)" }}>
                claude-hindsight serve
              </code>{" "}
              running on :7227?
            </>
          )}
        </div>
      </div>
    </div>
  );
}
