/**
 * Renders a sub-agent task completion notification.
 *
 * Displays status badge, summary, usage stats, task ID, and result as markdown.
 * parseTaskNotification() lives in ./utils.ts — this file is purely rendering.
 */

import { ContentSection } from "./tool-displays";
import { MarkdownContent } from "@/components/ui/MarkdownContent";
import type { TaskNotificationCardProps } from "./types";

export function TaskNotificationCard({ task }: TaskNotificationCardProps) {
  const isComplete = task.status === "completed";
  const statusColor = isComplete ? "var(--emerald)" : "var(--amber)";

  return (
    <>
      <ContentSection label="Task Complete" color="var(--purple)">
        <div style={{
          padding: "14px 16px",
          background: "var(--bg-2)",
          border: "1px solid var(--border-1)",
          borderRadius: "var(--radius-md)",
          display: "flex", flexDirection: "column", gap: "10px",
        }}>
          {/* Status badge */}
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <span style={{
              display: "inline-flex", alignItems: "center", gap: "5px",
              padding: "3px 10px", borderRadius: "10px",
              background: `color-mix(in srgb, ${statusColor} 12%, transparent)`,
              border: `1px solid color-mix(in srgb, ${statusColor} 25%, transparent)`,
              fontFamily: "var(--font-mono)", fontSize: "10px", fontWeight: 600,
              color: statusColor, textTransform: "uppercase", letterSpacing: "0.06em",
            }}>
              {isComplete ? "\u2713" : "\u25cb"} {task.status}
            </span>
          </div>

          {/* Summary */}
          {task.summary && (
            <div style={{
              fontFamily: "var(--font-sans)", fontSize: "14px", fontWeight: 500,
              color: "var(--text-1)", lineHeight: 1.4,
            }}>
              {task.summary}
            </div>
          )}

          {/* Usage stats */}
          {(task.totalTokens || task.toolUses || task.durationMs) && (
            <div style={{ display: "flex", gap: "16px", paddingTop: "6px", borderTop: "1px solid var(--border-1)" }}>
              {task.totalTokens && (
                <StatItem label="Tokens" value={Number(task.totalTokens).toLocaleString()} color="var(--cyan)" />
              )}
              {task.toolUses && (
                <StatItem label="Tools" value={task.toolUses} color="var(--amber)" />
              )}
              {task.durationMs && (
                <StatItem label="Duration" value={`${(Number(task.durationMs) / 1000).toFixed(1)}s`} color="var(--text-2)" />
              )}
            </div>
          )}

          {/* Task ID */}
          {task.taskId && (
            <div style={{ fontFamily: "var(--font-mono)", fontSize: "10px", color: "var(--text-3)" }}>
              {task.taskId}
            </div>
          )}
        </div>
      </ContentSection>

      {/* Result as markdown */}
      {task.result && (
        <ContentSection label="Result" color="var(--emerald)">
          <MarkdownContent text={task.result} />
        </ContentSection>
      )}
    </>
  );
}

function StatItem({ label, value, color }: { label: string; value: string; color: string }) {
  return (
    <div style={{ fontFamily: "var(--font-mono)", fontSize: "11px" }}>
      <span style={{ color: "var(--text-3)" }}>{label} </span>
      <span style={{ color }}>{value}</span>
    </div>
  );
}
