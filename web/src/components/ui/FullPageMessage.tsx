/**
 * Full-page centered message for loading and error states.
 */

interface FullPageMessageProps {
  children: React.ReactNode;
  error?: boolean;
}

export function FullPageMessage({ children, error }: FullPageMessageProps) {
  return (
    <div style={{
      height: "calc(100vh - 56px)", display: "flex", alignItems: "center",
      justifyContent: "center", fontSize: "14px",
      color: error ? "var(--red)" : "var(--text-3)",
      fontFamily: "var(--font-sans)", textAlign: "center",
    }}>
      {children}
    </div>
  );
}
