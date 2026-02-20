"use client";

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <div
      style={{
        height: "100vh",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        flexDirection: "column",
        gap: "16px",
        background: "var(--bg-0)",
        color: "var(--text-1)",
        fontFamily: "var(--font-sans)",
      }}
    >
      <div style={{ fontSize: "14px", color: "var(--red)" }}>
        Something went wrong
      </div>
      <div style={{ fontSize: "13px", color: "var(--text-3)" }}>
        {error.message}
      </div>
      <button
        onClick={reset}
        style={{
          padding: "6px 16px",
          background: "var(--bg-2)",
          border: "1px solid var(--border-2)",
          borderRadius: "var(--radius-md)",
          color: "var(--text-1)",
          fontSize: "13px",
          cursor: "pointer",
          fontFamily: "var(--font-sans)",
        }}
      >
        Try again
      </button>
    </div>
  );
}
