/**
 * Language detection and keyword configuration for syntax highlighting.
 *
 * This module provides:
 * - File extension → language mapping
 * - Content-based heuristic language detection
 * - Per-language keyword sets for tokenization
 */

/** Map of file extensions to language identifiers. */
export const EXT_TO_LANG: Record<string, string> = {
  rs: "rust", js: "javascript", jsx: "javascript",
  ts: "typescript", tsx: "typescript", py: "python",
  go: "go", json: "json", toml: "toml", yaml: "yaml",
  yml: "yaml", md: "markdown", css: "css", sh: "bash", bash: "bash",
  html: "html", c: "c", cpp: "cpp", rb: "ruby",
};

/** Per-language keyword sets for syntax highlighting. */
export const LANGUAGE_KEYWORDS: Record<string, Set<string>> = {
  typescript: new Set(["import", "export", "from", "const", "let", "var", "function", "return", "if", "else", "for", "while", "class", "interface", "type", "extends", "implements", "new", "this", "async", "await", "default", "switch", "case", "break", "continue", "throw", "try", "catch", "finally", "typeof", "instanceof", "in", "of", "as", "null", "undefined", "true", "false", "void"]),
  javascript: new Set(["import", "export", "from", "const", "let", "var", "function", "return", "if", "else", "for", "while", "class", "extends", "new", "this", "async", "await", "default", "switch", "case", "break", "continue", "throw", "try", "catch", "finally", "typeof", "instanceof", "in", "of", "null", "undefined", "true", "false", "void"]),
  rust: new Set(["use", "fn", "pub", "let", "mut", "const", "struct", "enum", "impl", "trait", "mod", "self", "super", "crate", "return", "if", "else", "for", "while", "loop", "match", "break", "continue", "async", "await", "move", "where", "type", "as", "in", "true", "false", "Some", "None", "Ok", "Err"]),
  python: new Set(["import", "from", "def", "class", "return", "if", "elif", "else", "for", "while", "with", "as", "try", "except", "finally", "raise", "yield", "async", "await", "pass", "break", "continue", "in", "is", "not", "and", "or", "True", "False", "None", "self", "lambda"]),
  go: new Set(["package", "import", "func", "return", "if", "else", "for", "range", "switch", "case", "default", "break", "continue", "var", "const", "type", "struct", "interface", "map", "chan", "go", "defer", "select", "true", "false", "nil"]),
  bash: new Set(["if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case", "esac", "function", "return", "exit", "echo", "export", "local", "readonly", "in", "true", "false"]),
  html: new Set(["DOCTYPE", "html", "head", "body", "div", "span", "p", "a", "img", "script", "style", "link", "meta", "title", "h1", "h2", "h3", "h4", "h5", "h6", "ul", "ol", "li", "table", "tr", "td", "th", "form", "input", "button", "select", "option", "textarea", "label", "section", "header", "footer", "nav", "main", "article", "aside", "canvas", "svg", "path"]),
  css: new Set(["inherit", "initial", "unset", "none", "auto", "block", "inline", "flex", "grid", "relative", "absolute", "fixed", "sticky", "hidden", "visible", "solid", "dashed", "dotted", "transparent", "important", "normal", "bold", "italic", "center", "left", "right", "top", "bottom"]),
};

const EMPTY_SET = new Set<string>();

/** Get keyword set for a language (returns empty set for unknown languages). */
export function getKeywords(lang: string): Set<string> {
  return LANGUAGE_KEYWORDS[lang] ?? EMPTY_SET;
}

/**
 * Detect language from file path extension or content heuristics.
 *
 * Priority: file extension > content patterns > "text" fallback.
 */
export function detectLanguage(filePath?: string, content?: string): string {
  if (filePath) {
    const ext = filePath.split(".").pop()?.toLowerCase() ?? "";
    if (EXT_TO_LANG[ext]) return EXT_TO_LANG[ext];
  }

  const trimmed = content?.trim() ?? "";

  // JSON
  if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
    try { JSON.parse(trimmed); return "json"; } catch { /* not json */ }
  }

  // Heuristic patterns
  if (/^import\s+.+from\s+['"]|^export\s+(default\s+)?/m.test(trimmed)) return "typescript";
  if (/^\s*(const|let|var)\s+\w+\s*(:|=)/m.test(trimmed) && /(\?\?|=>|as\s+\w|interface\s|type\s)/.test(trimmed)) return "typescript";
  if (/^\s*(const|let|var)\s+\w+\s*=/.test(trimmed) && /(\?\.|=>|\.map\(|\.filter\(|\.forEach\()/.test(trimmed)) return "javascript";
  if (/^(use\s+\w|fn\s+\w|pub\s+(fn|struct|enum|mod|use)\s)/m.test(trimmed)) return "rust";
  if (/^(def\s+\w|class\s+\w|from\s+\w+\s+import)/m.test(trimmed)) return "python";
  if (/^(func\s+\w|package\s+\w)/m.test(trimmed)) return "go";
  if (/^\s*(#!\/bin\/(ba)?sh|\$\s)/m.test(trimmed)) return "bash";
  if (/^<!DOCTYPE\s|^<html|^<head|^<body|^<div|^<script|^<link|^<meta/im.test(trimmed)) return "html";
  if (/^(\.|#|@media|@import|@keyframes|:root)\s*\{|^\s*[a-z-]+\s*:\s*[^;]+;/m.test(trimmed)) return "css";

  return "text";
}
