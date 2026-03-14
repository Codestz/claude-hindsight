import { useState } from "react";
import { EventRow, type UnifiedEvent } from "./EventRow";
import { EmptyState } from "@/components/ui/EmptyState";

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

interface Props {
  events: UnifiedEvent[];
}

export function EventTimeline({ events }: Props) {
  const [filter, setFilter] = useState<Filter>("All");

  const filtered = events.filter((e) => matchesFilter(e, filter));

  return (
    <div>
      {/* Filter tabs */}
      <div
        style={{
          display: "flex",
          gap: "2px",
          padding: "12px 16px",
          borderBottom: "1px solid var(--border-1)",
        }}
      >
        {FILTERS.map((f) => {
          const active = filter === f;
          const count = f === "All" ? events.length : events.filter((e) => matchesFilter(e, f)).length;
          return (
            <button
              key={f}
              onClick={() => setFilter(f)}
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "11px",
                fontWeight: 600,
                padding: "4px 10px",
                borderRadius: "12px",
                border: "none",
                background: active ? "rgba(129, 140, 248, 0.10)" : "transparent",
                color: active ? "var(--indigo)" : "var(--text-3)",
                cursor: "pointer",
                transition: "all 0.12s",
                display: "flex",
                alignItems: "center",
                gap: "5px",
              }}
            >
              {f}
              <span
                style={{
                  fontSize: "10px",
                  color: active ? "var(--text-2)" : "var(--text-3)",
                  opacity: 0.7,
                }}
              >
                {count}
              </span>
            </button>
          );
        })}
      </div>

      {/* Event rows */}
      {filtered.length === 0 ? (
        <EmptyState title="No events" description={`No ${filter.toLowerCase()} events found`} />
      ) : (
        <div style={{ maxHeight: "520px", overflowY: "auto" }}>
          {filtered.map((event) => (
            <EventRow key={`${event.kind}-${event.id}`} event={event} />
          ))}
        </div>
      )}
    </div>
  );
}
