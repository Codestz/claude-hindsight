/**
 * Code block renderer with syntax highlighting.
 *
 * Renders code content with:
 * - Automatic language detection from file path or content heuristics
 * - Syntax highlighting for JSON, HTML/CSS/JS, TypeScript, Rust, Python, Go, Bash
 * - Optional file path header
 * - Language badge
 * - Error styling
 *
 * The actual tokenization and colorization logic lives in lib/syntax/.
 */

import React from "react";
import { detectLanguage } from "@/lib/syntax/languages";
import { colorizeJson, colorizeCode } from "@/lib/syntax/colorize";

import type { CodeRenderProps } from "./types";

export function CodeRender({
  content,
  language,
  filePath,
  error = false,
  maxHeight = "480px",
}: CodeRenderProps) {
  const lang = language ?? detectLanguage(filePath, content);
  const isJson = lang === "json";
  const hasHighlight = isJson || lang !== "text";

  return (
    <div style={{ position: "relative" }}>
      {/* File path header */}
      {filePath && (
        <div
          style={{
            fontFamily: "var(--font-mono)", fontSize: "11px",
            color: "var(--text-3)", padding: "6px 14px",
            background: "var(--bg-3)",
            borderRadius: "var(--radius-md) var(--radius-md) 0 0",
            border: `1px solid ${error ? "rgba(255,69,69,0.25)" : "var(--border-1)"}`,
            borderBottom: "none",
            overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
          }}
          title={filePath}
        >
          {filePath}
        </div>
      )}

      {/* Language badge */}
      <div
        style={{
          position: "absolute",
          top: filePath ? "38px" : "10px",
          right: "12px",
          fontFamily: "var(--font-mono)", fontSize: "10px",
          letterSpacing: "0.1em", textTransform: "uppercase",
          color: "var(--text-3)", pointerEvents: "none",
          background: "var(--bg-3)", padding: "2px 8px",
          borderRadius: "var(--radius-sm)", border: "1px solid var(--border-1)",
          zIndex: 1,
        }}
      >
        {lang}
      </div>

      {/* Code content */}
      <pre
        style={{
          margin: 0, padding: "14px 16px", paddingTop: "28px",
          background: "var(--bg-2)",
          border: `1px solid ${error ? "rgba(255,69,69,0.25)" : "var(--border-1)"}`,
          borderRadius: filePath ? "0 0 var(--radius-md) var(--radius-md)" : "var(--radius-md)",
          fontFamily: "var(--font-mono)", fontSize: "13px", lineHeight: 1.65,
          overflowX: "auto", whiteSpace: "pre-wrap", wordBreak: "break-word",
          maxHeight, overflowY: "auto",
        }}
      >
        {isJson ? (
          colorizeJson(content)
        ) : hasHighlight ? (
          colorizeCode(content, lang)
        ) : (
          <span style={{ color: error ? "var(--red)" : "var(--text-2)" }}>{content}</span>
        )}
      </pre>
    </div>
  );
}
