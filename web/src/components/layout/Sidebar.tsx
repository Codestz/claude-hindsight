import { useState, useEffect } from "react";
import { Link, useLocation } from "react-router-dom";
import { Search } from "lucide-react";
import { HindsightLogo } from "@/components/ui/HindsightLogo";
import type { SidebarProps } from "./types";
import { SIDEBAR_W, SIDEBAR_COLLAPSED_W, SIDEBAR_BREAKPOINT, NAV_ITEMS, isNavActive } from "./config";

export function Sidebar({ onSearch }: SidebarProps) {
  const { pathname } = useLocation();
  const [collapsed, setCollapsed] = useState(false);
  const [hoveredTo, setHoveredTo] = useState<string | null>(null);

  useEffect(() => {
    const check = () => setCollapsed(window.innerWidth < SIDEBAR_BREAKPOINT);
    check();
    window.addEventListener("resize", check);
    return () => window.removeEventListener("resize", check);
  }, []);

  const w = collapsed ? SIDEBAR_COLLAPSED_W : SIDEBAR_W;

  let lastSection: string | undefined;

  return (
    <aside
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        bottom: 0,
        width: w,
        background: "var(--bg-1)",
        borderRight: "1px solid var(--border-1)",
        display: "flex",
        flexDirection: "column",
        zIndex: 40,
        transition: "width 0.2s cubic-bezier(0.4, 0, 0.2, 1)",
        overflow: "hidden",
      }}
    >
      {/* ── Brand ──────────────────────────────────────── */}
      <Link
        to="/"
        style={{
          display: "flex",
          alignItems: "center",
          gap: "12px",
          padding: collapsed ? "24px 16px" : "24px 20px",
          textDecoration: "none",
          flexShrink: 0,
        }}
      >
        <HindsightLogo size={collapsed ? 24 : 28} />
        {!collapsed && (
          <div style={{ display: "flex", alignItems: "baseline", gap: "8px" }}>
            <span
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "15px",
                fontWeight: 600,
                letterSpacing: "-0.01em",
                color: "var(--text-1)",
              }}
            >
              Hindsight
            </span>
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "10px",
                fontWeight: 500,
                color: "var(--text-3)",
              }}
            >
              v2.2
            </span>
          </div>
        )}
      </Link>

      {/* ── Search trigger ─────────────────────────────── */}
      <div style={{ padding: collapsed ? "0 10px 12px" : "0 14px 16px", flexShrink: 0 }}>
        <button
          onClick={onSearch}
          style={{
            display: "flex",
            alignItems: "center",
            gap: "10px",
            width: "100%",
            padding: collapsed ? "7px" : "7px 10px",
            borderRadius: "var(--radius-md)",
            border: "1px solid var(--border-2)",
            background: "var(--bg-2)",
            color: "var(--text-3)",
            fontSize: "13px",
            cursor: "pointer",
            transition: "border-color 0.15s, background 0.15s",
            justifyContent: collapsed ? "center" : "flex-start",
            fontFamily: "var(--font-sans)",
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.borderColor = "var(--border-3)";
            e.currentTarget.style.background = "var(--bg-3)";
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.borderColor = "var(--border-2)";
            e.currentTarget.style.background = "var(--bg-2)";
          }}
        >
          <Search size={14} strokeWidth={2} />
          {!collapsed && (
            <>
              <span style={{ flex: 1, textAlign: "left" }}>Search</span>
              <kbd
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "10px",
                  fontWeight: 500,
                  color: "var(--text-3)",
                  background: "var(--bg-0)",
                  border: "1px solid var(--border-2)",
                  borderRadius: "var(--radius-sm)",
                  padding: "0 5px",
                  lineHeight: "18px",
                }}
              >
                ⌘K
              </kbd>
            </>
          )}
        </button>
      </div>

      {/* ── Navigation ─────────────────────────────────── */}
      <nav style={{ flex: 1, padding: "0 10px", display: "flex", flexDirection: "column", gap: "1px" }}>
        {NAV_ITEMS.map((item) => {
          const showSection = item.section && item.section !== lastSection;
          if (item.section) lastSection = item.section;

          const active = isNavActive(item.to, pathname);
          const hovered = hoveredTo === item.to;
          const Icon = item.icon;

          return (
            <div key={item.to}>
              {showSection && !collapsed && (
                <div
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "10px",
                    fontWeight: 500,
                    letterSpacing: "0.08em",
                    textTransform: "uppercase",
                    color: "var(--text-3)",
                    padding: "20px 10px 6px",
                  }}
                >
                  {item.section}
                </div>
              )}
              {showSection && collapsed && <div style={{ height: "16px" }} />}

              <Link
                to={item.to}
                onMouseEnter={() => setHoveredTo(item.to)}
                onMouseLeave={() => setHoveredTo(null)}
                title={collapsed ? item.label : undefined}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "10px",
                  padding: "8px 10px",
                  borderRadius: "var(--radius-md)",
                  textDecoration: "none",
                  fontSize: "13px",
                  fontWeight: active ? 500 : 400,
                  color: active ? "var(--text-1)" : hovered ? "var(--text-2)" : "var(--text-3)",
                  background: active
                    ? "rgba(129, 140, 248, 0.08)"
                    : hovered
                    ? "rgba(255, 255, 255, 0.03)"
                    : "transparent",
                  transition: "color 0.15s, background 0.15s",
                  justifyContent: collapsed ? "center" : "flex-start",
                  position: "relative",
                }}
              >
                {/* Active indicator — indigo bar */}
                {active && (
                  <span
                    aria-hidden="true"
                    style={{
                      position: "absolute",
                      left: 0,
                      top: "50%",
                      transform: "translateY(-50%)",
                      width: "3px",
                      height: "16px",
                      borderRadius: "0 2px 2px 0",
                      background: "var(--indigo)",
                    }}
                  />
                )}
                <Icon
                  size={16}
                  strokeWidth={active ? 2 : 1.7}
                />
                {!collapsed && <span>{item.label}</span>}
              </Link>
            </div>
          );
        })}
      </nav>
    </aside>
  );
}

// Constants re-exported from ./config via ./index.ts
