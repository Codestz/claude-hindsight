import React from "react";

// ── Language detection ────────────────────────────────────────
function detectLanguage(filePath?: string, content?: string): string {
  if (filePath) {
    const ext = filePath.split(".").pop()?.toLowerCase() ?? "";
    const map: Record<string, string> = {
      rs: "rust", js: "javascript", jsx: "javascript",
      ts: "typescript", tsx: "typescript", py: "python",
      go: "go", json: "json", toml: "toml", yaml: "yaml",
      yml: "yaml", md: "markdown", css: "css", sh: "bash", bash: "bash",
      html: "html", c: "c", cpp: "cpp", rb: "ruby",
    };
    if (map[ext]) return map[ext];
  }
  // Heuristic: try JSON parse
  const trimmed = content?.trim() ?? "";
  if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
    try { JSON.parse(trimmed); return "json"; } catch {}
  }
  return "text";
}

// ── JSON colorization ─────────────────────────────────────────
function colorizeJson(json: string): React.ReactNode {
  const tokens = json.split(/(\"[^\"]*\"\s*:)|(\"[^\"]*\")|([-\d.]+)|(true|false|null)|([{}[\],])/g);
  return tokens.filter(Boolean).map((token, i) => {
    if (/^"[^"]*"\s*:$/.test(token))
      return <span key={i} style={{ color: "var(--cyan)" }}>{token}</span>;
    if (/^"[^"]*"$/.test(token))
      return <span key={i} style={{ color: "var(--green)" }}>{token}</span>;
    if (/^[-\d.]+$|^(true|false|null)$/.test(token))
      return <span key={i} style={{ color: "var(--amber)" }}>{token}</span>;
    return <span key={i} style={{ color: "var(--text-3)" }}>{token}</span>;
  });
}

// ── Component ─────────────────────────────────────────────────
interface CodeRenderProps {
  content: string;
  language?: string;
  filePath?: string;
  error?: boolean;
  maxHeight?: string;
}

export function CodeRender({
  content,
  language,
  filePath,
  error = false,
  maxHeight = "480px",
}: CodeRenderProps) {
  const lang = language ?? detectLanguage(filePath, content);
  const isJson = lang === "json";

  return (
    <div style={{ position: "relative" }}>
      {/* Language badge */}
      <div
        style={{
          position: "absolute", top: "10px", right: "12px",
          fontFamily: "var(--font-mono)", fontSize: "10px",
          letterSpacing: "0.1em", textTransform: "uppercase",
          color: "var(--text-3)", pointerEvents: "none",
        }}
      >
        {lang}
      </div>
      <pre
        style={{
          margin: 0, padding: "14px 16px", paddingTop: "28px",
          background: "var(--bg-2)",
          border: `1px solid ${error ? "rgba(255,69,69,0.25)" : "var(--border-1)"}`,
          borderRadius: "var(--radius-md)",
          fontFamily: "var(--font-mono)", fontSize: "13px", lineHeight: 1.65,
          overflowX: "auto", whiteSpace: "pre-wrap", wordBreak: "break-word",
          maxHeight, overflowY: "auto",
        }}
      >
        {isJson ? (
          colorizeJson(content)
        ) : (
          <span style={{ color: error ? "var(--red)" : "var(--text-2)" }}>{content}</span>
        )}
      </pre>
    </div>
  );
}
