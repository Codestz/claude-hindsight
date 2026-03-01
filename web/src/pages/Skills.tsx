import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { api } from "@/lib/api";
import type { SkillConfig } from "@/lib/types";
import { PageShell } from "@/components/ui/PageShell";
import { LoadingState } from "@/components/ui/LoadingState";
import { ErrorState } from "@/components/ui/ErrorState";
import { EmptyState } from "@/components/ui/EmptyState";
import { Sparkles, Cpu, Wrench, Terminal } from "lucide-react";

function ScopeBadge({ scope }: { scope: string }) {
  const isGlobal = scope === "global";
  const isPlugin = scope.startsWith("plugin:");
  const label = isPlugin ? scope.replace("plugin:", "") : scope;
  const color = isGlobal ? "var(--cyan)" : isPlugin ? "var(--purple)" : "var(--amber)";

  return (
    <span
      style={{
        fontFamily: "var(--font-mono)",
        fontSize: "10px",
        fontWeight: 600,
        color,
        background: `color-mix(in srgb, ${color} 12%, transparent)`,
        border: `1px solid color-mix(in srgb, ${color} 25%, transparent)`,
        borderRadius: "var(--radius-sm)",
        padding: "1px 6px",
      }}
    >
      {label}
    </span>
  );
}

function SkillCard({ skill }: { skill: SkillConfig }) {
  const [hovered, setHovered] = useState(false);

  return (
    <Link
      to={`/skills/${encodeURIComponent(skill.name)}`}
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
        }}
      >
        {/* Header */}
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <span style={{ color: "var(--cyan)", display: "flex" }}>
            <Sparkles size={18} strokeWidth={2} />
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
            {skill.name}
          </span>
          <ScopeBadge scope={skill.scope} />
        </div>

        {/* Description */}
        {skill.description && (
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
            {skill.description}
          </div>
        )}

        {/* Meta row */}
        <div style={{ display: "flex", flexWrap: "wrap", gap: "8px", marginTop: "auto" }}>
          {skill.user_invocable && (
            <span
              style={{
                display: "flex",
                alignItems: "center",
                gap: "4px",
                fontFamily: "var(--font-mono)",
                fontSize: "11px",
                color: "var(--green)",
              }}
            >
              <Terminal size={11} />
              /{skill.name}
            </span>
          )}
          {skill.model && (
            <span
              style={{
                display: "flex",
                alignItems: "center",
                gap: "4px",
                fontFamily: "var(--font-mono)",
                fontSize: "11px",
                color: "var(--text-3)",
              }}
            >
              <Cpu size={11} />
              {skill.model.replace(/^claude-/, "").replace(/-\d{8}$/, "")}
            </span>
          )}
          {skill.allowed_tools && skill.allowed_tools.length > 0 && (
            <span
              style={{
                display: "flex",
                alignItems: "center",
                gap: "4px",
                fontFamily: "var(--font-mono)",
                fontSize: "11px",
                color: "var(--text-3)",
              }}
            >
              <Wrench size={11} />
              {skill.allowed_tools.length} tools
            </span>
          )}
        </div>
      </div>
    </Link>
  );
}

function FilterButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        fontFamily: "var(--font-mono)",
        fontSize: "11px",
        fontWeight: 600,
        padding: "4px 10px",
        borderRadius: "var(--radius-sm)",
        border: active ? "1px solid var(--accent)" : "1px solid var(--border-2)",
        background: active ? "rgba(0,255,136,0.08)" : "transparent",
        color: active ? "var(--accent)" : "var(--text-3)",
        cursor: "pointer",
        transition: "all 0.12s",
      }}
    >
      {children}
    </button>
  );
}

export default function SkillsPage() {
  const [skills, setSkills] = useState<SkillConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [scopeFilter, setScopeFilter] = useState<string>("all");

  useEffect(() => {
    api.skills()
      .then(setSkills)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <PageShell><LoadingState /></PageShell>;
  if (error) return <PageShell><ErrorState message={error} /></PageShell>;

  const scopes = Array.from(new Set(skills.map((s) => s.scope))).sort();
  const filtered = scopeFilter === "all" ? skills : skills.filter((s) => s.scope === scopeFilter);

  return (
    <PageShell>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <h1
            style={{
              fontSize: "20px",
              fontWeight: 600,
              color: "var(--text-1)",
              fontFamily: "var(--font-sans)",
              margin: 0,
            }}
          >
            Skills
          </h1>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)" }}>
            {skills.length}
          </span>
        </div>

        {scopes.length > 1 && (
          <div style={{ display: "flex", gap: "4px" }}>
            <FilterButton active={scopeFilter === "all"} onClick={() => setScopeFilter("all")}>
              All
            </FilterButton>
            {scopes.map((s) => (
              <FilterButton key={s} active={scopeFilter === s} onClick={() => setScopeFilter(s)}>
                {s}
              </FilterButton>
            ))}
          </div>
        )}
      </div>

      {filtered.length === 0 ? (
        <EmptyState
          title="No skills found"
          description="Add skill directories to ~/.claude/skills/ or your project's .claude/skills/ directory"
        />
      ) : (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))",
            gap: "12px",
          }}
        >
          {filtered.map((skill) => (
            <SkillCard key={`${skill.scope}-${skill.name}`} skill={skill} />
          ))}
        </div>
      )}
    </PageShell>
  );
}
