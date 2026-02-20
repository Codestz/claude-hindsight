"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const nav = [
  { href: "/", label: "Dashboard", icon: "◈" },
  { href: "/projects", label: "Projects", icon: "◉" },
  { href: "/search", label: "Search", icon: "◎" },
];

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside
      className="fixed left-0 top-0 h-screen w-52 flex flex-col"
      style={{
        background: "rgba(13,13,15,0.95)",
        borderRight: "1px solid rgba(255,255,255,0.06)",
        backdropFilter: "blur(12px)",
      }}
    >
      {/* Logo */}
      <div className="px-5 py-5 border-b" style={{ borderColor: "rgba(255,255,255,0.06)" }}>
        <div className="flex items-center gap-2">
          <span className="text-accent-cyan text-lg font-bold mono">HS</span>
          <div>
            <div className="text-text-primary text-sm font-semibold">Hindsight</div>
            <div className="text-text-muted text-xs">Session Observer</div>
          </div>
        </div>
      </div>

      {/* Nav */}
      <nav className="flex-1 px-3 py-4 space-y-1">
        {nav.map((item) => {
          const active = pathname === item.href;
          return (
            <Link
              key={item.href}
              href={item.href}
              className={`flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-all duration-150 ${
                active
                  ? "bg-accent-cyan/10 text-accent-cyan"
                  : "text-text-muted hover:text-text-primary hover:bg-white/5"
              }`}
            >
              <span className="text-base">{item.icon}</span>
              <span>{item.label}</span>
            </Link>
          );
        })}
      </nav>

      {/* Footer */}
      <div className="px-5 py-4 border-t" style={{ borderColor: "rgba(255,255,255,0.06)" }}>
        <div className="text-text-muted text-xs mono">v0.1.0</div>
      </div>
    </aside>
  );
}
