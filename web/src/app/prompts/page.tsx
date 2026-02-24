"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { api } from "@/lib/api";
import type { PromptEntry } from "@/lib/types";
import { timeAgo, shortModel, shortId } from "@/lib/utils";
import { SectionHeader } from "@/components/ui/SectionHeader";

const MAX_W = "1400px";
const PAGE_PAD = "28px";
const SECTION_GAP = "20px";

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
        <div
          style={{
            height: "60vh",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: "14px",
            color: "var(--text-3)",
          }}
        >
          Loading...
        </div>
      </PageShell>
    );
  }

  if (error) {
    return (
      <PageShell>
        <div
          style={{
            height: "60vh",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexDirection: "column",
            gap: "10px",
            textAlign: "center",
          }}
        >
          <div style={{ fontSize: "14px", color: "var(--red)" }}>{error}</div>
          <div style={{ fontSize: "13px", color: "var(--text-3)" }}>
            Is{" "}
            <code
              style={{
                fontFamily: "var(--font-mono)",
                color: "var(--accent)",
              }}
            >
              hindsight serve
            </code>{" "}
            running on :7227?
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
          <div
            style={{
              padding: "40px 20px",
              textAlign: "center",
              fontSize: "13px",
              color: "var(--text-3)",
            }}
          >
            No prompts found
          </div>
        </Card>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
          {prompts.map((p, i) => {
            const hovered = hoveredIdx === i;
            return (
              <Link
                key={`${p.session_id}-${i}`}
                href={`/sessions/${encodeURIComponent(p.session_id)}/`}
                onMouseEnter={() => setHoveredIdx(i)}
                onMouseLeave={() => setHoveredIdx(null)}
                style={{
                  display: "block",
                  padding: "16px 20px",
                  background: hovered ? "var(--bg-2)" : "var(--bg-1)",
                  border: `1px solid ${hovered ? "var(--border-3)" : "var(--border-1)"}`,
                  borderRadius: "var(--radius-lg)",
                  textDecoration: "none",
                  color: "inherit",
                  transition: "background 0.1s, border-color 0.15s",
                }}
              >
                {/* Prompt text — main content, up to 3 lines */}
                <div
                  style={{
                    fontSize: "14px",
                    lineHeight: "1.5",
                    color: "var(--text-1)",
                    display: "-webkit-box",
                    WebkitLineClamp: 3,
                    WebkitBoxOrient: "vertical",
                    overflow: "hidden",
                    marginBottom: "10px",
                  }}
                >
                  {p.prompt_text}
                </div>

                {/* Meta row */}
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "16px",
                    fontSize: "12px",
                    fontFamily: "var(--font-mono)",
                    color: "var(--text-3)",
                  }}
                >
                  <span
                    style={{
                      color: "var(--amber)",
                      fontWeight: 500,
                    }}
                  >
                    {p.project_name}
                  </span>

                  <span style={{ color: "var(--border-3)" }}>·</span>

                  <span>{shortModel(p.model)}</span>

                  <span style={{ color: "var(--border-3)" }}>·</span>

                  <span>{p.timestamp ? timeAgo(p.timestamp) : "—"}</span>

                  <span style={{ color: "var(--border-3)" }}>·</span>

                  <span style={{ color: "var(--text-3)" }}>
                    {shortId(p.session_id)}
                  </span>
                </div>
              </Link>
            );
          })}
        </div>
      )}
    </PageShell>
  );
}

function PageShell({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        maxWidth: MAX_W,
        margin: "0 auto",
        padding: `36px ${PAGE_PAD}`,
        display: "flex",
        flexDirection: "column",
        gap: SECTION_GAP,
      }}
    >
      {children}
    </div>
  );
}

function Card({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        background: "var(--bg-1)",
        border: "1px solid var(--border-1)",
        borderRadius: "var(--radius-lg)",
        overflow: "hidden",
      }}
    >
      {children}
    </div>
  );
}
