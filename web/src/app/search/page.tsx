"use client";

import { useEffect, useState, useCallback } from "react";
import { useSearchParams } from "next/navigation";
import { Suspense } from "react";
import { api } from "@/lib/api";
import type { SessionFile } from "@/lib/types";
import { Header } from "@/components/layout/Header";
import { SessionCard } from "@/components/sessions/SessionCard";

function SearchContent() {
  const searchParams = useSearchParams();
  const initialQ = searchParams.get("q") ?? "";

  const [query, setQuery] = useState(initialQ);
  const [errorsOnly, setErrorsOnly] = useState(false);
  const [toolFilter, setToolFilter] = useState("");
  const [results, setResults] = useState<SessionFile[]>([]);
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);

  const doSearch = useCallback(
    async (q: string, errors: boolean, tool: string) => {
      setLoading(true);
      setSearched(true);
      try {
        const res = await api.search({
          q: q.trim() || undefined,
          errors: errors || undefined,
          tool: tool.trim() || undefined,
        });
        setResults(res);
      } catch (e) {
        console.error(e);
      } finally {
        setLoading(false);
      }
    },
    []
  );

  useEffect(() => {
    const t = setTimeout(() => {
      if (query.trim() || errorsOnly || toolFilter.trim()) {
        doSearch(query, errorsOnly, toolFilter);
      } else {
        setResults([]);
        setSearched(false);
      }
    }, 300);
    return () => clearTimeout(t);
  }, [query, errorsOnly, toolFilter, doSearch]);

  useEffect(() => {
    if (initialQ) doSearch(initialQ, false, "");
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="flex flex-col flex-1">
      <Header title="Search" subtitle="Full-text search across all sessions" />

      <div className="flex-1 p-4 space-y-4">
        {/* Search bar */}
        <div style={{ background: "var(--bg-2)", border: "1px solid var(--border-2)" }}>
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="search sessions, messages, projects…"
            autoFocus
            className="w-full mono text-sm outline-none"
            style={{
              background: "transparent",
              color: "var(--text-1)",
              padding: "0.875rem 1rem",
              caretColor: "var(--accent)",
              borderBottom: (errorsOnly || toolFilter) ? "1px solid var(--border)" : "none",
            }}
          />

          {/* Filter row */}
          {(errorsOnly || toolFilter || true) && (
            <div
              className="flex items-center gap-3 flex-wrap px-4 py-2"
              style={{ display: "flex" }}
            >
              <button
                onClick={() => setErrorsOnly((e) => !e)}
                className="mono transition-colors"
                style={{
                  fontSize: "10px",
                  padding: "2px 8px",
                  letterSpacing: "0.08em",
                  textTransform: "uppercase",
                  border: errorsOnly ? "1px solid rgba(255,69,69,0.4)" : "1px solid var(--border-2)",
                  color: errorsOnly ? "var(--red)" : "var(--text-3)",
                  background: errorsOnly ? "rgba(255,69,69,0.08)" : "transparent",
                  cursor: "pointer",
                }}
              >
                Errors only
              </button>

              <div className="flex items-center gap-2">
                <span className="mono" style={{ fontSize: "10px", color: "var(--text-3)", letterSpacing: "0.06em" }}>@tool:</span>
                <input
                  type="text"
                  value={toolFilter}
                  onChange={(e) => setToolFilter(e.target.value)}
                  placeholder="Read, Edit, Bash…"
                  className="mono outline-none"
                  style={{
                    background: "transparent",
                    color: "var(--text-1)",
                    fontSize: "11px",
                  }}
                />
              </div>

              {(query || errorsOnly || toolFilter) && (
                <button
                  onClick={() => {
                    setQuery("");
                    setErrorsOnly(false);
                    setToolFilter("");
                  }}
                  className="mono ml-auto"
                  style={{ fontSize: "10px", color: "var(--text-3)", cursor: "pointer", background: "none", border: "none" }}
                  onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-1)"; }}
                  onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-3)"; }}
                >
                  Clear
                </button>
              )}
            </div>
          )}
        </div>

        {/* Results */}
        {loading && (
          <div className="mono text-center py-8 animate-pulse" style={{ color: "var(--text-3)" }}>Searching…</div>
        )}

        {!loading && searched && results.length === 0 && (
          <div className="text-center py-16">
            <div className="mono text-sm" style={{ color: "var(--text-2)" }}>No sessions found</div>
            <div className="mono text-xs mt-1" style={{ color: "var(--text-3)" }}>Try different keywords or remove filters</div>
          </div>
        )}

        {!loading && !searched && (
          <div className="text-center py-16">
            <div className="mono text-sm" style={{ color: "var(--text-2)" }}>Type to search</div>
            <div className="mono text-xs mt-1" style={{ color: "var(--text-3)" }}>
              Searches session messages, project names, and more
            </div>
          </div>
        )}

        {!loading && results.length > 0 && (
          <div>
            <div className="mono mb-3" style={{ fontSize: "10px", color: "var(--text-3)", letterSpacing: "0.06em" }}>
              {results.length} result{results.length !== 1 ? "s" : ""}
            </div>
            <div className="space-y-1">
              {results.map((s) => (
                <SessionCard key={s.session_id} session={s} />
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default function SearchPage() {
  return (
    <Suspense fallback={
      <div className="flex flex-col flex-1">
        <Header title="Search" />
        <div className="flex-1 flex items-center justify-center">
          <div className="mono text-sm animate-pulse" style={{ color: "var(--text-2)" }}>Loading…</div>
        </div>
      </div>
    }>
      <SearchContent />
    </Suspense>
  );
}
