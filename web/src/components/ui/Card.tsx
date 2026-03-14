interface CardProps {
  children: React.ReactNode;
  padding?: string;
  glow?: string;
  animate?: boolean;
}

export function Card({ children, padding, glow, animate }: CardProps) {
  return (
    <div
      className={animate ? "animate-in" : undefined}
      style={{
        background: "var(--bg-1)",
        border: "1px solid var(--border-1)",
        borderRadius: "var(--radius-lg)",
        overflow: "hidden",
        padding,
        position: "relative",
        boxShadow: "var(--shadow-md)",
      }}
    >
      {glow && (
        <div
          aria-hidden="true"
          style={{
            position: "absolute",
            top: 0,
            left: "10%",
            right: "10%",
            height: "1px",
            background: `linear-gradient(90deg, transparent, ${glow}, transparent)`,
            opacity: 0.5,
          }}
        />
      )}
      {children}
    </div>
  );
}
