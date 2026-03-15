/**
 * Types for the Activity event system.
 *
 * UnifiedEvent is the common format that all hook event types
 * (tool, subagent, lifecycle, permission) are normalized into.
 */

export type EventKind = "tool" | "tool_failure" | "subagent" | "permission" | "lifecycle";

export interface UnifiedEvent {
  kind: EventKind;
  id: number;
  session_id: string;
  occurred_at: number;
  label: string;
  detail?: string | null;
  error?: string | null;
}

export interface EventKindConfig {
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  color: string;
}

export type EventFilter = "All" | "Tools" | "Agents" | "Lifecycle" | "Errors";
