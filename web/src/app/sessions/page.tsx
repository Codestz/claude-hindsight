"use client";

import Link from "next/link";

export default function SessionsPage() {
  return (
    <div style={{
      height: "calc(100vh - 56px)", display: "flex", alignItems: "center",
      justifyContent: "center", fontSize: "14px",
      color: "var(--text-3)", fontFamily: "var(--font-sans)", textAlign: "center",
    }}>
      No session selected. Pick one from{" "}
      <Link href="/projects/" style={{ color: "var(--accent)", marginLeft: "4px" }}>
        Projects
      </Link>
      .
    </div>
  );
}
