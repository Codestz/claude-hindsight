// Typed fetch wrappers for the Hindsight REST API

import type {
  GlobalAnalytics,
  NodeResponse,
  ProjectAnalytics,
  ProjectStats,
  SessionFile,
  TreeResponse,
} from "./types";

// In production, the Rust server serves both the static files and the API
// on the same origin. In dev, next.config.ts proxies /api/* to :7227.
const BASE = "/api";

async function get<T>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`, { cache: "no-store" });
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(`API ${path} → ${res.status}: ${text}`);
  }
  return res.json() as Promise<T>;
}

export const api = {
  projects(): Promise<ProjectStats[]> {
    return get("/projects");
  },

  globalAnalytics(): Promise<GlobalAnalytics> {
    return get("/analytics/global");
  },

  globalSparkline(days = 14): Promise<number[]> {
    return get(`/analytics/global/sparkline?days=${days}`);
  },

  projectAnalytics(project: string): Promise<ProjectAnalytics> {
    return get(`/analytics/${encodeURIComponent(project)}`);
  },

  sessions(opts?: { project?: string; limit?: number }): Promise<SessionFile[]> {
    const params = new URLSearchParams();
    if (opts?.project) params.set("project", opts.project);
    if (opts?.limit) params.set("limit", String(opts.limit));
    const qs = params.toString();
    return get(`/sessions${qs ? `?${qs}` : ""}`);
  },

  session(id: string): Promise<SessionFile> {
    return get(`/sessions/${encodeURIComponent(id)}`);
  },

  sessionNodes(id: string): Promise<TreeResponse> {
    return get(`/sessions/${encodeURIComponent(id)}/nodes`);
  },

  search(opts: {
    q?: string;
    project?: string;
    tool?: string;
    errors?: boolean;
  }): Promise<SessionFile[]> {
    const params = new URLSearchParams();
    if (opts.q) params.set("q", opts.q);
    if (opts.project) params.set("project", opts.project);
    if (opts.tool) params.set("tool", opts.tool);
    if (opts.errors) params.set("errors", "true");
    return get(`/search?${params.toString()}`);
  },

  eventsUrl(sessionId: string): string {
    return `${BASE}/events?session_id=${encodeURIComponent(sessionId)}`;
  },
};

export type { NodeResponse };
