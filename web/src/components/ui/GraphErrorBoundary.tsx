/**
 * Error boundary for lazy-loaded 3D graph component.
 *
 * Catches WebGL failures and shows a user-friendly fallback.
 */

import React from "react";

export class GraphErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean }
> {
  state = { hasError: false };

  static getDerivedStateFromError() {
    return { hasError: true };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{
          display: "flex", alignItems: "center", justifyContent: "center",
          height: "100%", color: "var(--text-3)", fontFamily: "var(--font-mono)",
          fontSize: "12px", flexDirection: "column", gap: "8px",
        }}>
          <span style={{ fontSize: "18px" }}>⚠</span>
          <span>3D graph failed to load</span>
          <span style={{ fontSize: "10px", color: "var(--text-3)" }}>WebGL may not be available</span>
        </div>
      );
    }
    return this.props.children;
  }
}
