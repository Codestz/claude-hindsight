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

  const statusDotCls =
    status === "live" ? "dot dot--pulse" :
    status === "connecting" ? "dot dot--amber dot--pulse" :
    "dot dot--red";

  const statusLabel =
    status === "live" ? "LIVE" :
    status === "connecting" ? "CONNECTING" :
    "DISCONNECTED";

  return (
    <div className="flex flex-col flex-1">
      <Header
        title={id ? `Live: ${shortId(id)}` : "Live Watch"}
        subtitle="Real-time session stream"
        actions={
          <div className="flex items-center gap-4">
            {totalCost > 0 && (
              <div className="mono tabular text-sm" style={{ color: "var(--amber)" }}>{formatCost(totalCost)}</div>
            )}
            {totalTokens > 0 && (
              <div className="mono tabular text-sm" style={{ color: "var(--cyan)" }}>{formatTokens(totalTokens)} tok</div>
            )}
            <div className="flex items-center gap-2">
              <span className={statusDotCls} aria-hidden="true" />
              <span className="mono" style={{
                fontSize: "9px",
                letterSpacing: "0.1em",
                color: status === "live" ? "var(--accent)" : status === "connecting" ? "var(--amber)" : "var(--red)",
              }}>
                {statusLabel}
              </span>
            </div>
            {id && (
              <Link
                href={`/sessions?id=${encodeURIComponent(id)}`}
                className="mono text-xs"
                style={{ color: "var(--text-2)", fontSize: "11px" }}
              >
                ← Detail
              </Link>
            )}
          </div>
        }
      />

      <div className="flex-1 overflow-auto p-4">
        {nodes.length === 0 && status !== "disconnected" && (
          <div className="text-center py-16">
            <div className="mono text-sm animate-pulse" style={{ color: "var(--text-3)" }}>Waiting for events…</div>
          </div>
        )}
        {nodes.length === 0 && status === "disconnected" && (
          <div className="text-center py-16">
            <div className="mono text-sm mb-2" style={{ color: "var(--red)" }}>Stream ended</div>
            {id && (
              <Link
                href={`/sessions?id=${encodeURIComponent(id)}`}
                className="mono text-xs hover:underline"
                style={{ color: "var(--accent)" }}
              >
                View full session →
              </Link>
            )}
          </div>
        )}
        <div className="space-y-0.5">
          {nodes.map((node, i) => (
            <div
              key={node.uuid ?? i}
              style={{ animation: "fadeIn 0.15s ease forwards" }}
            >
              <NodeRow node={node} defaultExpanded={false} />
            </div>
          ))}
        </div>
        <div ref={bottomRef} />
      </div>

      <div
        className="px-6 py-3 flex items-center justify-between"
        style={{
          borderTop: "1px solid var(--border)",
          background: "rgba(5,5,5,0.95)",
        }}
      >
        <div className="mono text-2xs" style={{ color: "var(--text-3)" }}>
          {nodes.length} node{nodes.length !== 1 ? "s" : ""}
        </div>
        {status === "disconnected" && (
          <button
            onClick={() => window.location.reload()}
            className="mono text-xs hover:underline"
            style={{ color: "var(--accent)", background: "none", border: "none", cursor: "pointer" }}
          >
            Reconnect
          </button>
        )}
      </div>

      <style>{`
        @keyframes fadeIn {
          from { opacity: 0; transform: translateY(4px); }
          to   { opacity: 1; transform: translateY(0); }
        }
      `}</style>
    </div>
  );
}

export default function LivePage() {
  return (
    <Suspense fallback={
      <div className="flex flex-col flex-1">
        <Header title="Live Watch" />
        <div className="flex-1 flex items-center justify-center">
          <div className="mono text-sm animate-pulse" style={{ color: "var(--text-2)" }}>Connecting…</div>
        </div>
      </div>
    }>
      <LiveWatch />
    </Suspense>
  );
}
