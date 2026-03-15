import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import ForceGraph3D from "react-force-graph-3d";
import * as THREE from "three";
import { UnrealBloomPass } from "three/examples/jsm/postprocessing/UnrealBloomPass.js";
import type { NodeResponse } from "@/lib/types";
import { isTaskNotification } from "@/lib/node-meta";
import type { ExecutionGraphProps, GraphNode, GraphLink } from "./types";

// ── Colors ───────────────────────────────────────────────────
const COLORS: Record<string, string> = {
  cyan: "#38BDF8", green: "#34D399", magenta: "#A78BFA",
  yellow: "#F59E0B", amber: "#F59E0B", blue: "#38BDF8",
  gray: "#4a4a5a", white: "#ECECF1",
};
const SELECTED = "#818CF8";
const ERROR = "#FB7185";
const TASK_COLOR = "#c084fc";
const BG = "#0a0a0e";

function nodeHex(n: NodeResponse): string {
  if (isTaskNotification(n)) return TASK_COLOR;
  return COLORS[n.color] ?? "#A1A1B5";
}

function nodeRadius(n: NodeResponse): number {
  if (isTaskNotification(n)) return 4;
  if (n.node_type === "user" && n.color !== "blue") return 5;
  if (n.node_type === "assistant" && n.color === "green") return 4.5;
  if (n.color === "yellow") return 3.5;
  if (n.color === "blue") return 3;
  if (n.color === "magenta") return 3;
  if (n.node_type === "progress") return 1.5;
  return 1.5;
}

// ── Graph data ───────────────────────────────────────────────
function buildGraph(roots: NodeResponse[]): { nodes: GraphNode[]; links: GraphLink[] } {
  const nodes: GraphNode[] = [];
  const links: GraphLink[] = [];
  const walk = (n: NodeResponse, pid: string | null) => {
    const id = n.uuid ?? `n${nodes.length}`;
    nodes.push({ id, node: n, color: nodeHex(n), r: nodeRadius(n) });
    if (pid) links.push({ source: pid, target: id });
    for (const c of n.children ?? []) walk(c, id);
  };
  for (const root of roots) walk(root, null);
  return { nodes, links };
}

// ── Performance tier based on node count ─────────────────────
function perfTier(count: number): "high" | "medium" | "low" {
  if (count < 300) return "high";
  if (count < 1000) return "medium";
  return "low";
}

// ── Shared geometry cache (avoid creating per-node) ──────────
let geoCache = new Map<string, THREE.SphereGeometry>();
function getSphereGeo(r: number, segs: number): THREE.SphereGeometry {
  const key = `${r}-${segs}`;
  if (!geoCache.has(key)) geoCache.set(key, new THREE.SphereGeometry(r, segs, segs));
  return geoCache.get(key)!;
}
function disposeGeoCache() {
  for (const geo of geoCache.values()) geo.dispose();
  geoCache = new Map();
}

// ── Component ────────────────────────────────────────────────
export function ExecutionGraph({ roots, selectedId, onSelect }: ExecutionGraphProps) {
  const fgRef = useRef<any>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const bloomAdded = useRef(false);

  const graphData = useMemo(() => buildGraph(roots), [roots]);
  const tier = perfTier(graphData.nodes.length);

  // Dispose geometry cache on unmount
  useEffect(() => () => disposeGeoCache(), []);

  // Add bloom only for small graphs
  useEffect(() => {
    const fg = fgRef.current;
    if (!fg || bloomAdded.current) return;
    const renderer = fg.renderer?.();
    if (!renderer) return;
    bloomAdded.current = true;

    // Only add bloom for high/medium perf tiers
    if (tier !== "low") {
      const bloom = new UnrealBloomPass(
        new THREE.Vector2(window.innerWidth, window.innerHeight),
        tier === "high" ? 0.8 : 0.4,
        0.4,
        0.85,
      );
      fg.postProcessingComposer?.().addPass(bloom);
    }

    const scene = fg.scene?.();
    if (scene) {
      scene.add(new THREE.AmbientLight(0x404060, 0.8));
      const dirLight = new THREE.DirectionalLight(0xffffff, 0.3);
      dirLight.position.set(100, 200, 100);
      scene.add(dirLight);

      // Starfield — fewer for large graphs
      const starCount = tier === "high" ? 1500 : tier === "medium" ? 500 : 200;
      const starsGeo = new THREE.BufferGeometry();
      const positions = new Float32Array(starCount * 3);
      for (let i = 0; i < starCount; i++) {
        positions[i * 3] = (Math.random() - 0.5) * 1200;
        positions[i * 3 + 1] = (Math.random() - 0.5) * 1200;
        positions[i * 3 + 2] = (Math.random() - 0.5) * 1200;
      }
      starsGeo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
      scene.add(new THREE.Points(starsGeo,
        new THREE.PointsMaterial({ color: 0x444466, size: 0.5, transparent: true, opacity: 0.5 }),
      ));
    }
  }, [graphData, tier]);

  // Tune forces — weaker for large graphs (faster convergence)
  useEffect(() => {
    const fg = fgRef.current;
    if (!fg) return;
    if (tier === "low") {
      fg.d3Force("charge")?.strength(-80).distanceMax(250);
      fg.d3Force("link")?.distance(30).strength(0.7);
      fg.d3Force("center")?.strength(0.5);
    } else {
      fg.d3Force("charge")?.strength(-150).distanceMax(400);
      fg.d3Force("link")?.distance(40).strength(0.6);
      fg.d3Force("center")?.strength(0.4);
    }
  }, [graphData, tier]);

  // Fit to view
  useEffect(() => {
    const t = setTimeout(() => fgRef.current?.zoomToFit(800, 80), tier === "low" ? 3500 : 2500);
    return () => clearTimeout(t);
  }, [graphData, tier]);

  // Click → select + fly
  const handleClick = useCallback((g: any) => {
    onSelect(g.node);
    const d = 50;
    const ratio = 1 + d / Math.hypot(g.x, g.y, g.z);
    fgRef.current?.cameraPosition(
      { x: g.x * ratio, y: g.y * ratio, z: g.z * ratio },
      g, 800,
    );
  }, [onSelect]);

  // Custom 3D nodes — performance-scaled
  const nodeObj = useCallback((g: any) => {
    const sel = g.node.uuid === selectedId;
    const err = g.node.has_error;
    const hex = sel ? SELECTED : err ? ERROR : g.color;
    const r = g.r as number;

    // Low tier: single mesh, basic material, low-poly
    if (tier === "low") {
      const segs = r > 3 ? 8 : 6;
      const mat = new THREE.MeshBasicMaterial({
        color: hex,
        transparent: !sel,
        opacity: sel ? 1 : 0.8,
      });
      const mesh = new THREE.Mesh(getSphereGeo(r, segs), mat);
      return mesh;
    }

    // Medium/High: group with glow
    const group = new THREE.Group();
    const segs = tier === "high" ? (r > 3 ? 16 : 10) : (r > 3 ? 10 : 6);

    const mat = new THREE.MeshStandardMaterial({
      color: hex,
      emissive: hex,
      emissiveIntensity: sel ? 1.2 : 0.5,
      metalness: 0.3,
      roughness: 0.4,
    });
    group.add(new THREE.Mesh(getSphereGeo(r, segs), mat));

    // Glow — only for larger nodes or selected
    if (sel || r >= 3) {
      group.add(new THREE.Mesh(
        getSphereGeo(r * 2.2, 6),
        new THREE.MeshBasicMaterial({ color: hex, transparent: true, opacity: sel ? 0.12 : 0.04 }),
      ));
    }

    // Selection ring
    if (sel) {
      group.add(new THREE.Mesh(
        new THREE.TorusGeometry(r + 2, 0.4, 6, 20),
        new THREE.MeshBasicMaterial({ color: SELECTED, transparent: true, opacity: 0.7 }),
      ));
    }

    // Task ring
    if (isTaskNotification(g.node) && !sel) {
      group.add(new THREE.Mesh(
        new THREE.TorusGeometry(r + 1.5, 0.3, 6, 16),
        new THREE.MeshBasicMaterial({ color: TASK_COLOR, transparent: true, opacity: 0.5 }),
      ));
    }

    // Error pip
    if (err && !sel) {
      const pip = new THREE.Mesh(
        getSphereGeo(1.2, 4),
        new THREE.MeshBasicMaterial({ color: ERROR }),
      );
      pip.position.set(r + 1, r + 1, 0);
      group.add(pip);
    }

    return group;
  }, [selectedId, tier]);

  // Tooltip — escape HTML to prevent XSS from node content
  const nodeLabel = useCallback((g: any) => {
    const n = g.node as NodeResponse;
    const isTask = isTaskNotification(n);
    const tool = n.tool_name ?? n.tool_use?.name;
    const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
    const badge = isTask
      ? `<span style="color:#c084fc">[Task]</span> `
      : tool ? `<span style="color:#F59E0B">[${esc(tool)}]</span> ` : "";
    const rawText = isTask
      ? (typeof n.message?.content === "string"
          ? n.message.content.match(/<summary>([\s\S]*?)<\/summary>/)?.[1] ?? n.label
          : n.label)
      : (n.summary || n.label);
    const text = esc(rawText);
    return `<div style="font:11px/1.4 'Geist Mono',monospace;background:rgba(10,10,14,0.92);padding:6px 10px;border-radius:6px;border:1px solid rgba(129,140,248,0.2);color:#ECECF1;max-width:320px;word-wrap:break-word">${badge}${text}</div>`;
  }, []);

  // Link styling — simpler for large graphs
  const linkColor = useCallback((link: any) => {
    if (tier === "low") return "rgba(255,255,255,0.04)";
    const s = typeof link.source === "object" ? link.source.id : link.source;
    const t = typeof link.target === "object" ? link.target.id : link.target;
    if (s === selectedId || t === selectedId) return "rgba(129,140,248,0.4)";
    return "rgba(255,255,255,0.06)";
  }, [selectedId, tier]);

  const linkWidth = useCallback((link: any) => {
    if (tier === "low") return 0.2;
    const s = typeof link.source === "object" ? link.source.id : link.source;
    const t = typeof link.target === "object" ? link.target.id : link.target;
    if (s === selectedId || t === selectedId) return 1.2;
    return 0.3;
  }, [selectedId, tier]);

  // Container size
  const [dims, setDims] = useState({ w: 800, h: 600 });
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const ro = new ResizeObserver((e) => {
      for (const entry of e) setDims({ w: entry.contentRect.width, h: entry.contentRect.height });
    });
    ro.observe(el);
    setDims({ w: el.clientWidth, h: el.clientHeight });
    return () => ro.disconnect();
  }, []);

  return (
    <div ref={containerRef} style={{ width: "100%", height: "100%", position: "relative" }}>
      <ForceGraph3D
        ref={fgRef}
        graphData={graphData}
        width={dims.w}
        height={dims.h}
        backgroundColor={BG}
        dagMode="radialout"
        dagLevelDistance={tier === "low" ? 35 : 50}
        // Nodes
        nodeThreeObject={nodeObj}
        nodeLabel={nodeLabel}
        onNodeClick={handleClick}
        enableNodeDrag={true}
        // Links — scale down for performance
        linkColor={linkColor}
        linkWidth={linkWidth}
        linkOpacity={0.6}
        linkCurvature={tier === "low" ? 0 : 0.15}
        linkCurveRotation={tier === "low" ? 0 : 0.5}
        linkDirectionalArrowLength={tier === "low" ? 0 : 2.5}
        linkDirectionalArrowRelPos={0.92}
        linkDirectionalArrowColor={() => "rgba(129,140,248,0.25)"}
        linkDirectionalParticles={tier === "low" ? 0 : tier === "medium" ? 1 : 2}
        linkDirectionalParticleWidth={0.5}
        linkDirectionalParticleSpeed={0.005}
        linkDirectionalParticleColor={() => "rgba(129,140,248,0.6)"}
        // Navigation
        enableNavigationControls={true}
        showNavInfo={false}
        // Simulation — faster convergence for large graphs
        warmupTicks={tier === "low" ? 60 : tier === "medium" ? 100 : 150}
        cooldownTime={tier === "low" ? 2000 : tier === "medium" ? 3500 : 5000}
      />

      {/* Legend */}
      <div style={{
        position: "absolute", bottom: "12px", left: "12px",
        display: "flex", gap: "10px", flexWrap: "wrap",
        padding: "6px 12px", borderRadius: "8px",
        background: "rgba(10,10,14,0.85)", border: "1px solid rgba(255,255,255,0.06)",
      }}>
        {[
          { l: "User", c: COLORS.cyan }, { l: "Asst", c: COLORS.green },
          { l: "Tool", c: COLORS.yellow }, { l: "Result", c: COLORS.blue },
          { l: "Think", c: COLORS.magenta }, { l: "Task", c: "#c084fc" },
          { l: "Error", c: ERROR },
        ].map(({ l, c }) => (
          <div key={l} style={{ display: "flex", alignItems: "center", gap: "4px" }}>
            <span style={{ width: "6px", height: "6px", borderRadius: "50%", background: c }} />
            <span style={{ fontFamily: "var(--font-mono)", fontSize: "9px", color: "#63637A", letterSpacing: "0.04em" }}>{l}</span>
          </div>
        ))}
      </div>

      {/* Performance + controls */}
      <div style={{
        position: "absolute", top: "10px", right: "10px",
        fontFamily: "var(--font-mono)", fontSize: "9px", color: "#4a4a5a",
        background: "rgba(10,10,14,0.7)", padding: "4px 8px", borderRadius: "6px",
        border: "1px solid rgba(255,255,255,0.04)",
      }}>
        {graphData.nodes.length} nodes {tier !== "high" && `\u00b7 ${tier} detail`}
        {" \u00b7 "} Drag: orbit &middot; Scroll: zoom
      </div>
    </div>
  );
}
