/**
 * Code colorization functions for syntax highlighting.
 *
 * Provides JSON, HTML (with embedded CSS/JS), and generic code colorization.
 * Uses the language keyword sets from ./languages.ts.
 *
 * These are pure render functions — no state, no side effects.
 */

import React from "react";
import { getKeywords } from "./languages";

// ── JSON ─────────────────────────────────────────────────────

export function colorizeJson(json: string): React.ReactNode {
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

// ── Generic code colorization ────────────────────────────────

/** Entry point — routes HTML to embedded handler, others to block colorizer. */
export function colorizeCode(code: string, lang: string): React.ReactNode {
  if (lang === "html") return colorizeHtmlWithEmbedded(code);
  return colorizeBlock(code, lang);
}

/** Split HTML by <style>/<script> blocks and colorize each segment in its language. */
function colorizeHtmlWithEmbedded(code: string): React.ReactNode {
  const parts: React.ReactNode[] = [];
  const embedRe = /(<style[^>]*>)([\s\S]*?)(<\/style>)|(<script[^>]*>)([\s\S]*?)(<\/script>)/gi;
  let lastIdx = 0;
  let segKey = 0;
  let m: RegExpExecArray | null;

  while ((m = embedRe.exec(code)) !== null) {
    if (m.index > lastIdx) {
      parts.push(<React.Fragment key={`h${segKey++}`}>{colorizeBlock(code.slice(lastIdx, m.index), "html")}</React.Fragment>);
    }
    if (m[1] != null) {
      parts.push(<React.Fragment key={`st${segKey++}`}>{colorizeBlock(m[1], "html")}</React.Fragment>);
      parts.push(<React.Fragment key={`sc${segKey++}`}>{colorizeBlock(m[2], "css")}</React.Fragment>);
      parts.push(<React.Fragment key={`se${segKey++}`}>{colorizeBlock(m[3], "html")}</React.Fragment>);
    } else {
      parts.push(<React.Fragment key={`jt${segKey++}`}>{colorizeBlock(m[4], "html")}</React.Fragment>);
      parts.push(<React.Fragment key={`jc${segKey++}`}>{colorizeBlock(m[5], "javascript")}</React.Fragment>);
      parts.push(<React.Fragment key={`je${segKey++}`}>{colorizeBlock(m[6], "html")}</React.Fragment>);
    }
    lastIdx = m.index + m[0].length;
  }

  if (lastIdx < code.length) {
    parts.push(<React.Fragment key={`t${segKey++}`}>{colorizeBlock(code.slice(lastIdx), "html")}</React.Fragment>);
  }

  return <>{parts}</>;
}

/** Colorize a single-language block line by line. */
function colorizeBlock(code: string, lang: string): React.ReactNode {
  const lines = code.split("\n");
  return lines.map((line, li) => {
    const parts: React.ReactNode[] = [];
    let remaining = line;
    let ki = 0;

    while (remaining.length > 0) {
      // HTML comments <!-- -->
      if (lang === "html" && remaining.includes("<!--")) {
        const m = remaining.match(/^(.*?)(<!--[\s\S]*?-->)(.*)?$/);
        if (m) {
          if (m[1]) { parts.push(...tokenizeLine(m[1], lang, ki)); ki += m[1].length; }
          parts.push(<span key={`c${li}-${ki}`} style={{ color: "var(--text-3)", fontStyle: "italic" }}>{m[2]}</span>);
          remaining = m[3] ?? "";
          ki += m[2].length;
          continue;
        }
      }
      // CSS comments /* */
      if (lang === "css" && remaining.includes("/*")) {
        const m = remaining.match(/^(.*?)(\/\*[\s\S]*?\*\/)(.*)?$/);
        if (m) {
          if (m[1]) { parts.push(...tokenizeLine(m[1], lang, ki)); ki += m[1].length; }
          parts.push(<span key={`c${li}-${ki}`} style={{ color: "var(--text-3)", fontStyle: "italic" }}>{m[2]}</span>);
          remaining = m[3] ?? "";
          ki += m[2].length;
          continue;
        }
      }
      // Line comments (// or #)
      if (lang !== "html" && lang !== "css") {
        const cm = (lang === "python" || lang === "bash" || lang === "ruby")
          ? remaining.match(/^(.*?)(#.*)$/)
          : remaining.match(/^(.*?)(\/\/.*)$/);
        if (cm && cm[1] !== undefined) {
          if (cm[1]) { parts.push(...tokenizeLine(cm[1], lang, ki)); ki += cm[1].length; }
          parts.push(<span key={`c${li}-${ki}`} style={{ color: "var(--text-3)", fontStyle: "italic" }}>{cm[2]}</span>);
          remaining = "";
          break;
        }
      }

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

// ── Token-level colorization ─────────────────────────────────

/** Tokenize a single line and return colored spans. */
function tokenizeLine(line: string, lang: string, keyOffset: number): React.ReactNode[] {
  const results: React.ReactNode[] = [];
  const keywords = getKeywords(lang);

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
      const [, str, tag, tagClose, attr, num, word, ws] = m;
      if (str) results.push(<span key={key} style={{ color: "var(--success)" }}>{str}</span>);
      else if (tag) results.push(<span key={key} style={{ color: "var(--rose)" }}>{tag}</span>);
      else if (tagClose) results.push(<span key={key} style={{ color: "var(--rose)" }}>{tagClose}</span>);
      else if (attr) results.push(<span key={key} style={{ color: "var(--amber)" }}>{attr}</span>);
      else if (num) results.push(<span key={key} style={{ color: "var(--amber)" }}>{num}</span>);
      else if (word && keywords.has(word)) results.push(<span key={key} style={{ color: "var(--violet)" }}>{word}</span>);
      else if (word) results.push(<span key={key} style={{ color: "var(--text-2)" }}>{word}</span>);
      else if (ws) results.push(<span key={key}>{ws}</span>);
      else results.push(<span key={key} style={{ color: "var(--text-3)" }}>{full}</span>);
    } else if (lang === "css") {
      const [, str, prop, selector, atRule, num, word, punct, ws] = m;
      if (str) results.push(<span key={key} style={{ color: "var(--success)" }}>{str}</span>);
      else if (prop) results.push(<span key={key} style={{ color: "var(--info)" }}>{prop}</span>);
      else if (selector) results.push(<span key={key} style={{ color: "var(--amber)" }}>{selector}</span>);
      else if (atRule) results.push(<span key={key} style={{ color: "var(--violet)" }}>{atRule}</span>);
      else if (num) results.push(<span key={key} style={{ color: "var(--amber)" }}>{num}</span>);
      else if (word && keywords.has(word)) results.push(<span key={key} style={{ color: "var(--violet)" }}>{word}</span>);
      else if (word) results.push(<span key={key} style={{ color: "var(--success)" }}>{word}</span>);
      else if (punct) results.push(<span key={key} style={{ color: "var(--text-3)" }}>{punct}</span>);
      else if (ws) results.push(<span key={key}>{ws}</span>);
      else results.push(<span key={key} style={{ color: "var(--text-3)" }}>{full}</span>);
    } else {
      const [, str, num, word, ws, op] = m;
      if (str) results.push(<span key={key} style={{ color: "var(--success)" }}>{str}</span>);
      else if (num) results.push(<span key={key} style={{ color: "var(--amber)" }}>{num}</span>);
      else if (word && keywords.has(word)) results.push(<span key={key} style={{ color: "var(--violet)" }}>{word}</span>);
      else if (word) results.push(<span key={key} style={{ color: "var(--text-2)" }}>{word}</span>);
      else if (ws) results.push(<span key={key}>{ws}</span>);
      else if (op) results.push(<span key={key} style={{ color: "var(--text-3)" }}>{op}</span>);
      else results.push(<span key={key}>{full}</span>);
    }
  }

  return results;
}
