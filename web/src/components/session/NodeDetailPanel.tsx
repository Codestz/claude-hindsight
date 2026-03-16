/**
 * Node detail panel — renders the full content of a selected node.
 *
 * Uses useNodeContent hook for all content parsing/extraction.
 * This component is purely rendering — no parsing logic inline.
 */

import { useEffect, useState } from "react";
import { shortId } from "@/lib/utils";
import { CodeRender } from "@/components/ui/CodeRender";
import { MarkdownContent } from "@/components/ui/MarkdownContent";
import { EmptyState } from "@/components/ui/EmptyState";
import { ImagePreview } from "./ImagePreview";
import { TaskNotificationCard } from "./TaskNotificationCard";
import { parseTaskNotification } from "./utils";
import { useNodeContent } from "@/hooks/useNodeContent";
import {
  ContentSection, TokenFooter, EmptyResult,
  WriteToolDisplay, BashToolDisplay, EditToolDisplay,
  TaskCreateDisplay, ReadToolDisplay, GenericToolInput,
  SerenaResultDisplay, parseSerenaResult,
} from "./tool-displays";
import type { NodeDetailPanelProps } from "./types";

// ── Content detection helpers ────────────────────────────────

function looksLikeMarkdown(text: string): boolean {
  const lines = text.split("\n").slice(0, 30);
  let mdSignals = 0;
  for (const line of lines) {
    if (/^#{1,4}\s/.test(line)) mdSignals += 2;
    if (/\*\*[^*]+\*\*/.test(line)) mdSignals++;
    if (/^[-*]\s/.test(line)) mdSignals++;
    if (/^\|.*\|/.test(line)) mdSignals++;
    if (/\[.+\]\(.+\)/.test(line)) mdSignals++;
    if (/^>\s/.test(line)) mdSignals++;
  }
  return mdSignals >= 3;
}

function tryExtractTextArray(content: string): string | null {
  const trimmed = content.trim();
  if (!trimmed.startsWith("[")) return null;
  try {
    const arr = JSON.parse(trimmed);
    if (!Array.isArray(arr)) return null;
    const texts: string[] = [];
    for (const item of arr) {
      if (typeof item !== "object" || item === null) return null;
      if (item.type === "text" && typeof item.text === "string") texts.push(item.text);
      else if (item.type === "image") { /* skip */ }
      else return null;
    }
    return texts.length > 0 ? texts.join("\n\n") : null;
  } catch { return null; }
}

// ── Component ────────────────────────────────────────────────

export function NodeDetailPanel({ node, flatNodes, onNavigate }: NodeDetailPanelProps) {
  if (!node) {
    return <EmptyState icon="◎" title="Select a node to inspect" description="Click any row in the execution list to view its details here." />;
  }
  return <PanelContent node={node} flatNodes={flatNodes} onNavigate={onNavigate} />;
}

function PanelContent({ node, flatNodes, onNavigate }: { node: import("@/lib/types").NodeResponse; flatNodes?: import("@/lib/types").NodeResponse[]; onNavigate?: (node: import("@/lib/types").NodeResponse) => void }) {
  const [showRaw, setShowRaw] = useState(false);
  useEffect(() => { setShowRaw(false); }, [node.uuid]);

  const {
    meta, tokenUsage,
    isThinking, isToolCall, isToolResult,
    textBlocks, thinkBlocks,
    toolCallName, toolCallInput, displayToolName, mcpServer,
    isTaskCall, isTaskCreateCall, isWriteCall, isReadCall, isEditCall, isBashCall,
    turFile, resolvedFilePath,
    effectiveIsError, hasCleanFile, displayResultContent,
    bodyText, thinkingText,
    extractedImages, linkedNode,
    progress, topLevelToolResult,
  } = useNodeContent(node, flatNodes);

  return (
    <div style={{ padding: "24px 28px" }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "20px", paddingBottom: "14px", borderBottom: "1px solid var(--border-1)" }}>
        <span style={{ width: "8px", height: "8px", borderRadius: "50%", background: meta.color, flexShrink: 0 }} />
        <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", fontWeight: 600, color: meta.color }}>{meta.badge}</span>
        {node.uuid && <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)" }}>{shortId(node.uuid)}</span>}
        <span style={{ flex: 1 }} />
        {node.timestamp != null && (
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)" }}>
            {new Date(node.timestamp).toLocaleTimeString("en-US", { hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit" })}
          </span>
        )}
        <button onClick={() => setShowRaw((r) => !r)} style={{ padding: "2px 8px", background: showRaw ? "var(--bg-3)" : "transparent", border: "1px solid var(--border-1)", borderRadius: "var(--radius-sm)", fontFamily: "var(--font-mono)", fontSize: "9px", letterSpacing: "0.06em", textTransform: "uppercase", color: showRaw ? "var(--text-1)" : "var(--text-3)", cursor: "pointer", transition: "all 0.1s" }}>
          {showRaw ? "UI" : "RAW"}
        </button>
        {linkedNode && onNavigate && (
          <button onClick={() => onNavigate(linkedNode)} title={isToolCall ? "Jump to result" : "Jump to call"} style={{ padding: "2px 8px", background: "transparent", border: "1px solid var(--border-1)", borderRadius: "var(--radius-sm)", fontFamily: "var(--font-mono)", fontSize: "9px", letterSpacing: "0.06em", color: "var(--indigo)", cursor: "pointer", transition: "all 0.1s", display: "flex", alignItems: "center", gap: "4px" }}>
            {isToolCall ? "\u2192 RESULT" : "\u2190 CALL"}
          </button>
        )}
      </div>

      {showRaw ? (
        <ContentSection label="Raw Node" color="var(--text-3)">
          <CodeRender content={JSON.stringify(node, null, 2)} />
        </ContentSection>
      ) : (
        <>
          {/* Thinking */}
          {(isThinking || thinkingText) && thinkingText && (
            <ContentSection label="Thinking" color="var(--purple)">
              <div style={{ display: "flex", gap: "5px", marginBottom: "14px", alignItems: "center" }}>
                {[0, 0.3, 0.6].map((delay, i) => (
                  <span key={i} style={{ display: "inline-block", width: "6px", height: "6px", borderRadius: "50%", background: "var(--purple)", animation: `thinking-pulse-dot 1.5s ease-in-out ${delay}s infinite` }} />
                ))}
                <span style={{ fontFamily: "var(--font-mono)", fontSize: "10px", letterSpacing: "0.1em", textTransform: "uppercase", color: "var(--purple)", opacity: 0.6, marginLeft: "8px" }}>Processing</span>
              </div>
              <p style={{ fontFamily: "var(--font-sans)", fontSize: "14px", lineHeight: 1.75, color: "var(--purple)", fontStyle: "italic", margin: 0, whiteSpace: "pre-wrap", wordBreak: "break-word", animation: "thinking-breathe 3s ease-in-out infinite", opacity: 0.85 }}>
                {thinkingText}
              </p>
            </ContentSection>
          )}
          {isThinking && !thinkingText && textBlocks.length > 0 && (
            <ContentSection label="Response"><MarkdownContent text={textBlocks.map((b) => b.text).join("\n\n")} /></ContentSection>
          )}

          {/* Tool call */}
          {isToolCall && toolCallName && (
            <>
              <ContentSection label="Tool">
                <div style={{ display: "flex", alignItems: "baseline", gap: "8px" }}>
                  <span style={{ fontFamily: "var(--font-mono)", fontSize: "16px", fontWeight: 600, color: "var(--amber)" }}>{displayToolName}</span>
                  {mcpServer && <span style={{ fontFamily: "var(--font-mono)", fontSize: "10px", color: "var(--text-3)", background: "var(--bg-3)", padding: "1px 6px", borderRadius: "var(--radius-sm)", border: "1px solid var(--border-1)" }}>{mcpServer}</span>}
                </div>
              </ContentSection>
              {isReadCall && toolCallInput ? <ReadToolDisplay input={toolCallInput} />
                : isWriteCall && toolCallInput ? <WriteToolDisplay input={toolCallInput} />
                : isEditCall && toolCallInput ? <EditToolDisplay input={toolCallInput} />
                : isTaskCreateCall && toolCallInput ? <TaskCreateDisplay input={toolCallInput} />
                : isBashCall && toolCallInput ? <BashToolDisplay input={toolCallInput} />
                : isTaskCall && toolCallInput ? (
                  <>
                    {toolCallInput.subagent_type && <ContentSection label="Agent Type"><span style={{ display: "inline-flex", alignItems: "center", padding: "4px 12px", borderRadius: "var(--radius-md)", background: "rgba(160,90,255,0.12)", border: "1px solid rgba(160,90,255,0.25)", fontFamily: "var(--font-mono)", fontSize: "13px", fontWeight: 600, color: "var(--purple)" }}>&#10227; {String(toolCallInput.subagent_type)}</span></ContentSection>}
                    {toolCallInput.description && <ContentSection label="Description"><p style={{ fontFamily: "var(--font-sans)", fontSize: "14px", lineHeight: 1.6, color: "var(--text-2)", margin: 0 }}>{String(toolCallInput.description)}</p></ContentSection>}
                    {toolCallInput.prompt && <ContentSection label="Prompt" color="var(--amber)"><MarkdownContent text={String(toolCallInput.prompt)} /></ContentSection>}
                  </>
                ) : toolCallInput && Object.keys(toolCallInput).length > 0 ? <GenericToolInput input={toolCallInput} toolName={toolCallName} /> : null}
            </>
          )}

          {/* Tool result */}
          {isToolResult && (
            <ContentSection label={effectiveIsError ? "Error Output" : "Output"} color={effectiveIsError ? "var(--red)" : "var(--cyan)"}>
              {extractedImages.length > 0 && displayResultContent != null ? (
                <div style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", padding: "8px 0" }}>
                  {extractedImages.length} image{extractedImages.length > 1 ? "s" : ""} attached — see preview below
                </div>
              ) : displayResultContent != null ? (() => {
                if (displayResultContent === "") return <EmptyResult />;
                const textArrayContent = tryExtractTextArray(displayResultContent);
                if (textArrayContent) return looksLikeMarkdown(textArrayContent) ? <MarkdownContent text={textArrayContent} /> : <div style={{ fontFamily: "var(--font-sans)", fontSize: "14px", lineHeight: 1.65, color: "var(--text-2)", whiteSpace: "pre-wrap", wordBreak: "break-word" }}>{textArrayContent}</div>;
                const serena = parseSerenaResult(displayResultContent);
                if (serena) return <SerenaResultDisplay content={displayResultContent} />;
                if (looksLikeMarkdown(displayResultContent)) return <MarkdownContent text={displayResultContent} />;
                return <CodeRender content={displayResultContent} filePath={resolvedFilePath} error={effectiveIsError} />;
              })() : (topLevelToolResult?.file || turFile) ? (() => {
                const f = topLevelToolResult?.file ?? turFile;
                const fp = f?.file_path ?? (f as any)?.filePath ?? resolvedFilePath;
                const content = f?.content ?? "";
                const numLines = (f as any)?.numLines ?? (f as any)?.num_lines;
                const startLine = (f as any)?.startLine;
                const lineInfo = startLine != null && numLines != null ? `lines ${startLine}–${startLine + numLines}` : numLines != null ? `${numLines} lines` : null;
                return (<div>{fp && <div style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", marginBottom: "8px" }}>{fp}{lineInfo && ` \u00b7 ${lineInfo}`}</div>}{content ? <CodeRender content={content} filePath={fp ?? undefined} /> : <EmptyResult />}</div>);
              })() : <EmptyResult label={node.summary || node.label || undefined} />}
              {topLevelToolResult?.duration_ms != null && <div style={{ marginTop: "10px", fontSize: "12px", fontFamily: "var(--font-mono)", color: "var(--text-3)" }}>{topLevelToolResult.duration_ms}ms</div>}
            </ContentSection>
          )}

          {/* Images */}
          {extractedImages.length > 0 && (
            <ContentSection label={`Image${extractedImages.length > 1 ? `s (${extractedImages.length})` : ""}`} color="var(--purple)">
              <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
                {extractedImages.map((img, i) => <ImagePreview key={i} img={img} index={i} />)}
              </div>
            </ContentSection>
          )}

          {/* Progress */}
          {node.node_type === "progress" && (
            <ContentSection label="Progress" color="var(--amber)">
              {progress?.message && <p style={{ fontFamily: "var(--font-sans)", fontSize: "14px", color: "var(--text-2)", margin: 0 }}>{progress.message}</p>}
              {progress?.percentage != null && <div style={{ marginTop: "8px" }}><div style={{ height: "4px", background: "var(--bg-3)", borderRadius: "2px", overflow: "hidden" }}><div style={{ height: "100%", width: `${progress.percentage}%`, background: "var(--amber)" }} /></div></div>}
              {!progress?.message && <div style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--text-3)" }}>{node.label}</div>}
            </ContentSection>
          )}

          {/* File snapshot */}
          {node.node_type === "file-history-snapshot" && (() => {
            const snap = (node as any).snapshot;
            const files = snap?.trackedFileBackups ? Object.keys(snap.trackedFileBackups) : [];
            return (<ContentSection label="File Snapshot" color="var(--purple)">{snap?.timestamp && <div style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", marginBottom: "8px" }}>{snap.timestamp}</div>}{files.length > 0 ? <div style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--text-2)" }}>{files.map((f: string) => <div key={f}>{f}</div>)}</div> : <div style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--text-3)" }}>No files tracked</div>}</ContentSection>);
          })()}

          {/* Plain text / Task notification */}
          {!isThinking && !isToolCall && !isToolResult && node.node_type !== "progress" && node.node_type !== "file-history-snapshot" && (() => {
            const taskNotif = parseTaskNotification(bodyText);
            if (taskNotif) return <TaskNotificationCard task={taskNotif} />;
            return bodyText ? <ContentSection label="Content"><MarkdownContent text={bodyText} /></ContentSection>
              : <ContentSection label="Details"><div style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--text-3)" }}>{node.label}</div></ContentSection>;
          })()}
        </>
      )}

      {tokenUsage && <TokenFooter usage={tokenUsage} />}
    </div>
  );
}
