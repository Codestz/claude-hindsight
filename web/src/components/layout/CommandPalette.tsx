"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { SearchIcon } from "@/components/icons/SearchIcon";

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
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const router = useRouter();

  const filtered = QUICK_LINKS.filter((l) =>
    l.label.toLowerCase().includes(query.toLowerCase())
  );

  // Focus input and reset selection when opened; clear query on close
  useEffect(() => {
    if (open) {
      inputRef.current?.focus();
      setSelectedIndex(0);
    } else {
      setQuery("");
    }
  }, [open]);

  // Reset selection index when filtered results change
  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  // Keyboard navigation (↑↓ Enter) + Escape to close
  useEffect(() => {
    if (!open) return;

    const handler = (e: KeyboardEvent) => {
      switch (e.key) {
        case "Escape":
          onClose();
          break;
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((i) =>
            filtered.length === 0 ? 0 : (i + 1) % filtered.length
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((i) =>
            filtered.length === 0 ? 0 : i <= 0 ? filtered.length - 1 : i - 1
          );
          break;
        case "Enter":
          if (filtered[selectedIndex]) {
            router.push(filtered[selectedIndex].href);
            onClose();
          }
          break;
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose, filtered, selectedIndex, router]);

  if (!open) return null;

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
          <SearchIcon size={14} stroke="var(--text-3)" />
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
              {filtered.map((link, i) => (
                <Link
                  key={link.href}
                  href={link.href}
                  onClick={onClose}
                  onMouseEnter={() => setSelectedIndex(i)}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    padding: "8px 16px",
                    fontSize: "13px",
                    color: "var(--text-1)",
                    cursor: "pointer",
                    background: i === selectedIndex ? "var(--bg-2)" : "transparent",
                    transition: "background 0.1s",
                    textDecoration: "none",
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
                </Link>
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
