"use client";

import { useEffect, useState, useCallback } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { Suspense } from "react";
import { api } from "@/lib/api";
import type { SessionFile } from "@/lib/types";
import { Header } from "@/components/layout/Header";
import { SessionCard } from "@/components/sessions/SessionCard";

function SearchContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
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

  // Debounced search
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

  // Initial search from URL param
  useEffect(() => {
    if (initialQ) doSearch(initialQ, false, "");
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="flex flex-col flex-1">
      <Header title="Search" subtitle="Full-text search across all sessions" />

      <div className="flex-1 p-6 space-y-6">
        {/* Search bar */}
        <div className="card p-4 rounded-lg space-y-3">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search sessions, messages, projects…"
            autoFocus
            className="w-full bg-transparent text-text-primary placeholder:text-text-muted outline-none text-base"
          />

          {/* Filter chips */}
          <div className="flex items-center gap-3 flex-wrap">
            <button
              onClick={() => setErrorsOnly((e) => !e)}
              className={`text-xs px-3 py-1 rounded-full border transition-all ${
                errorsOnly
                  ? "text-accent-red border-accent-red/50 bg-accent-red/10"
                  : "text-text-muted border-white/10 hover:border-white/20"
              }`}
            >
              Errors only
            </button>

            <div className="flex items-center gap-2">
              <span className="text-text-muted text-xs">@tool:</span>
              <input
                type="text"
                value={toolFilter}
                onChange={(e) => setToolFilter(e.target.value)}
                placeholder="Read, Edit, Bash…"
                className="bg-transparent text-text-primary text-xs placeholder:text-text-muted outline-none mono"
              />
            </div>

            {(query || errorsOnly || toolFilter) && (
              <button
                onClick={() => {
                  setQuery("");
                  setErrorsOnly(false);
                  setToolFilter("");
                }}
                className="text-xs text-text-muted hover:text-text-primary ml-auto"
              >
                Clear
              </button>
            )}
          </div>
        </div>

        {/* Results */}
        {loading && (
          <div className="text-center py-8 text-text-muted animate-pulse">Searching…</div>
        )}

        {!loading && searched && results.length === 0 && (
          <div className="text-center py-16">
            <div className="text-text-muted text-sm">No sessions found</div>
            <div className="text-text-muted text-xs mt-1">Try different keywords or remove filters</div>
          </div>
        )}

        {!loading && !searched && (
          <div className="text-center py-16">
            <div className="text-text-muted text-sm">Type to search</div>
            <div className="text-text-muted text-xs mt-1">
              Searches session messages, project names, and more
            </div>
          </div>
        )}

        {!loading && results.length > 0 && (
          <div>
            <div className="text-text-muted text-xs mb-3 mono">
              {results.length} result{results.length !== 1 ? "s" : ""}
            </div>
            <div className="space-y-3">
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
          <div className="text-text-muted animate-pulse">Loading…</div>
        </div>
      </div>
    }>
      <SearchContent />
    </Suspense>
  );
}
