"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { LayoutDashboard, FolderOpen, Search } from "lucide-react";

const nav = [
  { href: "/", label: "Dashboard", icon: LayoutDashboard },
  { href: "/projects", label: "Projects", icon: FolderOpen },
  { href: "/search", label: "Search", icon: Search },
];

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside
      className="fixed left-0 top-0 h-screen flex flex-col"
      style={{
        width: "220px",
        background: "var(--bg-2)",
        borderRight: "1px solid var(--border)",
      }}
    >
      {/* Brand */}
      <div
        className="flex items-center gap-2.5 px-4 flex-shrink-0"
        style={{ height: "48px", borderBottom: "1px solid var(--border)" }}
      >
        <span
          style={{
            fontWeight: 800,
            fontSize: "0.8125rem",
            letterSpacing: "0.15em",
            color: "var(--text-1)",
            textTransform: "uppercase",
          }}
        >
          HINDSIGHT
        </span>
        <span className="tbadge tbadge-ok" style={{ fontSize: "7px" }}>
          v1.0.0
        </span>
      </div>

      {/* Nav */}
      <nav className="flex-1 overflow-y-auto py-3" aria-label="Main navigation">
        <ul className="list-none px-2 m-0 space-y-px">
          {nav.map((item) => {
            const active = pathname === item.href;
            const Icon = item.icon;
            return (
              <li key={item.href}>
                <Link
                  href={item.href}
                  className="flex items-center gap-2.5 px-3 py-2 text-sm transition-colors duration-100 outline-none"
                  style={{
                    color: active ? "var(--text-1)" : "var(--text-2)",
                    background: active ? "rgba(0,255,136,0.07)" : "transparent",
                    borderLeft: active ? "2px solid var(--accent)" : "2px solid transparent",
                    fontWeight: 500,
                    fontSize: "13px",
                  }}
                  onMouseEnter={(e) => {
                    if (!active) (e.currentTarget as HTMLElement).style.color = "var(--text-1)";
                  }}
                  onMouseLeave={(e) => {
                    if (!active) (e.currentTarget as HTMLElement).style.color = "var(--text-2)";
                  }}
                >
                  <Icon
                    size={13}
                    aria-hidden="true"
                    style={{ color: active ? "var(--accent)" : "var(--text-3)" }}
                  />
                  <span>{item.label}</span>
                </Link>
              </li>
            );
          })}
        </ul>
      </nav>
    </aside>
  );
}
