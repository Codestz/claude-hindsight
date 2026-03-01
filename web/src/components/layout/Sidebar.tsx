import { useState, useEffect } from "react";
import { Link, useLocation } from "react-router-dom";
import {
  LayoutDashboard,
  Activity,
  FolderOpen,
  MessageSquare,
  Bot,
  Sparkles,
  Search,
} from "lucide-react";

const SIDEBAR_W = 220;
const SIDEBAR_COLLAPSED_W = 52;
const BREAKPOINT = 1024;

interface NavItem {
  to: string;
  label: string;
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  section?: string;
}

const NAV_ITEMS: NavItem[] = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard },
  { to: "/activity", label: "Activity", icon: Activity },
  { to: "/projects", label: "Projects", icon: FolderOpen, section: "Analyze" },
  { to: "/prompts", label: "Prompts", icon: MessageSquare },
  { to: "/agents", label: "Agents", icon: Bot, section: "Configure" },
  { to: "/skills", label: "Skills", icon: Sparkles },
];

function isActive(to: string, pathname: string): boolean {
  if (to === "/") return pathname === "/";
  return pathname.startsWith(to) || (to === "/projects" && pathname.startsWith("/sessions"));
}

export function Sidebar({ onSearch }: { onSearch: () => void }) {
  const { pathname } = useLocation();
  const [collapsed, setCollapsed] = useState(false);
  const [hoveredTo, setHoveredTo] = useState<string | null>(null);

  useEffect(() => {
    const check = () => setCollapsed(window.innerWidth < BREAKPOINT);
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
        transition: "width 0.2s ease",
        overflow: "hidden",
      }}
    >
      {/* Brand */}
      <Link
        to="/"
        style={{
          display: "flex",
          alignItems: "center",
          gap: "10px",
          padding: collapsed ? "20px 14px" : "20px 18px",
          textDecoration: "none",
          flexShrink: 0,
        }}
      >
        <span
          style={{
            width: "24px",
            height: "24px",
            borderRadius: "6px",
            background: "linear-gradient(135deg, var(--green), rgba(0, 255, 136, 0.4))",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: "12px",
            fontWeight: 700,
            color: "#000",
            flexShrink: 0,
          }}
        >
          H
        </span>
        {!collapsed && (
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "13px",
                fontWeight: 700,
                letterSpacing: "0.06em",
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
                color: "var(--accent)",
                background: "rgba(0, 255, 136, 0.08)",
                border: "1px solid rgba(0, 255, 136, 0.18)",
                borderRadius: "var(--radius-sm)",
                padding: "1px 5px",
              }}
            >
              v2.0
            </span>
          </div>
        )}
      </Link>

      {/* Nav */}
      <nav style={{ flex: 1, padding: "4px 8px", display: "flex", flexDirection: "column", gap: "2px" }}>
        {NAV_ITEMS.map((item) => {
          const showSection = item.section && item.section !== lastSection;
          if (item.section) lastSection = item.section;

          const active = isActive(item.to, pathname);
          const hovered = hoveredTo === item.to;
          const Icon = item.icon;

          return (
            <div key={item.to}>
              {showSection && !collapsed && (
                <div
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "10px",
                    fontWeight: 600,
                    letterSpacing: "0.1em",
                    textTransform: "uppercase",
                    color: "var(--text-3)",
                    padding: "16px 10px 6px",
                  }}
                >
                  {item.section}
                </div>
              )}
              {showSection && collapsed && <div style={{ height: "12px" }} />}

              <Link
                to={item.to}
                onMouseEnter={() => setHoveredTo(item.to)}
                onMouseLeave={() => setHoveredTo(null)}
                title={collapsed ? item.label : undefined}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "10px",
                  padding: collapsed ? "8px 10px" : "8px 10px",
                  borderRadius: "var(--radius-md)",
                  textDecoration: "none",
                  fontSize: "13px",
                  fontWeight: active ? 500 : 400,
                  color: active ? "var(--text-1)" : hovered ? "var(--text-2)" : "var(--text-3)",
                  background: active ? "var(--bg-2)" : hovered ? "rgba(255,255,255,0.03)" : "transparent",
                  transition: "color 0.12s, background 0.12s",
                  justifyContent: collapsed ? "center" : "flex-start",
                }}
              >
                <Icon
                  size={16}
                  strokeWidth={active ? 2.2 : 1.8}
                />
                {!collapsed && <span>{item.label}</span>}
              </Link>
            </div>
          );
        })}
      </nav>

      {/* Search button */}
      <div style={{ padding: "8px", flexShrink: 0 }}>
        <button
          onClick={onSearch}
          style={{
            display: "flex",
            alignItems: "center",
            gap: "10px",
            width: "100%",
            padding: collapsed ? "8px 10px" : "8px 10px",
            borderRadius: "var(--radius-md)",
            border: "1px solid var(--border-2)",
            background: "transparent",
            color: "var(--text-3)",
            fontSize: "13px",
            cursor: "pointer",
            transition: "border-color 0.12s, color 0.12s",
            justifyContent: collapsed ? "center" : "flex-start",
            fontFamily: "var(--font-sans)",
          }}
          onMouseEnter={(e) => {
            (e.currentTarget as HTMLElement).style.borderColor = "var(--border-3)";
            (e.currentTarget as HTMLElement).style.color = "var(--text-2)";
          }}
          onMouseLeave={(e) => {
            (e.currentTarget as HTMLElement).style.borderColor = "var(--border-2)";
            (e.currentTarget as HTMLElement).style.color = "var(--text-3)";
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
                  color: "var(--text-3)",
                  background: "var(--bg-2)",
                  border: "1px solid var(--border-2)",
                  borderRadius: "var(--radius-sm)",
                  padding: "0 4px",
                  lineHeight: "16px",
                }}
              >
                ⌘K
              </kbd>
            </>
          )}
        </button>
      </div>
    </aside>
  );
}

export { SIDEBAR_W, SIDEBAR_COLLAPSED_W, BREAKPOINT };
