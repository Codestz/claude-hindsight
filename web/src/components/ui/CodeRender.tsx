import React from "react";

// ── Language detection ────────────────────────────────────────
function detectLanguage(filePath?: string, content?: string): string {
  if (filePath) {
    const ext = filePath.split(".").pop()?.toLowerCase() ?? "";
    const map: Record<string, string> = {
      rs: "rust", js: "javascript", jsx: "javascript",
      ts: "typescript", tsx: "typescript", py: "python",
      go: "go", json: "json", toml: "toml", yaml: "yaml",
      yml: "yaml", md: "markdown", css: "css", sh: "bash", bash: "bash",
      html: "html", c: "c", cpp: "cpp", rb: "ruby",
    };
    if (map[ext]) return map[ext];
  }
  const trimmed = content?.trim() ?? "";
  // JSON
  if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
    try { JSON.parse(trimmed); return "json"; } catch { /* not json */ }
  }
  // Heuristic: detect common code patterns
  if (/^import\s+.+from\s+['"]|^export\s+(default\s+)?/m.test(trimmed)) return "typescript";
  // TS/JS: const/let/var declarations, arrow functions, type annotations
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

// ── JSON colorization ─────────────────────────────────────────
function colorizeJson(json: string): React.ReactNode {
  const tokens = json.split(/(\"[^\"]*\"\s*:)|(\"[^\"]*\")|([-\d.]+)|(true|false|null)|([{}[\],])/g);
  return tokens.filter(Boolean).map((token, i) => {
    if (/^"[^"]*"\s*:$/.test(token))
      return <span key={i} style={{ color: "var(--info)" }}>{token}</span>;
    if (/^"[^"]*"$/.test(token))
      return <span key={i} style={{ color: "var(--success)" }}>{token}</span>;
    if (/^[-\d.]+$|^(true|false|null)$/.test(token))
      return <span key={i} style={{ color: "var(--amber)" }}>{token}</span>;
    return <span key={i} style={{ color: "var(--text-3)" }}>{token}</span>;
  });
}

// ── Basic code colorization ───────────────────────────────────
// Not a full parser — just enough to make code scannable.
function colorizeCode(code: string, lang: string): React.ReactNode {
  // For HTML, split into segments with embedded <style> and <script> blocks
  if (lang === "html") {
    return colorizeHtmlWithEmbedded(code);
  }
  return colorizeBlock(code, lang);
}

/** Split HTML into segments, colorizing <style> content as CSS and <script> content as JS */
function colorizeHtmlWithEmbedded(code: string): React.ReactNode {
  const parts: React.ReactNode[] = [];
  // Regex to find <style>...</style> and <script>...</script> blocks
  const embedRe = /(<style[^>]*>)([\s\S]*?)(<\/style>)|(<script[^>]*>)([\s\S]*?)(<\/script>)/gi;
  let lastIdx = 0;
  let segKey = 0;
  let m: RegExpExecArray | null;

  while ((m = embedRe.exec(code)) !== null) {
    // HTML before this embedded block
    if (m.index > lastIdx) {
      parts.push(<React.Fragment key={`h${segKey++}`}>{colorizeBlock(code.slice(lastIdx, m.index), "html")}</React.Fragment>);
    }

    if (m[1] != null) {
      // <style>...</style>
      parts.push(<React.Fragment key={`st${segKey++}`}>{colorizeBlock(m[1], "html")}</React.Fragment>);
      parts.push(<React.Fragment key={`sc${segKey++}`}>{colorizeBlock(m[2], "css")}</React.Fragment>);
      parts.push(<React.Fragment key={`se${segKey++}`}>{colorizeBlock(m[3], "html")}</React.Fragment>);
    } else {
      // <script>...</script>
      parts.push(<React.Fragment key={`jt${segKey++}`}>{colorizeBlock(m[4], "html")}</React.Fragment>);
      parts.push(<React.Fragment key={`jc${segKey++}`}>{colorizeBlock(m[5], "javascript")}</React.Fragment>);
      parts.push(<React.Fragment key={`je${segKey++}`}>{colorizeBlock(m[6], "html")}</React.Fragment>);
    }

    lastIdx = m.index + m[0].length;
  }

  // Remaining HTML after last embedded block
  if (lastIdx < code.length) {
    parts.push(<React.Fragment key={`t${segKey++}`}>{colorizeBlock(code.slice(lastIdx), "html")}</React.Fragment>);
  }

  return <>{parts}</>;
}

/** Colorize a block of code in a single language */
function colorizeBlock(code: string, lang: string): React.ReactNode {
  const lines = code.split("\n");
  return lines.map((line, li) => {
    const parts: React.ReactNode[] = [];
    let remaining = line;
    let ki = 0;

    // Process line left-to-right
    while (remaining.length > 0) {
      // HTML comments <!-- -->
      if ((lang === "html") && remaining.includes("<!--")) {
        const htmlCommentMatch = remaining.match(/^(.*?)(<!--[\s\S]*?-->)(.*)?$/);
        if (htmlCommentMatch) {
          if (htmlCommentMatch[1]) { parts.push(...tokenizeLine(htmlCommentMatch[1], lang, ki)); ki += htmlCommentMatch[1].length; }
          parts.push(<span key={`c${li}-${ki}`} style={{ color: "var(--text-3)", fontStyle: "italic" }}>{htmlCommentMatch[2]}</span>);
          remaining = htmlCommentMatch[3] ?? "";
          ki += htmlCommentMatch[2].length;
          continue;
        }
      }
      // CSS comments /* */
      if ((lang === "css") && remaining.includes("/*")) {
        const cssCommentMatch = remaining.match(/^(.*?)(\/\*[\s\S]*?\*\/)(.*)?$/);
        if (cssCommentMatch) {
          if (cssCommentMatch[1]) { parts.push(...tokenizeLine(cssCommentMatch[1], lang, ki)); ki += cssCommentMatch[1].length; }
          parts.push(<span key={`c${li}-${ki}`} style={{ color: "var(--text-3)", fontStyle: "italic" }}>{cssCommentMatch[2]}</span>);
          remaining = cssCommentMatch[3] ?? "";
          ki += cssCommentMatch[2].length;
          continue;
        }
      }
      // Line comments (// or #)
      if (lang !== "html" && lang !== "css") {
        const commentMatch = lang === "python" || lang === "bash" || lang === "ruby"
          ? remaining.match(/^(.*?)(#.*)$/)
          : remaining.match(/^(.*?)(\/\/.*)$/);
        if (commentMatch && commentMatch[1] !== undefined) {
          if (commentMatch[1]) {
            parts.push(...tokenizeLine(commentMatch[1], lang, ki));
            ki += commentMatch[1].length;
          }
          parts.push(<span key={`c${li}-${ki}`} style={{ color: "var(--text-3)", fontStyle: "italic" }}>{commentMatch[2]}</span>);
          remaining = "";
          break;
        }
      }

      // No comment found — tokenize the whole line
      parts.push(...tokenizeLine(remaining, lang, ki));
      remaining = "";
    }

    return (
      <React.Fragment key={li}>
        {parts}
        {li < lines.length - 1 ? "\n" : ""}
      </React.Fragment>
    );
  });
}

// Module-level keyword sets — created once, reused across all tokenizeLine calls
const KW_SETS: Record<string, Set<string>> = {
  typescript: new Set(["import", "export", "from", "const", "let", "var", "function", "return", "if", "else", "for", "while", "class", "interface", "type", "extends", "implements", "new", "this", "async", "await", "default", "switch", "case", "break", "continue", "throw", "try", "catch", "finally", "typeof", "instanceof", "in", "of", "as", "null", "undefined", "true", "false", "void"]),
  javascript: new Set(["import", "export", "from", "const", "let", "var", "function", "return", "if", "else", "for", "while", "class", "extends", "new", "this", "async", "await", "default", "switch", "case", "break", "continue", "throw", "try", "catch", "finally", "typeof", "instanceof", "in", "of", "null", "undefined", "true", "false", "void"]),
  rust: new Set(["use", "fn", "pub", "let", "mut", "const", "struct", "enum", "impl", "trait", "mod", "self", "super", "crate", "return", "if", "else", "for", "while", "loop", "match", "break", "continue", "async", "await", "move", "where", "type", "as", "in", "true", "false", "Some", "None", "Ok", "Err"]),
  python: new Set(["import", "from", "def", "class", "return", "if", "elif", "else", "for", "while", "with", "as", "try", "except", "finally", "raise", "yield", "async", "await", "pass", "break", "continue", "in", "is", "not", "and", "or", "True", "False", "None", "self", "lambda"]),
  go: new Set(["package", "import", "func", "return", "if", "else", "for", "range", "switch", "case", "default", "break", "continue", "var", "const", "type", "struct", "interface", "map", "chan", "go", "defer", "select", "true", "false", "nil"]),
  bash: new Set(["if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case", "esac", "function", "return", "exit", "echo", "export", "local", "readonly", "in", "true", "false"]),
  html: new Set(["DOCTYPE", "html", "head", "body", "div", "span", "p", "a", "img", "script", "style", "link", "meta", "title", "h1", "h2", "h3", "h4", "h5", "h6", "ul", "ol", "li", "table", "tr", "td", "th", "form", "input", "button", "select", "option", "textarea", "label", "section", "header", "footer", "nav", "main", "article", "aside", "canvas", "svg", "path"]),
  css: new Set(["inherit", "initial", "unset", "none", "auto", "block", "inline", "flex", "grid", "relative", "absolute", "fixed", "sticky", "hidden", "visible", "solid", "dashed", "dotted", "transparent", "important", "normal", "bold", "italic", "center", "left", "right", "top", "bottom"]),
};
const EMPTY_KW_SET = new Set<string>();

function tokenizeLine(line: string, lang: string, keyOffset: number): React.ReactNode[] {
  const results: React.ReactNode[] = [];
  const keywords = KW_SETS[lang] ?? EMPTY_KW_SET;

  // For HTML: special regex that captures tags, attributes, etc.
  const tokenRe = lang === "html"
    ? /("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')|(<!?\/?[a-zA-Z][\w-]*)|(\/>|>)|([a-zA-Z][\w-]*(?==))|(\b\d[\d_.]*\b)|(\b[a-zA-Z_]\w*\b)|(\s+)|(.)/g
    : lang === "css"
    ? /("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')|([a-zA-Z-]+(?=\s*:))|([.#][\w-]+)|(@[\w-]+)|(\b\d[\d_.]*(?:px|em|rem|%|vh|vw|s|ms|deg)?\b)|(\b[a-zA-Z_][\w-]*\b)|([{}();:,])|(\s+)|(.)/g
    : /("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|`(?:[^`\\]|\\.)*`)|(\b\d[\d_.]*\b)|(\b[a-zA-Z_]\w*\b)|(\s+)|(.)/g;

  let m: RegExpExecArray | null;
  let idx = 0;
  while ((m = tokenRe.exec(line)) !== null) {
    const key = keyOffset + idx++;
    const full = m[0];

    if (lang === "html") {
      const [, str, tag, tagClose, attr, num, word, ws, op] = m;
      if (str) { results.push(<span key={key} style={{ color: "var(--success)" }}>{str}</span>); }
      else if (tag) { results.push(<span key={key} style={{ color: "var(--rose)" }}>{tag}</span>); }
      else if (tagClose) { results.push(<span key={key} style={{ color: "var(--rose)" }}>{tagClose}</span>); }
      else if (attr) { results.push(<span key={key} style={{ color: "var(--amber)" }}>{attr}</span>); }
      else if (num) { results.push(<span key={key} style={{ color: "var(--amber)" }}>{num}</span>); }
      else if (word && keywords.has(word)) { results.push(<span key={key} style={{ color: "var(--violet)" }}>{word}</span>); }
      else if (word) { results.push(<span key={key} style={{ color: "var(--text-2)" }}>{word}</span>); }
      else if (ws) { results.push(<span key={key}>{ws}</span>); }
      else { results.push(<span key={key} style={{ color: "var(--text-3)" }}>{full}</span>); }
    } else if (lang === "css") {
      const [, str, prop, selector, atRule, num, word, punct, ws, op] = m;
      if (str) { results.push(<span key={key} style={{ color: "var(--success)" }}>{str}</span>); }
      else if (prop) { results.push(<span key={key} style={{ color: "var(--info)" }}>{prop}</span>); }
      else if (selector) { results.push(<span key={key} style={{ color: "var(--amber)" }}>{selector}</span>); }
      else if (atRule) { results.push(<span key={key} style={{ color: "var(--violet)" }}>{atRule}</span>); }
      else if (num) { results.push(<span key={key} style={{ color: "var(--amber)" }}>{num}</span>); }
      else if (word && keywords.has(word)) { results.push(<span key={key} style={{ color: "var(--violet)" }}>{word}</span>); }
      else if (word) { results.push(<span key={key} style={{ color: "var(--success)" }}>{word}</span>); }
      else if (punct) { results.push(<span key={key} style={{ color: "var(--text-3)" }}>{punct}</span>); }
      else if (ws) { results.push(<span key={key}>{ws}</span>); }
      else { results.push(<span key={key} style={{ color: "var(--text-3)" }}>{full}</span>); }
    } else {
      const [, str, num, word, ws, op] = m;
      if (str) { results.push(<span key={key} style={{ color: "var(--success)" }}>{str}</span>); }
      else if (num) { results.push(<span key={key} style={{ color: "var(--amber)" }}>{num}</span>); }
      else if (word && keywords.has(word)) { results.push(<span key={key} style={{ color: "var(--violet)" }}>{word}</span>); }
      else if (word) { results.push(<span key={key} style={{ color: "var(--text-2)" }}>{word}</span>); }
      else if (ws) { results.push(<span key={key}>{ws}</span>); }
      else if (op) { results.push(<span key={key} style={{ color: "var(--text-3)" }}>{op}</span>); }
      else { results.push(<span key={key}>{full}</span>); }
    }
  }

  return results;
}

// ── Component ─────────────────────────────────────────────────
interface CodeRenderProps {
  content: string;
  language?: string;
  filePath?: string;
  error?: boolean;
  maxHeight?: string;
}

export function CodeRender({
  content,
  language,
  filePath,
  error = false,
  maxHeight = "480px",
}: CodeRenderProps) {
  const lang = language ?? detectLanguage(filePath, content);
  const isJson = lang === "json";
  const hasHighlight = isJson || lang !== "text";

  return (
    <div style={{ position: "relative" }}>
      {/* File path header */}
      {filePath && (
        <div
          style={{
            fontFamily: "var(--font-mono)", fontSize: "11px",
            color: "var(--text-3)", padding: "6px 14px",
            background: "var(--bg-3)",
            borderRadius: "var(--radius-md) var(--radius-md) 0 0",
            border: `1px solid ${error ? "rgba(255,69,69,0.25)" : "var(--border-1)"}`,
            borderBottom: "none",
            overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
          }}
          title={filePath}
        >
          {filePath}
        </div>
      )}
      {/* Language badge */}
      <div
        style={{
          position: "absolute",
          top: filePath ? "38px" : "10px",
          right: "12px",
          fontFamily: "var(--font-mono)", fontSize: "10px",
          letterSpacing: "0.1em", textTransform: "uppercase",
          color: "var(--text-3)", pointerEvents: "none",
          background: "var(--bg-3)", padding: "2px 8px",
          borderRadius: "var(--radius-sm)", border: "1px solid var(--border-1)",
          zIndex: 1,
        }}
      >
        {lang}
      </div>
      <pre
        style={{
          margin: 0, padding: "14px 16px", paddingTop: "28px",
          background: "var(--bg-2)",
          border: `1px solid ${error ? "rgba(255,69,69,0.25)" : "var(--border-1)"}`,
          borderRadius: filePath ? "0 0 var(--radius-md) var(--radius-md)" : "var(--radius-md)",
          fontFamily: "var(--font-mono)", fontSize: "13px", lineHeight: 1.65,
          overflowX: "auto", whiteSpace: "pre-wrap", wordBreak: "break-word",
          maxHeight, overflowY: "auto",
        }}
      >
        {isJson ? (
          colorizeJson(content)
        ) : hasHighlight ? (
          colorizeCode(content, lang)
        ) : (
          <span style={{ color: error ? "var(--red)" : "var(--text-2)" }}>{content}</span>
        )}
      </pre>
    </div>
  );
}
