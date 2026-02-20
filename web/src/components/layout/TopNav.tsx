"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { CommandPalette } from "./CommandPalette";

function SearchIcon() {
  return (
    <svg
      width="13"
      height="13"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
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

const NAV = [
  { href: "/",        label: "Dashboard" },
  { href: "/projects", label: "Projects" },
];

// A href is active when the current path is at or below it.
// /sessions is treated as a child of /projects (no standalone list page).
function isActive(href: string, pathname: string): boolean {
  if (href === "/") return pathname === "/";
  return (
    pathname.startsWith(href) ||
    (href === "/projects" && pathname.startsWith("/sessions"))
  );
}

export function TopNav() {
  const pathname = usePathname();
  const [cmdOpen, setCmdOpen] = useState(false);
  const [hoveredHref, setHoveredHref] = useState<string | null>(null);

  // ⌘K / Ctrl+K shortcut
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setCmdOpen((o) => !o);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  return (
    <>
      <header
        style={{
          position: "sticky",
          top: 0,
          zIndex: 50,
          height: "56px",
          background: "rgba(5, 5, 5, 0.92)",
          borderBottom: "1px solid var(--border-1)",
          backdropFilter: "blur(12px)",
          WebkitBackdropFilter: "blur(12px)",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            maxWidth: "1400px",
            margin: "0 auto",
            padding: "0 24px",
            height: "100%",
            gap: "4px",
          }}
        >
          {/* ── Brand ──────────────────────────────────────── */}
          <Link
            href="/"
            style={{
              display: "flex",
              alignItems: "center",
              gap: "7px",
              marginRight: "20px",
              flexShrink: 0,
              textDecoration: "none",
            }}
          >
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "13px",
                fontWeight: 700,
                letterSpacing: "0.08em",
                textTransform: "uppercase",
                color: "var(--text-1)",
              }}
            >
              Hindsight
            </span>
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "9px",
                fontWeight: 600,
                letterSpacing: "0.05em",
                color: "var(--accent)",
                background: "rgba(0, 255, 136, 0.08)",
                border: "1px solid rgba(0, 255, 136, 0.18)",
                borderRadius: "var(--radius-sm)",
                padding: "1px 5px",
              }}
            >
              v1.0
            </span>
          </Link>

          {/* ── Nav items ──────────────────────────────────── */}
          <nav
            aria-label="Main navigation"
            style={{ display: "flex", alignItems: "center", gap: "2px" }}
          >
            {NAV.map((item) => {
              const active = isActive(item.href, pathname);
              const hovered = hoveredHref === item.href;

              return (
                <Link
                  key={item.href}
                  href={item.href}
                  onMouseEnter={() => setHoveredHref(item.href)}
                  onMouseLeave={() => setHoveredHref(null)}
                  style={{
                    padding: "4px 12px",
                    borderRadius: "var(--radius-md)",
                    fontSize: "13px",
                    fontWeight: active ? 500 : 400,
                    color: active
                      ? "var(--text-1)"
                      : hovered
                      ? "var(--text-2)"
                      : "var(--text-3)",
                    background: active
                      ? "var(--bg-2)"
                      : hovered
                      ? "var(--bg-1)"
                      : "transparent",
                    borderBottom: active
                      ? "1px solid rgba(0, 255, 136, 0.3)"
                      : "1px solid transparent",
                    transition: "color 0.12s, background 0.12s",
                    textDecoration: "none",
                    lineHeight: "22px",
                  }}
                >
                  {item.label}
                </Link>
              );
            })}
          </nav>

          {/* ── Search trigger (⌘K) ────────────────────────── */}
          <button
            onClick={() => setCmdOpen(true)}
            aria-label="Open search (⌘K)"
            style={{
              marginLeft: "auto",
              display: "flex",
              alignItems: "center",
              gap: "8px",
              padding: "5px 10px 5px 8px",
              background: "var(--bg-1)",
              border: "1px solid var(--border-2)",
              borderRadius: "var(--radius-md)",
              cursor: "pointer",
              color: "var(--text-3)",
              fontSize: "12px",
              fontFamily: "var(--font-sans)",
              transition: "border-color 0.12s, color 0.12s",
              minWidth: "180px",
            }}
            onMouseEnter={(e) => {
              const el = e.currentTarget as HTMLButtonElement;
              el.style.borderColor = "var(--border-3)";
              el.style.color = "var(--text-2)";
            }}
            onMouseLeave={(e) => {
              const el = e.currentTarget as HTMLButtonElement;
              el.style.borderColor = "var(--border-2)";
              el.style.color = "var(--text-3)";
            }}
          >
            <SearchIcon />
            <span style={{ flex: 1, textAlign: "left" }}>Search…</span>
            <span
              style={{
                display: "flex",
                alignItems: "center",
                gap: "2px",
                flexShrink: 0,
              }}
            >
              <kbd
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "10px",
                  color: "var(--text-3)",
                  background: "var(--bg-2)",
                  border: "1px solid var(--border-2)",
                  borderRadius: "var(--radius-sm)",
                  padding: "0 4px",
                  lineHeight: "16px",
                }}
              >
                ⌘
              </kbd>
              <kbd
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "10px",
                  color: "var(--text-3)",
                  background: "var(--bg-2)",
                  border: "1px solid var(--border-2)",
                  borderRadius: "var(--radius-sm)",
                  padding: "0 4px",
                  lineHeight: "16px",
                }}
              >
                K
              </kbd>
            </span>
          </button>
        </div>
      </header>

      <CommandPalette open={cmdOpen} onClose={() => setCmdOpen(false)} />
    </>
  );
}
