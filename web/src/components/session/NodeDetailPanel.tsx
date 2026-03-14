import { useEffect, useState } from "react";
import type { ContentBlock, NodeResponse } from "@/lib/types";
import { getNodeMeta, getThinkingText, getTokenUsage } from "@/lib/node-meta";
import { shortId } from "@/lib/utils";
import { CodeRender } from "@/components/ui/CodeRender";
import { MarkdownContent } from "@/components/ui/MarkdownContent";
import { EmptyState } from "@/components/ui/EmptyState";
import {
  ContentSection,
  TokenFooter,
  EmptyResult,
  WriteToolDisplay,
  BashToolDisplay,
  EditToolDisplay,
  TaskCreateDisplay,
  SerenaResultDisplay,
  parseSerenaResult,
  stripLineNumbers,
} from "./tool-displays";

interface NodeDetailPanelProps {
  node: NodeResponse | null;
  flatNodes?: NodeResponse[];
  onNavigate?: (node: NodeResponse) => void;
}

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

// ── Parse <task-notification> XML from user nodes ────────────
interface TaskNotification {
  taskId: string | null;
  status: string | null;
  summary: string | null;
  result: string | null;
  totalTokens: string | null;
  toolUses: string | null;
  durationMs: string | null;
}

function parseTaskNotification(content: string): TaskNotification | null {
  if (!content.includes("<task-notification>")) return null;
  const extract = (tag: string): string | null => {
    const m = content.match(new RegExp(`<${tag}>([\\s\\S]*?)</${tag}>`));
    return m?.[1]?.trim() ?? null;
  };
  return {
    taskId: extract("task-id"),
    status: extract("status"),
    summary: extract("summary"),
    result: extract("result"),
    totalTokens: extract("total_tokens"),
    toolUses: extract("tool_uses"),
    durationMs: extract("duration_ms"),
  };
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
  const isEditCall       = isToolCall && toolNameLc === "edit";
  const isBashCall       = isToolCall && toolNameLc === "bash";

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
  type ImageData = { mediaType: string; data: string };
  const extractedImages: ImageData[] = [];

  // From message content blocks
  for (const img of imageBlocks) {
    if (img.source?.data && img.source?.media_type) {
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
        if (src?.data && src?.media_type) {
          extractedImages.push({ mediaType: src.media_type, data: src.data });
        }
      }
    }
  };

  tryExtractImagesFromContent(toolResultBlock?.content);
  tryExtractImagesFromContent(topLevelToolResult?.content);

  // ── Find linked node (tool call ↔ result) ──────────────────
  const linkedNode: NodeResponse | null = (() => {
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
  })();

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
                <span style={{ fontFamily: "var(--font-mono)", fontSize: "16px", fontWeight: 600, color: "var(--amber)" }}>
                  {toolCallName}
                </span>
              </ContentSection>

              {isWriteCall && toolCallInput ? (
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
                  <ContentSection label="Input" color="var(--amber)">
                    <CodeRender content={JSON.stringify(toolCallInput, null, 2)} />
                  </ContentSection>
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
                  const serena = parseSerenaResult(displayResultContent);
                  if (serena) return <SerenaResultDisplay content={displayResultContent} />;
                  // Render markdown-like results as markdown
                  if (looksLikeMarkdown(displayResultContent)) {
                    return <MarkdownContent text={displayResultContent} />;
                  }
                  const resultFilePath =
                    (typeof toolCallInput?.file_path === "string" ? toolCallInput.file_path : undefined)
                    ?? node.file_paths?.[0];
                  return (
                    <CodeRender
                      content={displayResultContent}
                      filePath={resultFilePath}
                      error={effectiveIsError}
                    />
                  );
                })()
              ) : topLevelToolResult?.file ? (
                <div>
                  {topLevelToolResult.file.file_path && (
                    <div style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", marginBottom: "8px" }}>
                      {topLevelToolResult.file.file_path}
                      {topLevelToolResult.file.num_lines != null && ` \u00b7 ${topLevelToolResult.file.num_lines} lines`}
                    </div>
                  )}
                  {topLevelToolResult.file.content ? (
                    <CodeRender
                      content={topLevelToolResult.file.content}
                      filePath={topLevelToolResult.file.file_path ?? undefined}
                    />
                  ) : (
                    <EmptyResult />
                  )}
                </div>
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
                return <TaskNotificationDisplay task={taskNotif} />;
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

// ── Image preview with lightbox ──────────────────────────────
function ImagePreview({ img, index }: { img: { mediaType: string; data: string }; index: number }) {
  const [lightbox, setLightbox] = useState(false);
  const [dims, setDims] = useState<{ w: number; h: number } | null>(null);
  const src = `data:${img.mediaType};base64,${img.data}`;
  const sizeKb = Math.round(img.data.length * 0.75 / 1024);

  return (
    <>
      <div style={{
        borderRadius: "var(--radius-md)",
        overflow: "hidden",
        border: "1px solid var(--border-1)",
        background: "var(--bg-2)",
      }}>
        <img
          src={src}
          alt={`Image ${index + 1}`}
          onLoad={(e) => {
            const el = e.currentTarget;
            setDims({ w: el.naturalWidth, h: el.naturalHeight });
          }}
          style={{
            display: "block",
            maxWidth: "100%",
            maxHeight: "300px",
            objectFit: "contain",
            margin: "0 auto",
            background: "var(--bg-1)",
            imageRendering: "auto",
          }}
        />
        <div style={{
          padding: "4px 10px",
          fontFamily: "var(--font-mono)", fontSize: "9px",
          color: "var(--text-3)", borderTop: "1px solid var(--border-1)",
          display: "flex", justifyContent: "space-between", alignItems: "center",
        }}>
          <span>{img.mediaType} · {sizeKb}KB{dims ? ` · ${dims.w}×${dims.h}` : ""}</span>
          <button
            onClick={() => setLightbox(true)}
            style={{
              fontFamily: "var(--font-mono)", fontSize: "9px", fontWeight: 600,
              color: "var(--indigo)", background: "rgba(129,140,248,0.08)",
              border: "1px solid rgba(129,140,248,0.2)",
              borderRadius: "var(--radius-sm)",
              padding: "2px 8px", cursor: "pointer",
              transition: "all 0.1s",
            }}
          >
            EXPAND
          </button>
        </div>
      </div>

      {/* Lightbox overlay */}
      {lightbox && (
        <div
          onClick={() => setLightbox(false)}
          style={{
            position: "fixed", inset: 0,
            background: "rgba(0,0,0,0.85)",
            zIndex: 200,
            display: "flex", alignItems: "center", justifyContent: "center",
            cursor: "zoom-out",
            animation: "fadeIn 0.15s ease forwards",
          }}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              position: "relative",
              maxWidth: "95vw",
              maxHeight: "95vh",
              cursor: "default",
            }}
          >
            <img
              src={src}
              alt={`Image ${index + 1} full`}
              style={{
                display: "block",
                maxWidth: "95vw",
                maxHeight: "90vh",
                objectFit: "contain",
                borderRadius: "var(--radius-md)",
                boxShadow: "var(--shadow-xl)",
              }}
            />
            {/* Close + info bar */}
            <div style={{
              display: "flex", justifyContent: "space-between", alignItems: "center",
              marginTop: "8px", padding: "0 4px",
            }}>
              <span style={{
                fontFamily: "var(--font-mono)", fontSize: "10px", color: "rgba(255,255,255,0.5)",
              }}>
                {img.mediaType} · {sizeKb}KB{dims ? ` · ${dims.w}×${dims.h} (transcript resolution)` : ""}
              </span>
              <button
                onClick={() => setLightbox(false)}
                style={{
                  fontFamily: "var(--font-mono)", fontSize: "10px", fontWeight: 600,
                  color: "var(--text-1)", background: "rgba(255,255,255,0.1)",
                  border: "1px solid rgba(255,255,255,0.15)",
                  borderRadius: "var(--radius-sm)",
                  padding: "4px 12px", cursor: "pointer",
                }}
              >
                CLOSE
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

// ── Task notification display ────────────────────────────────
function TaskNotificationDisplay({ task }: { task: TaskNotification }) {
  const isComplete = task.status === "completed";
  const statusColor = isComplete ? "var(--emerald)" : "var(--amber)";

  return (
    <>
      {/* Header card */}
      <ContentSection label="Task Complete" color="var(--purple)">
        <div style={{
          padding: "14px 16px",
          background: "var(--bg-2)",
          border: "1px solid var(--border-1)",
          borderRadius: "var(--radius-md)",
          display: "flex", flexDirection: "column", gap: "10px",
        }}>
          {/* Status + summary */}
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
                <div style={{ fontFamily: "var(--font-mono)", fontSize: "11px" }}>
                  <span style={{ color: "var(--text-3)" }}>Tokens </span>
                  <span style={{ color: "var(--cyan)" }}>{Number(task.totalTokens).toLocaleString()}</span>
                </div>
              )}
              {task.toolUses && (
                <div style={{ fontFamily: "var(--font-mono)", fontSize: "11px" }}>
                  <span style={{ color: "var(--text-3)" }}>Tools </span>
                  <span style={{ color: "var(--amber)" }}>{task.toolUses}</span>
                </div>
              )}
              {task.durationMs && (
                <div style={{ fontFamily: "var(--font-mono)", fontSize: "11px" }}>
                  <span style={{ color: "var(--text-3)" }}>Duration </span>
                  <span style={{ color: "var(--text-2)" }}>{(Number(task.durationMs) / 1000).toFixed(1)}s</span>
                </div>
              )}
            </div>
          )}

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
