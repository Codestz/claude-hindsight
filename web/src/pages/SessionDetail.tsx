import React, { lazy, Suspense, useEffect, useMemo, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { api } from "@/lib/api";
import type { NodeResponse, OtelSessionSummary, SessionFile } from "@/lib/types";
import { flattenTree, computeSessionStats, filterNodes } from "@/lib/node-utils";
import { SessionHeader } from "@/components/session/SessionHeader";
import { SessionFilterBar } from "@/components/session/SessionFilterBar";
import { TimelineScrubber } from "@/components/session/TimelineScrubber";
import { ResizablePanel } from "@/components/ui/ResizablePanel";
import { ExecutionList } from "@/components/session/ExecutionList";
import { NodeDetailPanel } from "@/components/session/NodeDetailPanel";
// Lazy-load the 3D graph (Three.js is heavy)
const ExecutionGraph = lazy(() =>
  import("@/components/session/ExecutionGraph").then((m) => ({ default: m.ExecutionGraph })),
);

export type ViewMode = "list" | "graph";
export type SortOrder = "newest" | "oldest";

// ── Root export ───────────────────────────────────────────────
export default function SessionDetailPage() {
  const { id: rawId } = useParams<{ id: string }>();
  const id = rawId ? decodeURIComponent(rawId) : null;

  if (!id) {
    return <FullPageMessage>Loading&hellip;</FullPageMessage>;
  }

  return <SessionDetail id={id} />;
}

// ── Session detail — two-panel resizable layout ─────────────
function SessionDetail({ id }: { id: string }) {
  const [session, setSession] = useState<SessionFile | null>(null);
  const [roots, setRoots] = useState<NodeResponse[]>([]);
  const [selected, setSelected] = useState<NodeResponse | null>(null);
  const [otelSummary, setOtelSummary] = useState<OtelSessionSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filterTypes, setFilterTypes] = useState<Set<string>>(new Set());
  const [filterKeyword, setFilterKeyword] = useState("");
  const [autoScroll, setAutoScroll] = useState(true);
  const autoScrollRef = useRef(true);
  const [activeFilePaths, setActiveFilePaths] = useState<Set<string>>(new Set());
  const [viewMode, setViewMode] = useState<ViewMode>("list");
  const [sortOrder, setSortOrder] = useState<SortOrder>("newest");
  const autoSelectedRef = useRef(false);

  const toggleFilterType = (type: string) => {
    setFilterTypes((prev) => {
      const next = new Set(prev);
      if (next.has(type)) next.delete(type);
      else next.add(type);
      return next;
    });
  };

  // ── Initial data fetch ──────────────────────────────────────
  useEffect(() => {
    Promise.all([
      api.session(id),
      api.sessionNodes(id),
      api.otelSessionSummary(id).catch(() => null),
    ])
      .then(([s, tree, otel]) => {
        setSession(s);
        setRoots(tree.roots);
        setOtelSummary(otel);
      })
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, [id]);

  // ── Live SSE subscription ───────────────────────────────────
  useEffect(() => {
    const seenUuids = new Set<string>();

    const collectUuids = (nodes: NodeResponse[]) => {
      for (const n of nodes) {
        if (n.uuid) seenUuids.add(n.uuid);
        if (n.children?.length) collectUuids(n.children);
      }
    };
    setRoots((current) => { collectUuids(current); return current; });

    const es = new EventSource(api.eventsUrl(id));

    es.onmessage = (evt) => {
      try {
        const node: NodeResponse = JSON.parse(evt.data);
        const uid = node.uuid ?? node.node_type + Math.random();
        if (seenUuids.has(uid)) return;
        seenUuids.add(uid);
        setRoots((prev) => [...prev, node]);
      } catch {
        // ignore parse errors / heartbeats
      }
    };

    return () => es.close();
  }, [id]);

  // ── Derived state ───────────────────────────────────────────
  const flatNodes = useMemo(() => flattenTree(roots), [roots]);
  const stats = useMemo(() => computeSessionStats(flatNodes, otelSummary), [flatNodes, otelSummary]);
  const filteredNodes = useMemo(
    () => filterNodes(flatNodes, { types: filterTypes, keyword: filterKeyword }, activeFilePaths),
    [flatNodes, filterTypes, filterKeyword, activeFilePaths],
  );

  // Apply sort order
  const sortedNodes = useMemo(() => {
    if (sortOrder === "oldest") return filteredNodes;
    // "newest" — reverse so latest nodes appear first
    return [...filteredNodes].reverse();
  }, [filteredNodes, sortOrder]);

  // ── Auto-select first node on load ────────────────────────
  useEffect(() => {
    if (autoSelectedRef.current || sortedNodes.length === 0) return;
    autoSelectedRef.current = true;
    setSelected(sortedNodes[0]);
  }, [sortedNodes]);

  // ── File path filtering ─────────────────────────────────────
  const removeFilePath = (path: string) => {
    setActiveFilePaths((prev) => {
      const next = new Set(prev);
      next.delete(path);
      return next;
    });
  };

  if (loading) return <FullPageMessage>Loading&hellip;</FullPageMessage>;
  if (error || !session)
    return <FullPageMessage error>{error ?? "Session not found"}</FullPageMessage>;

  const leftPanel = viewMode === "graph" ? (
    <GraphErrorBoundary>
    <Suspense fallback={<div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%", color: "var(--text-3)", fontFamily: "var(--font-mono)", fontSize: "12px" }}>Loading 3D graph&hellip;</div>}>
      <ExecutionGraph
        roots={roots}
        selectedId={selected?.uuid ?? null}
        onSelect={setSelected}
      />
    </Suspense>
    </GraphErrorBoundary>
  ) : (
    <ExecutionList
      nodes={sortedNodes}
      selectedId={selected?.uuid ?? null}
      onSelect={setSelected}
      autoScroll={autoScroll}
      newestFirst={sortOrder === "newest"}
    />
  );

  return (
    <div style={{
      height: "calc(100vh - 56px)",
      display: "flex", flexDirection: "column",
      padding: "12px 20px 0",
      boxSizing: "border-box",
    }}>
      <SessionHeader session={session} otelSummary={otelSummary} stats={stats} />

      <SessionFilterBar
        filterTypes={filterTypes}
        onToggleType={toggleFilterType}
        filterKeyword={filterKeyword}
        onKeywordChange={setFilterKeyword}
        autoScroll={autoScroll}
        onToggleAutoScroll={() => {
          const next = !autoScroll;
          setAutoScroll(next);
          autoScrollRef.current = next;
        }}
        activeFilePaths={activeFilePaths}
        onRemoveFilePath={removeFilePath}
        onClearFilePaths={() => setActiveFilePaths(new Set())}
        viewMode={viewMode}
        onViewModeChange={setViewMode}
        sortOrder={sortOrder}
        onSortOrderChange={setSortOrder}
        filteredCount={filteredNodes.length}
        totalCount={flatNodes.length}
      />

      <TimelineScrubber
        nodes={flatNodes}
        selectedId={selected?.uuid ?? null}
        onSelect={setSelected}
        turnCosts={stats.turnCosts}
      />

      <div style={{ flex: 1, minHeight: 0 }}>
        <ResizablePanel
          storageKey="session-detail-panel"
          left={leftPanel}
          right={
            <NodeDetailPanel
              node={selected}
              flatNodes={flatNodes}
              onNavigate={setSelected}
            />
          }
        />
      </div>
    </div>
  );
}

// Error boundary for lazy-loaded components (3D graph can fail on WebGL issues)
class GraphErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean }
> {
  state = { hasError: false };
  static getDerivedStateFromError() { return { hasError: true }; }
  render() {
    if (this.state.hasError) {
      return (
        <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%", color: "var(--text-3)", fontFamily: "var(--font-mono)", fontSize: "12px", flexDirection: "column", gap: "8px" }}>
          <span style={{ fontSize: "18px" }}>⚠</span>
          <span>3D graph failed to load</span>
          <span style={{ fontSize: "10px", color: "var(--text-3)" }}>WebGL may not be available</span>
        </div>
      );
    }
    return this.props.children;
  }
}

function FullPageMessage({ children, error }: { children: React.ReactNode; error?: boolean }) {
  return (
    <div style={{
      height: "calc(100vh - 56px)", display: "flex", alignItems: "center",
      justifyContent: "center", fontSize: "14px",
      color: error ? "var(--red)" : "var(--text-3)",
      fontFamily: "var(--font-sans)", textAlign: "center",
    }}>
      {children}
    </div>
  );
}
