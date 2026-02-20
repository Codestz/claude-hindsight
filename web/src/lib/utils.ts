// Utility functions for formatting

export function formatTokens(tokens: number): string {
  if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toFixed(1)}M`;
  if (tokens >= 1_000) return `${(tokens / 1_000).toFixed(1)}K`;
  return tokens.toString();
}

export function formatCost(cost: number): string {
  if (cost < 0.001) return "<$0.001";
  if (cost < 1) return `$${cost.toFixed(3)}`;
  return `$${cost.toFixed(2)}`;
}

export function formatBytes(bytes: number): string {
  if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(1)} GB`;
  if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`;
  if (bytes >= 1_024) return `${(bytes / 1_024).toFixed(1)} KB`;
  return `${bytes} B`;
}

export function timeAgo(unixSeconds: number): string {
  const now = Math.floor(Date.now() / 1000);
  const diff = now - unixSeconds;

  if (diff < 60) return "just now";
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;

  const date = new Date(unixSeconds * 1000);
  return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
}

export function shortId(id: string): string {
  return id.slice(0, 8);
}

export function nodeTypeColor(nodeType: string, color?: string): string {
  if (color) {
    const map: Record<string, string> = {
      cyan: "text-accent-cyan",
      green: "text-accent-green",
      yellow: "text-accent-yellow",
      magenta: "text-accent-magenta",
      red: "text-accent-red",
      white: "text-text-primary",
    };
    return map[color] ?? "text-text-muted";
  }
  const typeMap: Record<string, string> = {
    tool_use: "text-accent-cyan",
    tool_result: "text-accent-green",
    user: "text-accent-green",
    assistant: "text-text-primary",
    thinking: "text-accent-magenta",
    error: "text-accent-red",
  };
  return typeMap[nodeType] ?? "text-text-muted";
}
