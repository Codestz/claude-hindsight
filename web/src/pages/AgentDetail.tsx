import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { api } from "@/lib/api";
import type { AgentConfig } from "@/lib/types";
import { PageShell } from "@/components/ui/PageShell";
import { Card } from "@/components/ui/Card";
import { LoadingState } from "@/components/ui/LoadingState";
import { ErrorState } from "@/components/ui/ErrorState";
import {
  Bot,
  ChevronLeft,
  Cpu,
  Wrench,
  Sparkles,
  Shield,
  RotateCcw,
  FileText,
} from "lucide-react";

function ConfigRow({ icon: Icon, label, value }: {
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  label: string;
  value: string;
}) {
  return (
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
        <Icon size={14} strokeWidth={2} />
      </span>
      <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", width: "120px", flexShrink: 0 }}>
        {label}
      </span>
      <span style={{ fontFamily: "var(--font-mono)", fontSize: "13px", color: "var(--text-1)" }}>
        {value}
      </span>
    </div>
  );
}

function TagList({ items, color }: { items: string[]; color: string }) {
  return (
    <div style={{ display: "flex", flexWrap: "wrap", gap: "4px" }}>
      {items.map((item) => (
        <span
          key={item}
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "11px",
            fontWeight: 500,
            color,
            background: `color-mix(in srgb, ${color} 10%, transparent)`,
            border: `1px solid color-mix(in srgb, ${color} 20%, transparent)`,
            borderRadius: "var(--radius-sm)",
            padding: "2px 8px",
          }}
        >
          {item}
        </span>
      ))}
    </div>
  );
}

export default function AgentDetailPage() {
  const { name } = useParams<{ name: string }>();
  const [agent, setAgent] = useState<AgentConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!name) return;
    api.agent(name)
      .then(setAgent)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, [name]);

  if (loading) return <PageShell><LoadingState /></PageShell>;
  if (error || !agent) return <PageShell><ErrorState message={error ?? "Agent not found"} /></PageShell>;

  return (
    <PageShell>
      {/* Back link */}
      <Link
        to="/agents"
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
        Agents
      </Link>

      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", gap: "14px" }}>
        <span
          style={{
            width: "40px",
            height: "40px",
            borderRadius: "var(--radius-lg)",
            background: "color-mix(in srgb, var(--purple) 12%, transparent)",
            border: "1px solid color-mix(in srgb, var(--purple) 25%, transparent)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "var(--purple)",
            flexShrink: 0,
          }}
        >
          <Bot size={20} />
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
            {agent.name}
          </h1>
          {agent.description && (
            <div
              style={{
                fontSize: "14px",
                color: "var(--text-2)",
                fontFamily: "var(--font-sans)",
                marginTop: "4px",
              }}
            >
              {agent.description}
            </div>
          )}
        </div>
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px", alignItems: "start" }}>
        {/* Config table */}
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
            {agent.model && <ConfigRow icon={Cpu} label="model" value={agent.model} />}
            {agent.permission_mode && <ConfigRow icon={Shield} label="permissions" value={agent.permission_mode} />}
            {agent.max_turns != null && <ConfigRow icon={RotateCcw} label="max_turns" value={String(agent.max_turns)} />}
            {agent.background != null && <ConfigRow icon={Bot} label="background" value={agent.background ? "true" : "false"} />}
            {agent.isolation && <ConfigRow icon={Bot} label="isolation" value={agent.isolation} />}
            <ConfigRow icon={FileText} label="scope" value={agent.scope} />
          </div>
        </Card>

        {/* Relationships */}
        <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
          {/* Tools */}
          {agent.tools && agent.tools.length > 0 && (
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
                <Wrench size={12} /> Tools ({agent.tools.length})
              </div>
              <TagList items={agent.tools} color="var(--amber)" />
            </Card>
          )}

          {/* Skills */}
          {agent.skills && agent.skills.length > 0 && (
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
                <Sparkles size={12} /> Skills ({agent.skills.length})
              </div>
              <div style={{ display: "flex", flexWrap: "wrap", gap: "4px" }}>
                {agent.skills.map((skill) => (
                  <Link
                    key={skill}
                    to={`/skills/${encodeURIComponent(skill)}`}
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "11px",
                      fontWeight: 500,
                      color: "var(--cyan)",
                      background: "color-mix(in srgb, var(--cyan) 10%, transparent)",
                      border: "1px solid color-mix(in srgb, var(--cyan) 20%, transparent)",
                      borderRadius: "var(--radius-sm)",
                      padding: "2px 8px",
                      textDecoration: "none",
                    }}
                  >
                    {skill}
                  </Link>
                ))}
              </div>
            </Card>
          )}

          {/* File path */}
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
              {agent.file_path}
            </div>
          </Card>
        </div>
      </div>

      {/* System prompt / body */}
      {agent.body && (
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
            System prompt
          </div>
          <div
            style={{
              padding: "16px",
              fontFamily: "var(--font-mono)",
              fontSize: "13px",
              lineHeight: 1.6,
              color: "var(--text-2)",
              whiteSpace: "pre-wrap",
              maxHeight: "500px",
              overflowY: "auto",
            }}
          >
            {agent.body}
          </div>
        </Card>
      )}
    </PageShell>
  );
}
