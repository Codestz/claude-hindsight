// Formatting helpers — pure functions, no side effects.

export function formatBytes(n: number): string {
  if (n >= 1_073_741_824) return `${(n / 1_073_741_824).toFixed(1)} GB`;
  if (n >= 1_048_576)     return `${(n / 1_048_576).toFixed(1)} MB`;
  if (n >= 1_024)         return `${(n / 1_024).toFixed(1)} KB`;
  return `${n} B`;
}

// Unix seconds → precise relative time (includes seconds)
export function relativeTime(unixSeconds: number): string {
  const diff = Math.floor(Date.now() / 1000) - unixSeconds;
  if (diff < 60)     return `${diff}s ago`;
  if (diff < 3_600)  return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86_400) return `${Math.floor(diff / 3_600)}h ago`;
  return `${Math.floor(diff / 86_400)}d ago`;
}

// Unix seconds → human-readable relative time (coarser, for display)
export function timeAgo(unixSeconds: number): string {
  const diff = Math.floor(Date.now() / 1000) - unixSeconds;
  if (diff < 60)     return "just now";
  if (diff < 3_600)  return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86_400) return `${Math.floor(diff / 3_600)}h ago`;
  if (diff < 604_800)return `${Math.floor(diff / 86_400)}d ago`;
  return new Date(unixSeconds * 1000).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
  });
}

// First 8 chars of a session ID for display
export function shortId(id: string): string {
  return id.slice(0, 8);
}

// "claude-sonnet-4-6-20241022" → "sonnet-4-6"
export function shortModel(model: string | null): string {
  if (!model) return "—";
  return model.replace(/^claude-/, "").replace(/-\d{8}$/, "");
}

// 1234 → "1.2K", 1234567 → "1.2M"
export function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000)     return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

// 0.05 → "$0.05", 1.234 → "$1.23"
export function formatCost(usd: number): string {
  return `$${usd.toFixed(2)}`;
}

// Group mcp__<server>__<tool> entries by server name
export function extractMcpServers(topTools: [string, number][]): [string, number][] {
  const servers: Record<string, number> = {};
  for (const [name, count] of topTools) {
    if (name.startsWith("mcp__")) {
      const parts = name.split("__");
      const server = parts[1] ?? name;
      servers[server] = (servers[server] ?? 0) + count;
    }
  }
  return Object.entries(servers).sort((a, b) => b[1] - a[1]) as [string, number][];
}

// Milliseconds timestamp → HH:MM:SS
export function formatTimestamp(ms: number): string {
  return new Date(ms).toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

// Time-of-day greeting
export function greeting(): string {
  const h = new Date().getHours();
  if (h < 12) return "Good morning";
  if (h < 17) return "Good afternoon";
  return "Good evening";
}

// Milliseconds → human-readable duration (e.g. "4m 12s")
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const s = ms / 1000;
  if (s < 60) return `${s.toFixed(0)}s`;
  const m = Math.floor(s / 60);
  const rem = Math.round(s % 60);
  return `${m}m ${rem}s`;
}

// Show last 2 path segments: "/a/b/c/d.ts" → ".../c/d.ts"
export function shortPath(path: string): string {
  const parts = path.replace(/\\/g, "/").split("/").filter(Boolean);
  if (parts.length <= 2) return parts.join("/");
  return `.../${parts.slice(-2).join("/")}`;
}
