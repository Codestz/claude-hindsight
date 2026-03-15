export function LoadingState() {
  return (
    <div
      style={{
        height: "60vh",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: "16px",
      }}
    >
      {/* Pulsing prism dot */}
      <div
        style={{
          width: "28px",
          height: "28px",
          borderRadius: "7px",
          background: "linear-gradient(135deg, var(--indigo), var(--violet))",
          opacity: 0.5,
          animation: "thinking-breathe 2s ease-in-out infinite",
        }}
      />
      <span
        style={{
          fontSize: "13px",
          color: "var(--text-3)",
          fontFamily: "var(--font-sans)",
          fontWeight: 400,
        }}
      >
        Loading
      </span>
    </div>
  );
}
