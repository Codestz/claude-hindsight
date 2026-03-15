/**
 * Specialized display components for each tool type.
 *
 * Each component renders the INPUT section for a specific Claude Code tool.
 */

import { CodeRender } from "@/components/ui/CodeRender";
import { MarkdownContent } from "@/components/ui/MarkdownContent";
import { ContentSection, EmptyResult } from "./primitives";

// ── Read ─────────────────────────────────────────────────────

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
        padding: "10px 14px", background: "var(--bg-2)",
        border: "1px solid var(--border-1)", borderRadius: "var(--radius-md)",
        display: "flex", flexDirection: "column", gap: "6px",
      }}>
        <span style={{ fontFamily: "var(--font-mono)", fontSize: "14px", fontWeight: 600, color: "var(--cyan)", wordBreak: "break-all" }}>
          {shortName}
        </span>
        {filePath && shortName !== filePath && (
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)", wordBreak: "break-all" }}>
            {filePath}
          </span>
        )}
        <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)" }}>
          {lineRange}
        </span>
      </div>
    </ContentSection>
  );
}

// ── Write ────────────────────────────────────────────────────

export function WriteToolDisplay({ input }: { input: Record<string, unknown> }) {
  const filePath = typeof input.file_path === "string" ? input.file_path : null;
  const content  = typeof input.content   === "string" ? input.content  : null;

  return (
    <>
      {filePath && (
        <ContentSection label="File">
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--green)", wordBreak: "break-all" }}>
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

// ── Bash ─────────────────────────────────────────────────────

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

// ── Edit (diff view) ─────────────────────────────────────────

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
        border: "1px solid var(--border-1)", fontFamily: "var(--font-mono)", fontSize: "13px",
      }}>
        {/* Header bar */}
        <div style={{
          display: "flex", alignItems: "center", justifyContent: "space-between",
          padding: "8px 14px", background: "var(--bg-2)", borderBottom: "1px solid var(--border-1)",
        }}>
          <span title={filePath ?? undefined} style={{
            color: "var(--text-2)", fontSize: "12px",
            overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: "65%",
          }}>
            {fileName}
          </span>
          <div style={{ display: "flex", alignItems: "center", gap: "12px", flexShrink: 0 }}>
            {removedLines.length > 0 && (
              <span style={{ color: "var(--red)", opacity: 0.75, fontSize: "12px" }}>&minus;{removedLines.length}</span>
            )}
            {addedLines.length > 0 && (
              <span style={{ color: "var(--green)", opacity: 0.75, fontSize: "12px" }}>+{addedLines.length}</span>
            )}
            {replaceAll && (
              <span style={{
                padding: "1px 7px", borderRadius: "var(--radius-sm)",
                border: "1px solid rgba(255,181,71,0.25)", fontSize: "10px",
                letterSpacing: "0.06em", textTransform: "uppercase", color: "var(--amber)", opacity: 0.85,
              }}>all</span>
            )}
          </div>
        </div>

        {/* Diff lines */}
        <div style={{ maxHeight: "480px", overflowY: "auto" }}>
          {removedLines.map((line, i) => (
            <div key={`r${i}`} style={{
              display: "flex", alignItems: "flex-start",
              background: "rgba(255,69,69,0.055)", borderLeft: "2px solid rgba(255,69,69,0.38)",
            }}>
              <span style={{ width: "34px", flexShrink: 0, textAlign: "center", padding: "3px 0", lineHeight: 1.6, color: "rgba(255,100,100,0.55)", userSelect: "none", fontSize: "12px" }}>&minus;</span>
              <span style={{ flex: 1, padding: "3px 14px 3px 0", color: "var(--text-3)", lineHeight: 1.6, whiteSpace: "pre-wrap", wordBreak: "break-all" }}>{line || "\u00a0"}</span>
            </div>
          ))}
          {removedLines.length > 0 && addedLines.length > 0 && (
            <div style={{ height: "1px", background: "var(--border-1)" }} />
          )}
          {addedLines.map((line, i) => (
            <div key={`a${i}`} style={{
              display: "flex", alignItems: "flex-start",
              background: "rgba(52, 211, 153, 0.04)", borderLeft: "2px solid rgba(52, 211, 153, 0.3)",
            }}>
              <span style={{ width: "34px", flexShrink: 0, textAlign: "center", padding: "3px 0", lineHeight: 1.6, color: "rgba(0,220,100,0.5)", userSelect: "none", fontSize: "12px" }}>+</span>
              <span style={{ flex: 1, padding: "3px 14px 3px 0", color: "var(--text-1)", lineHeight: 1.6, whiteSpace: "pre-wrap", wordBreak: "break-all" }}>{line || "\u00a0"}</span>
            </div>
          ))}
        </div>
      </div>
    </ContentSection>
  );
}

// ── TaskCreate ───────────────────────────────────────────────

export function TaskCreateDisplay({ input }: { input: Record<string, unknown> }) {
  const subject     = typeof input.subject     === "string" ? input.subject     : null;
  const description = typeof input.description === "string" ? input.description : null;
  const activeForm  = typeof input.activeForm  === "string" ? input.activeForm  : null;

  return (
    <>
      {subject && (
        <ContentSection label="Subject">
          <p style={{ fontFamily: "var(--font-sans)", fontSize: "16px", fontWeight: 600, color: "var(--text-1)", margin: 0, lineHeight: 1.4 }}>
            {subject}
          </p>
        </ContentSection>
      )}
      {activeForm && (
        <ContentSection label="Active Form" color="var(--cyan)">
          <span style={{ display: "inline-flex", alignItems: "center", gap: "6px", fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--cyan)" }}>
            <span style={{ opacity: 0.5 }}>&#10227;</span> {activeForm}
          </span>
        </ContentSection>
      )}
      {description && (
        <ContentSection label="Description">
          <p style={{ fontFamily: "var(--font-sans)", fontSize: "14px", lineHeight: 1.65, color: "var(--text-2)", margin: 0 }}>
            {description}
          </p>
        </ContentSection>
      )}
    </>
  );
}

// ── Generic input ────────────────────────────────────────────

export function GenericToolInput({ input, toolName }: { input: Record<string, unknown>; toolName?: string }) {
  const entries = Object.entries(input);
  if (entries.length === 0) return null;

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
              <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)", flexShrink: 0, minWidth: "80px" }}>{key}</span>
              <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-1)", wordBreak: "break-all" }}>{String(value)}</span>
            </div>
          ))}
        </div>
      </ContentSection>
    );
  }

  return (
    <ContentSection label="Input" color="var(--amber)">
      <CodeRender content={JSON.stringify(input, null, 2)} />
    </ContentSection>
  );
}
