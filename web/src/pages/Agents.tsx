import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { api } from "@/lib/api";
import type { AgentConfig, AgentGroup } from "@/lib/types";
import { PageShell } from "@/components/ui/PageShell";
import { LoadingState } from "@/components/ui/LoadingState";
import { ErrorState } from "@/components/ui/ErrorState";
import { EmptyState } from "@/components/ui/EmptyState";
import { ScopeBadge } from "@/components/ui/ScopeBadge";
import { FilterChips } from "@/components/ui/FilterChips";
import { Bot, Cpu, Wrench, Sparkles } from "lucide-react";

// ── Single-scope card ────────────────────────────────────────

function AgentCard({ agent }: { agent: AgentConfig }) {
  const [hovered, setHovered] = useState(false);

  return (
    <Link
      to={`/agents/${encodeURIComponent(agent.name)}`}
      style={{ textDecoration: "none" }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div
        style={{
          background: hovered ? "var(--bg-2)" : "var(--bg-1)",
          border: `1px solid ${hovered ? "var(--border-2)" : "var(--border-1)"}`,
          borderRadius: "var(--radius-lg)",
          padding: "20px",
          transition: "all 0.15s",
          display: "flex",
          flexDirection: "column",
          gap: "12px",
          height: "100%",
          boxShadow: hovered ? "var(--shadow-md)" : "var(--shadow-sm)",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <span style={{ color: "var(--purple)", display: "flex" }}>
            <Bot size={18} strokeWidth={2} />
          </span>
          <span
            style={{
              fontSize: "14px",
              fontWeight: 600,
              color: "var(--text-1)",
              fontFamily: "var(--font-mono)",
              flex: 1,
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
          >
            {agent.name}
          </span>
          <ScopeBadge scope={agent.scope} />
        </div>

        {agent.description && (
          <div
            style={{
              fontSize: "13px",
              color: "var(--text-2)",
              fontFamily: "var(--font-sans)",
              lineHeight: 1.5,
              display: "-webkit-box",
              WebkitLineClamp: 2,
              WebkitBoxOrient: "vertical",
              overflow: "hidden",
            }}
          >
            {agent.description}
          </div>
        )}

        <div style={{ display: "flex", flexWrap: "wrap", gap: "8px", marginTop: "auto" }}>
          {agent.model && (
            <span style={{ display: "flex", alignItems: "center", gap: "4px", fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)", background: "var(--bg-2)", padding: "2px 8px", borderRadius: "var(--radius-sm)" }}>
              <Cpu size={11} />{agent.model.replace(/^claude-/, "").replace(/-\d{8}$/, "")}
            </span>
          )}
          {agent.tools && agent.tools.length > 0 && (
            <span style={{ display: "flex", alignItems: "center", gap: "4px", fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)", background: "var(--bg-2)", padding: "2px 8px", borderRadius: "var(--radius-sm)" }}>
              <Wrench size={11} />{agent.tools.length} tools
            </span>
          )}
          {agent.skills && agent.skills.length > 0 && (
            <span style={{ display: "flex", alignItems: "center", gap: "4px", fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)", background: "var(--bg-2)", padding: "2px 8px", borderRadius: "var(--radius-sm)" }}>
              <Sparkles size={11} />{agent.skills.length} skills
            </span>
          )}
        </div>
      </div>
    </Link>
  );
}

// ── Multi-scope merged card ──────────────────────────────────

function MergedAgentCard({ group }: { group: AgentGroup }) {
  const [hovered, setHovered] = useState(false);
  const first = group.items[0];

  return (
    <Link
      to={`/agents/${encodeURIComponent(group.name)}`}
      style={{ textDecoration: "none" }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div
        style={{
          background: hovered ? "var(--bg-2)" : "var(--bg-1)",
          border: `1px solid ${hovered ? "var(--border-2)" : "var(--border-1)"}`,
          borderRadius: "var(--radius-lg)",
          padding: "20px",
          transition: "all 0.15s",
          display: "flex",
          flexDirection: "column",
          gap: "12px",
          height: "100%",
          boxShadow: hovered ? "var(--shadow-md)" : "var(--shadow-sm)",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <span style={{ color: "var(--purple)", display: "flex" }}>
            <Bot size={18} strokeWidth={2} />
          </span>
          <span
            style={{
              fontSize: "14px",
              fontWeight: 600,
              color: "var(--text-1)",
              fontFamily: "var(--font-mono)",
              flex: 1,
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
          >
            {group.name}
          </span>
          <div style={{ display: "flex", gap: "4px" }}>
            {group.items.map((a) => <ScopeBadge key={a.file_path} scope={a.scope} />)}
          </div>
        </div>

        {first.description && (
          <div
            style={{
              fontSize: "13px",
              color: "var(--text-2)",
              fontFamily: "var(--font-sans)",
              lineHeight: 1.5,
              display: "-webkit-box",
              WebkitLineClamp: 2,
              WebkitBoxOrient: "vertical",
              overflow: "hidden",
            }}
          >
            {first.description}
          </div>
        )}

        <div style={{ display: "flex", flexWrap: "wrap", gap: "8px", marginTop: "auto" }}>
          {first.model && (
            <span style={{ display: "flex", alignItems: "center", gap: "4px", fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)", background: "var(--bg-2)", padding: "2px 8px", borderRadius: "var(--radius-sm)" }}>
              <Cpu size={11} />{first.model.replace(/^claude-/, "").replace(/-\d{8}$/, "")}
            </span>
          )}
          {first.tools && first.tools.length > 0 && (
            <span style={{ display: "flex", alignItems: "center", gap: "4px", fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)", background: "var(--bg-2)", padding: "2px 8px", borderRadius: "var(--radius-sm)" }}>
              <Wrench size={11} />{first.tools.length} tools
            </span>
          )}
          {first.skills && first.skills.length > 0 && (
            <span style={{ display: "flex", alignItems: "center", gap: "4px", fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)", background: "var(--bg-2)", padding: "2px 8px", borderRadius: "var(--radius-sm)" }}>
              <Sparkles size={11} />{first.skills.length} skills
            </span>
          )}
          {group.identical ? (
            <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)" }}>
              Shared across {group.items.length} scopes
            </span>
          ) : (
            <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--amber)" }}>
              ⚠ Differs across scopes
            </span>
          )}
        </div>
      </div>
    </Link>
  );
}

// ── Page ─────────────────────────────────────────────────────

export default function AgentsPage() {
  const [groups, setGroups] = useState<AgentGroup[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeScopes, setActiveScopes] = useState<Set<string>>(new Set());

  useEffect(() => {
    api.agents()
      .then(setGroups)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <PageShell><LoadingState /></PageShell>;
  if (error) return <PageShell><ErrorState message={error} /></PageShell>;

  const scopes = Array.from(new Set(groups.flatMap((g) => g.items.map((a) => a.scope)))).sort();
  const filtered = activeScopes.size === 0
    ? groups
    : groups.filter((g) => g.items.some((a) => activeScopes.has(a.scope.toLowerCase())));

  return (
    <PageShell>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <h1 style={{ fontSize: "20px", fontWeight: 600, color: "var(--text-1)", fontFamily: "var(--font-sans)", margin: 0 }}>
            Agents
          </h1>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)" }}>
            {groups.length}
          </span>
        </div>

        {scopes.length > 1 && (
          <FilterChips
            options={scopes}
            active={activeScopes}
            onToggle={(s) => {
              setActiveScopes((prev) => {
                const next = new Set(prev);
                if (next.has(s)) next.delete(s); else next.add(s);
                return next;
              });
            }}
          />
        )}
      </div>

      {filtered.length === 0 ? (
        <EmptyState
          title="No agents found"
          description="Add agent markdown files to ~/.claude/agents/ or your project's .claude/agents/ directory"
        />
      ) : (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))", gap: "12px" }}>
          {filtered.map((group, i) => (
            <div key={group.name} className="animate-in" style={{ "--delay": `${i * 0.06}s` } as React.CSSProperties}>
              {group.items.length === 1
                ? <AgentCard agent={group.items[0]} />
                : <MergedAgentCard group={group} />}
            </div>
          ))}
        </div>
      )}
    </PageShell>
  );
}
