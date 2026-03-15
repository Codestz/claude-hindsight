/**
 * Types for node tree components.
 */

import type { NodeResponse } from "@/lib/types";
import type { NodeMeta } from "@/lib/node-meta";
import type { NodeFilter } from "@/lib/node-utils";

export interface NodeRowProps {
  node: NodeResponse;
  depth: number;
  meta: NodeMeta;
  isSelected: boolean;
  hasChildren: boolean;
  isExpanded: boolean;
  onSelect: () => void;
  onToggle: () => void;
  promptScore?: number;
}

export interface NodeTreeProps {
  nodes: NodeResponse[];
  onSelect: (node: NodeResponse) => void;
  selectedId: string | null;
  filter?: NodeFilter;
}
