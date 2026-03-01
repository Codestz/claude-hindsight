interface PageShellProps {
  children: React.ReactNode;
  maxWidth?: string;
}

export function PageShell({ children, maxWidth = "1200px" }: PageShellProps) {
  return (
    <div
      style={{
        maxWidth,
        margin: "0 auto",
        padding: "36px 28px",
        display: "flex",
        flexDirection: "column",
        gap: "20px",
      }}
    >
      {children}
    </div>
  );
}
