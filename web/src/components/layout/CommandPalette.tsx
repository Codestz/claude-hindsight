"use client";

import { useEffect, useRef, useState } from "react";

interface Props {
  open: boolean;
  onClose: () => void;
}

// Quick-links shown before the user types anything
const QUICK_LINKS = [
  { label: "Dashboard", href: "/", category: "Navigate" },
  { label: "Projects", href: "/projects", category: "Navigate" },
];

export function CommandPalette({ open, onClose }: Props) {
  const [query, setQuery] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus when opened, clear on close
  useEffect(() => {
    if (open) {
      // Small delay so the element is mounted
      const t = setTimeout(() => inputRef.current?.focus(), 40);
      return () => clearTimeout(t);
    } else {
      setQuery("");
    }
  }, [open]);

  // Escape to close
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  if (!open) return null;

  const filtered = QUICK_LINKS.filter((l) =>
    l.label.toLowerCase().includes(query.toLowerCase())
  );

  return (
    // Backdrop
    <div
      onClick={onClose}
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 200,
        background: "rgba(0, 0, 0, 0.65)",
        backdropFilter: "blur(3px)",
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "center",
        paddingTop: "15vh",
      }}
    >
      {/* Panel — stop click from closing when interacting with panel */}
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          width: "100%",
          maxWidth: "560px",
          margin: "0 16px",
          background: "var(--bg-1)",
          border: "1px solid var(--border-2)",
          borderRadius: "var(--radius-lg)",
          boxShadow: "0 32px 80px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(0,255,136,0.06)",
          overflow: "hidden",
        }}
      >
        {/* Search row */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "10px",
            padding: "12px 16px",
            borderBottom: "1px solid var(--border-1)",
          }}
        >
          <SearchIcon />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search sessions, projects, tools…"
            style={{
              flex: 1,
              background: "transparent",
              border: "none",
              outline: "none",
              color: "var(--text-1)",
              fontSize: "14px",
              fontFamily: "var(--font-sans)",
              caretColor: "var(--accent)",
            }}
          />
          <Kbd>ESC</Kbd>
        </div>

        {/* Results / quick links */}
        <div style={{ padding: "6px 0", maxHeight: "380px", overflowY: "auto" }}>
          {filtered.length > 0 ? (
            <>
              <div
                style={{
                  padding: "4px 16px 6px",
                  fontSize: "10px",
                  fontFamily: "var(--font-mono)",
                  fontWeight: 600,
                  letterSpacing: "0.1em",
                  textTransform: "uppercase",
                  color: "var(--text-3)",
                }}
              >
                {query ? "Results" : "Quick links"}
              </div>
              {filtered.map((link) => (
                <a
                  key={link.href}
                  href={link.href}
                  onClick={onClose}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    padding: "8px 16px",
                    fontSize: "13px",
                    color: "var(--text-1)",
                    cursor: "pointer",
                    transition: "background 0.1s",
                    textDecoration: "none",
                  }}
                  onMouseEnter={(e) => {
                    (e.currentTarget as HTMLElement).style.background = "var(--bg-2)";
                  }}
                  onMouseLeave={(e) => {
                    (e.currentTarget as HTMLElement).style.background = "transparent";
                  }}
                >
                  <span>{link.label}</span>
                  <span
                    style={{
                      fontSize: "10px",
                      fontFamily: "var(--font-mono)",
                      color: "var(--text-3)",
                    }}
                  >
                    {link.category}
                  </span>
                </a>
              ))}
            </>
          ) : (
            <div
              style={{
                padding: "28px 16px",
                textAlign: "center",
                fontSize: "13px",
                color: "var(--text-3)",
              }}
            >
              No results for &ldquo;{query}&rdquo;
            </div>
          )}
        </div>

        {/* Footer hint */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "16px",
            padding: "8px 16px",
            borderTop: "1px solid var(--border-1)",
            fontSize: "11px",
            color: "var(--text-3)",
          }}
        >
          <span style={{ display: "flex", alignItems: "center", gap: "4px" }}>
            <Kbd>↑↓</Kbd> navigate
          </span>
          <span style={{ display: "flex", alignItems: "center", gap: "4px" }}>
            <Kbd>↵</Kbd> open
          </span>
          <span style={{ display: "flex", alignItems: "center", gap: "4px" }}>
            <Kbd>ESC</Kbd> close
          </span>
        </div>
      </div>
    </div>
  );
}

function SearchIcon() {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="var(--text-3)"
      strokeWidth="2.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      style={{ flexShrink: 0 }}
    >
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.35-4.35" />
    </svg>
  );
}

function Kbd({ children }: { children: React.ReactNode }) {
  return (
    <kbd
      style={{
        fontFamily: "var(--font-mono)",
        fontSize: "10px",
        fontWeight: 500,
        color: "var(--text-3)",
        background: "var(--bg-2)",
        border: "1px solid var(--border-2)",
        borderRadius: "var(--radius-sm)",
        padding: "1px 5px",
        lineHeight: 1.6,
        whiteSpace: "nowrap",
      }}
    >
      {children}
    </kbd>
  );
}
