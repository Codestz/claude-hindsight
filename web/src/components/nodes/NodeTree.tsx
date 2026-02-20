"use client";

import { useState } from "react";
import type { NodeResponse } from "@/lib/types";
import { getNodeMeta, isInternalNode } from "@/lib/node-meta";
import { NodeRow } from "./NodeRow";

interface NodeTreeProps {
  nodes: NodeResponse[];
  onSelect: (node: NodeResponse) => void;
  selectedId: string | null;
}

export function NodeTree({ nodes, onSelect, selectedId }: NodeTreeProps) {
  return (
    <div>
      {nodes.map((node, i) => (
        <NodeTreeItem
          key={node.uuid ?? `root-${i}`}
          node={node}
          depth={0}
          onSelect={onSelect}
          selectedId={selectedId}
        />
      ))}
    </div>
  );
}

function NodeTreeItem({
  node,
  depth,
  onSelect,
  selectedId,
}: {
  node: NodeResponse;
  depth: number;
  onSelect: (node: NodeResponse) => void;
  selectedId: string | null;
}) {
  // Internal/decorative nodes start collapsed to reduce noise.
  const [expanded, setExpanded] = useState(!isInternalNode(node));

  const hasChildren = node.children.length > 0;
  const isSelected = node.uuid != null && node.uuid === selectedId;
  const meta = getNodeMeta(node);

  return (
    <>
      <NodeRow
        node={node}
        depth={depth}
        meta={meta}
        isSelected={isSelected}
        hasChildren={hasChildren}
        isExpanded={expanded}
        onSelect={() => onSelect(node)}
        onToggle={() => setExpanded((e) => !e)}
      />
      {hasChildren &&
        expanded &&
        node.children.map((child, i) => (
          <NodeTreeItem
            key={child.uuid ?? `${depth}-${i}`}
            node={child}
            depth={depth + 1}
            onSelect={onSelect}
            selectedId={selectedId}
          />
        ))}
    </>
  );
}
