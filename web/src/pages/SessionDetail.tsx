import React, { lazy, Suspense, useMemo } from "react";
import { useParams } from "react-router-dom";
import { flattenTree, computeSessionStats } from "@/lib/node-utils";
import { useSessionData, useNodeFiltering } from "@/hooks";
import { SessionHeader } from "@/components/session/SessionHeader";
import { SessionFilterBar } from "@/components/session/SessionFilterBar";
import { TimelineScrubber } from "@/components/session/TimelineScrubber";
import { ResizablePanel } from "@/components/ui/ResizablePanel";
import { ExecutionList } from "@/components/session/ExecutionList";
import { NodeDetailPanel } from "@/components/session/NodeDetailPanel";

export type ViewMode = "list" | "graph";
export type { SortOrder } from "@/hooks";

// Lazy-load the 3D graph (Three.js is heavy)
const ExecutionGraph = lazy(() =>
  import("@/components/session/ExecutionGraph").then((m) => ({ default: m.ExecutionGraph })),
);

// Error boundary for lazy-loaded components (WebGL can fail)
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

// ── Root export ───────────────────────────────────────────────
export default function SessionDetailPage() {
  const { id: rawId } = useParams<{ id: string }>();
  const id = rawId ? decodeURIComponent(rawId) : null;

  if (!id) return <FullPageMessage>Loading&hellip;</FullPageMessage>;
  return <SessionDetail id={id} />;
}

// ── Session detail — two-panel resizable layout ─────────────
function SessionDetail({ id }: { id: string }) {
  const { session, roots, otelSummary, loading, error } = useSessionData(id);
  const [viewMode, setViewMode] = React.useState<ViewMode>("list");

  // Derived state
  const flatNodes = useMemo(() => flattenTree(roots), [roots]);
  const stats = useMemo(() => computeSessionStats(flatNodes, otelSummary), [flatNodes, otelSummary]);

  const {
    filterTypes, filterKeyword, sortOrder, autoScroll, activeFilePaths,
    selected, filteredNodes, sortedNodes,
    toggleFilterType, setFilterKeyword, setSortOrder, toggleAutoScroll,
    setSelected, removeFilePath, clearFilePaths,
  } = useNodeFiltering(flatNodes);

  if (loading) return <FullPageMessage>Loading&hellip;</FullPageMessage>;
  if (error || !session) return <FullPageMessage error>{error ?? "Session not found"}</FullPageMessage>;

  const leftPanel = viewMode === "graph" ? (
    <GraphErrorBoundary>
      <Suspense fallback={<div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%", color: "var(--text-3)", fontFamily: "var(--font-mono)", fontSize: "12px" }}>Loading 3D graph&hellip;</div>}>
        <ExecutionGraph roots={roots} selectedId={selected?.uuid ?? null} onSelect={setSelected} />
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
    <div style={{ height: "calc(100vh - 56px)", display: "flex", flexDirection: "column", padding: "12px 20px 0", boxSizing: "border-box" }}>
      <SessionHeader session={session} otelSummary={otelSummary} stats={stats} />

      <SessionFilterBar
        filterTypes={filterTypes}
        onToggleType={toggleFilterType}
        filterKeyword={filterKeyword}
        onKeywordChange={setFilterKeyword}
        autoScroll={autoScroll}
        onToggleAutoScroll={toggleAutoScroll}
        activeFilePaths={activeFilePaths}
        onRemoveFilePath={removeFilePath}
        onClearFilePaths={clearFilePaths}
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
          right={<NodeDetailPanel node={selected} flatNodes={flatNodes} onNavigate={setSelected} />}
        />
      </div>
    </div>
  );
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
