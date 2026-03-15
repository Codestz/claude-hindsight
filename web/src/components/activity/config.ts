/**
 * Configuration for activity event kinds.
 *
 * Maps each EventKind to its icon component and semantic color.
 * Extend this when adding new event types.
 */

import { Wrench, Bot, Shield, Zap, AlertTriangle } from "lucide-react";
import type { EventKind, EventKindConfig, EventFilter, UnifiedEvent } from "./types";

/** Ordered list of filter options for the activity timeline tabs. */
export const EVENT_FILTERS: readonly EventFilter[] = ["All", "Tools", "Agents", "Lifecycle", "Errors"];

export const KIND_CONFIG: Record<EventKind, EventKindConfig> = {
  tool:         { icon: Wrench,        color: "var(--amber)" },
  tool_failure: { icon: AlertTriangle, color: "var(--red)" },
  subagent:     { icon: Bot,           color: "var(--purple)" },
  permission:   { icon: Shield,        color: "var(--cyan)" },
  lifecycle:    { icon: Zap,           color: "var(--green)" },
};

/** Check if an event matches a filter tab. */
export function matchesEventFilter(event: UnifiedEvent, filter: EventFilter): boolean {
  switch (filter) {
    case "All":       return true;
    case "Tools":     return event.kind === "tool";
    case "Agents":    return event.kind === "subagent";
    case "Lifecycle": return event.kind === "lifecycle" || event.kind === "permission";
    case "Errors":    return event.kind === "tool_failure";
  }
}
