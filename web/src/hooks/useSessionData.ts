/**
 * Custom hook for session data fetching and live SSE subscription.
 *
 * Encapsulates:
 * - Initial data fetch (session metadata, node tree, OTEL summary)
 * - Live SSE subscription for real-time node updates
 * - UUID deduplication to prevent duplicate nodes
 *
 * @param id - Session ID to load
 * @returns Session data, loading state, and error
 */

import { useEffect, useRef, useState } from "react";
import { api } from "@/lib/api";
import type { NodeResponse, OtelSessionSummary, SessionFile } from "@/lib/types";

interface SessionDataState {
  session: SessionFile | null;
  roots: NodeResponse[];
  otelSummary: OtelSessionSummary | null;
  loading: boolean;
  error: string | null;
}

export function useSessionData(id: string): SessionDataState {
  const [session, setSession] = useState<SessionFile | null>(null);
  const [roots, setRoots] = useState<NodeResponse[]>([]);
  const [otelSummary, setOtelSummary] = useState<OtelSessionSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Initial data fetch
  useEffect(() => {
    let cancelled = false;

    Promise.all([
      api.session(id),
      api.sessionNodes(id),
      api.otelSessionSummary(id).catch(() => null),
    ])
      .then(([s, tree, otel]) => {
        if (cancelled) return;
        setSession(s);
        setRoots(tree.roots);
        setOtelSummary(otel);
      })
      .catch((e: Error) => {
        if (!cancelled) setError(e.message);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => { cancelled = true; };
  }, [id]);

  // Live SSE subscription for real-time node streaming
  useEffect(() => {
    const seenUuids = new Set<string>();

    // Collect existing UUIDs to avoid duplicates
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
        // Ignore parse errors / heartbeats
      }
    };

    return () => es.close();
  }, [id]);

  return { session, roots, otelSummary, loading, error };
}
