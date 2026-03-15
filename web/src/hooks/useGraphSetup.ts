/**
 * Hook for 3D graph scene setup — bloom, lighting, starfield, forces.
 *
 * Runs once after the graph mounts. Adapts visual quality to node count.
 */

import { useEffect, useRef } from "react";
import * as THREE from "three";
import { UnrealBloomPass } from "three/examples/jsm/postprocessing/UnrealBloomPass.js";

type PerfTier = "high" | "medium" | "low";

export function useGraphSetup(
  fgRef: React.MutableRefObject<any>,
  tier: PerfTier,
  nodeCount: number,
) {
  const bloomAdded = useRef(false);

  // Scene setup: bloom, lights, starfield
  useEffect(() => {
    const fg = fgRef.current;
    if (!fg || bloomAdded.current) return;
    const renderer = fg.renderer?.();
    if (!renderer) return;
    bloomAdded.current = true;

    // Bloom post-processing (skip for large graphs)
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

      // Starfield for depth reference
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
  }, [fgRef, nodeCount, tier]);

  // Force tuning
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
  }, [fgRef, nodeCount, tier]);

  // Fit to view after layout settles
  useEffect(() => {
    const t = setTimeout(
      () => fgRef.current?.zoomToFit(800, 80),
      tier === "low" ? 3500 : 2500,
    );
    return () => clearTimeout(t);
  }, [fgRef, nodeCount, tier]);
}
