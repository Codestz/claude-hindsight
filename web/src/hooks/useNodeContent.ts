/**
 * Hook that parses a NodeResponse into display-ready content.
 *
 * Extracts:
 * - Content blocks (text, thinking, image, tool_use, tool_result)
 * - Tool call info (name, input, MCP parsing)
 * - Tool result content with error detection
 * - File path resolution
 * - Image data extraction
 * - Linked node (tool call ↔ result navigation)
 */

import { useMemo } from "react";
import type { ContentBlock, NodeResponse } from "@/lib/types";
import { getNodeMeta, getThinkingText, getTokenUsage } from "@/lib/node-meta";
import { extractTurFile, resolveFilePath } from "@/components/session/tool-displays/resolve-file-path";
import { stripLineNumbers } from "@/components/session/tool-displays/strip-utils";
import type { ImageData } from "@/components/session/types";

/** Safe media types for data: URI rendering. */
const SAFE_IMAGE_TYPES = new Set(["image/jpeg", "image/png", "image/gif", "image/webp"]);

export function useNodeContent(node: NodeResponse, flatNodes?: NodeResponse[]) {
  const meta = getNodeMeta(node);
  const tokenUsage = getTokenUsage(node);

  const isThinking   = node.node_type === "assistant" && node.color === "magenta";
  const isToolCall   = node.node_type === "assistant" && node.color === "yellow";
  const isToolResult = node.node_type === "user"      && node.color === "blue";

  // Parse content blocks
  const contentBlocks = Array.isArray(node.message?.content)
    ? (node.message!.content as ContentBlock[])
    : [];

  const textBlocks  = contentBlocks.filter((b) => b.type === "text") as { type: "text"; text: string }[];
  const thinkBlocks = contentBlocks.filter((b) => b.type === "thinking") as { type: "thinking"; thinking: string }[];
  const imageBlocks = contentBlocks.filter((b) => b.type === "image") as {
    type: "image"; source: { type: string; media_type: string; data: string };
  }[];
  const toolUseBlock = contentBlocks.find((b) => b.type === "tool_use") as
    | { type: "tool_use"; id: string; name: string; input: Record<string, unknown> }
    | undefined;
  const toolResultBlock = contentBlocks.find((b) => b.type === "tool_result") as
    | { type: "tool_result"; tool_use_id: string; content: unknown; is_error: boolean | null }
    | undefined;

  // Legacy text
  const legacyText = typeof node.message?.content === "string" ? node.message.content : null;

  // Tool call info
  const topLevelToolUse    = node.tool_use;
  const topLevelToolResult = node.tool_result;
  const toolCallName  = toolUseBlock?.name  ?? topLevelToolUse?.name ?? node.tool_name;
  const toolCallInput = toolUseBlock?.input ?? (topLevelToolUse?.input as Record<string, unknown> | undefined);

  // Tool type detection
  const toolNameLc       = toolCallName?.toLowerCase() ?? "";
  const isTaskCall       = isToolCall && ["task", "todowrite"].includes(toolNameLc);
  const isTaskCreateCall = isToolCall && toolNameLc === "taskcreate";
  const isWriteCall      = isToolCall && toolNameLc === "write";
  const isReadCall       = isToolCall && toolNameLc === "read";
  const isEditCall       = isToolCall && toolNameLc === "edit";
  const isBashCall       = isToolCall && toolNameLc === "bash";

  // MCP tool name parsing
  const mcpParts = toolCallName?.match(/^mcp__([^_]+(?:__[^_]+)*)__([^_]+(?:__[^_]+)*)$/);
  const mcpServer = mcpParts?.[1]?.replace(/__/g, "-") ?? null;
  const mcpShortTool = mcpParts?.[2]?.replace(/__/g, ".") ?? null;
  const displayToolName = mcpShortTool ?? toolCallName;

  // File path resolution
  const turFile = extractTurFile(node);
  const resolvedFilePath = resolveFilePath(node, turFile, toolCallInput);

  // Error detection
  const resultIsError = toolResultBlock?.is_error ?? topLevelToolResult?.is_error ?? false;
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

  // Clean file content
  const hasCleanFile = !!topLevelToolResult?.file?.content || !!turFile?.content;

  // Display result content
  const displayResultContent: string | null = hasCleanFile
    ? null
    : (() => {
        let raw: string | null = null;
        if (toolResultBlock?.content != null) {
          if (typeof toolResultBlock.content === "string") {
            raw = toolResultBlock.content;
          } else if (typeof toolResultBlock.content === "object" && !Array.isArray(toolResultBlock.content)) {
            const obj = toolResultBlock.content as Record<string, unknown>;
            raw = typeof obj.content === "string" ? obj.content : JSON.stringify(toolResultBlock.content, null, 2);
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

  // Body text
  const bodyText =
    legacyText ??
    (textBlocks.length > 0 ? textBlocks.map((b) => b.text).join("\n\n") : null) ??
    node.summary ?? "";

  // Thinking text
  const thinkingText =
    getThinkingText(node) ??
    (thinkBlocks.length > 0 ? thinkBlocks.map((b) => b.thinking).join("\n\n") : null) ??
    null;

  // Image extraction
  const extractedImages: ImageData[] = [];
  for (const img of imageBlocks) {
    if (img.source?.data && img.source?.media_type && SAFE_IMAGE_TYPES.has(img.source.media_type)) {
      extractedImages.push({ mediaType: img.source.media_type, data: img.source.data });
    }
  }
  const tryExtractImagesFromContent = (content: unknown) => {
    if (!content) return;
    let arr: unknown[] | null = null;
    if (Array.isArray(content)) arr = content;
    else if (typeof content === "string") {
      try { const p = JSON.parse(content); if (Array.isArray(p)) arr = p; } catch { /* */ }
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

  // Linked node (tool call ↔ result)
  const linkedNode = useMemo(() => {
    if (!flatNodes) return null;
    if (isToolCall) {
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

  return {
    meta, tokenUsage,
    isThinking, isToolCall, isToolResult,
    textBlocks, thinkBlocks,
    toolCallName, toolCallInput, displayToolName,
    mcpServer,
    isTaskCall, isTaskCreateCall, isWriteCall, isReadCall, isEditCall, isBashCall,
    turFile, resolvedFilePath,
    effectiveIsError, hasCleanFile, displayResultContent,
    bodyText, thinkingText,
    extractedImages,
    linkedNode,
    progress: node.progress,
    topLevelToolResult,
  };
}
