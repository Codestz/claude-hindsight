// Typed fetch wrappers for the Hindsight REST API.
// In dev: vite.config.ts proxies /api/* → :7227
// In prod: Rust server serves both static files and the API on the same origin.

import type {
  AgentConfig,
  GlobalAnalytics,
  HookActivitySummary,
  HookLifecycleEvent,
  HookPermissionEvent,
  HookSubagentEvent,
  HookToolEvent,
  NodeResponse,
  OtelGlobalSummary,
  OtelLogDto,
  OtelSessionSummary,
  PromptEntry,
  ProjectAnalytics,
  ProjectStats,
  SessionFile,
  SessionTelemetry,
  SkillConfig,
  Sparkline,
  TelemetrySummary,
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

  prompts(opts?: { project?: string; limit?: number }): Promise<PromptEntry[]> {
    const p = new URLSearchParams();
    if (opts?.project) p.set("project", opts.project);
    if (opts?.limit)   p.set("limit", String(opts.limit));
    const qs = p.toString();
    return get(`/prompts${qs ? `?${qs}` : ""}`);
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

  telemetrySummary(): Promise<TelemetrySummary> {
    return get("/telemetry/summary");
  },

  telemetrySessions(): Promise<SessionTelemetry[]> {
    return get("/telemetry/sessions");
  },

  otelSessionSummary(sessionId: string): Promise<OtelSessionSummary> {
    return get(`/otel/session-summary?session_id=${encodeURIComponent(sessionId)}`);
  },

  otelGlobalSummary(): Promise<OtelGlobalSummary> {
    return get("/otel/global-summary");
  },

  otelLogs(opts?: { session_id?: string; event?: string }): Promise<OtelLogDto[]> {
    const p = new URLSearchParams();
    if (opts?.session_id) p.set("session_id", opts.session_id);
    if (opts?.event)      p.set("event", opts.event);
    const qs = p.toString();
    return get(`/otel/logs${qs ? `?${qs}` : ""}`);
  },

  // ── Hook events ──────────────────────────────────────

  hookToolEvents(opts?: { session_id?: string; event?: string; limit?: number }): Promise<HookToolEvent[]> {
    const p = new URLSearchParams();
    if (opts?.session_id) p.set("session_id", opts.session_id);
    if (opts?.event)      p.set("event", opts.event);
    if (opts?.limit)      p.set("limit", String(opts.limit));
    const qs = p.toString();
    return get(`/hooks/tool-events${qs ? `?${qs}` : ""}`);
  },

  hookToolFailures(opts?: { session_id?: string; limit?: number }): Promise<HookToolEvent[]> {
    const p = new URLSearchParams();
    if (opts?.session_id) p.set("session_id", opts.session_id);
    if (opts?.limit)      p.set("limit", String(opts.limit));
    const qs = p.toString();
    return get(`/hooks/tool-failures${qs ? `?${qs}` : ""}`);
  },

  hookSubagentEvents(opts?: { session_id?: string; limit?: number }): Promise<HookSubagentEvent[]> {
    const p = new URLSearchParams();
    if (opts?.session_id) p.set("session_id", opts.session_id);
    if (opts?.limit)      p.set("limit", String(opts.limit));
    const qs = p.toString();
    return get(`/hooks/subagent-events${qs ? `?${qs}` : ""}`);
  },

  hookPermissionEvents(opts?: { session_id?: string; limit?: number }): Promise<HookPermissionEvent[]> {
    const p = new URLSearchParams();
    if (opts?.session_id) p.set("session_id", opts.session_id);
    if (opts?.limit)      p.set("limit", String(opts.limit));
    const qs = p.toString();
    return get(`/hooks/permission-events${qs ? `?${qs}` : ""}`);
  },

  hookLifecycleEvents(opts?: { session_id?: string; event?: string; limit?: number }): Promise<HookLifecycleEvent[]> {
    const p = new URLSearchParams();
    if (opts?.session_id) p.set("session_id", opts.session_id);
    if (opts?.event)      p.set("event", opts.event);
    if (opts?.limit)      p.set("limit", String(opts.limit));
    const qs = p.toString();
    return get(`/hooks/lifecycle-events${qs ? `?${qs}` : ""}`);
  },

  hookActivitySummary(): Promise<HookActivitySummary> {
    return get("/hooks/activity-summary");
  },

  // ── Agents & Skills ──────────────────────────────────

  agents(): Promise<AgentConfig[]> {
    return get("/agents");
  },

  agent(name: string): Promise<AgentConfig> {
    return get(`/agents/${encodeURIComponent(name)}`);
  },

  skills(): Promise<SkillConfig[]> {
    return get("/skills");
  },

  skill(name: string): Promise<SkillConfig> {
    return get(`/skills/${encodeURIComponent(name)}`);
  },
};

export type { NodeResponse, Sparkline };
