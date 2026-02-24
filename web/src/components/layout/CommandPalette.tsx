"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { SearchIcon } from "@/components/icons/SearchIcon";
import { api } from "@/lib/api";
import type { SessionFile } from "@/lib/types";
import { shortId, shortModel, timeAgo } from "@/lib/utils";

interface Props {
  open: boolean;
  onClose: () => void;
}

// Quick-links shown before the user types anything
const QUICK_LINKS = [
  { label: "Dashboard", href: "/", category: "Navigate" },
  { label: "Projects", href: "/projects", category: "Navigate" },
];

type ResultItem =
  | { kind: "link"; label: string; href: string; category: string }
  | { kind: "session"; session: SessionFile };

export function CommandPalette({ open, onClose }: Props) {
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [sessionResults, setSessionResults] = useState<SessionFile[]>([]);
  const [searching, setSearching] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const router = useRouter();

  // Build unified result list
  const items: ResultItem[] = [];

  // Quick links (always shown, filtered by query)
  const filteredLinks = QUICK_LINKS.filter((l) =>
    l.label.toLowerCase().includes(query.toLowerCase())
  );
  for (const link of filteredLinks) {
    items.push({ kind: "link", ...link });
  }

  // Session results (from API search)
  for (const session of sessionResults) {
    items.push({ kind: "session", session });
  }

  // Focus input and reset selection when opened; clear query on close
  useEffect(() => {
    if (open) {
      inputRef.current?.focus();
      setSelectedIndex(0);
      setSessionResults([]);
    } else {
      setQuery("");
      setSessionResults([]);
    }
  }, [open]);

  // Debounced search
  const doSearch = useCallback((q: string) => {
    if (debounceRef.current) clearTimeout(debounceRef.current);

    if (!q.trim()) {
      setSessionResults([]);
      setSearching(false);
      return;
    }

    setSearching(true);
    debounceRef.current = setTimeout(async () => {
      try {
        // Parse special syntax: @tool, errors
        let searchQ = q;
        let tool: string | undefined;
        let errors = false;

        const toolMatch = q.match(/@(\w+)/);
        if (toolMatch) {
          tool = toolMatch[1];
          searchQ = q.replace(/@\w+/, "").trim();
        }
        if (searchQ.toLowerCase().includes("errors")) {
          errors = true;
          searchQ = searchQ.replace(/errors/i, "").trim();
        }

        const results = await api.search({ q: searchQ || undefined, tool, errors: errors || undefined });
        setSessionResults(results.slice(0, 10));
      } catch {
        setSessionResults([]);
      } finally {
        setSearching(false);
      }
    }, 300);
  }, []);

  // Reset selection and trigger search when query changes
  useEffect(() => {
    setSelectedIndex(0);
    doSearch(query);
  }, [query, doSearch]);

  // Keyboard navigation (↑↓ Enter) + Escape to close
  useEffect(() => {
    if (!open) return;

    const handler = (e: KeyboardEvent) => {
      switch (e.key) {
        case "Escape":
          onClose();
          break;
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((i) =>
            items.length === 0 ? 0 : (i + 1) % items.length
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((i) =>
            items.length === 0 ? 0 : i <= 0 ? items.length - 1 : i - 1
          );
          break;
        case "Enter": {
          const item = items[selectedIndex];
          if (item) {
            if (item.kind === "link") {
              router.push(item.href);
            } else {
              router.push(`/sessions/${encodeURIComponent(item.session.session_id)}/`);
            }
            onClose();
          }
          break;
        }
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose, items, selectedIndex, router]);

  if (!open) return null;

  return (
    // Backdrop
    <div
      onClick={onClose}
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 200,
        background: "rgba(0, 0, 0, 0.65)",
        backdropFilter: "blur(3px)",
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "center",
        paddingTop: "15vh",
      }}
    >
      {/* Panel */}
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          width: "100%",
          maxWidth: "560px",
          margin: "0 16px",
          background: "var(--bg-1)",
          border: "1px solid var(--border-2)",
          borderRadius: "var(--radius-lg)",
          boxShadow: "0 32px 80px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(0,255,136,0.06)",
          overflow: "hidden",
        }}
      >
        {/* Search row */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "10px",
            padding: "12px 16px",
            borderBottom: "1px solid var(--border-1)",
          }}
        >
          <SearchIcon size={14} stroke="var(--text-3)" />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search sessions… (@tool, errors)"
            style={{
              flex: 1,
              background: "transparent",
              border: "none",
              outline: "none",
              color: "var(--text-1)",
              fontSize: "14px",
              fontFamily: "var(--font-sans)",
              caretColor: "var(--accent)",
            }}
          />
          <Kbd>ESC</Kbd>
        </div>

        {/* Results */}
        <div style={{ padding: "6px 0", maxHeight: "420px", overflowY: "auto" }}>
          {/* Quick links section */}
          {filteredLinks.length > 0 && (
            <>
              <SectionHeader>{query ? "Quick links" : "Quick links"}</SectionHeader>
              {filteredLinks.map((link, i) => {
                const globalIdx = i;
                return (
                  <Link
                    key={link.href}
                    href={link.href}
                    onClick={onClose}
                    onMouseEnter={() => setSelectedIndex(globalIdx)}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      padding: "8px 16px",
                      fontSize: "13px",
                      color: "var(--text-1)",
                      cursor: "pointer",
                      background: globalIdx === selectedIndex ? "var(--bg-2)" : "transparent",
                      transition: "background 0.1s",
                      textDecoration: "none",
                    }}
                  >
                    <span>{link.label}</span>
                    <span style={{ fontSize: "10px", fontFamily: "var(--font-mono)", color: "var(--text-3)" }}>
                      {link.category}
                    </span>
                  </Link>
                );
              })}
            </>
          )}

          {/* Sessions section */}
          {sessionResults.length > 0 && (
            <>
              <SectionHeader>Sessions</SectionHeader>
              {sessionResults.map((session, i) => {
                const globalIdx = filteredLinks.length + i;
                return (
                  <div
                    key={session.session_id}
                    onClick={() => {
                      router.push(`/sessions/${encodeURIComponent(session.session_id)}/`);
                      onClose();
                    }}
                    onMouseEnter={() => setSelectedIndex(globalIdx)}
                    style={{
                      display: "flex",
                      flexDirection: "column",
                      gap: "2px",
                      padding: "8px 16px",
                      cursor: "pointer",
                      background: globalIdx === selectedIndex ? "var(--bg-2)" : "transparent",
                      transition: "background 0.1s",
                    }}
                  >
                    <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                      <span style={{
                        fontSize: "13px", color: "var(--text-1)",
                        overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1,
                      }}>
                        {session.first_message || shortId(session.session_id)}
                      </span>
                      <span style={{ fontSize: "11px", fontFamily: "var(--font-mono)", color: "var(--text-3)", flexShrink: 0 }}>
                        {timeAgo(session.modified_at)}
                      </span>
                    </div>
                    <div style={{ display: "flex", alignItems: "center", gap: "8px", fontSize: "11px", color: "var(--text-3)" }}>
                      <span style={{ fontFamily: "var(--font-mono)" }}>{session.project_name}</span>
                      {session.model && (
                        <span style={{ fontFamily: "var(--font-mono)" }}>{shortModel(session.model)}</span>
                      )}
                      {session.error_count > 0 && (
                        <span style={{ color: "var(--red)" }}>{session.error_count} errors</span>
                      )}
                    </div>
                  </div>
                );
              })}
            </>
          )}

          {/* Loading/empty states */}
          {searching && (
            <div style={{ padding: "20px 16px", textAlign: "center", fontSize: "13px", color: "var(--text-3)" }}>
              Searching…
            </div>
          )}
          {query && !searching && sessionResults.length === 0 && filteredLinks.length === 0 && (
            <div style={{ padding: "28px 16px", textAlign: "center", fontSize: "13px", color: "var(--text-3)" }}>
              No results for &ldquo;{query}&rdquo;
            </div>
          )}
        </div>

        {/* Footer hint */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "16px",
            padding: "8px 16px",
            borderTop: "1px solid var(--border-1)",
            fontSize: "11px",
            color: "var(--text-3)",
          }}
        >
          <span style={{ display: "flex", alignItems: "center", gap: "4px" }}>
            <Kbd>↑↓</Kbd> navigate
          </span>
          <span style={{ display: "flex", alignItems: "center", gap: "4px" }}>
            <Kbd>↵</Kbd> open
          </span>
          <span style={{ display: "flex", alignItems: "center", gap: "4px" }}>
            <Kbd>ESC</Kbd> close
          </span>
          <span style={{ marginLeft: "auto", fontFamily: "var(--font-mono)", fontSize: "10px" }}>
            @tool · errors
          </span>
        </div>
      </div>
    </div>
  );
}

function SectionHeader({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        padding: "4px 16px 6px",
        fontSize: "10px",
        fontFamily: "var(--font-mono)",
        fontWeight: 600,
        letterSpacing: "0.1em",
        textTransform: "uppercase",
        color: "var(--text-3)",
      }}
    >
      {children}
    </div>
  );
}

function Kbd({ children }: { children: React.ReactNode }) {
  return (
    <kbd
      style={{
        fontFamily: "var(--font-mono)",
        fontSize: "10px",
        fontWeight: 500,
        color: "var(--text-3)",
        background: "var(--bg-2)",
        border: "1px solid var(--border-2)",
        borderRadius: "var(--radius-sm)",
        padding: "1px 5px",
        lineHeight: 1.6,
        whiteSpace: "nowrap",
      }}
    >
      {children}
    </kbd>
  );
}
