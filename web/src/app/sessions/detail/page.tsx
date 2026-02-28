"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { api } from "@/lib/api";
import type { ContentBlock, NodeResponse, SessionFile, TokenUsage } from "@/lib/types";
import { getNodeMeta, getThinkingText, getTokenUsage } from "@/lib/node-meta";
import { formatBytes, shortId, shortModel, timeAgo } from "@/lib/utils";
import { NodeTree, type NodeFilter } from "@/components/nodes/NodeTree";
import { CodeRender } from "@/components/ui/CodeRender";

// ── Root export ───────────────────────────────────────────────
// Reads the session ID from window.location.pathname at runtime.
// The Rust server serves this page for any /sessions/:id/ path.
export default function SessionDetailPage() {
  const [id, setId] = useState<string | null>(null);

  useEffect(() => {
    // pathname looks like /sessions/abc123/ — extract the second segment
    const segments = window.location.pathname.split("/").filter(Boolean);
    // segments[0] = "sessions", segments[1] = session id
    if (segments.length >= 2) {
      setId(decodeURIComponent(segments[1]));
    }
  }, []);

  if (!id) {
    return <FullPageMessage>Loading…</FullPageMessage>;
  }

  return <SessionDetail id={id} />;
}

// ── Session detail — full viewport split layout ───────────────
function SessionDetail({ id }: { id: string }) {
  const [session, setSession] = useState<SessionFile | null>(null);
  const [roots, setRoots] = useState<NodeResponse[]>([]);
  const [selected, setSelected] = useState<NodeResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filterTypes, setFilterTypes] = useState<Set<string>>(new Set());
  const [filterKeyword, setFilterKeyword] = useState("");

  const nodeFilter: NodeFilter = { types: filterTypes, keyword: filterKeyword };

  const toggleFilterType = (type: string) => {
    setFilterTypes((prev) => {
      const next = new Set(prev);
      if (next.has(type)) next.delete(type);
      else next.add(type);
      return next;
    });
  };

  useEffect(() => {
    Promise.all([api.session(id), api.sessionNodes(id)])
      .then(([s, tree]) => {
        setSession(s);
        setRoots(tree.roots);
        const firstUser = findFirst(tree.roots, (n) => n.node_type === "user");
        setSelected(firstUser ?? tree.roots[0] ?? null);
      })
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, [id]);

  // Live SSE subscription — appends new nodes as they arrive from the server.
  // Each session tab connects to its own stream so multiple concurrent sessions work.
  useEffect(() => {
    const seenUuids = new Set<string>();
    const es = new EventSource(api.eventsUrl(id));

    es.onmessage = (evt) => {
      try {
        const node: NodeResponse = JSON.parse(evt.data);
        const uid = node.uuid ?? node.node_type + Math.random();
        if (seenUuids.has(uid)) return;
        seenUuids.add(uid);
        setRoots((prev) => [...prev, node]);
      } catch {
        // ignore parse errors / heartbeats
      }
    };

    return () => es.close();
  }, [id]);

  if (loading) return <FullPageMessage>Loading…</FullPageMessage>;
  if (error || !session)
    return <FullPageMessage error>{error ?? "Session not found"}</FullPageMessage>;

  return (
    <div
      style={{
        height: "calc(100vh - 56px)",
        display: "flex",
        flexDirection: "column",
        maxWidth: "1400px",
        margin: "0 auto",
        padding: "24px 28px 0",
        boxSizing: "border-box",
      }}
    >
      {/* ── Breadcrumb ──────────────────────────────────── */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "8px",
          marginBottom: "16px",
          flexShrink: 0,
        }}
      >
        <BackLink href={`/projects/${encodeURIComponent(session.project_name)}/`}>
          ← {session.project_name}
        </BackLink>
        <span style={{ color: "var(--border-3)", fontSize: "13px" }}>/</span>
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "13px",
            color: "var(--text-2)",
          }}
        >
          {shortId(session.session_id)}
        </span>
      </div>

      {/* ── Session meta bento ──────────────────────────── */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: session.subagent_models?.length ? "repeat(5, 1fr)" : "repeat(4, 1fr)",
          background: "var(--bg-1)",
          border: "1px solid var(--border-1)",
          borderRadius: "var(--radius-lg)",
          overflow: "hidden",
          flexShrink: 0,
          marginBottom: "16px",
        }}
      >
        <MetaCell label="Model" accent>{shortModel(session.model)}</MetaCell>
        <MetaCell label="Errors" color={session.error_count > 0 ? "var(--red)" : undefined}>
          {session.error_count.toLocaleString()}
        </MetaCell>
        <MetaCell label="Size">{formatBytes(session.file_size)}</MetaCell>
        <MetaCell label="Created">{timeAgo(session.created_at)}</MetaCell>
        {session.subagent_models?.length ? (
          <MetaCell label="Sub-agents" color="var(--purple)">
            <span style={{ display: "flex", flexWrap: "wrap", gap: "4px" }}>
              {session.subagent_models.map((m) => (
                <span
                  key={m}
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "11px",
                    padding: "2px 6px",
                    borderRadius: "var(--radius-sm)",
                    background: "rgba(160,90,255,0.12)",
                    border: "1px solid rgba(160,90,255,0.25)",
                    color: "var(--purple)",
                  }}
                >
                  {shortModel(m)}
                </span>
              ))}
            </span>
          </MetaCell>
        ) : null}
      </div>

      {/* ── Split panels ─────────────────────────────────── */}
      <div
        style={{
          flex: 1,
          display: "flex",
          overflow: "hidden",
          border: "1px solid var(--border-1)",
          borderRadius: "var(--radius-lg)",
          marginBottom: "24px",
        }}
      >
        {/* ── Tree panel (35%) ──────────────────────────── */}
        <div
          style={{
            width: "35%",
            flexShrink: 0,
            overflowY: "auto",
            borderRight: "1px solid var(--border-1)",
            background: "var(--bg-1)",
          }}
        >
          <div
            style={{
              borderBottom: "1px solid var(--border-1)",
              position: "sticky",
              top: 0,
              background: "var(--bg-1)",
              zIndex: 1,
            }}
          >
            <div
              style={{
                padding: "12px 16px 8px",
                fontFamily: "var(--font-mono)",
                fontSize: "11px",
                fontWeight: 700,
                letterSpacing: "0.08em",
                textTransform: "uppercase",
                color: "var(--text-3)",
              }}
            >
              Execution Tree
            </div>

            {/* Filter bar */}
            <div style={{
              display: "flex", flexWrap: "wrap", gap: "4px",
              padding: "0 16px 10px",
              alignItems: "center",
            }}>
              {(["User", "Assistant", "Tool", "Error", "Thinking", "Prompt"] as const).map(
                (label) => {
                  const key = label.toLowerCase();
                  const active = filterTypes.has(key);
                  return (
                    <button
                      key={key}
                      onClick={() => toggleFilterType(key)}
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "10px",
                        fontWeight: 600,
                        letterSpacing: "0.05em",
                        padding: "2px 8px",
                        borderRadius: "var(--radius-sm)",
                        border: active
                          ? "1px solid var(--accent)"
                          : "1px solid var(--border-2)",
                        background: active ? "rgba(0,255,136,0.08)" : "transparent",
                        color: active ? "var(--accent)" : "var(--text-3)",
                        cursor: "pointer",
                        transition: "all 0.12s",
                      }}
                    >
                      {label}
                    </button>
                  );
                }
              )}
              <input
                type="text"
                value={filterKeyword}
                onChange={(e) => setFilterKeyword(e.target.value)}
                placeholder="Search…"
                style={{
                  flex: 1,
                  minWidth: "60px",
                  fontFamily: "var(--font-mono)",
                  fontSize: "11px",
                  padding: "2px 8px",
                  borderRadius: "var(--radius-sm)",
                  border: "1px solid var(--border-2)",
                  background: "transparent",
                  color: "var(--text-2)",
                  outline: "none",
                  caretColor: "var(--accent)",
                }}
              />
            </div>
          </div>
          <NodeTree
            nodes={roots}
            onSelect={setSelected}
            selectedId={selected?.uuid ?? null}
            filter={nodeFilter}
          />
        </div>

        {/* ── Detail panel (65%) ────────────────────────── */}
        <div style={{ flex: 1, overflowY: "auto", background: "var(--bg-0)" }}>
          {selected ? (
            <NodeDetailPanel node={selected} />
          ) : (
            <div
              style={{
                height: "100%",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                fontSize: "14px",
                color: "var(--text-3)",
                fontFamily: "var(--font-sans)",
              }}
            >
              Select a node to inspect
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// ── Strip N→ line number prefix (Read tool results) ───────────
function stripLineNumbers(text: string): string {
  const lines = text.split("\n");
  const firstNonEmpty = lines.find((l) => l.trim() !== "");
  if (firstNonEmpty && /^\d+→/.test(firstNonEmpty)) {
    return lines.map((l) => l.replace(/^\d+→/, "")).join("\n");
  }
  return text;
}

// ── Node detail panel ─────────────────────────────────────────
function NodeDetailPanel({ node }: { node: NodeResponse }) {
  const [showRaw, setShowRaw] = useState(false);
  const meta = getNodeMeta(node);
  const tokenUsage = getTokenUsage(node);

  // Reset to UI mode when node changes
  useEffect(() => { setShowRaw(false); }, [node.uuid]);

  // Semantic flags
  const isThinking   = node.node_type === "assistant" && node.color === "magenta";
  const isToolCall   = node.node_type === "assistant" && node.color === "yellow";
  const isToolResult = node.node_type === "user"      && node.color === "blue";

  // ── Extract content from message.content blocks ──────────
  const contentBlocks = Array.isArray(node.message?.content)
    ? (node.message!.content as ContentBlock[])
    : [];

  const textBlocks  = contentBlocks.filter((b) => b.type === "text") as { type: "text"; text: string }[];
  const thinkBlocks = contentBlocks.filter((b) => b.type === "thinking") as { type: "thinking"; thinking: string }[];
  const toolUseBlock  = contentBlocks.find((b) => b.type === "tool_use") as
    | { type: "tool_use"; id: string; name: string; input: Record<string, unknown> }
    | undefined;
  const toolResultBlock = contentBlocks.find((b) => b.type === "tool_result") as
    | { type: "tool_result"; tool_use_id: string; content: unknown; is_error: boolean | null }
    | undefined;

  const legacyText = typeof node.message?.content === "string" ? node.message.content : null;

  const topLevelToolUse    = node.tool_use;
  const topLevelToolResult = node.tool_result;

  const toolCallName  = toolUseBlock?.name  ?? topLevelToolUse?.name;
  const toolCallInput = toolUseBlock?.input ?? (topLevelToolUse?.input as Record<string, unknown> | undefined);

  // ── Tool specialisation flags ────────────────────────────
  const toolNameLc       = toolCallName?.toLowerCase() ?? "";
  const isTaskCall       = isToolCall && ["task", "todowrite"].includes(toolNameLc);
  const isTaskCreateCall = isToolCall && toolNameLc === "taskcreate";
  const isWriteCall      = isToolCall && toolNameLc === "write";
  const isEditCall       = isToolCall && toolNameLc === "edit";
  const isBashCall       = isToolCall && toolNameLc === "bash";

  // ── Tool result content & error detection ───────────────
  // Error detection: check explicit flag AND look for <tool_use_error> tag
  const resultIsError =
    toolResultBlock?.is_error
    ?? topLevelToolResult?.is_error
    ?? false;

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

  // Display content: prefer clean file.content (no N→ line numbers).
  // Only fall through to raw content when file.content is absent.
  const hasCleanFile = !!topLevelToolResult?.file?.content;

  const displayResultContent: string | null = hasCleanFile
    ? null  // JSX will use the file branch which shows file.content
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

  // Body text for plain text nodes
  const bodyText =
    legacyText ??
    (textBlocks.length > 0 ? textBlocks.map((b) => b.text).join("\n\n") : null) ??
    node.summary ??
    "";

  // Thinking text
  const thinkingText =
    getThinkingText(node) ??
    (thinkBlocks.length > 0 ? thinkBlocks.map((b) => b.thinking).join("\n\n") : null) ??
    null;

  const progress = node.progress;

  return (
    <div style={{ padding: "24px 28px" }}>
      {/* ── Header ─────────────────────────────────── */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "12px",
          marginBottom: "24px",
          paddingBottom: "18px",
          borderBottom: "1px solid var(--border-1)",
        }}
      >
        <span
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: "7px",
            padding: "4px 12px",
            borderRadius: "var(--radius-md)",
            background: "var(--bg-2)",
            border: "1px solid var(--border-2)",
          }}
        >
          <span style={{ color: meta.color, fontSize: "16px" }}>{meta.icon}</span>
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "12px",
              fontWeight: 700,
              letterSpacing: "0.08em",
              textTransform: "uppercase",
              color: meta.color,
            }}
          >
            {meta.badge}
          </span>
        </span>

        {node.uuid && (
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)" }}>
            {shortId(node.uuid)}
          </span>
        )}

        {node.timestamp != null && (
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", marginLeft: "auto" }}>
            {new Date(node.timestamp).toLocaleTimeString("en-US", {
              hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit",
            })}
          </span>
        )}

        <button
          onClick={() => setShowRaw((r) => !r)}
          style={{
            marginLeft: node.timestamp != null ? "12px" : "auto",
            padding: "3px 10px",
            background: showRaw ? "var(--bg-3)" : "transparent",
            border: "1px solid var(--border-2)",
            borderRadius: "var(--radius-sm)",
            fontFamily: "var(--font-mono)", fontSize: "10px",
            letterSpacing: "0.08em", textTransform: "uppercase",
            color: showRaw ? "var(--text-1)" : "var(--text-3)",
            cursor: "pointer", transition: "all 0.12s",
          }}
        >
          {showRaw ? "UI" : "JSON"}
        </button>
      </div>

      {showRaw ? (
        <ContentSection label="Raw Node" color="var(--text-3)">
          <CodeRender content={JSON.stringify(node, null, 2)} />
        </ContentSection>
      ) : (
        <>
          {/* ── Thinking content ──────────────────────── */}
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

          {/* ── Tool call ─────────────────────────────── */}
          {isToolCall && toolCallName && (
            <>
              <ContentSection label="Tool">
                <span style={{ fontFamily: "var(--font-mono)", fontSize: "16px", fontWeight: 600, color: "var(--amber)" }}>
                  {toolCallName}
                </span>
              </ContentSection>

              {/* ── Write tool: file_path + content ────── */}
              {isWriteCall && toolCallInput ? (
                <WriteToolDisplay input={toolCallInput} />

              /* ── Edit tool: diff view ───────────────── */
              ) : isEditCall && toolCallInput ? (
                <EditToolDisplay input={toolCallInput} />

              /* ── TaskCreate structured display ──────── */
              ) : isTaskCreateCall && toolCallInput ? (
                <TaskCreateDisplay input={toolCallInput} />

              /* ── Bash tool: command ─────────────────── */
              ) : isBashCall && toolCallInput ? (
                <BashToolDisplay input={toolCallInput} />

              /* ── Task / SubAgent structured display ─── */
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
                        ⟳ {String(toolCallInput.subagent_type)}
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

              /* ── Generic tool: JSON input ───────────── */
              ) : (
                toolCallInput && Object.keys(toolCallInput).length > 0 && (
                  <ContentSection label="Input" color="var(--amber)">
                    <CodeRender content={JSON.stringify(toolCallInput, null, 2)} />
                  </ContentSection>
                )
              )}
            </>
          )}

          {/* ── Tool result ───────────────────────────── */}
          {isToolResult && (
            <ContentSection
              label={effectiveIsError ? "Error Output" : "Output"}
              color={effectiveIsError ? "var(--red)" : "var(--cyan)"}
            >
              {displayResultContent != null ? (
                (() => {
                  if (displayResultContent === "") return <EmptyResult />;
                  const serena = parseSerenaResult(displayResultContent);
                  if (serena) return <SerenaResultDisplay content={displayResultContent} />;
                  return (
                    <CodeRender
                      content={displayResultContent}
                      filePath={typeof toolCallInput?.file_path === "string" ? toolCallInput.file_path : undefined}
                      error={effectiveIsError}
                    />
                  );
                })()
              ) : topLevelToolResult?.file ? (
                <div>
                  {topLevelToolResult.file.file_path && (
                    <div style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", marginBottom: "8px" }}>
                      {topLevelToolResult.file.file_path}
                      {topLevelToolResult.file.num_lines != null && ` · ${topLevelToolResult.file.num_lines} lines`}
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

          {/* ── Progress ──────────────────────────────── */}
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

          {/* ── Plain text (user / assistant text, system, etc.) ── */}
          {!isThinking && !isToolCall && !isToolResult && node.node_type !== "progress" && (
            <>
              {bodyText ? (
                <ContentSection label="Content">
                  <MarkdownContent text={bodyText} />
                </ContentSection>
              ) : (
                <ContentSection label="Details">
                  <div style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--text-3)" }}>
                    {node.label}
                  </div>
                </ContentSection>
              )}
            </>
          )}
        </>
      )}

      {/* ── Token usage footer ────────────────────── */}
      {tokenUsage && <TokenFooter usage={tokenUsage} />}
    </div>
  );
}

// ── Serena MCP result rendering ───────────────────────────────

type SerenaResult =
  | { type: "text"; text: string }
  | { type: "structured"; data: Record<string, unknown> };

function parseSerenaResult(content: string): SerenaResult | null {
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

function PathList({ items }: { items: string[] }) {
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

function SerenaResultDisplay({ content }: { content: string }) {
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
        <ContentSection label={`Dirs · ${dirs.length}`} color="var(--cyan)">
          <PathList items={dirs} />
        </ContentSection>
      )}
      {files.length > 0 && (
        <ContentSection label={`Files · ${files.length}`} color="var(--green)">
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

// ── Specialised tool displays ─────────────────────────────────

function WriteToolDisplay({ input }: { input: Record<string, unknown> }) {
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

function BashToolDisplay({ input }: { input: Record<string, unknown> }) {
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

function TaskCreateDisplay({ input }: { input: Record<string, unknown> }) {
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
            <span style={{ opacity: 0.5 }}>⟳</span> {activeForm}
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

function EditToolDisplay({ input }: { input: Record<string, unknown> }) {
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

        {/* ── Header bar ── */}
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
                −{removedLines.length}
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

        {/* ── Diff lines ── */}
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
              }}>−</span>
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
              background: "rgba(0,255,136,0.04)",
              borderLeft: "2px solid rgba(0,255,136,0.3)",
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

// ── Markdown renderer ─────────────────────────────────────────

function MarkdownContent({ text }: { text: string }) {
  const segments = text.split(/(```[\w]*\n[\s\S]*?```)/g);

  return (
    <div style={{ fontFamily: "var(--font-sans)", fontSize: "15px", lineHeight: 1.75 }}>
      {segments.map((seg, i) => {
        const fenceMatch = seg.match(/^```([\w]*)\n([\s\S]*?)```$/);
        if (fenceMatch) {
          const lang = fenceMatch[1] || undefined;
          const code = fenceMatch[2];
          return (
            <div key={i} style={{ marginBottom: "16px" }}>
              <CodeRender content={code} language={lang || undefined} />
            </div>
          );
        }
        return <MarkdownText key={i} text={seg} />;
      })}
    </div>
  );
}

function MarkdownText({ text }: { text: string }) {
  const lines = text.split("\n");
  const out: React.ReactNode[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    const headMatch = line.match(/^(#{1,6})\s+(.+)$/);
    if (headMatch) {
      const level = headMatch[1].length;
      const sizes = ["22px", "20px", "18px", "16px", "15px", "14px"];
      out.push(
        <div key={i} style={{
          fontSize: sizes[level - 1] ?? "14px",
          fontWeight: 700,
          color: "var(--text-1)",
          marginBottom: "8px",
          marginTop: out.length > 0 ? "16px" : 0,
          lineHeight: 1.3,
        }}>
          {renderInline(headMatch[2])}
        </div>
      );
      i++;
      continue;
    }

    const bulletMatch = line.match(/^[-*]\s+(.+)$/);
    if (bulletMatch) {
      out.push(
        <div key={i} style={{ display: "flex", gap: "8px", marginBottom: "3px" }}>
          <span style={{ color: "var(--text-3)", flexShrink: 0, userSelect: "none" }}>·</span>
          <span style={{ color: "var(--text-1)" }}>{renderInline(bulletMatch[1])}</span>
        </div>
      );
      i++;
      continue;
    }

    if (/^---+$|^===+$/.test(line.trim())) {
      out.push(
        <hr key={i} style={{ border: "none", borderTop: "1px solid var(--border-1)", margin: "12px 0" }} />
      );
      i++;
      continue;
    }

    if (line.trim() === "") {
      out.push(<div key={i} style={{ height: "10px" }} />);
      i++;
      continue;
    }

    out.push(
      <div key={i} style={{ color: "var(--text-1)", marginBottom: "2px" }}>
        {renderInline(line)}
      </div>
    );
    i++;
  }

  return <>{out}</>;
}

function renderInline(text: string): React.ReactNode {
  const parts = text.split(/(\*\*[^*\n]+\*\*|\*[^*\n]+\*|`[^`\n]+`)/g);
  if (parts.length === 1) return text;

  return (
    <>
      {parts.map((part, i) => {
        if (/^\*\*[^*\n]+\*\*$/.test(part))
          return <strong key={i} style={{ color: "var(--text-1)", fontWeight: 700 }}>{part.slice(2, -2)}</strong>;
        if (/^\*[^*\n]+\*$/.test(part))
          return <em key={i} style={{ color: "var(--text-2)", fontStyle: "italic" }}>{part.slice(1, -1)}</em>;
        if (/^`[^`\n]+`$/.test(part))
          return (
            <code key={i} style={{
              fontFamily: "var(--font-mono)", fontSize: "12px",
              background: "var(--bg-3)", padding: "1px 5px",
              borderRadius: "var(--radius-sm)", color: "var(--cyan)",
            }}>
              {part.slice(1, -1)}
            </code>
          );
        return <span key={i}>{part}</span>;
      })}
    </>
  );
}

// ── Empty result placeholder ──────────────────────────────────

function EmptyResult({ label }: { label?: string }) {
  return (
    <div style={{
      display: "flex", alignItems: "center", justifyContent: "center",
      padding: "20px", gap: "8px",
      background: "var(--bg-2)", border: "1px solid var(--border-1)",
      borderRadius: "var(--radius-md)",
    }}>
      <span style={{ color: "var(--text-3)", fontSize: "16px" }}>∅</span>
      <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", letterSpacing: "0.05em" }}>
        {label ?? "no result"}
      </span>
    </div>
  );
}

// ── NodeDetailPanel helpers ───────────────────────────────────

function ContentSection({
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

function TokenFooter({ usage }: { usage: TokenUsage }) {
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

// ── Page-level layout helpers ─────────────────────────────────

function MetaCell({
  label, children, accent, color,
}: { label: string; children: React.ReactNode; accent?: boolean; color?: string }) {
  return (
    <div style={{ padding: "18px 22px", borderRight: "1px solid var(--border-1)", position: "relative" }}>
      {accent && (
        <div aria-hidden="true" style={{
          position: "absolute", top: 0, left: 0, right: 0,
          height: "2px", background: "var(--accent)",
        }} />
      )}
      <div style={{
        fontFamily: "var(--font-mono)", fontSize: "11px", fontWeight: 600,
        letterSpacing: "0.08em", textTransform: "uppercase",
        color: accent ? "var(--accent)" : "var(--text-3)", marginBottom: "8px",
      }}>
        {label}
      </div>
      <div style={{
        fontFamily: "var(--font-mono)", fontSize: "16px", fontWeight: 600,
        fontVariantNumeric: "tabular-nums", color: color ?? "var(--text-1)", lineHeight: 1,
      }}>
        {children}
      </div>
    </div>
  );
}

function BackLink({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <Link
      href={href}
      style={{
        fontSize: "13px", color: "var(--text-3)", textDecoration: "none",
        fontFamily: "var(--font-mono)", transition: "color 0.12s", flexShrink: 0,
      }}
      onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-2)"; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-3)"; }}
    >
      {children}
    </Link>
  );
}

function FullPageMessage({ children, error }: { children: React.ReactNode; error?: boolean }) {
  return (
    <div style={{
      height: "calc(100vh - 56px)", display: "flex", alignItems: "center",
      justifyContent: "center", fontSize: "14px",
      color: error ? "var(--red)" : "var(--text-3)",
      fontFamily: "var(--font-sans)", textAlign: "center",
    }}>
      {children}
    </div>
  );
}

function findFirst(nodes: NodeResponse[], pred: (n: NodeResponse) => boolean): NodeResponse | null {
  for (const node of nodes) {
    if (pred(node)) return node;
    const found = findFirst(node.children, pred);
    if (found) return found;
  }
  return null;
}
