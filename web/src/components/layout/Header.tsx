"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";

interface HeaderProps {
  title: string;
  subtitle?: string;
  actions?: React.ReactNode;
}

export function Header({ title, subtitle, actions }: HeaderProps) {
  const router = useRouter();

  const handleKeydown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      router.push(`/search?q=${encodeURIComponent((e.target as HTMLInputElement).value)}`);
    }
  };

  return (
    <header
      className="sticky top-0 z-10 flex items-center gap-4 px-6 py-4"
      style={{
        background: "rgba(13,13,15,0.85)",
        borderBottom: "1px solid rgba(255,255,255,0.06)",
        backdropFilter: "blur(12px)",
      }}
    >
      <div className="flex-1 min-w-0">
        <h1 className="text-text-primary font-semibold text-lg truncate">{title}</h1>
        {subtitle && <p className="text-text-muted text-sm truncate">{subtitle}</p>}
      </div>

      {/* Quick search */}
      <div className="hidden md:flex items-center gap-2">
        <div className="relative">
          <input
            type="text"
            placeholder="Search sessions…"
            onKeyDown={handleKeydown}
            className="w-48 px-3 py-1.5 text-sm rounded-md mono text-text-muted
                       placeholder:text-text-muted/50 outline-none focus:ring-1 focus:ring-accent-cyan/50
                       transition-all"
            style={{
              background: "rgba(255,255,255,0.05)",
              border: "1px solid rgba(255,255,255,0.08)",
            }}
          />
          <kbd className="absolute right-2 top-1/2 -translate-y-1/2 text-[10px] text-text-muted opacity-50">
            ↵
          </kbd>
        </div>
      </div>

      {actions && <div className="flex items-center gap-2">{actions}</div>}
    </header>
  );
}
