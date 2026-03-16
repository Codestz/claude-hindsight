/**
 * Shared types for session detail components.
 *
 * All component prop interfaces and internal types live here.
 * Components import types from this file — no inline definitions.
 */

import type { NodeResponse, OtelSessionSummary, SessionFile, SessionStats, TurnCost } from "@/lib/types";
import type { SortOrder } from "@/hooks";
import type { ViewMode } from "@/pages/SessionDetail";

// ── NodeDetailPanel ──────────────────────────────────────────

export interface NodeDetailPanelProps {
  node: NodeResponse | null;
  flatNodes?: NodeResponse[];
  onNavigate?: (node: NodeResponse) => void;
}

// ── ExecutionRow ─────────────────────────────────────────────

export interface ExecutionRowProps {
  node: NodeResponse;
  isSelected: boolean;
  onSelect: () => void;
}

// ── ExecutionList ────────────────────────────────────────────

export interface ExecutionListProps {
  nodes: NodeResponse[];
  selectedId: string | null;
  onSelect: (node: NodeResponse) => void;
  autoScroll: boolean;
  newestFirst?: boolean;
}

export type DisplayItem =
  | { kind: "node"; node: NodeResponse }
  | { kind: "group"; nodes: NodeResponse[]; label: string };

// ── ExecutionGraph ───────────────────────────────────────────

export interface ExecutionGraphProps {
  roots: NodeResponse[];
  selectedId: string | null;
  onSelect: (node: NodeResponse) => void;
}

export interface GraphNode {
  id: string;
  node: NodeResponse;
  color: string;
  r: number;
}

export interface GraphLink {
  source: string;
  target: string;
}

// ── SessionHeader ────────────────────────────────────────────

export interface SessionHeaderProps {
  session: SessionFile;
  otelSummary: OtelSessionSummary | null;
  stats: SessionStats;
}

// ── SessionFilterBar ─────────────────────────────────────────

export interface SessionFilterBarProps {
  filterTypes: Set<string>;
  onToggleType: (type: string) => void;
  filterKeyword: string;
  onKeywordChange: (keyword: string) => void;
  autoScroll: boolean;
  onToggleAutoScroll: () => void;
  activeFilePaths: Set<string>;
  onRemoveFilePath: (path: string) => void;
  onClearFilePaths: () => void;
  viewMode?: ViewMode;
  onViewModeChange?: (mode: ViewMode) => void;
  sortOrder?: SortOrder;
  onSortOrderChange?: (order: SortOrder) => void;
  filteredCount?: number;
  totalCount?: number;
}

// ── TimelineScrubber ─────────────────────────────────────────

export interface TimelineScrubberProps {
  nodes: NodeResponse[];
  selectedId: string | null;
  onSelect: (node: NodeResponse) => void;
  turnCosts?: TurnCost[];
}

// ── ImagePreview ─────────────────────────────────────────────

export interface ImageData {
  mediaType: string;
  data: string;
}

export interface ImagePreviewProps {
  img: ImageData;
  index: number;
}

// ── TaskNotification ─────────────────────────────────────────

export interface TaskNotification {
  taskId: string | null;
  status: string | null;
  summary: string | null;
  result: string | null;
  totalTokens: string | null;
  toolUses: string | null;
  durationMs: string | null;
}

export interface TaskNotificationCardProps {
  task: TaskNotification;
}

// ── ThinkingBlock ────────────────────────────────────────────

export interface ThinkingBlockProps {
  text: string;
}

// ── ProgressIndicator ────────────────────────────────────────

export interface ProgressIndicatorProps {
  percentage: number;
  label?: string;
}
