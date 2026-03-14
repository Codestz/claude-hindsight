// Extracted tool-specific renderers — pure display components, no state.
// Used by both ToolCallCard (inline) and NodeDetailDrawer (full detail).

import type { ContentBlock, NodeResponse, TokenUsage } from "@/lib/types";
import { CodeRender } from "@/components/ui/CodeRender";
import { MarkdownContent } from "@/components/ui/MarkdownContent";

// ── Content section wrapper ─────────────────────────────────────
export function ContentSection({
  label, color, children,
}: { label: string; color?: string; children: React.ReactNode }) {
  return (
    <div style={{ marginBottom: "24px" }}>
      <div style={{
        fontFamily: "var(--font-mono)", fontSize: "11px", fontWeight: 700,
        letterSpacing: "0.1em", textTransform: "uppercase",
        color: color ?? "var(--text-3)", marginBottom: "10px",
      }}>
        {label}
      </div>
      {children}
    </div>
  );
}

// ── Token usage footer ──────────────────────────────────────────
export function TokenFooter({ usage }: { usage: TokenUsage }) {
  const items = [
    { label: "In",          value: usage.input_tokens },
    { label: "Out",         value: usage.output_tokens },
    { label: "Cache write", value: usage.cache_creation_input_tokens },
    { label: "Cache read",  value: usage.cache_read_input_tokens },
  ].filter((i) => i.value != null && i.value > 0);

  if (items.length === 0) return null;

  return (
    <div style={{
      display: "flex", gap: "20px", paddingTop: "18px",
      borderTop: "1px solid var(--border-1)", marginTop: "8px",
      background: "color-mix(in srgb, var(--sky) 3%, var(--bg-1))",
      margin: "8px -24px 0", padding: "18px 24px 0",
    }}>
      {items.map((item) => (
        <div key={item.label} style={{ display: "flex", gap: "6px", alignItems: "baseline" }}>
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "11px",
            color: "var(--text-3)", textTransform: "uppercase", letterSpacing: "0.06em",
          }}>{item.label}</span>
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "13px",
            fontVariantNumeric: "tabular-nums", color: "var(--cyan)",
          }}>{item.value!.toLocaleString()}</span>
        </div>
      ))}
    </div>
  );
}

// ── Empty result placeholder ────────────────────────────────────
export function EmptyResult({ label }: { label?: string }) {
  return (
    <div style={{
      display: "flex", alignItems: "center", justifyContent: "center",
      padding: "20px", gap: "8px",
      background: "var(--bg-2)", border: "1px solid var(--border-1)",
      borderRadius: "var(--radius-md)",
    }}>
      <span style={{ color: "var(--text-3)", fontSize: "16px" }}>&#8709;</span>
      <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", letterSpacing: "0.05em" }}>
        {label ?? "no result"}
      </span>
    </div>
  );
}

// ── Read tool ───────────────────────────────────────────────────
export function ReadToolDisplay({ input }: { input: Record<string, unknown> }) {
  const filePath = typeof input.file_path === "string" ? input.file_path : null;
  const offset   = typeof input.offset   === "number" ? input.offset   : null;
  const limit    = typeof input.limit    === "number" ? input.limit    : null;

  const shortName = filePath?.split("/").pop() ?? filePath;
  const lineRange = offset != null && limit != null
    ? `lines ${offset}–${offset + limit}`
    : offset != null ? `from line ${offset}` : limit != null ? `first ${limit} lines` : "entire file";

  return (
    <ContentSection label="File" color="var(--cyan)">
      <div style={{
        padding: "10px 14px",
        background: "var(--bg-2)",
        border: "1px solid var(--border-1)",
        borderRadius: "var(--radius-md)",
        display: "flex", flexDirection: "column", gap: "6px",
      }}>
        <span style={{
          fontFamily: "var(--font-mono)", fontSize: "14px", fontWeight: 600,
          color: "var(--cyan)", wordBreak: "break-all",
        }}>
          {shortName}
        </span>
        {filePath && shortName !== filePath && (
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "11px",
            color: "var(--text-3)", wordBreak: "break-all",
          }}>
            {filePath}
          </span>
        )}
        <span style={{
          fontFamily: "var(--font-mono)", fontSize: "11px",
          color: "var(--text-3)",
        }}>
          {lineRange}
        </span>
      </div>
    </ContentSection>
  );
}

// ── Generic tool input as labeled fields ────────────────────────
export function GenericToolInput({ input, toolName }: { input: Record<string, unknown>; toolName?: string }) {
  const entries = Object.entries(input);
  if (entries.length === 0) return null;

  // For simple inputs (1-3 short values), show as inline fields
  const isSimple = entries.length <= 4 && entries.every(([, v]) =>
    typeof v === "string" ? v.length < 200 : typeof v === "number" || typeof v === "boolean"
  );

  if (isSimple) {
    return (
      <ContentSection label="Input" color="var(--amber)">
        <div style={{
          display: "flex", flexDirection: "column", gap: "6px",
          padding: "10px 14px", background: "var(--bg-2)",
          border: "1px solid var(--border-1)", borderRadius: "var(--radius-md)",
        }}>
          {entries.map(([key, value]) => (
            <div key={key} style={{ display: "flex", gap: "8px", alignItems: "baseline" }}>
              <span style={{
                fontFamily: "var(--font-mono)", fontSize: "11px",
                color: "var(--text-3)", flexShrink: 0, minWidth: "80px",
              }}>
                {key}
              </span>
              <span style={{
                fontFamily: "var(--font-mono)", fontSize: "12px",
                color: "var(--text-1)", wordBreak: "break-all",
              }}>
                {String(value)}
              </span>
            </div>
          ))}
        </div>
      </ContentSection>
    );
  }

  // Complex inputs: JSON code block
  return (
    <ContentSection label="Input" color="var(--amber)">
      <CodeRender content={JSON.stringify(input, null, 2)} />
    </ContentSection>
  );
}

// ── Write tool ──────────────────────────────────────────────────
export function WriteToolDisplay({ input }: { input: Record<string, unknown> }) {
  const filePath = typeof input.file_path === "string" ? input.file_path : null;
  const content  = typeof input.content   === "string" ? input.content  : null;

  return (
    <>
      {filePath && (
        <ContentSection label="File">
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "13px",
            color: "var(--green)", wordBreak: "break-all",
          }}>
            {filePath}
          </span>
        </ContentSection>
      )}
      {content != null ? (
        <ContentSection label="Content" color="var(--green)">
          <CodeRender content={content} filePath={filePath ?? undefined} />
        </ContentSection>
      ) : (
        <ContentSection label="Content" color="var(--green)">
          <EmptyResult />
        </ContentSection>
      )}
    </>
  );
}

// ── Bash tool ───────────────────────────────────────────────────
export function BashToolDisplay({ input }: { input: Record<string, unknown> }) {
  const command = typeof input.command === "string" ? input.command : null;
  const stdin   = typeof input.stdin   === "string" ? input.stdin   : null;

  return (
    <>
      {command != null ? (
        <ContentSection label="Command" color="var(--amber)">
          <CodeRender content={command} language="bash" />
        </ContentSection>
      ) : (
        <ContentSection label="Input" color="var(--amber)">
          <CodeRender content={JSON.stringify(input, null, 2)} />
        </ContentSection>
      )}
      {stdin && (
        <ContentSection label="Stdin" color="var(--text-3)">
          <CodeRender content={stdin} language="text" />
        </ContentSection>
      )}
    </>
  );
}

// ── Edit tool (diff view) ───────────────────────────────────────
export function EditToolDisplay({ input }: { input: Record<string, unknown> }) {
  const filePath   = typeof input.file_path  === "string" ? input.file_path  : null;
  const oldString  = typeof input.old_string === "string" ? input.old_string : null;
  const newString  = typeof input.new_string === "string" ? input.new_string : null;
  const replaceAll = input.replace_all === true;

  const removedLines = oldString?.split("\n") ?? [];
  const addedLines   = newString?.split("\n")  ?? [];
  const fileName = filePath?.split("/").pop() ?? filePath ?? "edit";

  return (
    <ContentSection label="Edit" color="var(--text-3)">
      <div style={{
        borderRadius: "var(--radius-md)", overflow: "hidden",
        border: "1px solid var(--border-1)",
        fontFamily: "var(--font-mono)", fontSize: "13px",
      }}>
        {/* Header bar */}
        <div style={{
          display: "flex", alignItems: "center", justifyContent: "space-between",
          padding: "8px 14px",
          background: "var(--bg-2)",
          borderBottom: "1px solid var(--border-1)",
        }}>
          <span
            title={filePath ?? undefined}
            style={{
              color: "var(--text-2)", fontSize: "12px",
              overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
              maxWidth: "65%",
            }}
          >
            {fileName}
          </span>

          <div style={{ display: "flex", alignItems: "center", gap: "12px", flexShrink: 0 }}>
            {removedLines.length > 0 && (
              <span style={{ color: "var(--red)", opacity: 0.75, fontSize: "12px" }}>
                &minus;{removedLines.length}
              </span>
            )}
            {addedLines.length > 0 && (
              <span style={{ color: "var(--green)", opacity: 0.75, fontSize: "12px" }}>
                +{addedLines.length}
              </span>
            )}
            {replaceAll && (
              <span style={{
                padding: "1px 7px", borderRadius: "var(--radius-sm)",
                border: "1px solid rgba(255,181,71,0.25)",
                fontSize: "10px", letterSpacing: "0.06em", textTransform: "uppercase",
                color: "var(--amber)", opacity: 0.85,
              }}>
                all
              </span>
            )}
          </div>
        </div>

        {/* Diff lines */}
        <div style={{ maxHeight: "480px", overflowY: "auto" }}>
          {removedLines.map((line, i) => (
            <div key={`r${i}`} style={{
              display: "flex", alignItems: "flex-start",
              background: "rgba(255,69,69,0.055)",
              borderLeft: "2px solid rgba(255,69,69,0.38)",
            }}>
              <span style={{
                width: "34px", flexShrink: 0, textAlign: "center",
                padding: "3px 0", lineHeight: 1.6,
                color: "rgba(255,100,100,0.55)", userSelect: "none", fontSize: "12px",
              }}>&minus;</span>
              <span style={{
                flex: 1, padding: "3px 14px 3px 0",
                color: "var(--text-3)", lineHeight: 1.6,
                whiteSpace: "pre-wrap", wordBreak: "break-all",
              }}>
                {line || "\u00a0"}
              </span>
            </div>
          ))}

          {removedLines.length > 0 && addedLines.length > 0 && (
            <div style={{ height: "1px", background: "var(--border-1)" }} />
          )}

          {addedLines.map((line, i) => (
            <div key={`a${i}`} style={{
              display: "flex", alignItems: "flex-start",
              background: "rgba(52, 211, 153, 0.04)",
              borderLeft: "2px solid rgba(52, 211, 153, 0.3)",
            }}>
              <span style={{
                width: "34px", flexShrink: 0, textAlign: "center",
                padding: "3px 0", lineHeight: 1.6,
                color: "rgba(0,220,100,0.5)", userSelect: "none", fontSize: "12px",
              }}>+</span>
              <span style={{
                flex: 1, padding: "3px 14px 3px 0",
                color: "var(--text-1)", lineHeight: 1.6,
                whiteSpace: "pre-wrap", wordBreak: "break-all",
              }}>
                {line || "\u00a0"}
              </span>
            </div>
          ))}
        </div>
      </div>
    </ContentSection>
  );
}

// ── TaskCreate display ──────────────────────────────────────────
export function TaskCreateDisplay({ input }: { input: Record<string, unknown> }) {
  const subject     = typeof input.subject     === "string" ? input.subject     : null;
  const description = typeof input.description === "string" ? input.description : null;
  const activeForm  = typeof input.activeForm  === "string" ? input.activeForm  : null;

  return (
    <>
      {subject && (
        <ContentSection label="Subject">
          <p style={{
            fontFamily: "var(--font-sans)", fontSize: "16px", fontWeight: 600,
            color: "var(--text-1)", margin: 0, lineHeight: 1.4,
          }}>
            {subject}
          </p>
        </ContentSection>
      )}
      {activeForm && (
        <ContentSection label="Active Form" color="var(--cyan)">
          <span style={{
            display: "inline-flex", alignItems: "center", gap: "6px",
            fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--cyan)",
          }}>
            <span style={{ opacity: 0.5 }}>&#10227;</span> {activeForm}
          </span>
        </ContentSection>
      )}
      {description && (
        <ContentSection label="Description">
          <p style={{
            fontFamily: "var(--font-sans)", fontSize: "14px",
            lineHeight: 1.65, color: "var(--text-2)", margin: 0,
          }}>
            {description}
          </p>
        </ContentSection>
      )}
    </>
  );
}

// ── Serena MCP result rendering ─────────────────────────────────

type SerenaResult =
  | { type: "text"; text: string }
  | { type: "structured"; data: Record<string, unknown> };

export function parseSerenaResult(content: string): SerenaResult | null {
  try {
    const outer = JSON.parse(content);
    if (typeof outer !== "object" || outer === null || !("result" in outer)) return null;
    const resultVal = (outer as Record<string, unknown>).result;
    if (typeof resultVal !== "string") return null;
    try {
      const inner = JSON.parse(resultVal);
      if (typeof inner === "object" && inner !== null)
        return { type: "structured", data: inner as Record<string, unknown> };
    } catch {
      // resultVal is plain text, not JSON
    }
    return { type: "text", text: resultVal };
  } catch {
    return null;
  }
}

export function PathList({ items }: { items: string[] }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "2px" }}>
      {items.map((item, i) => (
        <div
          key={i}
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "12px",
            color: "var(--text-2)",
            padding: "2px 0",
            borderBottom: "1px solid var(--border-1)",
            wordBreak: "break-all",
          }}
        >
          {item}
        </div>
      ))}
    </div>
  );
}

export function SerenaResultDisplay({ content }: { content: string }) {
  const parsed = parseSerenaResult(content);
  if (!parsed) return <CodeRender content={content} />;
  if (parsed.type === "text") return <MarkdownContent text={parsed.text} />;

  const { data } = parsed;
  const dirs  = Array.isArray(data.dirs)  ? (data.dirs  as string[]) : [];
  const files = Array.isArray(data.files) ? (data.files as string[]) : [];
  const rest  = Object.entries(data).filter(([k]) => k !== "dirs" && k !== "files");

  return (
    <div>
      {dirs.length > 0 && (
        <ContentSection label={`Dirs \u00b7 ${dirs.length}`} color="var(--cyan)">
          <PathList items={dirs} />
        </ContentSection>
      )}
      {files.length > 0 && (
        <ContentSection label={`Files \u00b7 ${files.length}`} color="var(--green)">
          <PathList items={files} />
        </ContentSection>
      )}
      {rest.map(([k, v]) => (
        <ContentSection key={k} label={k}>
          <CodeRender content={typeof v === "string" ? v : JSON.stringify(v, null, 2)} />
        </ContentSection>
      ))}
    </div>
  );
}

// ── Strip N-> line number prefix (Read tool results) ────────────
// The Read tool returns content in `cat -n` format:
//   "     1→import React from 'react';"
// Leading spaces + digits + arrow (U+2192) + optional tab/space
export function stripLineNumbers(text: string): string {
  const lines = text.split("\n");
  const firstNonEmpty = lines.find((l) => l.trim() !== "");
  if (firstNonEmpty && /^\s*\d+\u2192/.test(firstNonEmpty)) {
    return lines.map((l) => l.replace(/^\s*\d+\u2192\t?/, "")).join("\n");
  }
  return text;
}

// ── Resolve tool call info from a node ──────────────────────────
export function resolveToolCall(node: NodeResponse): {
  toolCallName: string | undefined;
  toolCallInput: Record<string, unknown> | undefined;
} {
  const contentBlocks = Array.isArray(node.message?.content)
    ? (node.message!.content as ContentBlock[])
    : [];

  const toolUseBlock = contentBlocks.find((b) => b.type === "tool_use") as
    | { type: "tool_use"; id: string; name: string; input: Record<string, unknown> }
    | undefined;

  const topLevelToolUse = node.tool_use;

  return {
    toolCallName: toolUseBlock?.name ?? topLevelToolUse?.name ?? node.tool_name,
    toolCallInput: toolUseBlock?.input ?? (topLevelToolUse?.input as Record<string, unknown> | undefined),
  };
}

// ── Resolve tool result info from a node ────────────────────────
export function resolveToolResult(node: NodeResponse): {
  displayResultContent: string | null;
  effectiveIsError: boolean;
  hasCleanFile: boolean;
} {
  const contentBlocks = Array.isArray(node.message?.content)
    ? (node.message!.content as ContentBlock[])
    : [];

  const toolResultBlock = contentBlocks.find((b) => b.type === "tool_result") as
    | { type: "tool_result"; tool_use_id: string; content: unknown; is_error: boolean | null }
    | undefined;

  const topLevelToolResult = node.tool_result;

  const resultIsError =
    toolResultBlock?.is_error ?? topLevelToolResult?.is_error ?? false;

  const rawErrorCheckStr: string | null = (() => {
    if (toolResultBlock?.content != null) {
      return typeof toolResultBlock.content === "string"
        ? toolResultBlock.content
        : JSON.stringify(toolResultBlock.content);
    }
    return topLevelToolResult?.content ?? null;
  })();

  const hasErrorTag = rawErrorCheckStr?.includes("<tool_use_error>") ?? false;
  const effectiveIsError = !!resultIsError || hasErrorTag;

  const hasCleanFile = !!topLevelToolResult?.file?.content;

  const displayResultContent: string | null = hasCleanFile
    ? null
    : (() => {
        let raw: string | null = null;
        if (toolResultBlock?.content != null) {
          raw = typeof toolResultBlock.content === "string"
            ? toolResultBlock.content
            : JSON.stringify(toolResultBlock.content, null, 2);
        } else if (topLevelToolResult?.content != null) {
          raw = topLevelToolResult.content;
        }
        if (raw == null) return null;
        raw = stripLineNumbers(raw);
        raw = raw.replace(/<tool_use_error>([\s\S]*?)<\/tool_use_error>/g, "$1").trim();
        return raw || null;
      })();

  return { displayResultContent, effectiveIsError, hasCleanFile };
}
