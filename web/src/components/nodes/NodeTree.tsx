"use client";

import type { TreeResponse } from "@/lib/types";
import { NodeRow } from "./NodeRow";

interface NodeTreeProps {
  tree: TreeResponse;
}

export function NodeTree({ tree }: NodeTreeProps) {
  return (
    <div className="space-y-0.5">
      {tree.roots.map((root, i) => (
        <NodeRow key={root.uuid ?? i} node={root} defaultExpanded={true} />
      ))}
      {tree.roots.length === 0 && (
        <div className="mono text-sm text-center py-8" style={{ color: "var(--text-3)" }}>No nodes in this session</div>
      )}
    </div>
  );
}
