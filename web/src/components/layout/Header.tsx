"use client";

import { useRouter } from "next/navigation";
import { Search } from "lucide-react";

interface HeaderProps {
  title: string;
  subtitle?: string;
  actions?: React.ReactNode;
}

export function Header({ title, subtitle, actions }: HeaderProps) {
  const router = useRouter();

  const handleKeydown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      router.push(`/search?q=${encodeURIComponent(e.currentTarget.value)}`);
    }
  };

  return (
    <header
      className="sticky top-0 z-10 flex items-center gap-3 px-5"
      style={{
        height: "48px",
        background: "rgba(5,5,5,0.97)",
        backdropFilter: "blur(12px)",
        WebkitBackdropFilter: "blur(12px)",
        borderBottom: "1px solid var(--border)",
      }}
    >
      <div className="flex-1 min-w-0">
        <h1
          className="truncate"
          style={{
            fontWeight: 600,
            color: "var(--text-1)",
            fontSize: "0.875rem",
            letterSpacing: "-0.01em",
          }}
        >
          {title}
        </h1>
        {subtitle && (
          <p className="text-xs truncate" style={{ color: "var(--text-2)" }}>
            {subtitle}
          </p>
        )}
      </div>

      <div className="hidden md:flex items-center">
        <label htmlFor="header-search" className="sr-only">Search sessions</label>
        <div className="relative">
          <Search
            size={12}
            className="absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none"
            style={{ color: "var(--text-3)" }}
            aria-hidden="true"
          />
          <input
            id="header-search"
            type="search"
            name="q"
            placeholder="search sessions… (⌘K)"
            autoComplete="off"
            spellCheck={false}
            onKeyDown={handleKeydown}
            className="mono text-xs outline-none transition-colors duration-100"
            style={{
              width: "220px",
              paddingLeft: "2rem",
              paddingRight: "0.75rem",
              paddingTop: "0.35rem",
              paddingBottom: "0.35rem",
              background: "var(--bg-2)",
              border: "1px solid var(--border-2)",
              borderRadius: 0,
              color: "var(--text-1)",
              caretColor: "var(--accent)",
              fontSize: "11px",
            }}
          />
        </div>
      </div>

      {actions && <div className="flex items-center gap-2">{actions}</div>}
    </header>
  );
}
