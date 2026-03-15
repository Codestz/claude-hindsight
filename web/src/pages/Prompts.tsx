import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { api } from "@/lib/api";
import type { PromptEntry } from "@/lib/types";
import { timeAgo, shortModel, shortId } from "@/lib/utils";
import { PageShell } from "@/components/ui/PageShell";
import { Card } from "@/components/ui/Card";
import { SectionHeader } from "@/components/ui/SectionHeader";

export default function PromptsPage() {
  const [prompts, setPrompts] = useState<PromptEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hoveredIdx, setHoveredIdx] = useState<number | null>(null);

  useEffect(() => {
    api
      .prompts({ limit: 200 })
      .then(setPrompts)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <PageShell>
        <div style={{ height: "60vh", display: "flex", alignItems: "center", justifyContent: "center", fontSize: "14px", color: "var(--text-3)" }}>
          Loading...
        </div>
      </PageShell>
    );
  }

  if (error) {
    return (
      <PageShell>
        <div style={{ height: "60vh", display: "flex", alignItems: "center", justifyContent: "center", flexDirection: "column", gap: "10px", textAlign: "center" }}>
          <div style={{ fontSize: "14px", color: "var(--red)" }}>{error}</div>
          <div style={{ fontSize: "13px", color: "var(--text-3)" }}>
            Is <code style={{ fontFamily: "var(--font-mono)", color: "var(--accent)" }}>hindsight serve</code> running on :7227?
          </div>
        </div>
      </PageShell>
    );
  }

  return (
    <PageShell>
      <SectionHeader title="Prompts" count={prompts.length} />

      {prompts.length === 0 ? (
        <Card>
          <div style={{ padding: "40px 20px", textAlign: "center", fontSize: "13px", color: "var(--text-3)" }}>
            No prompts found
          </div>
        </Card>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
          {prompts.map((p, i) => {
            const hovered = hoveredIdx === i;
            return (
              <div key={`${p.session_id}-${i}`} className="animate-in" style={{ "--delay": `${Math.min(i * 0.04, 0.4)}s` } as React.CSSProperties}>
              <Link
                to={`/sessions/${encodeURIComponent(p.session_id)}`}
                onMouseEnter={() => setHoveredIdx(i)}
                onMouseLeave={() => setHoveredIdx(null)}
                style={{
                  display: "block",
                  padding: "16px 20px",
                  background: hovered ? "var(--bg-2)" : "var(--bg-1)",
                  border: `1px solid ${hovered ? "var(--border-3)" : "var(--border-1)"}`,
                  borderLeft: hovered ? "2px solid var(--indigo)" : "2px solid transparent",
                  borderRadius: "var(--radius-lg)",
                  textDecoration: "none",
                  color: "inherit",
                  transition: "background 0.1s, border-color 0.15s, box-shadow 0.15s",
                  boxShadow: hovered ? "var(--shadow-md)" : "var(--shadow-sm)",
                }}
              >
                <div style={{
                  fontSize: "14px", lineHeight: "1.5", color: "var(--text-1)",
                  display: "-webkit-box", WebkitLineClamp: 3, WebkitBoxOrient: "vertical",
                  overflow: "hidden", marginBottom: "10px",
                }}>
                  {p.prompt_text}
                </div>
                <div style={{
                  display: "flex", alignItems: "center", gap: "16px",
                  fontSize: "12px", fontFamily: "var(--font-mono)", color: "var(--text-3)",
                }}>
                  <span style={{ color: "var(--amber)", fontWeight: 500 }}>{p.project_name}</span>
                  <span style={{ color: "var(--border-3)" }}>·</span>
                  <span>{shortModel(p.model)}</span>
                  <span style={{ color: "var(--border-3)" }}>·</span>
                  <span>{p.timestamp ? timeAgo(p.timestamp) : "—"}</span>
                  <span style={{ color: "var(--border-3)" }}>·</span>
                  <span style={{ color: "var(--text-3)" }}>{shortId(p.session_id)}</span>
                </div>
              </Link>
              </div>
            );
          })}
        </div>
      )}
    </PageShell>
  );
}
