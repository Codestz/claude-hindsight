import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { api } from "@/lib/api";
import type { SkillConfig, SkillReference } from "@/lib/types";
import { PageShell } from "@/components/ui/PageShell";
import { Card } from "@/components/ui/Card";
import { LoadingState } from "@/components/ui/LoadingState";
import { ErrorState } from "@/components/ui/ErrorState";
import {
  Sparkles,
  ChevronLeft,
  ChevronRight,
  ChevronDown,
  Cpu,
  Wrench,
  Terminal,
  Bot,
  FileText,
  BookOpen,
  Scale,
  Webhook,
} from "lucide-react";
import { MarkdownContent } from "@/components/ui/MarkdownContent";
import { ConfigRow } from "@/components/ui/ConfigRow";

function ReferenceRow({ item }: { item: SkillReference }) {
  const [open, setOpen] = useState(false);
  return (
    <div style={{ borderBottom: "1px solid var(--border-1)" }}>
      <button
        onClick={() => setOpen(!open)}
        style={{
          display: "flex",
          alignItems: "center",
          gap: "8px",
          width: "100%",
          padding: "10px 16px",
          background: "none",
          border: "none",
          cursor: "pointer",
          fontFamily: "var(--font-mono)",
          fontSize: "13px",
          color: "var(--text-1)",
          textAlign: "left",
        }}
      >
        {open ? (
          <ChevronDown size={14} style={{ color: "var(--text-3)", flexShrink: 0 }} />
        ) : (
          <ChevronRight size={14} style={{ color: "var(--text-3)", flexShrink: 0 }} />
        )}
        {item.name}
      </button>
      {open && (
        <div
          style={{
            padding: "0 16px 14px 38px",
            maxHeight: "400px",
            overflowY: "auto",
          }}
        >
          <MarkdownContent text={item.content} />
        </div>
      )}
    </div>
  );
}

function ReferencesSection({
  references,
  category,
  label,
  icon: Icon,
}: {
  references: SkillReference[];
  category: string;
  label: string;
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
}) {
  const filtered = references.filter((r) => r.category === category);
  if (filtered.length === 0) return null;

  return (
    <Card>
      <div
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "10px",
          fontWeight: 600,
          letterSpacing: "0.1em",
          textTransform: "uppercase",
          color: "var(--text-3)",
          padding: "14px 16px",
          borderBottom: "1px solid var(--border-1)",
          display: "flex",
          alignItems: "center",
          gap: "6px",
        }}
      >
        <Icon size={12} strokeWidth={2} />
        {label} ({filtered.length})
      </div>
      <div>
        {filtered.map((r) => (
          <ReferenceRow key={r.path} item={r} />
        ))}
      </div>
    </Card>
  );
}

export default function SkillDetailPage() {
  const { name } = useParams<{ name: string }>();
  const [skill, setSkill] = useState<SkillConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!name) return;
    api.skill(name)
      .then(setSkill)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, [name]);

  if (loading) return <PageShell><LoadingState /></PageShell>;
  if (error || !skill) return <PageShell><ErrorState message={error ?? "Skill not found"} /></PageShell>;

  return (
    <PageShell>
      {/* Back link */}
      <Link
        to="/skills"
        style={{
          display: "flex",
          alignItems: "center",
          gap: "4px",
          fontSize: "13px",
          color: "var(--text-3)",
          textDecoration: "none",
          width: "fit-content",
        }}
      >
        <ChevronLeft size={14} />
        Skills
      </Link>

      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", gap: "14px" }}>
        <span
          style={{
            width: "40px",
            height: "40px",
            borderRadius: "var(--radius-lg)",
            background: "color-mix(in srgb, var(--cyan) 12%, transparent)",
            border: "1px solid color-mix(in srgb, var(--cyan) 25%, transparent)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "var(--cyan)",
            flexShrink: 0,
          }}
        >
          <Sparkles size={20} />
        </span>
        <div>
          <h1
            style={{
              fontSize: "20px",
              fontWeight: 600,
              fontFamily: "var(--font-mono)",
              color: "var(--text-1)",
              margin: 0,
            }}
          >
            {skill.name}
          </h1>
          {skill.description && (
            <div
              style={{
                fontSize: "14px",
                color: "var(--text-2)",
                fontFamily: "var(--font-sans)",
                marginTop: "4px",
              }}
            >
              {skill.description}
            </div>
          )}
        </div>
      </div>

      {/* Config + Tools row (2-col) */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px", alignItems: "start" }}>
        <Card>
          <div
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "10px",
              fontWeight: 600,
              letterSpacing: "0.1em",
              textTransform: "uppercase",
              color: "var(--text-3)",
              padding: "14px 16px 0",
            }}
          >
            Configuration
          </div>
          <div style={{ marginTop: "10px" }}>
            {skill.user_invocable != null && (
              <ConfigRow icon={Terminal} label="invocable" value={skill.user_invocable ? `true (/${skill.name})` : "false"} />
            )}
            {skill.model && <ConfigRow icon={Cpu} label="model" value={skill.model} />}
            {skill.context && <ConfigRow icon={FileText} label="context" value={skill.context} />}
            {skill.agent && (
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "10px",
                  padding: "10px 16px",
                  borderBottom: "1px solid var(--border-1)",
                }}
              >
                <span style={{ color: "var(--text-3)", display: "flex", flexShrink: 0 }}>
                  <Bot size={14} strokeWidth={2} />
                </span>
                <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", width: "120px", flexShrink: 0 }}>
                  agent
                </span>
                <Link
                  to={`/agents/${encodeURIComponent(skill.agent)}`}
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "13px",
                    color: "var(--purple)",
                    textDecoration: "none",
                  }}
                >
                  {skill.agent}
                </Link>
              </div>
            )}
            <ConfigRow icon={FileText} label="scope" value={skill.scope} />
          </div>
        </Card>

        <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
          {skill.allowed_tools && skill.allowed_tools.length > 0 && (
            <Card padding="16px">
              <div
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "10px",
                  fontWeight: 600,
                  letterSpacing: "0.1em",
                  textTransform: "uppercase",
                  color: "var(--text-3)",
                  marginBottom: "10px",
                  display: "flex",
                  alignItems: "center",
                  gap: "6px",
                }}
              >
                <Wrench size={12} /> Allowed tools ({skill.allowed_tools.length})
              </div>
              <div style={{ display: "flex", flexWrap: "wrap", gap: "4px" }}>
                {skill.allowed_tools.map((tool) => (
                  <span
                    key={tool}
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "11px",
                      fontWeight: 500,
                      color: "var(--amber)",
                      background: "color-mix(in srgb, var(--amber) 10%, transparent)",
                      border: "1px solid color-mix(in srgb, var(--amber) 20%, transparent)",
                      borderRadius: "var(--radius-sm)",
                      padding: "2px 8px",
                    }}
                  >
                    {tool}
                  </span>
                ))}
              </div>
            </Card>
          )}

          {/* Hooks */}
          {skill.hooks != null && (
            <Card>
              <div
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "10px",
                  fontWeight: 600,
                  letterSpacing: "0.1em",
                  textTransform: "uppercase",
                  color: "var(--text-3)",
                  padding: "14px 16px",
                  borderBottom: "1px solid var(--border-1)",
                  display: "flex",
                  alignItems: "center",
                  gap: "6px",
                }}
              >
                <Webhook size={12} /> Hooks
              </div>
              <pre
                style={{
                  padding: "16px",
                  fontFamily: "var(--font-mono)",
                  fontSize: "12px",
                  lineHeight: 1.6,
                  color: "var(--text-2)",
                  margin: 0,
                  whiteSpace: "pre-wrap",
                  overflowX: "auto",
                }}
              >
                {String(JSON.stringify(skill.hooks, null, 2))}
              </pre>
            </Card>
          )}

          <Card padding="16px">
            <div
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "10px",
                fontWeight: 600,
                letterSpacing: "0.1em",
                textTransform: "uppercase",
                color: "var(--text-3)",
                marginBottom: "8px",
              }}
            >
              Source file
            </div>
            <div
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "12px",
                color: "var(--text-2)",
                wordBreak: "break-all",
              }}
            >
              {skill.file_path}
            </div>
          </Card>
        </div>
      </div>

      {/* Instructions / body */}
      {skill.body && (
        <Card>
          <div
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "10px",
              fontWeight: 600,
              letterSpacing: "0.1em",
              textTransform: "uppercase",
              color: "var(--text-3)",
              padding: "14px 16px",
              borderBottom: "1px solid var(--border-1)",
            }}
          >
            Instructions
          </div>
          <div
            style={{
              padding: "16px",
              maxHeight: "500px",
              overflowY: "auto",
            }}
          >
            <MarkdownContent text={skill.body} />
          </div>
        </Card>
      )}

      {/* Skill references and rules */}
      {skill.references && skill.references.length > 0 && (
        <>
          <ReferencesSection
            references={skill.references}
            category="reference"
            label="References"
            icon={BookOpen}
          />
          <ReferencesSection
            references={skill.references}
            category="rule"
            label="Rules"
            icon={Scale}
          />
        </>
      )}
    </PageShell>
  );
}
