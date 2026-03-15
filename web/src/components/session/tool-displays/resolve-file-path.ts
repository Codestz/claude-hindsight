/**
 * Resolve file path from all possible locations in a node's data.
 *
 * Claude Code stores file paths in multiple places depending on the tool type
 * and transcript format version. This function checks all known locations
 * in priority order.
 *
 * The resolved file path is used for syntax highlighting language detection.
 */

import type { NodeResponse } from "@/lib/types";

interface TurFile {
  content?: string;
  filePath?: string;
  file_path?: string;
  numLines?: number;
  startLine?: number;
  totalLines?: number;
}

/** Extract toolUseResult.file metadata if present. */
export function extractTurFile(node: NodeResponse): TurFile | undefined {
  const tur = node.toolUseResult as Record<string, unknown> | null | undefined;
  if (tur && typeof tur === "object" && tur.file && typeof tur.file === "object") {
    return tur.file as TurFile;
  }
  return undefined;
}

/**
 * Resolve file path from all possible sources in a node.
 *
 * Priority:
 * 1. toolUseResult.file.filePath (Read tool results — most reliable)
 * 2. Raw message content blocks (filePath as sibling of text/content)
 * 3. Tool call input (Read, Write, Edit all have file_path)
 * 4. tool_result content block with nested filePath
 * 5. Top-level tool_result.file
 * 6. Backend-computed file_paths array
 */
export function resolveFilePath(
  node: NodeResponse,
  turFile: TurFile | undefined,
  toolCallInput: Record<string, unknown> | undefined,
): string | undefined {
  // 1. toolUseResult.file.filePath
  if (turFile) {
    if (typeof turFile.filePath === "string") return turFile.filePath;
    if (typeof turFile.file_path === "string") return turFile.file_path;
  }

  // 2. Raw message content blocks
  const rawContent = node.message?.content;
  if (Array.isArray(rawContent)) {
    for (const block of rawContent) {
      if (typeof block === "object" && block !== null) {
        const b = block as Record<string, unknown>;
        if (typeof b.filePath === "string") return b.filePath;
        if (typeof b.file_path === "string") return b.file_path;
        if (typeof b.content === "object" && b.content !== null && !Array.isArray(b.content)) {
          const inner = b.content as Record<string, unknown>;
          if (typeof inner.filePath === "string") return inner.filePath;
        }
      }
    }
  }

  // 3. Tool call input
  if (typeof toolCallInput?.file_path === "string") return toolCallInput.file_path;

  // 4. tool_result content block
  const contentBlocks = Array.isArray(node.message?.content) ? node.message!.content : [];
  const toolResultBlock = contentBlocks.find((b: any) => b.type === "tool_result") as
    | { content: unknown } | undefined;
  if (toolResultBlock?.content && typeof toolResultBlock.content === "object" && !Array.isArray(toolResultBlock.content)) {
    const obj = toolResultBlock.content as Record<string, unknown>;
    if (typeof obj.filePath === "string") return obj.filePath;
    if (typeof obj.file_path === "string") return obj.file_path;
  }

  // 5. Top-level tool_result.file
  if (node.tool_result?.file?.file_path) return node.tool_result.file.file_path;

  // 6. Backend-computed file_paths
  if (node.file_paths?.[0]) return node.file_paths[0];

  return undefined;
}
