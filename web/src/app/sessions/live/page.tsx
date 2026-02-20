"use client";

import { useEffect, useRef, useState, Suspense } from "react";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { api } from "@/lib/api";
import type { NodeResponse } from "@/lib/types";
import { formatCost, formatTokens, shortId } from "@/lib/utils";
import { Header } from "@/components/layout/Header";
import { NodeRow } from "@/components/nodes/NodeRow";

type Status = "connecting" | "live" | "disconnected";

function LiveWatch() {
  const searchParams = useSearchParams();
  const id = searchParams.get("id") ?? "";

  const [nodes, setNodes] = useState<NodeResponse[]>([]);
  const [status, setStatus] = useState<Status>("connecting");
  const [totalCost, setTotalCost] = useState(0);
  const [totalTokens, setTotalTokens] = useState(0);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!id) { setStatus("disconnected"); return; }
    const url = api.eventsUrl(id);
    const es = new EventSource(url);

    es.onopen = () => setStatus("live");
    es.onmessage = (e) => {
      try {
        const node = JSON.parse(e.data) as NodeResponse;
        setNodes((prev) => [...prev, node]);
        const data = node as Record<string, unknown>;
        const cost = data.estimated_cost;
        const tokens = data.total_tokens;
        if (typeof cost === "number") setTotalCost((c) => c + cost);
        if (typeof tokens === "number") setTotalTokens((t) => t + tokens);
      } catch { /* heartbeat */ }
    };
    es.onerror = () => { setStatus("disconnected"); es.close(); };

    return () => es.close();
  }, [id]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [nodes]);

  const statusConfig = {
    connecting: { label: "Connecting…", color: "#64748b", dot: "bg-text-muted animate-pulse" },
    live: { label: "Live", color: "#4ade80", dot: "bg-accent-green animate-pulse" },
    disconnected: { label: "Disconnected", color: "#f87171", dot: "bg-accent-red" },
  };
  const sc = statusConfig[status];

  return (
    <div className="flex flex-col flex-1">
      <Header
        title={id ? `Live: ${shortId(id)}` : "Live Watch"}
        subtitle="Real-time session stream"
        actions={
          <div className="flex items-center gap-4">
            {totalCost > 0 && <div className="mono text-accent-yellow text-sm">{formatCost(totalCost)}</div>}
            {totalTokens > 0 && <div className="mono text-accent-green text-sm">{formatTokens(totalTokens)} tok</div>}
            <div className="flex items-center gap-2 px-3 py-1 rounded-full" style={{ background: "rgba(255,255,255,0.05)" }}>
              <span className={`w-2 h-2 rounded-full ${sc.dot}`} />
              <span className="text-xs" style={{ color: sc.color }}>{sc.label}</span>
            </div>
            {id && <Link href={`/sessions?id=${encodeURIComponent(id)}`} className="text-text-muted text-sm hover:text-text-primary">← Detail</Link>}
          </div>
        }
      />

      <div className="flex-1 overflow-auto p-4">
        {nodes.length === 0 && status !== "disconnected" && (
          <div className="text-center py-16">
            <div className="text-text-muted text-sm animate-pulse">Waiting for events…</div>
          </div>
        )}
        {nodes.length === 0 && status === "disconnected" && (
          <div className="text-center py-16">
            <div className="text-accent-red text-sm mb-2">Stream ended</div>
            {id && <Link href={`/sessions?id=${encodeURIComponent(id)}`} className="text-accent-cyan text-xs hover:underline">View full session →</Link>}
          </div>
        )}
        <div className="space-y-0.5">
          {nodes.map((node, i) => (
            <NodeRow key={node.uuid ?? i} node={node} defaultExpanded={false} />
          ))}
        </div>
        <div ref={bottomRef} />
      </div>

      <div className="px-6 py-3 border-t flex items-center justify-between" style={{ borderColor: "rgba(255,255,255,0.06)", background: "rgba(13,13,15,0.8)" }}>
        <div className="text-text-muted text-xs mono">{nodes.length} node{nodes.length !== 1 ? "s" : ""}</div>
        {status === "disconnected" && (
          <button onClick={() => window.location.reload()} className="text-accent-cyan text-xs hover:underline">Reconnect</button>
        )}
      </div>
    </div>
  );
}

export default function LivePage() {
  return (
    <Suspense fallback={
      <div className="flex flex-col flex-1">
        <Header title="Live Watch" />
        <div className="flex-1 flex items-center justify-center">
          <div className="text-text-muted animate-pulse">Connecting…</div>
        </div>
      </div>
    }>
      <LiveWatch />
    </Suspense>
  );
}
