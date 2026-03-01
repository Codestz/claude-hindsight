import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import type {
  HookActivitySummary,
  HookToolEvent,
  HookSubagentEvent,
  HookLifecycleEvent,
  HookPermissionEvent,
} from "@/lib/types";
import { PageShell } from "@/components/ui/PageShell";
import { Card } from "@/components/ui/Card";
import { LoadingState } from "@/components/ui/LoadingState";
import { ErrorState } from "@/components/ui/ErrorState";
import { EmptyState } from "@/components/ui/EmptyState";
import { EventTimeline } from "@/components/activity/EventTimeline";
import type { UnifiedEvent } from "@/components/activity/EventRow";
import {
  Wrench,
  Bot,
  Shield,
  AlertTriangle,
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
    id: e.id + 1_000_000, // offset to avoid key collisions
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

// ── Stat mini-card (inline, compact) ───────────────────────────

function MiniStat({
  icon: Icon,
  color,
  label,
  value,
}: {
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  color: string;
  label: string;
  value: number;
}) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: "10px",
        padding: "16px 20px",
        background: "var(--bg-1)",
        borderRadius: "var(--radius-lg)",
        border: "1px solid var(--border-1)",
        flex: "1 1 0",
        minWidth: "140px",
      }}
    >
      <span style={{ color, display: "flex" }}>
        <Icon size={18} strokeWidth={2} />
      </span>
      <div>
        <div
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "20px",
            fontWeight: 700,
            lineHeight: 1,
            color: "var(--text-1)",
            fontVariantNumeric: "tabular-nums",
          }}
        >
          {value.toLocaleString()}
        </div>
        <div
          style={{
            fontSize: "11px",
            color: "var(--text-3)",
            fontFamily: "var(--font-sans)",
            marginTop: "2px",
          }}
        >
          {label}
        </div>
      </div>
    </div>
  );
}

// ── Top tools bar chart (simple horizontal) ────────────────────

function TopToolsChart({ data }: { data: [string, number][] }) {
  if (data.length === 0) return <EmptyState title="No tool data" />;

  const max = Math.max(...data.map(([, v]) => v), 1);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
      {data.slice(0, 8).map(([name, count]) => (
        <div key={name} style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "11px",
              color: "var(--text-2)",
              width: "120px",
              textAlign: "right",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
              flexShrink: 0,
            }}
          >
            {name}
          </span>
          <div style={{ flex: 1, height: "16px", background: "var(--bg-2)", borderRadius: "2px", overflow: "hidden" }}>
            <div
              style={{
                height: "100%",
                width: `${(count / max) * 100}%`,
                background: "var(--amber)",
                borderRadius: "2px",
                minWidth: "2px",
                transition: "width 0.3s ease",
              }}
            />
          </div>
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "11px",
              color: "var(--text-3)",
              width: "40px",
              flexShrink: 0,
            }}
          >
            {count.toLocaleString()}
          </span>
        </div>
      ))}
    </div>
  );
}

// ── Recent errors list ─────────────────────────────────────────

function RecentErrors({ events }: { events: UnifiedEvent[] }) {
  const errors = events.filter((e) => e.kind === "tool_failure").slice(0, 6);

  if (errors.length === 0) {
    return (
      <div style={{ padding: "24px", textAlign: "center", fontSize: "13px", color: "var(--text-3)" }}>
        No recent errors
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column" }}>
      {errors.map((e) => (
        <div
          key={`err-${e.id}`}
          style={{
            padding: "10px 16px",
            borderBottom: "1px solid var(--border-1)",
            fontSize: "12px",
            fontFamily: "var(--font-mono)",
          }}
        >
          <div style={{ color: "var(--red)", marginBottom: "2px" }}>
            {e.label}
          </div>
          {e.error && (
            <div
              style={{
                color: "var(--text-3)",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
                maxWidth: "100%",
              }}
            >
              {e.error}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

// ── Main page ──────────────────────────────────────────────────

export default function ActivityPage() {
  const [summary, setSummary] = useState<HookActivitySummary | null>(null);
  const [events, setEvents] = useState<UnifiedEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

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

        // Merge & sort by occurred_at descending
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

  return (
    <PageShell>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <h1
          style={{
            fontSize: "20px",
            fontWeight: 600,
            color: "var(--text-1)",
            fontFamily: "var(--font-sans)",
            margin: 0,
          }}
        >
          Activity
        </h1>
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "11px",
            color: "var(--text-3)",
          }}
        >
          {events.length.toLocaleString()} events
        </span>
      </div>

      {/* Stat cards row */}
      {summary && (
        <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
          <MiniStat icon={Wrench} color="var(--amber)" label="Tool events" value={summary.total_tool_events} />
          <MiniStat icon={Bot} color="var(--purple)" label="Agent events" value={summary.total_subagent_events} />
          <MiniStat icon={Shield} color="var(--cyan)" label="Permission events" value={summary.total_permission_events} />
          <MiniStat icon={AlertTriangle} color="var(--red)" label="Recent errors" value={summary.recent_errors} />
        </div>
      )}

      {/* Main content: timeline + sidebar */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 340px", gap: "16px", alignItems: "start" }}>
        {/* Event timeline */}
        <Card>
          <EventTimeline events={events} />
        </Card>

        {/* Right column */}
        <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
          {/* Top tools */}
          <Card padding="16px">
            <div
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "10px",
                fontWeight: 600,
                letterSpacing: "0.1em",
                textTransform: "uppercase",
                color: "var(--text-3)",
                marginBottom: "14px",
              }}
            >
              Top tools
            </div>
            <TopToolsChart data={summary?.tool_event_counts ?? []} />
          </Card>

          {/* Recent errors */}
          <Card>
            <div
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "10px",
                fontWeight: 600,
                letterSpacing: "0.1em",
                textTransform: "uppercase",
                color: "var(--text-3)",
                padding: "14px 16px 0",
              }}
            >
              Recent errors
            </div>
            <RecentErrors events={events} />
          </Card>
        </div>
      </div>
    </PageShell>
  );
}
