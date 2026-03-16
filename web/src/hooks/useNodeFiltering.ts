/**
 * Custom hook for node filtering, sorting, and search state.
 *
 * Encapsulates:
 * - Filter chip state (type-based filtering)
 * - Keyword search
 * - Sort order (newest/oldest)
 * - Auto-scroll toggle
 * - File path filtering
 * - Derived filtered + sorted node list
 * - Auto-selection of first node
 *
 * @param flatNodes - Flat list of all nodes (from flattenTree)
 * @returns Filter state, handlers, and derived sorted nodes
 */

import { useEffect, useMemo, useRef, useState } from "react";
import type { NodeResponse } from "@/lib/types";
import { filterNodes } from "@/lib/node-utils";

export type SortOrder = "newest" | "oldest";

interface NodeFilteringResult {
  // State
  filterTypes: Set<string>;
  filterKeyword: string;
  sortOrder: SortOrder;
  autoScroll: boolean;
  activeFilePaths: Set<string>;
  selected: NodeResponse | null;
  filteredNodes: NodeResponse[];
  sortedNodes: NodeResponse[];

  // Handlers
  toggleFilterType: (type: string) => void;
  setFilterKeyword: (keyword: string) => void;
  setSortOrder: (order: SortOrder) => void;
  toggleAutoScroll: () => void;
  setSelected: (node: NodeResponse | null) => void;
  addFilePath: (path: string) => void;
  removeFilePath: (path: string) => void;
  clearFilePaths: () => void;
}

export function useNodeFiltering(flatNodes: NodeResponse[]): NodeFilteringResult {
  const [filterTypes, setFilterTypes] = useState<Set<string>>(new Set());
  const [filterKeyword, setFilterKeyword] = useState("");
  const [sortOrder, setSortOrder] = useState<SortOrder>("newest");
  const [autoScroll, setAutoScroll] = useState(true);
  const [activeFilePaths, setActiveFilePaths] = useState<Set<string>>(new Set());
  const [selected, setSelected] = useState<NodeResponse | null>(null);
  const autoSelectedRef = useRef(false);

  const toggleFilterType = (type: string) => {
    setFilterTypes((prev) => {
      const next = new Set(prev);
      if (next.has(type)) next.delete(type);
      else next.add(type);
      return next;
    });
  };

  const toggleAutoScroll = () => setAutoScroll((prev) => !prev);

  const addFilePath = (path: string) => {
    setActiveFilePaths((prev) => new Set(prev).add(path));
  };

  const removeFilePath = (path: string) => {
    setActiveFilePaths((prev) => {
      const next = new Set(prev);
      next.delete(path);
      return next;
    });
  };

  const clearFilePaths = () => setActiveFilePaths(new Set());

  // Derived: filtered nodes
  const filteredNodes = useMemo(
    () => filterNodes(flatNodes, { types: filterTypes, keyword: filterKeyword }, activeFilePaths),
    [flatNodes, filterTypes, filterKeyword, activeFilePaths],
  );

  // Derived: sorted nodes
  const sortedNodes = useMemo(() => {
    if (sortOrder === "oldest") return filteredNodes;
    return [...filteredNodes].reverse();
  }, [filteredNodes, sortOrder]);

  // Auto-select first node on initial load
  useEffect(() => {
    if (autoSelectedRef.current || sortedNodes.length === 0) return;
    autoSelectedRef.current = true;
    setSelected(sortedNodes[0]);
  }, [sortedNodes]);

  return {
    filterTypes,
    filterKeyword,
    sortOrder,
    autoScroll,
    activeFilePaths,
    selected,
    filteredNodes,
    sortedNodes,
    toggleFilterType,
    setFilterKeyword,
    setSortOrder,
    toggleAutoScroll,
    setSelected,
    addFilePath,
    removeFilePath,
    clearFilePaths,
  };
}
