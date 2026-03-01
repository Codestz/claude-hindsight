import { CodeRender } from "@/components/ui/CodeRender";

export function MarkdownContent({ text }: { text: string }) {
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
