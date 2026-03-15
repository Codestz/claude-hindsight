/**
 * Text processing utilities for tool results.
 */

/**
 * Strip line number prefixes from Read tool output.
 *
 * The Read tool returns content in `cat -n` format:
 *   "     1→import React from 'react';"
 * Leading spaces + digits + arrow (U+2192) + optional tab/space.
 */
export function stripLineNumbers(text: string): string {
  const lines = text.split("\n");
  const firstNonEmpty = lines.find((l) => l.trim() !== "");
  if (firstNonEmpty && /^\s*\d+\u2192/.test(firstNonEmpty)) {
    return lines.map((l) => l.replace(/^\s*\d+\u2192\t?/, "")).join("\n");
  }
  return text;
}
