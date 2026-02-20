// Typed fetch wrappers for the Hindsight REST API.
// In dev: next.config.ts proxies /api/* → :7227
// In prod: Rust server serves both static files and the API on the same origin.

import type {
  GlobalAnalytics,
  HealthCheck,
  NodeResponse,
  ProjectAnalytics,
  ProjectStats,
  SessionFile,
  Sparkline,
  TreeResponse,
} from "./types";

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

  globalSparkline(days = 14): Promise<Sparkline> {
    return get(`/analytics/global/sparkline?days=${days}`);
  },

  health(): Promise<HealthCheck> {
    return get("/health");
  },

  projectAnalytics(project: string): Promise<ProjectAnalytics> {
    return get(`/analytics/${encodeURIComponent(project)}`);
  },

  sessions(opts?: { project?: string; limit?: number }): Promise<SessionFile[]> {
    const p = new URLSearchParams();
    if (opts?.project) p.set("project", opts.project);
    if (opts?.limit)   p.set("limit", String(opts.limit));
    const qs = p.toString();
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
    const p = new URLSearchParams();
    if (opts.q)       p.set("q", opts.q);
    if (opts.project) p.set("project", opts.project);
    if (opts.tool)    p.set("tool", opts.tool);
    if (opts.errors)  p.set("errors", "true");
    return get(`/search?${p.toString()}`);
  },

  globalTopFiles(): Promise<[string, number][]> {
    return get("/analytics/global/files");
  },

  projectTopFiles(project: string): Promise<[string, number][]> {
    return get(`/analytics/${encodeURIComponent(project)}/files`);
  },

  eventsUrl(sessionId: string): string {
    return `${BASE}/events?session_id=${encodeURIComponent(sessionId)}`;
  },
};

export type { NodeResponse, Sparkline };
