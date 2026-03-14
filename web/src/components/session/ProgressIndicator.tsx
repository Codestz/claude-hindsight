import type { NodeResponse } from "@/lib/types";

interface ProgressIndicatorProps {
  node: NodeResponse;
}

export function ProgressIndicator({ node }: ProgressIndicatorProps) {
  const progress = node.progress;
  const label = progress?.message ?? node.label;
  const percentage = progress?.percentage;

  return (
    <div style={{
      display: "flex", justifyContent: "center",
      animation: "fadeSlideIn 0.3s ease forwards",
    }}>
      <div style={{
        display: "flex", alignItems: "center", gap: "8px",
        padding: "4px 16px",
        minHeight: "28px",
      }}>
        <span style={{
          fontSize: "11px", color: "var(--text-3)",
        }}>
          &#10227;
        </span>
        <span style={{
          fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)",
        }}>
          {label}
        </span>
        {percentage != null && (
          <div style={{
            width: "60px", height: "3px",
            background: "var(--bg-3)", borderRadius: "2px", overflow: "hidden",
          }}>
            <div style={{
              height: "100%", width: `${percentage}%`,
              background: "var(--amber)",
              transition: "width 0.3s ease",
            }} />
          </div>
        )}
      </div>
    </div>
  );
}
