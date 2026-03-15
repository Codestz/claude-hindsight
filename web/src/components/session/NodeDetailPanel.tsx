import { useEffect, useMemo, useState } from "react";
import type { ContentBlock, NodeResponse } from "@/lib/types";
import { getNodeMeta, getThinkingText, getTokenUsage } from "@/lib/node-meta";
import { shortId } from "@/lib/utils";
import { CodeRender } from "@/components/ui/CodeRender";
import { MarkdownContent } from "@/components/ui/MarkdownContent";
import { EmptyState } from "@/components/ui/EmptyState";
import { ImagePreview } from "./ImagePreview";
import { TaskNotificationCard } from "./TaskNotificationCard";
import { parseTaskNotification } from "./utils";
import {
  ContentSection,
  TokenFooter,
  EmptyResult,
  WriteToolDisplay,
  BashToolDisplay,
  EditToolDisplay,
  TaskCreateDisplay,
  ReadToolDisplay,
  GenericToolInput,
  SerenaResultDisplay,
  parseSerenaResult,
  stripLineNumbers,
} from "./tool-displays";
import { extractTurFile, resolveFilePath } from "./tool-displays/resolve-file-path";

import type { NodeDetailPanelProps, ImageData } from "./types";

// ── Detect if content is markdown (not code) ─────────────────
function looksLikeMarkdown(text: string): boolean {
  const lines = text.split("\n").slice(0, 30);
  let mdSignals = 0;
  for (const line of lines) {
    if (/^#{1,4}\s/.test(line)) mdSignals += 2;     // headers
    if (/\*\*[^*]+\*\*/.test(line)) mdSignals++;     // bold
    if (/^[-*]\s/.test(line)) mdSignals++;            // list items
    if (/^\|.*\|/.test(line)) mdSignals++;            // table rows
    if (/\[.+\]\(.+\)/.test(line)) mdSignals++;      // links
    if (/^>\s/.test(line)) mdSignals++;               // blockquotes
  }
  return mdSignals >= 3;
}


/** Parse JSON arrays of {type:"text", text:"..."} into clean text */
function tryExtractTextArray(content: string): string | null {
  const trimmed = content.trim();
  if (!trimmed.startsWith("[")) return null;
  try {
    const arr = JSON.parse(trimmed);
    if (!Array.isArray(arr)) return null;
    const texts: string[] = [];
    let hasImages = false;
    for (const item of arr) {
      if (typeof item !== "object" || item === null) return null;
      if (item.type === "text" && typeof item.text === "string") {
        texts.push(item.text);
      } else if (item.type === "image") {
        hasImages = true;
      } else {
        return null; // unknown block type — don't parse
      }
    }
    if (texts.length === 0 && !hasImages) return null;
    return texts.join("\n\n") || null;
  } catch {
    return null;
  }
}

export function NodeDetailPanel({ node, flatNodes, onNavigate }: NodeDetailPanelProps) {
  if (!node) {
    return (
      <EmptyState
        icon="◎"
        title="Select a node to inspect"
        description="Click any row in the execution list to view its details here."
      />
    );
  }

  return <PanelContent node={node} flatNodes={flatNodes} onNavigate={onNavigate} />;
}

function PanelContent({ node, flatNodes, onNavigate }: { node: NodeResponse; flatNodes?: NodeResponse[]; onNavigate?: (node: NodeResponse) => void }) {
  const [showRaw, setShowRaw] = useState(false);
  const meta = getNodeMeta(node);
  const tokenUsage = getTokenUsage(node);

  // Reset to UI mode when node changes
  useEffect(() => { setShowRaw(false); }, [node.uuid]);

  const isThinking   = node.node_type === "assistant" && node.color === "magenta";
  const isToolCall   = node.node_type === "assistant" && node.color === "yellow";
  const isToolResult = node.node_type === "user"      && node.color === "blue";

  const contentBlocks = Array.isArray(node.message?.content)
    ? (node.message!.content as ContentBlock[])
    : [];

  const textBlocks  = contentBlocks.filter((b) => b.type === "text") as { type: "text"; text: string }[];
  const thinkBlocks = contentBlocks.filter((b) => b.type === "thinking") as { type: "thinking"; thinking: string }[];
  const imageBlocks = contentBlocks.filter((b) => b.type === "image") as {
    type: "image"; source: { type: string; media_type: string; data: string };
  }[];
  const toolUseBlock  = contentBlocks.find((b) => b.type === "tool_use") as
    | { type: "tool_use"; id: string; name: string; input: Record<string, unknown> }
    | undefined;
  const toolResultBlock = contentBlocks.find((b) => b.type === "tool_result") as
    | { type: "tool_result"; tool_use_id: string; content: unknown; is_error: boolean | null }
    | undefined;

  const legacyText = typeof node.message?.content === "string" ? node.message.content : null;

  const topLevelToolUse    = node.tool_use;
  const topLevelToolResult = node.tool_result;

  const toolCallName  = toolUseBlock?.name  ?? topLevelToolUse?.name ?? node.tool_name;
  const toolCallInput = toolUseBlock?.input ?? (topLevelToolUse?.input as Record<string, unknown> | undefined);

  const toolNameLc       = toolCallName?.toLowerCase() ?? "";
  const isTaskCall       = isToolCall && ["task", "todowrite"].includes(toolNameLc);
  const isTaskCreateCall = isToolCall && toolNameLc === "taskcreate";
  const isWriteCall      = isToolCall && toolNameLc === "write";
  const isReadCall       = isToolCall && toolNameLc === "read";
  const isEditCall       = isToolCall && toolNameLc === "edit";
  const isBashCall       = isToolCall && toolNameLc === "bash";

  // MCP tool name parsing: mcp__server__tool → { server, shortTool }
  const mcpParts = toolCallName?.match(/^mcp__([^_]+(?:__[^_]+)*)__([^_]+(?:__[^_]+)*)$/);
  const mcpServer = mcpParts?.[1]?.replace(/__/g, "-") ?? null;
  const mcpShortTool = mcpParts?.[2]?.replace(/__/g, ".") ?? null;
  const displayToolName = mcpShortTool ?? toolCallName;

  // File path resolution (extracted to tool-displays/resolve-file-path.ts)
  const turFile = extractTurFile(node);
  const resolvedFilePath = resolveFilePath(node, turFile, toolCallInput);

  // Tool result content & error detection
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

  const hasCleanFile = !!topLevelToolResult?.file?.content || !!turFile?.content;

  const displayResultContent: string | null = hasCleanFile
    ? null
    : (() => {
        let raw: string | null = null;
        if (toolResultBlock?.content != null) {
          if (typeof toolResultBlock.content === "string") {
            raw = toolResultBlock.content;
          } else if (typeof toolResultBlock.content === "object" && !Array.isArray(toolResultBlock.content)) {
            // Content object with nested content string (e.g., Read result: {content, filePath, numLines})
            const obj = toolResultBlock.content as Record<string, unknown>;
            if (typeof obj.content === "string") {
              raw = obj.content;
            } else {
              raw = JSON.stringify(toolResultBlock.content, null, 2);
            }
          } else {
            raw = JSON.stringify(toolResultBlock.content, null, 2);
          }
        } else if (topLevelToolResult?.content != null) {
          raw = topLevelToolResult.content;
        }
        if (raw == null) return null;
        raw = stripLineNumbers(raw);
        raw = raw.replace(/<tool_use_error>([\s\S]*?)<\/tool_use_error>/g, "$1").trim();
        return raw || null;
      })();

  const bodyText =
    legacyText ??
    (textBlocks.length > 0 ? textBlocks.map((b) => b.text).join("\n\n") : null) ??
    node.summary ??
    "";

  const thinkingText =
    getThinkingText(node) ??
    (thinkBlocks.length > 0 ? thinkBlocks.map((b) => b.thinking).join("\n\n") : null) ??
    null;

  const progress = node.progress;

  // ── Extract images from all sources ────────────────────────
  const extractedImages: ImageData[] = [];

  // Safe media types for data: URI rendering (SVG can execute scripts)
  const SAFE_IMAGE_TYPES = new Set(["image/jpeg", "image/png", "image/gif", "image/webp"]);

  // From message content blocks
  for (const img of imageBlocks) {
    if (img.source?.data && img.source?.media_type && SAFE_IMAGE_TYPES.has(img.source.media_type)) {
      extractedImages.push({ mediaType: img.source.media_type, data: img.source.data });
    }
  }

  // From tool result content (JSON array with image objects)
  const tryExtractImagesFromContent = (content: unknown) => {
    if (!content) return;
    let arr: unknown[] | null = null;
    if (Array.isArray(content)) {
      arr = content;
    } else if (typeof content === "string") {
      try {
        const parsed = JSON.parse(content);
        if (Array.isArray(parsed)) arr = parsed;
      } catch { /* not JSON */ }
    }
    if (!arr) return;
    for (const item of arr) {
      if (typeof item === "object" && item !== null && (item as any).type === "image") {
        const src = (item as any).source;
        if (src?.data && src?.media_type && SAFE_IMAGE_TYPES.has(src.media_type)) {
          extractedImages.push({ mediaType: src.media_type, data: src.data });
        }
      }
    }
  };

  tryExtractImagesFromContent(toolResultBlock?.content);
  tryExtractImagesFromContent(topLevelToolResult?.content);

  // ── Find linked node (tool call ↔ result) ──────────────────
  const linkedNode: NodeResponse | null = useMemo(() => {
    if (!flatNodes) return null;
    if (isToolCall) {
      // Find the result node with matching tool_use_id
      const tuId = toolUseBlock?.id ?? topLevelToolUse?.id;
      if (!tuId) return null;
      return flatNodes.find((n) =>
        n.node_type === "user" && n.color === "blue" && (
          n.tool_result?.tool_use_id === tuId ||
          (Array.isArray(n.message?.content) &&
            n.message!.content.some((b: any) => b.type === "tool_result" && b.tool_use_id === tuId))
        )
      ) ?? null;
    }
    if (isToolResult) {
      // Find the call node with matching tool_use_id
      const tuId = toolResultBlock?.tool_use_id ?? topLevelToolResult?.tool_use_id;
      if (!tuId) return null;
      return flatNodes.find((n) =>
        n.node_type === "assistant" && n.color === "yellow" && (
          n.tool_use?.id === tuId ||
          (Array.isArray(n.message?.content) &&
            n.message!.content.some((b: any) => b.type === "tool_use" && b.id === tuId))
        )
      ) ?? null;
    }
    return null;
  }, [node.uuid, flatNodes, isToolCall, isToolResult, toolUseBlock?.id, topLevelToolUse?.id, toolResultBlock?.tool_use_id, topLevelToolResult?.tool_use_id]);

  return (
    <div style={{ padding: "24px 28px" }}>
      {/* Header */}
      <div style={{
        display: "flex", alignItems: "center", gap: "10px",
        marginBottom: "20px", paddingBottom: "14px",
        borderBottom: "1px solid var(--border-1)",
      }}>
        <span style={{
          width: "8px", height: "8px", borderRadius: "50%",
          background: meta.color, flexShrink: 0,
        }} />
        <span style={{
          fontFamily: "var(--font-mono)", fontSize: "12px", fontWeight: 600,
          color: meta.color,
        }}>
          {meta.badge}
        </span>

        {node.uuid && (
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)" }}>
            {shortId(node.uuid)}
          </span>
        )}

        <span style={{ flex: 1 }} />

        {node.timestamp != null && (
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)" }}>
            {new Date(node.timestamp).toLocaleTimeString("en-US", {
              hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit",
            })}
          </span>
        )}

        <button
          onClick={() => setShowRaw((r) => !r)}
          style={{
            padding: "2px 8px",
            background: showRaw ? "var(--bg-3)" : "transparent",
            border: "1px solid var(--border-1)",
            borderRadius: "var(--radius-sm)",
            fontFamily: "var(--font-mono)", fontSize: "9px",
            letterSpacing: "0.06em", textTransform: "uppercase",
            color: showRaw ? "var(--text-1)" : "var(--text-3)",
            cursor: "pointer", transition: "all 0.1s",
          }}
        >
          {showRaw ? "UI" : "RAW"}
        </button>

        {/* Linked node navigation */}
        {linkedNode && onNavigate && (
          <button
            onClick={() => onNavigate(linkedNode)}
            style={{
              padding: "2px 8px",
              background: "transparent",
              border: "1px solid var(--border-1)",
              borderRadius: "var(--radius-sm)",
              fontFamily: "var(--font-mono)", fontSize: "9px",
              letterSpacing: "0.06em",
              color: "var(--indigo)",
              cursor: "pointer", transition: "all 0.1s",
              display: "flex", alignItems: "center", gap: "4px",
            }}
            title={isToolCall ? "Jump to result" : "Jump to call"}
          >
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
          {/* Thinking content */}
          {(isThinking || thinkingText) && thinkingText && (
            <ContentSection label="Thinking" color="var(--purple)">
              <div style={{ display: "flex", gap: "5px", marginBottom: "14px", alignItems: "center" }}>
                {[0, 0.3, 0.6].map((delay, i) => (
                  <span key={i} style={{
                    display: "inline-block", width: "6px", height: "6px",
                    borderRadius: "50%", background: "var(--purple)",
                    animation: `thinking-pulse-dot 1.5s ease-in-out ${delay}s infinite`,
                  }} />
                ))}
                <span style={{
                  fontFamily: "var(--font-mono)", fontSize: "10px", letterSpacing: "0.1em",
                  textTransform: "uppercase", color: "var(--purple)", opacity: 0.6, marginLeft: "8px",
                }}>Processing</span>
              </div>
              <p style={{
                fontFamily: "var(--font-sans)", fontSize: "14px", lineHeight: 1.75,
                color: "var(--purple)", fontStyle: "italic", margin: 0,
                whiteSpace: "pre-wrap", wordBreak: "break-word",
                animation: "thinking-breathe 3s ease-in-out infinite",
                opacity: 0.85,
              }}>
                {thinkingText}
              </p>
            </ContentSection>
          )}

          {isThinking && !thinkingText && textBlocks.length > 0 && (
            <ContentSection label="Response">
              <MarkdownContent text={textBlocks.map((b) => b.text).join("\n\n")} />
            </ContentSection>
          )}

          {/* Tool call */}
          {isToolCall && toolCallName && (
            <>
              <ContentSection label="Tool">
                <div style={{ display: "flex", alignItems: "baseline", gap: "8px" }}>
                  <span style={{ fontFamily: "var(--font-mono)", fontSize: "16px", fontWeight: 600, color: "var(--amber)" }}>
                    {displayToolName}
                  </span>
                  {mcpServer && (
                    <span style={{
                      fontFamily: "var(--font-mono)", fontSize: "10px",
                      color: "var(--text-3)", background: "var(--bg-3)",
                      padding: "1px 6px", borderRadius: "var(--radius-sm)",
                      border: "1px solid var(--border-1)",
                    }}>
                      {mcpServer}
                    </span>
                  )}
                </div>
              </ContentSection>

              {isReadCall && toolCallInput ? (
                <ReadToolDisplay input={toolCallInput} />
              ) : isWriteCall && toolCallInput ? (
                <WriteToolDisplay input={toolCallInput} />
              ) : isEditCall && toolCallInput ? (
                <EditToolDisplay input={toolCallInput} />
              ) : isTaskCreateCall && toolCallInput ? (
                <TaskCreateDisplay input={toolCallInput} />
              ) : isBashCall && toolCallInput ? (
                <BashToolDisplay input={toolCallInput} />
              ) : isTaskCall && toolCallInput ? (
                <>
                  {toolCallInput.subagent_type && (
                    <ContentSection label="Agent Type">
                      <span style={{
                        display: "inline-flex", alignItems: "center",
                        padding: "4px 12px", borderRadius: "var(--radius-md)",
                        background: "rgba(160,90,255,0.12)",
                        border: "1px solid rgba(160,90,255,0.25)",
                        fontFamily: "var(--font-mono)", fontSize: "13px",
                        fontWeight: 600, color: "var(--purple)",
                      }}>
                        &#10227; {String(toolCallInput.subagent_type)}
                      </span>
                    </ContentSection>
                  )}
                  {toolCallInput.description && (
                    <ContentSection label="Description">
                      <p style={{
                        fontFamily: "var(--font-sans)", fontSize: "14px",
                        lineHeight: 1.6, color: "var(--text-2)", margin: 0,
                      }}>
                        {String(toolCallInput.description)}
                      </p>
                    </ContentSection>
                  )}
                  {toolCallInput.prompt && (
                    <ContentSection label="Prompt" color="var(--amber)">
                      <MarkdownContent text={String(toolCallInput.prompt)} />
                    </ContentSection>
                  )}
                </>
              ) : (
                toolCallInput && Object.keys(toolCallInput).length > 0 && (
                  <GenericToolInput input={toolCallInput} toolName={toolCallName} />
                )
              )}
            </>
          )}

          {/* Tool result */}
          {isToolResult && (
            <ContentSection
              label={effectiveIsError ? "Error Output" : "Output"}
              color={effectiveIsError ? "var(--red)" : "var(--cyan)"}
            >
              {/* If images were extracted from the result, skip code render for the image JSON */}
              {extractedImages.length > 0 && displayResultContent != null ? (
                <div style={{
                  fontFamily: "var(--font-mono)", fontSize: "12px",
                  color: "var(--text-3)", padding: "8px 0",
                }}>
                  {extractedImages.length} image{extractedImages.length > 1 ? "s" : ""} attached — see preview below
                </div>
              ) : displayResultContent != null ? (
                (() => {
                  if (displayResultContent === "") return <EmptyResult />;

                  // Parse [{type:"text", text:"..."}] arrays into clean text
                  const textArrayContent = tryExtractTextArray(displayResultContent);
                  if (textArrayContent) {
                    if (looksLikeMarkdown(textArrayContent)) {
                      return <MarkdownContent text={textArrayContent} />;
                    }
                    return (
                      <div style={{
                        fontFamily: "var(--font-sans)", fontSize: "14px",
                        lineHeight: 1.65, color: "var(--text-2)",
                        whiteSpace: "pre-wrap", wordBreak: "break-word",
                      }}>
                        {textArrayContent}
                      </div>
                    );
                  }

                  const serena = parseSerenaResult(displayResultContent);
                  if (serena) return <SerenaResultDisplay content={displayResultContent} />;
                  if (looksLikeMarkdown(displayResultContent)) {
                    return <MarkdownContent text={displayResultContent} />;
                  }
                  return (
                    <CodeRender
                      content={displayResultContent}
                      filePath={resolvedFilePath}
                      error={effectiveIsError}
                    />
                  );
                })()
              ) : (topLevelToolResult?.file || turFile) ? (
                (() => {
                  const f = topLevelToolResult?.file ?? turFile;
                  const fp = f?.file_path ?? (f as any)?.filePath ?? resolvedFilePath;
                  const content = f?.content ?? "";
                  const numLines = (f as any)?.numLines ?? (f as any)?.num_lines;
                  const startLine = (f as any)?.startLine;
                  const lineInfo = startLine != null && numLines != null
                    ? `lines ${startLine}–${startLine + numLines}`
                    : numLines != null ? `${numLines} lines` : null;
                  return (
                    <div>
                      {fp && (
                        <div style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", marginBottom: "8px" }}>
                          {fp}
                          {lineInfo && ` \u00b7 ${lineInfo}`}
                        </div>
                      )}
                      {content ? (
                        <CodeRender content={content} filePath={fp ?? undefined} />
                      ) : (
                        <EmptyResult />
                      )}
                    </div>
                  );
                })()
              ) : (
                <EmptyResult label={node.summary || node.label || undefined} />
              )}
              {topLevelToolResult?.duration_ms != null && (
                <div style={{ marginTop: "10px", fontSize: "12px", fontFamily: "var(--font-mono)", color: "var(--text-3)" }}>
                  {topLevelToolResult.duration_ms}ms
                </div>
              )}
            </ContentSection>
          )}

          {/* Images (from any node type) */}
          {extractedImages.length > 0 && (
            <ContentSection
              label={`Image${extractedImages.length > 1 ? `s (${extractedImages.length})` : ""}`}
              color="var(--purple)"
            >
              <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
                {extractedImages.map((img, i) => (
                  <ImagePreview key={i} img={img} index={i} />
                ))}
              </div>
            </ContentSection>
          )}

          {/* Progress */}
          {node.node_type === "progress" && (
            <ContentSection label="Progress" color="var(--amber)">
              {progress?.message && (
                <p style={{ fontFamily: "var(--font-sans)", fontSize: "14px", color: "var(--text-2)", margin: 0 }}>
                  {progress.message}
                </p>
              )}
              {progress?.percentage != null && (
                <div style={{ marginTop: "8px" }}>
                  <div style={{ height: "4px", background: "var(--bg-3)", borderRadius: "2px", overflow: "hidden" }}>
                    <div style={{ height: "100%", width: `${progress.percentage}%`, background: "var(--amber)" }} />
                  </div>
                </div>
              )}
              {!progress?.message && (
                <div style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--text-3)" }}>
                  {node.label}
                </div>
              )}
            </ContentSection>
          )}

          {/* File history snapshot */}
          {node.node_type === "file-history-snapshot" && (() => {
            const snap = (node as NodeResponse & { snapshot?: { trackedFileBackups?: Record<string, unknown>; timestamp?: string } }).snapshot;
            const files = snap?.trackedFileBackups ? Object.keys(snap.trackedFileBackups) : [];
            return (
              <ContentSection label="File Snapshot" color="var(--purple)">
                {snap?.timestamp && (
                  <div style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", marginBottom: "8px" }}>
                    {snap.timestamp}
                  </div>
                )}
                {files.length > 0 ? (
                  <div style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--text-2)" }}>
                    {files.map((f) => <div key={f}>{f}</div>)}
                  </div>
                ) : (
                  <div style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--text-3)" }}>
                    No files tracked at this point
                  </div>
                )}
              </ContentSection>
            );
          })()}

          {/* Plain text (user / assistant text, system, etc.) */}
          {!isThinking && !isToolCall && !isToolResult && node.node_type !== "progress" && node.node_type !== "file-history-snapshot" && (
            (() => {
              // Task notification from sub-agent completion
              const taskNotif = parseTaskNotification(bodyText);
              if (taskNotif) {
                return <TaskNotificationCard task={taskNotif} />;
              }
              return bodyText ? (
                <ContentSection label="Content">
                  <MarkdownContent text={bodyText} />
                </ContentSection>
              ) : (
                <ContentSection label="Details">
                  <div style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--text-3)" }}>
                    {node.label}
                  </div>
                </ContentSection>
              );
            })()
          )}
        </>
      )}

      {tokenUsage && <TokenFooter usage={tokenUsage} />}
    </div>
  );
}

// ImagePreview and TaskNotificationCard are imported from separate files

