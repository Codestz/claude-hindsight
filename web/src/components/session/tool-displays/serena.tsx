/**
 * Serena MCP result display and parsing.
 */

import { CodeRender } from "@/components/ui/CodeRender";
import { MarkdownContent } from "@/components/ui/MarkdownContent";
import { ContentSection } from "./primitives";

type SerenaResult =
  | { type: "text"; text: string }
  | { type: "structured"; data: Record<string, unknown> };

export function parseSerenaResult(content: string): SerenaResult | null {
  try {
    const outer = JSON.parse(content);
    if (typeof outer !== "object" || outer === null || !("result" in outer)) return null;
    const resultVal = (outer as Record<string, unknown>).result;
    if (typeof resultVal !== "string") return null;
    try {
      const inner = JSON.parse(resultVal);
      if (typeof inner === "object" && inner !== null)
        return { type: "structured", data: inner as Record<string, unknown> };
    } catch { /* plain text */ }
    return { type: "text", text: resultVal };
  } catch {
    return null;
  }
}

function PathList({ items }: { items: string[] }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "2px" }}>
      {items.map((item, i) => (
        <div key={i} style={{
          fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-2)",
          padding: "2px 0", borderBottom: "1px solid var(--border-1)", wordBreak: "break-all",
        }}>
          {item}
        </div>
      ))}
    </div>
  );
}

export function SerenaResultDisplay({ content }: { content: string }) {
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
        <ContentSection label={`Dirs \u00b7 ${dirs.length}`} color="var(--cyan)">
          <PathList items={dirs} />
        </ContentSection>
      )}
      {files.length > 0 && (
        <ContentSection label={`Files \u00b7 ${files.length}`} color="var(--green)">
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
