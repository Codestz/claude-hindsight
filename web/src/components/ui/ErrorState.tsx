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
        flexDirection: "column",
        gap: "10px",
        textAlign: "center",
      }}
    >
      <div style={{ fontSize: "14px", color: "var(--red)" }}>
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
  );
}
