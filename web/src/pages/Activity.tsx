import React, { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { api } from "@/lib/api";
import type {
  HookActivitySummary,
  HookToolEvent,
  HookSubagentEvent,
  HookLifecycleEvent,
  HookPermissionEvent,
} from "@/lib/types";
import { shortId } from "@/lib/utils";
import { PageShell } from "@/components/ui/PageShell";
import { Card } from "@/components/ui/Card";
import { LoadingState } from "@/components/ui/LoadingState";
import { ErrorState } from "@/components/ui/ErrorState";
import { EmptyState } from "@/components/ui/EmptyState";
import type { UnifiedEvent } from "@/components/activity/EventRow";
import {
  Wrench,
  Bot,
  Shield,
  AlertTriangle,
  Zap,
  ChevronDown,
  ChevronRight,
} from "lucide-react";

// ── Merge all event types into a sorted UnifiedEvent list ──────

function toolToUnified(e: HookToolEvent): UnifiedEvent {
  const isFailure = !!(e.error_message);
  return {
    kind: isFailure ? "tool_failure" : "tool",
    id: e.id,
    session_id: e.session_id,
    occurred_at: e.occurred_at,
    label: `${e.hook_event}${e.tool_name ? ` — ${e.tool_name}` : ""}`,
    detail: e.tool_input ?? e.tool_result,
    error: e.error_message,
  };
}

function subagentToUnified(e: HookSubagentEvent): UnifiedEvent {
  return {
    kind: "subagent",
    id: e.id,
    session_id: e.session_id,
    occurred_at: e.occurred_at,
    label: `${e.hook_event}${e.agent_type ? ` — ${e.agent_type}` : ""}${e.agent_name ? ` (${e.agent_name})` : ""}`,
  };
}

function lifecycleToUnified(e: HookLifecycleEvent): UnifiedEvent {
  return {
    kind: "lifecycle",
    id: e.id + 1_000_000,
    session_id: e.session_id,
    occurred_at: e.occurred_at,
    label: e.event_name,
    detail: e.attributes,
  };
}

function permissionToUnified(e: HookPermissionEvent): UnifiedEvent {
  return {
    kind: "permission",
    id: e.id + 2_000_000,
    session_id: e.session_id,
    occurred_at: e.occurred_at,
    label: `Permission request${e.tool_name ? ` — ${e.tool_name}` : ""}`,
    detail: e.tool_input,
  };
}

// ── Filter types ──────────────────────────────────────────────
const FILTERS = ["All", "Tools", "Agents", "Lifecycle", "Errors"] as const;
type Filter = (typeof FILTERS)[number];

function matchesFilter(event: UnifiedEvent, filter: Filter): boolean {
  switch (filter) {
    case "All":       return true;
    case "Tools":     return event.kind === "tool";
    case "Agents":    return event.kind === "subagent";
    case "Lifecycle": return event.kind === "lifecycle" || event.kind === "permission";
    case "Errors":    return event.kind === "tool_failure";
  }
}

// ── Kind config ───────────────────────────────────────────────
const KIND_CONFIG: Record<UnifiedEvent["kind"], { icon: React.ComponentType<{ size?: number; strokeWidth?: number }>; color: string }> = {
  tool:         { icon: Wrench,        color: "var(--amber)" },
  tool_failure: { icon: AlertTriangle, color: "var(--rose)" },
  subagent:     { icon: Bot,           color: "var(--violet)" },
  permission:   { icon: Shield,        color: "var(--sky)" },
  lifecycle:    { icon: Zap,           color: "var(--emerald)" },
};

function relativeTime(unixSec: number): string {
  const diff = Math.floor(Date.now() / 1000) - unixSec;
  if (diff < 60)     return `${diff}s ago`;
  if (diff < 3_600)  return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86_400) return `${Math.floor(diff / 3_600)}h ago`;
  return `${Math.floor(diff / 86_400)}d ago`;
}

// ── Inline event row ──────────────────────────────────────────
function ActivityRow({ event }: { event: UnifiedEvent }) {
  const [expanded, setExpanded] = useState(false);
  const cfg = KIND_CONFIG[event.kind];
  const Icon = cfg.icon;
  const hasDetail = !!(event.detail || event.error);

  return (
    <div style={{ borderBottom: "1px solid var(--border-1)" }}>
      <div
        onClick={() => hasDetail && setExpanded(!expanded)}
        style={{
          display: "flex",
          alignItems: "center",
          gap: "10px",
          padding: "8px 16px",
          cursor: hasDetail ? "pointer" : "default",
          transition: "background 0.1s",
        }}
        onMouseEnter={(e) => {
          if (hasDetail) (e.currentTarget as HTMLElement).style.background = "var(--bg-2)";
        }}
        onMouseLeave={(e) => {
          (e.currentTarget as HTMLElement).style.background = "transparent";
        }}
      >
        {/* Time */}
        <span style={{
          fontFamily: "var(--font-mono)", fontSize: "10px",
          color: "var(--text-3)", width: "50px", flexShrink: 0, textAlign: "right",
        }}>
          {relativeTime(event.occurred_at)}
        </span>

        {/* Dot */}
        <span style={{
          width: "6px", height: "6px", borderRadius: "50%",
          background: cfg.color, flexShrink: 0, opacity: 0.8,
        }} />

        {/* Icon */}
        <span style={{ color: cfg.color, flexShrink: 0, display: "flex", opacity: 0.7 }}>
          <Icon size={13} strokeWidth={2} />
        </span>

        {/* Label */}
        <span style={{
          fontSize: "12px",
          color: event.kind === "tool_failure" ? "var(--rose)" : "var(--text-1)",
          fontFamily: "var(--font-sans)", flex: 1,
          overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
        }}>
          {event.label}
        </span>

        {/* Session link */}
        <Link
          to={`/sessions/${encodeURIComponent(event.session_id)}`}
          onClick={(e) => e.stopPropagation()}
          style={{
            fontFamily: "var(--font-mono)", fontSize: "10px",
            color: "var(--text-3)", flexShrink: 0, textDecoration: "none",
          }}
          onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--indigo)"; }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-3)"; }}
        >
          {shortId(event.session_id)}
        </Link>

        {/* Expand */}
        {hasDetail && (
          <span style={{ color: "var(--text-3)", display: "flex", flexShrink: 0 }}>
            {expanded ? <ChevronDown size={11} /> : <ChevronRight size={11} />}
          </span>
        )}
      </div>

      {expanded && hasDetail && (
        <div style={{
          padding: "0 16px 10px 86px", fontSize: "11px",
          fontFamily: "var(--font-mono)", lineHeight: 1.5,
          whiteSpace: "pre-wrap", wordBreak: "break-all",
          color: event.error ? "var(--rose)" : "var(--text-3)",
          maxHeight: "180px", overflowY: "auto",
        }}>
          {event.error || event.detail}
        </div>
      )}
    </div>
  );
}

// ── Main page ──────────────────────────────────────────────────

export default function ActivityPage() {
  const [summary, setSummary] = useState<HookActivitySummary | null>(null);
  const [events, setEvents] = useState<UnifiedEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<Filter>("All");

  useEffect(() => {
    const limit = 200;

    Promise.all([
      api.hookActivitySummary(),
      api.hookToolEvents({ limit }),
      api.hookSubagentEvents({ limit }),
      api.hookLifecycleEvents({ limit }),
      api.hookPermissionEvents({ limit }),
    ])
      .then(([sum, tools, subagents, lifecycle, permissions]) => {
        setSummary(sum);

        const unified: UnifiedEvent[] = [
          ...tools.map(toolToUnified),
          ...subagents.map(subagentToUnified),
          ...lifecycle.map(lifecycleToUnified),
          ...permissions.map(permissionToUnified),
        ].sort((a, b) => b.occurred_at - a.occurred_at);

        setEvents(unified);
      })
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <PageShell><LoadingState /></PageShell>;
  if (error) return <PageShell><ErrorState message={error} /></PageShell>;

  const filtered = events.filter((e) => matchesFilter(e, filter));

  // Compute event kind distribution for the mini chart
  const kindCounts = {
    tools: events.filter((e) => e.kind === "tool").length,
    errors: events.filter((e) => e.kind === "tool_failure").length,
    agents: events.filter((e) => e.kind === "subagent").length,
    lifecycle: events.filter((e) => e.kind === "lifecycle" || e.kind === "permission").length,
  };
  const totalKinds = Math.max(kindCounts.tools + kindCounts.errors + kindCounts.agents + kindCounts.lifecycle, 1);

  const errorRate = summary && summary.total_tool_events > 0
    ? ((summary.recent_errors / summary.total_tool_events) * 100).toFixed(1)
    : "0";

  return (
    <PageShell>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div style={{ display: "flex", alignItems: "baseline", gap: "12px" }}>
          <h1 style={{
            fontSize: "20px", fontWeight: 600, color: "var(--text-1)",
            fontFamily: "var(--font-sans)", margin: 0,
          }}>
            Activity
          </h1>
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)",
          }}>
            {events.length.toLocaleString()} events
          </span>
        </div>
        {summary && summary.recent_errors > 0 && (
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "11px",
            color: "var(--rose)", background: "color-mix(in srgb, var(--rose) 8%, transparent)",
            border: "1px solid color-mix(in srgb, var(--rose) 15%, transparent)",
            borderRadius: "12px", padding: "3px 10px",
          }}>
            {errorRate}% error rate
          </span>
        )}
      </div>

      {/* Stats + Distribution row */}
      {summary && (
        <div className="animate-in" style={{ "--delay": "0s" } as React.CSSProperties}>
        <div style={{
          display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr 2fr",
          gap: "12px", alignItems: "stretch",
        }}>
          <StatTile icon={Wrench} color="var(--amber)" label="Tools" value={summary.total_tool_events} />
          <StatTile icon={Bot} color="var(--violet)" label="Agents" value={summary.total_subagent_events} />
          <StatTile icon={Shield} color="var(--sky)" label="Perms" value={summary.total_permission_events} />
          <StatTile icon={AlertTriangle} color="var(--rose)" label="Errors" value={summary.recent_errors} />

          {/* Distribution bar */}
          <div style={{
            background: "var(--bg-1)", border: "1px solid var(--border-1)",
            borderRadius: "var(--radius-lg)", padding: "14px 16px",
            display: "flex", flexDirection: "column", justifyContent: "center", gap: "8px",
          }}>
            <div style={{
              fontFamily: "var(--font-mono)", fontSize: "9px", fontWeight: 600,
              letterSpacing: "0.08em", textTransform: "uppercase", color: "var(--text-3)",
            }}>
              Event distribution
            </div>
            <div style={{ display: "flex", height: "8px", borderRadius: "4px", overflow: "hidden" }}>
              {kindCounts.tools > 0 && (
                <div style={{ width: `${(kindCounts.tools / totalKinds) * 100}%`, background: "var(--amber)", transition: "width 0.3s" }} />
              )}
              {kindCounts.agents > 0 && (
                <div style={{ width: `${(kindCounts.agents / totalKinds) * 100}%`, background: "var(--violet)", transition: "width 0.3s" }} />
              )}
              {kindCounts.lifecycle > 0 && (
                <div style={{ width: `${(kindCounts.lifecycle / totalKinds) * 100}%`, background: "var(--sky)", transition: "width 0.3s" }} />
              )}
              {kindCounts.errors > 0 && (
                <div style={{ width: `${(kindCounts.errors / totalKinds) * 100}%`, background: "var(--rose)", transition: "width 0.3s" }} />
              )}
            </div>
            <div style={{ display: "flex", gap: "12px", flexWrap: "wrap" }}>
              {[
                { label: "Tools", color: "var(--amber)", value: kindCounts.tools },
                { label: "Agents", color: "var(--violet)", value: kindCounts.agents },
                { label: "Lifecycle", color: "var(--sky)", value: kindCounts.lifecycle },
                { label: "Errors", color: "var(--rose)", value: kindCounts.errors },
              ].filter(s => s.value > 0).map(s => (
                <span key={s.label} style={{ display: "flex", alignItems: "center", gap: "4px", fontSize: "10px", color: "var(--text-3)" }}>
                  <span style={{ width: "6px", height: "6px", borderRadius: "50%", background: s.color }} />
                  {s.label}
                </span>
              ))}
            </div>
          </div>
        </div>
        </div>
      )}

      {/* Main content */}
      <div className="animate-in" style={{ "--delay": "0.06s", display: "grid", gridTemplateColumns: "1fr 320px", gap: "16px", alignItems: "start" } as React.CSSProperties}>
        {/* Event timeline */}
        <Card>
          {/* Filter tabs */}
          <div style={{
            display: "flex", gap: "2px", padding: "10px 16px",
            borderBottom: "1px solid var(--border-1)",
            background: "var(--bg-1)",
          }}>
            {FILTERS.map((f) => {
              const active = filter === f;
              const count = f === "All" ? events.length : events.filter((e) => matchesFilter(e, f)).length;
              return (
                <button
                  key={f}
                  onClick={() => setFilter(f)}
                  style={{
                    fontFamily: "var(--font-mono)", fontSize: "10px", fontWeight: 600,
                    padding: "3px 10px", borderRadius: "10px", border: "none",
                    background: active ? "rgba(129, 140, 248, 0.10)" : "transparent",
                    color: active ? "var(--indigo)" : "var(--text-3)",
                    cursor: "pointer", transition: "all 0.1s",
                    display: "flex", alignItems: "center", gap: "4px",
                  }}
                >
                  {f}
                  <span style={{ fontSize: "9px", opacity: 0.6 }}>{count}</span>
                </button>
              );
            })}
          </div>

          {/* Events */}
          {filtered.length === 0 ? (
            <EmptyState title="No events" description={`No ${filter.toLowerCase()} events found`} />
          ) : (
            <div style={{ maxHeight: "600px", overflowY: "auto" }}>
              {filtered.map((event) => (
                <ActivityRow key={`${event.kind}-${event.id}`} event={event} />
              ))}
            </div>
          )}
        </Card>

        {/* Right sidebar */}
        <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
          {/* Top tools */}
          <Card padding="14px 16px">
            <div style={{
              fontFamily: "var(--font-mono)", fontSize: "9px", fontWeight: 600,
              letterSpacing: "0.08em", textTransform: "uppercase",
              color: "var(--text-3)", marginBottom: "12px",
            }}>
              Top tools
            </div>
            <TopToolsChart data={summary?.tool_event_counts ?? []} />
          </Card>

          {/* Recent errors */}
          <Card glow="var(--rose)">
            <div style={{
              fontFamily: "var(--font-mono)", fontSize: "9px", fontWeight: 600,
              letterSpacing: "0.08em", textTransform: "uppercase",
              color: "var(--text-3)", padding: "14px 16px 0",
              display: "flex", alignItems: "center", justifyContent: "space-between",
            }}>
              <span>Recent errors</span>
              {summary && summary.recent_errors > 0 && (
                <span style={{
                  fontFamily: "var(--font-mono)", fontSize: "10px", fontWeight: 600,
                  color: "var(--rose)",
                }}>
                  {summary.recent_errors}
                </span>
              )}
            </div>
            <RecentErrors events={events} />
          </Card>
        </div>
      </div>
    </PageShell>
  );
}

// ── Stat tile ─────────────────────────────────────────────────
function StatTile({
  icon: Icon, color, label, value,
}: {
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  color: string;
  label: string;
  value: number;
}) {
  return (
    <div style={{
      background: "var(--bg-1)", border: "1px solid var(--border-1)",
      borderRadius: "var(--radius-lg)", padding: "14px 16px",
      display: "flex", flexDirection: "column", gap: "6px",
    }}>
      <span style={{ color, display: "flex", opacity: 0.8 }}>
        <Icon size={15} strokeWidth={2} />
      </span>
      <div style={{
        fontFamily: "var(--font-mono)", fontSize: "22px", fontWeight: 700,
        lineHeight: 1, color: "var(--text-1)", fontVariantNumeric: "tabular-nums",
      }}>
        {value.toLocaleString()}
      </div>
      <div style={{
        fontSize: "10px", color: "var(--text-3)", fontFamily: "var(--font-sans)",
      }}>
        {label}
      </div>
    </div>
  );
}

// ── Top tools chart ───────────────────────────────────────────
function TopToolsChart({ data }: { data: [string, number][] }) {
  if (data.length === 0) return <EmptyState title="No tool data" />;

  const max = Math.max(...data.map(([, v]) => v), 1);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "5px" }}>
      {data.slice(0, 8).map(([name, count]) => (
        <div key={name} style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "10px", color: "var(--text-2)",
            width: "80px", textAlign: "right", overflow: "hidden",
            textOverflow: "ellipsis", whiteSpace: "nowrap", flexShrink: 0,
          }}>
            {name}
          </span>
          <div style={{ flex: 1, height: "6px", background: "var(--bg-3)", borderRadius: "3px", overflow: "hidden" }}>
            <div style={{
              height: "100%", width: `${(count / max) * 100}%`,
              background: "var(--amber)", borderRadius: "3px",
              minWidth: "2px", transition: "width 0.3s ease",
            }} />
          </div>
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "10px", color: "var(--text-3)",
            width: "36px", flexShrink: 0, textAlign: "right",
          }}>
            {count.toLocaleString()}
          </span>
        </div>
      ))}
    </div>
  );
}

// ── Recent errors ─────────────────────────────────────────────
function RecentErrors({ events }: { events: UnifiedEvent[] }) {
  const errors = events.filter((e) => e.kind === "tool_failure").slice(0, 6);

  if (errors.length === 0) {
    return (
      <div style={{ padding: "20px", textAlign: "center", fontSize: "12px", color: "var(--text-3)" }}>
        No recent errors
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column" }}>
      {errors.map((e) => (
        <Link
          key={`err-${e.id}`}
          to={`/sessions/${encodeURIComponent(e.session_id)}`}
          style={{
            display: "block", padding: "8px 16px",
            borderBottom: "1px solid var(--border-1)",
            fontSize: "11px", fontFamily: "var(--font-mono)",
            textDecoration: "none", transition: "background 0.1s",
          }}
          onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.background = "var(--bg-2)"; }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.background = "transparent"; }}
        >
          <div style={{ color: "var(--rose)", marginBottom: "2px" }}>
            {e.label}
          </div>
          {e.error && (
            <div style={{
              color: "var(--text-3)", overflow: "hidden",
              textOverflow: "ellipsis", whiteSpace: "nowrap",
            }}>
              {e.error}
            </div>
          )}
        </Link>
      ))}
    </div>
  );
}
