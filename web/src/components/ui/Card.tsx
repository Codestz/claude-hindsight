interface CardProps {
  children: React.ReactNode;
  padding?: string;
}

export function Card({ children, padding }: CardProps) {
  return (
    <div
      style={{
        background: "var(--bg-1)",
        border: "1px solid var(--border-1)",
        borderRadius: "var(--radius-lg)",
        overflow: "hidden",
        padding,
      }}
    >
      {children}
    </div>
  );
}
