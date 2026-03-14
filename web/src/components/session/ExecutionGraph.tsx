import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import ForceGraph3D from "react-force-graph-3d";
import * as THREE from "three";
import { UnrealBloomPass } from "three/examples/jsm/postprocessing/UnrealBloomPass.js";
import type { NodeResponse } from "@/lib/types";

// ── Colors ───────────────────────────────────────────────────
const COLORS: Record<string, string> = {
  cyan: "#38BDF8",
  green: "#34D399",
  magenta: "#A78BFA",
  yellow: "#F59E0B",
  amber: "#F59E0B",
  blue: "#38BDF8",
  gray: "#4a4a5a",
  white: "#ECECF1",
};

const SELECTED = "#818CF8";
const ERROR = "#FB7185";
const TASK_COLOR = "#c084fc"; // lighter purple to distinguish from thinking
const BG = "#0a0a0e";

function isTaskNotification(n: NodeResponse): boolean {
  return n.node_type === "user"
    && typeof n.message?.content === "string"
    && n.message.content.includes("<task-notification>");
}

function nodeHex(n: NodeResponse): string {
  if (isTaskNotification(n)) return TASK_COLOR;
  return COLORS[n.color] ?? "#A1A1B5";
}

// Radius by semantic importance
function nodeRadius(n: NodeResponse): number {
  if (isTaskNotification(n)) return 4; // task completions are important
  if (n.node_type === "user" && n.color !== "blue") return 5;
  if (n.node_type === "assistant" && n.color === "green") return 4.5;
  if (n.color === "yellow") return 3.5; // tool call
  if (n.color === "blue") return 3; // tool result
  if (n.color === "magenta") return 3; // thinking
  if (n.node_type === "progress") return 1.5;
  return 1.5;
}

// ── Graph data ───────────────────────────────────────────────
interface GNode {
  id: string;
  node: NodeResponse;
  color: string;
  r: number;
}
interface GLink { source: string; target: string }
interface GData { nodes: GNode[]; links: GLink[] }

function buildGraph(roots: NodeResponse[]): GData {
  const nodes: GNode[] = [];
  const links: GLink[] = [];
  const walk = (n: NodeResponse, pid: string | null) => {
    const id = n.uuid ?? `n${nodes.length}`;
    nodes.push({ id, node: n, color: nodeHex(n), r: nodeRadius(n) });
    if (pid) links.push({ source: pid, target: id });
    for (const c of n.children ?? []) walk(c, id);
  };
  for (const root of roots) walk(root, null);
  return { nodes, links };
}

// ── Component ────────────────────────────────────────────────
interface Props {
  roots: NodeResponse[];
  selectedId: string | null;
  onSelect: (node: NodeResponse) => void;
}

export function ExecutionGraph({ roots, selectedId, onSelect }: Props) {
  const fgRef = useRef<any>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const bloomAdded = useRef(false);

  const graphData = useMemo(() => buildGraph(roots), [roots]);

  // Add bloom post-processing once
  useEffect(() => {
    const fg = fgRef.current;
    if (!fg || bloomAdded.current) return;
    const renderer = fg.renderer?.();
    if (!renderer) return;
    bloomAdded.current = true;
    const bloom = new UnrealBloomPass(
      new THREE.Vector2(window.innerWidth, window.innerHeight),
      0.8,   // strength — subtle, not washed out
      0.4,   // radius
      0.85,  // threshold — only bright emissive nodes glow
    );
    fg.postProcessingComposer?.().addPass(bloom);

    // Add ambient + directional light for better material shading
    const scene = fg.scene?.();
    if (scene) {
      scene.add(new THREE.AmbientLight(0x404060, 0.6));
      const dirLight = new THREE.DirectionalLight(0xffffff, 0.4);
      dirLight.position.set(100, 200, 100);
      scene.add(dirLight);

      // Starfield for depth reference
      const starsGeo = new THREE.BufferGeometry();
      const starPositions = new Float32Array(1500 * 3);
      for (let i = 0; i < 1500; i++) {
        starPositions[i * 3] = (Math.random() - 0.5) * 1200;
        starPositions[i * 3 + 1] = (Math.random() - 0.5) * 1200;
        starPositions[i * 3 + 2] = (Math.random() - 0.5) * 1200;
      }
      starsGeo.setAttribute("position", new THREE.BufferAttribute(starPositions, 3));
      const starsMat = new THREE.PointsMaterial({ color: 0x444466, size: 0.5, transparent: true, opacity: 0.5 });
      scene.add(new THREE.Points(starsGeo, starsMat));
    }
  }, [graphData]);

  // Tune forces
  useEffect(() => {
    const fg = fgRef.current;
    if (!fg) return;
    fg.d3Force("charge")?.strength(-150).distanceMax(400);
    fg.d3Force("link")?.distance(40).strength(0.6);
    fg.d3Force("center")?.strength(0.4);
  }, [graphData]);

  // Fit to view
  useEffect(() => {
    const t = setTimeout(() => fgRef.current?.zoomToFit(800, 80), 2500);
    return () => clearTimeout(t);
  }, [graphData]);

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

  // Custom 3D nodes
  const nodeObj = useCallback((g: any) => {
    const sel = g.node.uuid === selectedId;
    const err = g.node.has_error;
    const hex = sel ? SELECTED : err ? ERROR : g.color;
    const r = g.r as number;

    const group = new THREE.Group();

    // Core
    const mat = new THREE.MeshStandardMaterial({
      color: hex,
      emissive: hex,
      emissiveIntensity: sel ? 1.2 : 0.5,
      metalness: 0.3,
      roughness: 0.4,
    });
    group.add(new THREE.Mesh(new THREE.SphereGeometry(r, 20, 14), mat));

    // Outer glow
    group.add(new THREE.Mesh(
      new THREE.SphereGeometry(r * 2.2, 10, 8),
      new THREE.MeshBasicMaterial({ color: hex, transparent: true, opacity: sel ? 0.12 : 0.04 }),
    ));

    // Selection ring
    if (sel) {
      const ring = new THREE.Mesh(
        new THREE.TorusGeometry(r + 2, 0.4, 8, 32),
        new THREE.MeshBasicMaterial({ color: SELECTED, transparent: true, opacity: 0.7 }),
      );
      group.add(ring);
    }

    // Task ring indicator
    if (isTaskNotification(g.node) && !sel) {
      const taskRing = new THREE.Mesh(
        new THREE.TorusGeometry(r + 1.5, 0.3, 8, 24),
        new THREE.MeshBasicMaterial({ color: TASK_COLOR, transparent: true, opacity: 0.5 }),
      );
      group.add(taskRing);
    }

    // Error pip
    if (err && !sel) {
      const pip = new THREE.Mesh(
        new THREE.SphereGeometry(1.2, 8, 6),
        new THREE.MeshBasicMaterial({ color: ERROR }),
      );
      pip.position.set(r + 1, r + 1, 0);
      group.add(pip);
    }

    return group;
  }, [selectedId]);

  // Tooltip
  const nodeLabel = useCallback((g: any) => {
    const n = g.node as NodeResponse;
    const isTask = isTaskNotification(n);
    const tool = n.tool_name ?? n.tool_use?.name;
    const badge = isTask
      ? `<span style="color:#c084fc">[Task]</span> `
      : tool ? `<span style="color:#F59E0B">[${tool}]</span> ` : "";
    const text = isTask
      ? (typeof n.message?.content === "string"
          ? n.message.content.match(/<summary>([\s\S]*?)<\/summary>/)?.[1] ?? n.label
          : n.label)
      : (n.summary || n.label);
    return `<div style="font:11px/1.4 'Geist Mono',monospace;background:rgba(10,10,14,0.92);padding:6px 10px;border-radius:6px;border:1px solid rgba(129,140,248,0.2);color:#ECECF1;max-width:320px;word-wrap:break-word;backdrop-filter:blur(8px)">${badge}${text}</div>`;
  }, []);

  // Link color based on selection
  const linkColor = useCallback((link: any) => {
    const s = typeof link.source === "object" ? link.source.id : link.source;
    const t = typeof link.target === "object" ? link.target.id : link.target;
    if (s === selectedId || t === selectedId) return "rgba(129,140,248,0.4)";
    return "rgba(255,255,255,0.06)";
  }, [selectedId]);

  const linkWidth = useCallback((link: any) => {
    const s = typeof link.source === "object" ? link.source.id : link.source;
    const t = typeof link.target === "object" ? link.target.id : link.target;
    if (s === selectedId || t === selectedId) return 1.2;
    return 0.3;
  }, [selectedId]);

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
        // Radial outward layout — root at center, branches spiral out
        dagMode="radialout"
        dagLevelDistance={50}
        // Nodes
        nodeThreeObject={nodeObj}
        nodeLabel={nodeLabel}
        onNodeClick={handleClick}
        enableNodeDrag={true}
        // Links
        linkColor={linkColor}
        linkWidth={linkWidth}
        linkOpacity={0.6}
        linkCurvature={0.15}
        linkCurveRotation={0.5}
        linkDirectionalArrowLength={2.5}
        linkDirectionalArrowRelPos={0.92}
        linkDirectionalArrowColor={() => "rgba(129,140,248,0.25)"}
        linkDirectionalParticles={2}
        linkDirectionalParticleWidth={0.5}
        linkDirectionalParticleSpeed={0.005}
        linkDirectionalParticleColor={() => "rgba(129,140,248,0.6)"}
        // Nav
        enableNavigationControls={true}
        showNavInfo={false}
        // Simulation
        warmupTicks={150}
        cooldownTime={5000}
      />

      {/* Legend */}
      <div style={{
        position: "absolute", bottom: "12px", left: "12px",
        display: "flex", gap: "10px", flexWrap: "wrap",
        padding: "6px 12px", borderRadius: "8px",
        background: "rgba(10,10,14,0.85)", border: "1px solid rgba(255,255,255,0.06)",
        backdropFilter: "blur(8px)",
      }}>
        {[
          { l: "User", c: COLORS.cyan },
          { l: "Asst", c: COLORS.green },
          { l: "Tool", c: COLORS.yellow },
          { l: "Result", c: COLORS.blue },
          { l: "Think", c: COLORS.magenta },
          { l: "Task", c: "#c084fc" },
          { l: "Error", c: ERROR },
        ].map(({ l, c }) => (
          <div key={l} style={{ display: "flex", alignItems: "center", gap: "4px" }}>
            <span style={{ width: "6px", height: "6px", borderRadius: "50%", background: c }} />
            <span style={{ fontFamily: "var(--font-mono)", fontSize: "9px", color: "#63637A", letterSpacing: "0.04em" }}>{l}</span>
          </div>
        ))}
      </div>

      {/* Controls hint */}
      <div style={{
        position: "absolute", top: "10px", right: "10px",
        fontFamily: "var(--font-mono)", fontSize: "9px", color: "#4a4a5a",
        background: "rgba(10,10,14,0.7)", padding: "4px 8px", borderRadius: "6px",
        border: "1px solid rgba(255,255,255,0.04)",
      }}>
        Drag: orbit &middot; Scroll: zoom &middot; Right-drag: pan &middot; Click: select
      </div>
    </div>
  );
}
