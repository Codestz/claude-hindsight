import { useState } from "react";
import type { NodeResponse } from "@/lib/types";
import { getThinkingText } from "@/lib/node-meta";

interface ThinkingBlockProps {
  node: NodeResponse;
}

export function ThinkingBlock({ node }: ThinkingBlockProps) {
  const [expanded, setExpanded] = useState(false);
  const thinkingText = getThinkingText(node);

  if (!thinkingText) return null;

  return (
    <div
      onClick={() => setExpanded((e) => !e)}
      style={{
        cursor: "pointer",
        borderRadius: "var(--radius-md)",
        border: "1px solid color-mix(in srgb, var(--violet) 15%, transparent)",
        background: "color-mix(in srgb, var(--violet) 3%, var(--bg-1))",
        overflow: "hidden",
        transition: "all 0.2s ease",
        animation: "fadeSlideIn 0.3s ease forwards",
      }}
    >
      {/* Collapsed header */}
      <div style={{
        display: "flex", alignItems: "center", gap: "8px",
        padding: "8px 14px",
        minHeight: "36px",
      }}>
        {/* Pulse dots */}
        {[0, 0.3, 0.6].map((delay, i) => (
          <span key={i} style={{
            display: "inline-block", width: "5px", height: "5px",
            borderRadius: "50%", background: "var(--violet)",
            animation: `thinking-pulse-dot 1.5s ease-in-out ${delay}s infinite`,
          }} />
        ))}
        <span style={{
          fontFamily: "var(--font-mono)", fontSize: "11px", fontWeight: 600,
          letterSpacing: "0.06em", color: "var(--violet)", opacity: 0.7,
        }}>
          Thinking
        </span>
        <span style={{ marginLeft: "auto" }}>
          <span style={{
            display: "inline-block",
            transform: expanded ? "rotate(90deg)" : "rotate(0deg)",
            transition: "transform 0.2s ease",
            color: "var(--text-3)", fontSize: "12px",
          }}>
            &#9656;
          </span>
        </span>
      </div>

      {/* Expanded content */}
      {expanded && (
        <div style={{
          padding: "0 14px 14px",
          borderTop: "1px solid color-mix(in srgb, var(--violet) 10%, transparent)",
        }}>
          <p style={{
            fontFamily: "var(--font-sans)", fontSize: "13px", lineHeight: 1.75,
            color: "var(--violet)", fontStyle: "italic", margin: "10px 0 0",
            whiteSpace: "pre-wrap", wordBreak: "break-word",
            animation: "thinking-breathe 3s ease-in-out infinite",
            opacity: 0.85,
          }}>
            {thinkingText}
          </p>
        </div>
      )}
    </div>
  );
}
