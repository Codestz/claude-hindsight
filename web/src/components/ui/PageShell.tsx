interface PageShellProps {
  children: React.ReactNode;
  maxWidth?: string;
}

export function PageShell({ children, maxWidth = "1280px" }: PageShellProps) {
  return (
    <div
      style={{
        maxWidth,
        margin: "0 auto",
        padding: "28px 32px",
        display: "flex",
        flexDirection: "column",
        gap: "20px",
      }}
    >
      {children}
    </div>
  );
}
